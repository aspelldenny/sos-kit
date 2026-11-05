//! The shell reader shared by `sim-guard` (sos-cli) and the rules engine: quotes, comments,
//! `$(…)`, backticks, heredocs, line continuations. Moved from sos-cli `guards/sim.rs` (wave 1d);
//! the lexer is unchanged except that it now keeps heredoc bodies and reports unclosed quotes.

/// Longest first, so `&&` wins over `&`.
pub const OPS: &[&str] = &["&>>", "<<<", "<<-", ";;", "&&", "&>", "||", "|&", "<<", "<&", "<>", ">>", ">&", ">|",
    ";", "&", "|", "(", ")", "<", ">"];
pub const SEPARATORS: &[&str] = &[";", ";;", "&", "&&", "||", "|", "|&", "(", ")", "\n"];
pub const PREFIX: &[&str] = &["{", "}", "!", "if", "then", "elif", "else", "while", "until", "do"];
pub const SHELLS: &[&str] = &["sh", "bash", "zsh", "dash"];
pub const SHELL_ARG_OPTS: &[&str] = &["-o", "+o", "-O", "+O", "--rcfile", "--init-file"];

/// Wrappers before the real program, with their options that take a value.
pub fn wrapper_opts(name: &str) -> Option<&'static [&'static str]> {
    Some(match name {
        "env" => &["-u", "--unset", "-C", "--chdir", "-S", "--split-string", "-P", "-a", "--argv0"],
        "timeout" | "gtimeout" => &["-s", "--signal", "-k", "--kill-after"],
        "xcrun" => &["--sdk", "-sdk", "--toolchain", "-toolchain"],
        "sudo" => &["-u", "--user", "-g", "--group", "-h", "--host", "-p", "--prompt", "-D", "--chdir", "-C", "--close-from",
                    "-U", "--other-user", "-r", "--role", "-t", "--type", "-c", "--login-class", "-T", "--command-timeout",
                    "-R", "--chroot"],
        "nice" => &["-n", "--adjustment"],
        "caffeinate" => &["-t", "-w"],
        "time" => &["-f", "--format", "-o", "--output"],
        "nohup" | "setsid" => &[],
        "stdbuf" => &["-i", "--input", "-o", "--output", "-e", "--error"],
        "doas" => &["-u", "-C"],
        // Builtins that run their argument: `exec -a NAME prog`, `command -p prog`, `builtin source`.
        "exec" => &["-a"],
        "command" | "builtin" => &[],
        _ => return None,
    })
}

/// How many words a wrapper option uses (1 or 2), the way getopt reads it: a short cluster
/// (`-iu NAME`) takes the next word when its first value-taking letter is the last one; a long
/// option may be abbreviated (`--unse NAME`); `--opt=value` and `NAME=value` are one word.
pub fn opt_words(o: &str, opts: &[&str]) -> usize {
    if o.contains('=') || o == "-" || o == "--" || !o.starts_with('-') {
        return 1;
    }
    if opts.contains(&o) {
        return 2;  // also single-dash long options such as xcrun `-sdk`
    }
    if let Some(long) = o.strip_prefix("--") {
        let abbrev = opts.iter().any(|x| x.strip_prefix("--").is_some_and(|x| x.starts_with(long)));
        return if abbrev { 2 } else { 1 };
    }
    let letters: Vec<char> = o[1..].chars().collect();
    for (i, c) in letters.iter().enumerate() {
        let mut b = [0u8; 4];
        let short = format!("-{}", c.encode_utf8(&mut b));
        if opts.contains(&short.as_str()) {
            return if i + 1 == letters.len() { 2 } else { 1 };
        }
    }
    1
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    W(String),
    Op(&'static str),
    /// A heredoc body (index into [`Lexed::bodies`]); it follows the `<<` operator of its command.
    Body(usize),
}

/// Python's str.isspace() for one char (the Rust set lacks the four ASCII separators).
pub fn py_space(c: char) -> bool {
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
    unclosed: bool,
}

impl Lexer<'_> {
    /// The end of a `$(…)` or backtick substitution starting at i (its command goes to subs), or None.
    fn substitution(&mut self, i: usize) -> Option<usize> {
        let (s, n) = (self.s, self.s.len());
        if s[i] == '`' {
            let j = find(s, i + 1, "`").unwrap_or_else(|| {
                self.unclosed = true;
                n
            });
            self.subs.push(text(s, i + 1, j));
            return Some((j + 1).min(n));
        }
        if s[i] == '$' && i + 1 < n && s[i + 1] == '(' {
            let (mut depth, mut j) = (1i32, i + 2);
            while j < n && depth != 0 {
                if s[j] == '\'' || s[j] == '"' {  // a quoted ( or ) does not count
                    j = match find(s, j + 1, &s[j].to_string()) {
                        Some(k) => k + 1,
                        None => {
                            self.unclosed = true;
                            n
                        }
                    };
                    continue;
                }
                depth += match s[j] { '(' => 1, ')' => -1, _ => 0 };
                j += 1 + (s[j] == '\\') as usize;
            }
            let j = j.min(n);
            self.unclosed |= depth != 0;
            self.subs.push(text(s, i + 2, if depth == 0 { j - 1 } else { j }));
            return Some(j);
        }
        None
    }
}

/// What [`lex_full`] read.
pub struct Lexed {
    pub toks: Vec<Tok>,
    /// Commands inside `$(…)` and backticks the shell would run.
    pub subs: Vec<String>,
    /// Heredoc bodies, in the order of their `<<` operators.
    pub bodies: Vec<String>,
    /// For each body: the fd glued before its `<<` (`3<<`, `{fd}<<`); `None` is stdin.
    pub body_fd: Vec<Option<String>>,
    /// For each redirect operator token (index into `toks`) with a glued fd: that fd.
    pub op_fd: std::collections::HashMap<usize, String>,
    /// For each body: its tag was quoted (`<<'EOF'`), so the body does not expand.
    pub body_quoted: Vec<bool>,
    /// Word tokens (index into `toks`) holding a `$` or backtick outside single quotes: the outer
    /// shell expands them, so their final text is unknown here.
    pub expanding: std::collections::HashSet<usize>,
    /// A quote, `$(` or backtick was never closed.
    pub unclosed: bool,
}

/// (tokens, substitutions), see [`lex_full`].
pub fn lex(src: &str) -> (Vec<Tok>, Vec<String>) {
    let l = lex_full(src);
    (l.toks, l.subs)
}

/// Words with quotes removed and operators (or "\n"); substitutions are the commands inside
/// `$(…)` and backticks the shell would run (not in single quotes, nor in a heredoc with a quoted
/// tag). Comments are dropped; heredoc bodies go to `bodies`; an unclosed quote runs to the end
/// (and sets `unclosed`).
pub fn lex_full(src: &str) -> Lexed {
    let s: Vec<char> = src.chars().collect();
    let n = s.len();
    let mut lx = Lexer { s: &s, subs: Vec::new(), unclosed: false };
    let mut bodies: Vec<String> = Vec::new();
    let mut body_fd: Vec<Option<String>> = Vec::new();
    let mut op_fd = std::collections::HashMap::new();
    let mut body_quoted: Vec<bool> = Vec::new();
    let mut expanding = std::collections::HashSet::new();
    let mut exp = false;
    let mut toks: Vec<Tok> = Vec::new();
    let mut word: Option<String> = None;
    let mut heredocs: Vec<(String, bool, bool, usize)> = Vec::new();
    let mut i = 0;
    macro_rules! end {
        () => {
            if let Some(w) = word.take() {
                if std::mem::take(&mut exp) {
                    expanding.insert(toks.len());
                }
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
        exp |= c == '`' || c == '$';
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
            let j = find(&s, i + 1, "'").unwrap_or_else(|| {
                lx.unclosed = true;
                n
            });
            add!(text(&s, i + 1, j));
            i = j + 1;
        } else if c == '"' {
            let (mut j, mut buf) = (i + 1, String::new());
            while j < n && s[j] != '"' {
                if s[j] == '`' || s[j] == '$' {
                    exp = true;
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
            lx.unclosed |= j >= n;
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
            for (tag, tabs, quoted, idx) in std::mem::take(&mut heredocs) {
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
                bodies[idx] = text(&s, start, body_end);
                let mut k = start;
                while !quoted && k < body_end {
                    let e = if s[k] == '`' || s[k] == '$' { lx.substitution(k) } else { None };
                    k = e.unwrap_or(k + 1 + (s[k] == '\\') as usize);
                }
            }
        } else if word.is_none() && s[i..].starts_with(&['(', '(']) {  // arithmetic `(( n << 2 ))`
            // Not run as a command, but `$(…)`/backticks inside it still run.
            let mut k = i + 2;
            loop {
                if k >= n {
                    lx.unclosed = true;
                    break;
                }
                if s[k] == ')' && k + 1 < n && s[k + 1] == ')' {
                    k += 2;
                    break;
                }
                let e = if s[k] == '`' || s[k] == '$' { lx.substitution(k) } else { None };
                k = e.unwrap_or(k + 1 + (s[k] == '\\') as usize);
            }
            i = k.min(n);
        } else if ";&|()<>".contains(c) {
            // The fd glued to a redirect (`2>`, `3<<`, `{fd}<<`), if any.
            let mut fd: Option<String> = None;
            let is_fd = |w: &String| {
                let named = w.len() > 2 && w.starts_with('{') && w.ends_with('}')
                    && w[1..w.len() - 1].chars().all(|c| c.is_alphanumeric() || c == '_');
                named || (!w.is_empty() && w.chars().all(char::is_numeric))
            };
            if (c == '<' || c == '>') && word.as_ref().is_some_and(is_fd) {
                exp = false;
                fd = word.take();  // `2>`: a number glued to a redirect is the file descriptor, not a word (any Unicode numeric, a little wider than Python's isdigit, e.g. `½`: wider only blocks more)
            }
            end!();
            let op = *OPS.iter().find(|o| s[i..].starts_with(&o.chars().collect::<Vec<_>>())).expect("op");
            i += op.chars().count();
            if let Some(f) = &fd {
                op_fd.insert(toks.len(), f.clone());
            }
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
                    heredocs.push((tag, op == "<<-", quoted, bodies.len()));
                    toks.push(Tok::Body(bodies.len()));
                    bodies.push(String::new());
                    body_fd.push(fd.clone());
                    body_quoted.push(quoted);
                    i = p;
                }
            }
        } else {
            word.get_or_insert_with(String::new).push(c);
            i += 1;
        }
    }
    end!();
    Lexed { toks, subs: lx.subs, bodies, body_fd, op_fd, body_quoted, expanding, unclosed: lx.unclosed }
}

/// Word lists of the simple commands, redirects and their targets removed.
pub fn simple_commands(toks: &[Tok]) -> Vec<Vec<String>> {
    segments(toks).into_iter().map(|(w, _)| w).collect()
}

/// One simple command: its words, its heredoc bodies (indexes into [`Lexed::bodies`]) and its other
/// redirects as (operator, glued fd).
pub struct Seg {
    pub words: Vec<String>,
    pub bodies: Vec<usize>,
    pub redirs: Vec<(&'static str, Option<String>)>,
    /// Per word: the outer shell expands it (see [`Lexed::expanding`]).
    pub expands: Vec<bool>,
}

/// [`segments`] with the redirects kept, see [`Seg`].
pub fn segments_full(lx: &Lexed) -> Vec<Seg> {
    let new = || Seg { words: Vec::new(), bodies: Vec::new(), redirs: Vec::new(), expands: Vec::new() };
    let (mut out, mut seg, mut skip) = (Vec::new(), new(), false);
    for (i, t) in lx.toks.iter().enumerate() {
        match t {
            Tok::Op(o) if SEPARATORS.contains(o) => {
                out.push(std::mem::replace(&mut seg, new()));
                skip = false;
            }
            Tok::Op(o) => {
                skip = *o != "<<" && *o != "<<-";
                if skip {
                    seg.redirs.push((*o, lx.op_fd.get(&i).cloned()));
                }
            }
            Tok::Body(k) => seg.bodies.push(*k),
            Tok::W(_) if skip => skip = false,
            Tok::W(w) => {
                seg.words.push(w.clone());
                seg.expands.push(lx.expanding.contains(&i));
            }
        }
    }
    out.push(seg);
    out
}

/// The simple commands with the heredoc bodies (indexes into [`Lexed::bodies`]) fed to each.
pub fn segments(toks: &[Tok]) -> Vec<(Vec<String>, Vec<usize>)> {
    let (mut out, mut seg, mut bodies, mut skip) = (Vec::new(), Vec::new(), Vec::new(), false);
    for t in toks {
        match t {
            Tok::Op(o) if SEPARATORS.contains(o) => {
                out.push((std::mem::take(&mut seg), std::mem::take(&mut bodies)));
                skip = false;
            }
            // A redirect: its target is the next word (a heredoc tag was taken by lex()).
            Tok::Op(o) => skip = *o != "<<" && *o != "<<-",
            Tok::Body(k) => bodies.push(*k),
            Tok::W(_) if skip => skip = false,
            Tok::W(w) => seg.push(w.clone()),
        }
    }
    out.push((seg, bodies));
    out
}

pub fn basename(w: &str) -> &str {
    w.rsplit('/').next().unwrap_or(w)
}

/// `NAME=value` (Python `\w+=.*`).
pub fn is_assignment(w: &str) -> bool {
    match w.find('=') {
        Some(p) => p > 0 && w[..p].chars().all(|c| c.is_alphanumeric() || c == '_'),
        None => false,
    }
}

/// The program a simple command runs, after assignments, keywords and wrappers
/// (`CI=1 env timeout -s TERM 600 xcrun --sdk iphonesimulator xcodebuild …` -> `xcodebuild …`).
pub fn command_words(mut words: &[String]) -> &[String] {
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

