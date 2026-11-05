//! sim-guard — may this shell command start an iOS test run? (port of scripts/sim-guard, 04/11/2026)
//!
//! An `xcodebuild test` (or `test-without-building`) run must
//!   - pass `-parallel-testing-enabled NO`: without it Xcode clones the simulator once per test
//!     worker (tap-luyen 29/10 and 31/10/2026: 4 clones, load 490, CPU 98% on the 8 GB Mac);
//!   - name at most one `-destination`;
//!   - not share a command that sends something to the background (`&`) with another run;
//!   - start only when no other `xcodebuild test` is running (two sessions on one machine, 31/10/2026).
//!
//! Commands without xcodebuild pass untouched. The command is read the way the shell reads it
//! (quotes, comments, heredocs, line continuations, `sh -c "…"`); a command inside a script file is
//! not seen. Behaviour matches the Python original case for case (tests/adapters/run.sh runs both).

use std::process::Command;

const FIX: &str = "-parallel-testing-enabled NO -maximum-concurrent-test-simulator-destinations 1";
const ACTIONS: &[&str] = &["test", "test-without-building"];
/// xcodebuild options that take no value, so an action word may follow them (`-quiet test`).
const BOOL_OPTS: &[&str] = &["-quiet", "-verbose", "-json", "-allowProvisioningUpdates", "-allowProvisioningDeviceRegistration",
    "-skipMacroValidation", "-skipPackagePluginValidation", "-showBuildTimingSummary",
    "-hideShellScriptEnvironment", "-skipUnavailableActions", "-disableAutomaticPackageResolution"];
/// Longest first, so `&&` wins over `&`.
const OPS: &[&str] = &["&>>", "<<<", "<<-", ";;", "&&", "&>", "||", "|&", "<<", "<&", "<>", ">>", ">&", ">|",
    ";", "&", "|", "(", ")", "<", ">"];
const SEPARATORS: &[&str] = &[";", ";;", "&", "&&", "||", "|", "|&", "(", ")", "\n"];
const PREFIX: &[&str] = &["{", "}", "!", "if", "then", "elif", "else", "while", "until", "do", "time", "exec", "nohup", "command"];
const SHELLS: &[&str] = &["sh", "bash", "zsh", "dash"];
const SHELL_ARG_OPTS: &[&str] = &["-o", "+o", "-O", "+O", "--rcfile", "--init-file"];

/// Wrappers before the real program, with their options that take a value.
fn wrapper_opts(name: &str) -> Option<&'static [&'static str]> {
    Some(match name {
        "env" => &["-u", "--unset", "-C", "--chdir", "-S", "--split-string"],
        "timeout" | "gtimeout" => &["-s", "--signal", "-k", "--kill-after"],
        "xcrun" => &["--sdk", "-sdk", "--toolchain", "-toolchain"],
        "sudo" => &["-u", "-g", "-h", "-p", "-D", "-C", "-U"],
        "nice" => &["-n", "--adjustment"],
        "caffeinate" => &["-t", "-w"],
        _ => return None,
    })
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    W(String),
    Op(&'static str),
}

/// Python's str.isspace() for one char (the Rust set lacks the four ASCII separators).
fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

fn find(s: &[char], from: usize, pat: &str) -> Option<usize> {
    let p: Vec<char> = pat.chars().collect();
    (from..s.len()).find(|&i| s[i..].starts_with(&p))
}

fn text(s: &[char], a: usize, b: usize) -> String {
    let n = s.len();
    s[a.min(n)..b.min(n).max(a.min(n))].iter().collect()
}

struct Lexer<'a> {
    s: &'a [char],
    subs: Vec<String>,
}

impl Lexer<'_> {
    /// The end of a `$(…)` or backtick substitution starting at i (its command goes to subs), or None.
    fn substitution(&mut self, i: usize) -> Option<usize> {
        let (s, n) = (self.s, self.s.len());
        if s[i] == '`' {
            let j = find(s, i + 1, "`").unwrap_or(n);
            self.subs.push(text(s, i + 1, j));
            return Some((j + 1).min(n));
        }
        if s[i] == '$' && i + 1 < n && s[i + 1] == '(' {
            let (mut depth, mut j) = (1i32, i + 2);
            while j < n && depth != 0 {
                if s[j] == '\'' || s[j] == '"' {  // a quoted ( or ) does not count
                    j = match find(s, j + 1, &s[j].to_string()) { Some(k) => k + 1, None => n };
                    continue;
                }
                depth += match s[j] { '(' => 1, ')' => -1, _ => 0 };
                j += 1 + (s[j] == '\\') as usize;
            }
            let j = j.min(n);
            self.subs.push(text(s, i + 2, if depth == 0 { j - 1 } else { j }));
            return Some(j);
        }
        None
    }
}

/// (tokens, substitutions): words with quotes removed and operators (or "\n"); substitutions are the
/// commands inside `$(…)` and backticks the shell would run (not in single quotes, nor in a heredoc
/// with a quoted tag). Comments and heredoc bodies are dropped; an unclosed quote runs to the end.
fn lex(src: &str) -> (Vec<Tok>, Vec<String>) {
    let s: Vec<char> = src.chars().collect();
    let n = s.len();
    let mut lx = Lexer { s: &s, subs: Vec::new() };
    let mut toks: Vec<Tok> = Vec::new();
    let mut word: Option<String> = None;
    let mut heredocs: Vec<(String, bool, bool)> = Vec::new();
    let mut i = 0;
    macro_rules! end {
        () => {
            if let Some(w) = word.take() {
                toks.push(Tok::W(w));
            }
        };
    }
    macro_rules! add {
        ($p:expr) => {
            word.get_or_insert_with(String::new).push_str(&$p)
        };
    }
    while i < n {
        let c = s[i];
        let e = if c == '`' || c == '$' { lx.substitution(i) } else { None };
        if let Some(e) = e {
            add!(text(&s, i, e));
            i = e;
        } else if c == '\\' {
            if i + 1 < n && s[i + 1] == '\n' {  // line continuation: one word stream across the lines
                i += 2;
                continue;
            }
            add!(text(&s, i + 1, i + 2));
            i += 2;
        } else if c == '\'' {
            let j = find(&s, i + 1, "'").unwrap_or(n);
            add!(text(&s, i + 1, j));
            i = j + 1;
        } else if c == '"' {
            let (mut j, mut buf) = (i + 1, String::new());
            while j < n && s[j] != '"' {
                if s[j] == '`' || s[j] == '$' {
                    if let Some(e) = lx.substitution(j) {
                        buf.push_str(&text(&s, j, e));
                        j = e;
                        continue;
                    }
                }
                if s[j] == '\\' && j + 1 < n {
                    if s[j + 1] != '\n' {
                        if "\"\\$`".contains(s[j + 1]) {
                            buf.push(s[j + 1]);
                        } else {
                            buf.push_str(&text(&s, j, j + 2));
                        }
                    }
                    j += 2;
                    continue;
                }
                buf.push(s[j]);
                j += 1;
            }
            add!(buf);
            i = j + 1;
        } else if c == '#' && word.is_none() {
            while i < n && s[i] != '\n' {
                i += 1;
            }
        } else if c == ' ' || c == '\t' || c == '\r' {
            end!();
            i += 1;
        } else if c == '\n' {
            end!();
            toks.push(Tok::Op("\n"));
            i += 1;
            // The body is data up to its tag line; unquoted, it still expands.
            for (tag, tabs, quoted) in std::mem::take(&mut heredocs) {
                let start = i;
                let mut body_end = i;
                while i < n {
                    let (line, next) = match find(&s, i, "\n") {
                        Some(j) => (text(&s, i, j), j + 1),
                        None => (text(&s, i, n), n),
                    };
                    i = next;
                    let l = if tabs { line.trim_start_matches('\t') } else { line.as_str() };
                    if l == tag {
                        break;
                    }
                    body_end = i;
                }
                let mut k = start;
                while !quoted && k < body_end {
                    let e = if s[k] == '`' || s[k] == '$' { lx.substitution(k) } else { None };
                    k = e.unwrap_or(k + 1 + (s[k] == '\\') as usize);
                }
            }
        } else if word.is_none() && s[i..].starts_with(&['(', '(']) {  // arithmetic `(( n << 2 ))`
            i = find(&s, i + 2, "))").map_or(n, |j| j + 2);
        } else if ";&|()<>".contains(c) {
            if (c == '<' || c == '>') && word.as_ref().is_some_and(|w| !w.is_empty() && w.chars().all(|d| d.is_ascii_digit())) {
                word = None;  // `2>`: a number glued to a redirect is the file descriptor, not a word
            }
            end!();
            let op = *OPS.iter().find(|o| s[i..].starts_with(&o.chars().collect::<Vec<_>>())).expect("op");
            i += op.chars().count();
            toks.push(Tok::Op(op));
            if op == "<<" || op == "<<-" {
                while i < n && (s[i] == ' ' || s[i] == '\t') {
                    i += 1;
                }
                // The tag with its quotes removed: E"OF" is EOF.
                let (mut p, mut tag, mut quoted) = (i, String::new(), false);
                while p < n {
                    let ch = s[p];
                    if ch == '\'' || ch == '"' {
                        match find(&s, p + 1, &ch.to_string()) {
                            Some(q) => {
                                tag.push_str(&text(&s, p + 1, q));
                                quoted = true;
                                p = q + 1;
                            }
                            None => break,
                        }
                    } else if ch == '\\' && p + 1 < n && s[p + 1] != '\n' {
                        tag.push(s[p + 1]);
                        quoted = true;
                        p += 2;
                    } else if !py_space(ch) && !";&|()<>'\"\\".contains(ch) {
                        tag.push(ch);
                        p += 1;
                    } else {
                        break;
                    }
                }
                if p > i {
                    heredocs.push((tag, op == "<<-", quoted));
                    i = p;
                }
            }
        } else {
            word.get_or_insert_with(String::new).push(c);
            i += 1;
        }
    }
    end!();
    let subs = lx.subs;
    (toks, subs)
}

/// Word lists of the simple commands, redirects and their targets removed.
fn simple_commands(toks: &[Tok]) -> Vec<Vec<String>> {
    let (mut out, mut seg, mut skip) = (Vec::new(), Vec::new(), false);
    for t in toks {
        match t {
            Tok::Op(o) if SEPARATORS.contains(o) => {
                out.push(std::mem::take(&mut seg));
                skip = false;
            }
            // A redirect: its target is the next word (a heredoc tag was taken by lex()).
            Tok::Op(o) => skip = *o != "<<" && *o != "<<-",
            Tok::W(_) if skip => skip = false,
            Tok::W(w) => seg.push(w.clone()),
        }
    }
    out.push(seg);
    out
}

fn basename(w: &str) -> &str {
    w.rsplit('/').next().unwrap_or(w)
}

/// `NAME=value` (Python `\w+=.*`).
fn is_assignment(w: &str) -> bool {
    match w.find('=') {
        Some(p) => p > 0 && w[..p].chars().all(|c| c.is_alphanumeric() || c == '_'),
        None => false,
    }
}

/// The program a simple command runs, after assignments, keywords and wrappers
/// (`CI=1 env timeout -s TERM 600 xcrun --sdk iphonesimulator xcodebuild …` -> `xcodebuild …`).
fn command_words(mut words: &[String]) -> &[String] {
    while let Some(first) = words.first() {
        let w = basename(first);
        if is_assignment(first) || PREFIX.contains(&first.as_str()) {
            words = &words[1..];
        } else if let Some(opts) = wrapper_opts(w) {
            words = &words[1..];
            while let Some(o) = words.first() {
                if !(o.starts_with('-') || is_assignment(o)) {
                    break;
                }
                let takes = opts.contains(&o.as_str()) && !o.contains('=');
                words = &words[(if takes { 2 } else { 1 }).min(words.len())..];
            }
            if (w == "timeout" || w == "gtimeout") && !words.is_empty() {
                words = &words[1..];  // the duration
            }
        } else {
            return words;
        }
    }
    words
}

fn is_test_run(words: &[String]) -> bool {
    if words.is_empty() || basename(&words[0]) != "xcodebuild" {
        return false;
    }
    // An action word that is not the value of the option before it (`-scheme test` is a scheme);
    // `-only-testing:X` carries its own value.
    words[1..].iter().enumerate().any(|(i, w)| {
        let prev = &words[i];
        ACTIONS.contains(&w.as_str()) && (!prev.starts_with('-') || BOOL_OPTS.contains(&prev.as_str()) || prev.contains(':'))
    })
}

/// (xcodebuild test runs as word lists, does any part go to the background?), `sh -c "…"` included.
fn test_runs(cmd: &str, depth: u32) -> (Vec<Vec<String>>, bool) {
    let (toks, mut inner) = lex(cmd);
    let mut background = toks.contains(&Tok::Op("&"));
    let mut runs = Vec::new();
    for seg in simple_commands(&toks) {
        let words = command_words(&seg);
        if is_test_run(words) {
            runs.push(words.to_vec());
        } else if !words.is_empty() && SHELLS.contains(&basename(&words[0])) {  // sh -c, bash -lc, bash -e -c '…'
            let (mut k, mut has_c) = (1, false);
            while k < words.len() && words[k].chars().next().is_none_or(|c| c == '-' || c == '+') && words[k] != "--" {
                has_c = has_c || (words[k].contains('c') && !words[k].starts_with("--") && !words[k].starts_with('+'));
                k += if SHELL_ARG_OPTS.contains(&words[k].as_str()) { 2 } else { 1 };
            }
            if has_c && k < words.len() {
                inner.push(words[k].clone());
            }
        }
    }
    if depth < 3 {
        for sub in inner {
            let (more, bg) = test_runs(&sub, depth + 1);
            runs.extend(more);
            background = background || bg;
        }
    }
    (runs, background)
}

fn problems(words: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let vals: Vec<String> = words.windows(2).filter(|p| p[0] == "-parallel-testing-enabled").map(|p| p[1].to_lowercase()).collect();
    if vals != ["no"] {
        out.push("it does not pass `-parallel-testing-enabled NO`, so Xcode clones the simulator once per test worker".into());
    }
    if words.iter().filter(|w| *w == "-destination").count() > 1 {
        out.push("it names more than one -destination: each one boots its own simulator".into());
    }
    out
}

/// Where the program word `xcodebuild` ends in a `ps` args string (`(?:.*/)?xcodebuild(?:\s|$)`).
fn xcodebuild_end(cmd: &str) -> Option<usize> {
    let at = |start: usize| -> Option<usize> {
        let rest = cmd[start..].strip_prefix("xcodebuild")?;
        match rest.chars().next() {
            None => Some(cmd.len()),
            Some(c) if py_space(c) => Some(cmd.len() - rest.len() + c.len_utf8()),
            _ => None,
        }
    };
    // Greedy `.*/`: the last slash that works, else the start.
    cmd.match_indices('/').rev().find_map(|(p, _)| at(p + 1)).or_else(|| at(0))
}

/// `pid command` of every xcodebuild test run on this machine (empty where `ps` is missing).
fn running_tests() -> Vec<String> {
    let Ok(o) = Command::new("ps").args(["-axo", "pid=,args="]).output() else {
        return Vec::new();
    };
    let ps = String::from_utf8_lossy(&o.stdout);
    let mut found = Vec::new();
    for line in ps.lines() {
        let line = line.trim_matches(py_space);
        let (pid, cmd) = line.split_once(' ').unwrap_or((line, ""));
        let cmd = cmd.trim_matches(py_space);
        if let Some(end) = xcodebuild_end(cmd) {
            let mut words = vec!["xcodebuild".to_string()];
            words.extend(cmd[end..].split(py_space).filter(|w| !w.is_empty()).map(String::from));
            if is_test_run(&words) {
                found.push(format!("{pid} {}", cmd.chars().take(160).collect::<String>()));
            }
        }
    }
    found
}

/// The block message for `cmd`, or None when it may run.
pub fn check(cmd: &str) -> Option<String> {
    let (runs, background) = test_runs(cmd, 0);
    if runs.is_empty() {
        return None;
    }
    let mut why: Vec<String> = runs.iter().flat_map(|w| problems(w)).collect();
    if runs.len() > 1 && background {
        why.push("it has more than one xcodebuild test run and sends work to the background (`&`), so they may overlap".into());
    }
    let busy = running_tests();
    if !busy.is_empty() {
        why.push(format!("another xcodebuild test is already running on this machine:\n    {}", busy.join("\n    ")));
    }
    if why.is_empty() {
        return None;
    }
    Some(format!(
        "BLOCKED: this xcodebuild test would start more than one simulator: {}.\n\
         Why: every extra simulator is a full iOS instance; on Chủ nhà's 8 GB Mac a few of them stall the machine.\n\
         How to fix: run one xcodebuild test at a time, with one -destination and `{FIX}`; \
         if another run is going, wait for it to finish (do not kill another session's run).",
        why.join("; ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runs(cmd: &str) -> usize {
        test_runs(cmd, 0).0.len()
    }

    #[test]
    fn reads_the_command_like_the_shell() {
        assert_eq!(runs("xcodebuild test -scheme X"), 1);
        assert_eq!(runs("xcodebuild build -scheme test"), 0);
        assert_eq!(runs("xcodebuild -scheme X -quiet test"), 1);
        assert_eq!(runs("echo xcodebuild test; rg 'xcodebuild test' AGENTS.md"), 0);
        assert_eq!(runs("cat <<'EOF'\nxcodebuild test\nEOF\necho done"), 0);
        assert_eq!(runs("cat <<EOF > n.md\nx\nEOF\nxcodebuild test"), 1);
        assert_eq!(runs("cat <<EOF\n$(xcodebuild test)\nEOF"), 1);
        assert_eq!(runs("cat <<EOF\n\\$(xcodebuild test)\nEOF"), 0);
        assert_eq!(runs("cat <<E\"OF\"\nhello\nEOF\nxcodebuild test"), 1);
        assert_eq!(runs("echo '$(xcodebuild test)'"), 0);
        assert_eq!(runs("echo \"built: $(xcodebuild test)\""), 1);
        assert_eq!(runs("echo `xcodebuild -scheme X test`"), 1);
        assert_eq!(runs("xcodebu\\\nild test"), 1);
        assert_eq!(runs("((n = 1 << 2))\nxcodebuild test"), 1);
        assert_eq!(runs("env CI=1 timeout --signal=TERM 600 xcrun --sdk iphonesimulator xcodebuild test"), 1);
        assert_eq!(runs("bash -o pipefail -c 'xcodebuild test'"), 1);
        assert_eq!(runs("2>/dev/null xcodebuild test"), 1);
        assert_eq!(runs("xcodebuild -destination-timeout 60 >/dev/null test"), 1);
        assert_eq!(runs("ls app  # later; xcodebuild test"), 0);
    }

    #[test]
    fn finds_xcodebuild_in_ps_lines() {
        assert!(xcodebuild_end("/Applications/Xcode 26.app/Contents/Developer/usr/bin/xcodebuild test -scheme X").is_some());
        assert!(xcodebuild_end("rg xcodebuild test AGENTS.md").is_none());
        assert_eq!(xcodebuild_end("xcodebuild"), Some(10));
    }

    #[test]
    fn flag_and_destinations() {
        let ok = "xcodebuild test -scheme X -destination 'platform=iOS Simulator,name=A' -parallel-testing-enabled NO";
        assert!(problems(&test_runs(ok, 0).0[0]).is_empty());
        assert_eq!(problems(&test_runs(&format!("{ok} -destination B"), 0).0[0]).len(), 1);
        assert!(test_runs(&format!("{ok} & {ok}"), 0).1);
    }
}
