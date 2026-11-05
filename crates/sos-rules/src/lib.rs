//! `sos-rules`: the declarative rules engine behind `sos hook rules --file …`
//! (docs/plans/RUST_ARCHITECTURE_2026-11-05.md §2). A rules file says, per tool and event,
//! which commands or paths to block and what to do instead; the engine reads the hook payload,
//! parses shell commands with the same lexer as sim-guard, and fails closed on anything it
//! cannot read (payload, rules file, command) when a rule could apply.

pub mod cases;
pub mod check;
pub mod config;
pub mod paths;
pub mod shell;

use check::Runner;
use config::{Ev, Kind, Places, Rule, RuleSet, Test, COMMAND_TOOLS, PATH_TOOLS};
use regex::Regex;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

pub use config::load;

/// One thing the tool call would do.
pub enum Ask {
    Command(String),
    Path(String),
}

/// What the engine decided: the messages of the blocking rules (deduplicated; empty = allow)
/// and, with `explain`, how each rule was weighed.
#[derive(Default)]
pub struct Outcome {
    pub blocks: Vec<String>,
    pub trace: Vec<String>,
    /// The event was a post-tool event (the caller answers differently on Codex).
    pub post: bool,
}

impl Outcome {
    fn block(&mut self, m: String) {
        if !self.blocks.contains(&m) {
            self.blocks.push(m);
        }
    }
}

/// Every path named on any `*** <header>: <path>` line of a Codex apply_patch (Add/Update/Delete
/// File, Move to). Deliberately broad: a header we do not know still names a path to check.
pub fn patch_paths(patch: &str) -> Vec<String> {
    let ws = |c: char| c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c);
    patch.split('\n').filter_map(|line| {
        let s = line.trim_matches(ws);
        if !s.starts_with("***") {
            return None;
        }
        let path = s.split_once(':')?.1.trim_matches(ws);
        (!path.is_empty()).then(|| path.to_string())
    }).collect()
}

fn str_field<'a>(m: &'a Map<String, Value>, k: &str) -> Option<&'a str> {
    m.get(k).and_then(Value::as_str)
}

/// The actions of a tool call. `Ok(None)`: a tool the engine does not read. `Err`: a tool it
/// reads, with input it cannot read.
pub fn actions(tool: &str, input: Option<&Value>) -> Result<Option<Vec<Ask>>, String> {
    if !COMMAND_TOOLS.contains(&tool) && !PATH_TOOLS.contains(&tool) {
        return Ok(None);
    }
    let m = input.and_then(Value::as_object).ok_or("thiếu tool_input")?;
    let need = |k: &str| str_field(m, k).filter(|s| !s.is_empty()).ok_or(format!("tool_input.{k} thiếu hoặc không phải chuỗi"));
    let asks = match tool {
        "Bash" => vec![Ask::Command(str_field(m, "command").ok_or("tool_input.command thiếu hoặc không phải chuỗi")?.to_string())],
        "NotebookEdit" => {
            let p = str_field(m, "notebook_path").filter(|s| !s.is_empty()).map(Ok).unwrap_or_else(|| need("file_path"))?;
            vec![Ask::Path(p.to_string())]
        }
        "Glob" => {
            let pat = need("pattern")?;
            let p = match str_field(m, "path").filter(|s| !s.is_empty()) {
                Some(dir) if !Path::new(pat).is_absolute() && !pat.starts_with('~') => format!("{}/{pat}", dir.trim_end_matches(['/', '\\'])),
                _ => pat.to_string(),
            };
            vec![Ask::Path(p)]
        }
        "apply_patch" => {
            let patch = need("command")?;
            let ps = patch_paths(patch);
            if ps.is_empty() {
                return Err("không thấy đường dẫn nào trong apply_patch (gửi lại dạng '*** Begin Patch' chuẩn)".into());
            }
            ps.into_iter().map(Ask::Path).collect()
        }
        _ => vec![Ask::Path(need("file_path")?.to_string())],
    };
    Ok(Some(asks))
}

fn rule_tools(r: &Rule) -> Vec<&str> {
    if !r.tools.is_empty() {
        return r.tools.iter().map(String::as_str).collect();
    }
    match r.kind {
        Kind::Command(_) => COMMAND_TOOLS.to_vec(),
        Kind::Paths(_) => PATH_TOOLS.to_vec(),
    }
}

/// Program name for matching: last component by either separator, `.exe` dropped.
fn prog_name(w: &str) -> &str {
    let b = paths::basename_of(w);
    match b.len().checked_sub(4) {
        Some(k) if b.is_char_boundary(k) && b[k..].eq_ignore_ascii_case(".exe") => &b[..k],
        _ => b,
    }
}

fn strip_prefix_words(mut w: &[String]) -> &[String] {
    while let Some(f) = w.first() {
        if shell::is_assignment(f) || shell::PREFIX.contains(&f.as_str()) {
            w = &w[1..];
        } else {
            break;
        }
    }
    w
}

/// The programs a simple command runs: the command itself, then what each wrapper (`env`,
/// `sudo`, `timeout 5`, `nice`, `xcrun`…) runs in turn.
fn layers(seg: &[String]) -> Result<Vec<Vec<String>>, String> {
    let mut cur = strip_prefix_words(seg);
    let mut out = Vec::new();
    while let Some(first) = cur.first() {
        let b = prog_name(first);
        if b.contains('$') || b.contains('`') {
            return Err(format!("tên chương trình `{first}` là biến"));
        }
        if b == "eval" {
            return Err("`eval`".into());
        }
        // Runs another file's commands in this shell (spec §2: refuse rather than guess).
        if b == "source" || b == "." {
            return Err(format!("`{b}` chạy lệnh từ file khác"));
        }
        out.push(cur.to_vec());
        let Some(opts) = shell::wrapper_opts(b) else { break };
        let mut rest = &cur[1..];
        while let Some(o) = rest.first() {
            if !(o.starts_with('-') || shell::is_assignment(o)) {
                break;
            }
            // `env -S STR` splits STR into a new command line: refuse it in any spelling (`-vS`,
            // `-S…`, `--split-string`, `--spl`); a stray `S` inside a cluster also refuses (review 1d round 3).
            let long = o.strip_prefix("--").map(|l| l.split('=').next().unwrap_or(l));
            let split = match long {
                Some(l) => !l.is_empty() && "split-string".starts_with(l),
                None => o.starts_with('-') && o[1..].contains('S'),
            };
            if b == "env" && split {
                return Err("`env -S`".into());
            }
            rest = &rest[shell::opt_words(o, opts).min(rest.len())..];
        }
        if (b == "timeout" || b == "gtimeout") && !rest.is_empty() {
            rest = &rest[1..];
        }
        cur = strip_prefix_words(rest);
    }
    Ok(out)
}

/// `/dev/fd/N`, `/proc/…/fd/N`: a script read from an open file descriptor.
fn is_fd_path(w: &str) -> bool {
    w.starts_with("/dev/fd/") || (w.starts_with("/proc/") && w.contains("/fd/"))
}

/// Script names that make a shell read its commands from stdin.
const STDIN_SCRIPTS: &[&str] = &["-", "/dev/stdin", "/dev/fd/0", "/proc/self/fd/0"];

/// Every simple command the shell would run for `cmd`, as wrapper layers, following `sh -c`
/// strings, `$(…)`/backticks and heredocs fed to a shell. Err: something the engine cannot
/// follow (unclosed quote, `eval`, a variable as program, a shell reading stdin, nesting > 3).
pub fn parse_command(cmd: &str) -> Result<Vec<Vec<Vec<String>>>, String> {
    parse_at(cmd, 0)
}

fn parse_at(cmd: &str, depth: u32) -> Result<Vec<Vec<Vec<String>>>, String> {
    if depth > 3 {
        return Err("lồng lệnh quá 3 tầng".into());
    }
    let lx = shell::lex_full(cmd);
    if lx.unclosed {
        return Err("nháy, `$(` hoặc backtick chưa đóng".into());
    }
    let mut out = Vec::new();
    let mut inner: Vec<String> = lx.subs.clone();
    for shell::Seg { words: seg, bodies, redirs, expands } in shell::segments_full(&lx) {
        let ls = layers(&seg)?;
        for layer in &ls {
            // Layers are suffixes of the segment: word k of a layer is word `base + k` of `seg`.
            let base = seg.len() - layer.len();
            let b = prog_name(&layer[0]);
            if !shell::SHELLS.contains(&b) {
                continue;
            }
            let (mut k, mut has_c, mut has_s) = (1, false, false);
            while k < layer.len() && layer[k].chars().next().is_some_and(|c| c == '-' || c == '+') && layer[k] != "--" {
                let o = &layer[k];
                let short = !o.starts_with("--");
                // `+c` runs its string too (bash, sh, zsh): read `c`/`s` in `-` and `+` clusters.
                has_c = has_c || (short && o.contains('c'));
                has_s = has_s || (short && o.contains('s'));
                // `-o NAME`, and every `o`/`O` ending a cluster (`-eo pipefail`), takes the next word.
                // An `o` with letters after it (`zsh -oerrexit`) is read differently by each shell:
                // refuse (review 1d round 5).
                let value_opts = o[1..].trim_end_matches(['o', 'O']);
                if short && value_opts.contains(['o', 'O']) {
                    return Err(format!("`{b} {o}`: cờ shell có giá trị viết liền"));
                }
                k += if !short {
                    if shell::SHELL_ARG_OPTS.contains(&o.as_str()) { 2 } else { 1 }
                } else {
                    o.len() - value_opts.len()
                };
            }
            if k < layer.len() && layer[k] == "--" {
                k += 1;
            }
            // `bash /dev/stdin <<EOF`, `bash - <<EOF`: the script is stdin, like `bash -s`.
            let stdin_script = k < layer.len() && STDIN_SCRIPTS.contains(&layer[k].as_str());
            // `bash /dev/fd/3 3<<EOF`: the script is another fd; follow every heredoc of the command.
            // With `-s` the words after the options are arguments; the script is stdin (round 5).
            let fd_script = k < layer.len() && !has_s && !stdin_script && is_fd_path(&layer[k]);
            if has_c {
                if k < layer.len() {
                    // `bash -c ": $X"`: the outer shell expands it first, so the code run is unknown.
                    if expands[base + k] {
                        return Err(format!("`{b} -c` với chuỗi có biến hay `$(…)` của shell ngoài"));
                    }
                    inner.push(layer[k].clone());
                }
            } else if k >= layer.len() || has_s || stdin_script || fd_script {
                // The script comes from fd 0 (stdin) or fd N (`/dev/fd/N`): follow the heredocs on that
                // fd, and only when no other redirect could feed it (`<<< …`, `< f`, `3<&0`): review 1d
                // rounds 3 and 4.
                let fd = if fd_script { layer[k].rsplit('/').next().unwrap_or("").to_string() } else { "0".to_string() };
                let on_fd = |f: &Option<String>| f.as_deref().unwrap_or("0") == fd;
                let fed: Vec<usize> = bodies.iter().copied().filter(|&i| on_fd(&lx.body_fd[i])).collect();
                let other = redirs.iter().any(|(op, f)| {
                    let input = matches!(*op, "<" | "<<<" | "<&" | "<>");
                    (input && on_fd(f)) || f.as_deref() == Some(fd.as_str()) || f.as_deref().is_some_and(|f| f.starts_with('{'))
                });
                if fed.is_empty() || other {
                    let from = if fd_script { format!("`{}`", layer[k]) } else { "stdin".into() };
                    return Err(format!("`{b}` đọc lệnh từ {from}"));
                }
                // An unquoted tag expands `$X`, `$(…)`, backticks and removes `\` escapes before the
                // shell reads the body (`s\\⏎sh h` becomes `ssh h`, review 1d round 6).
                if fed.iter().any(|&i| !lx.body_quoted[i] && lx.bodies[i].contains(['$', '`', '\\'])) {
                    return Err(format!("`{b}` đọc heredoc không quote có biến, `$(…)` hay `\\`"));
                }
                inner.extend(fed.iter().map(|&i| lx.bodies[i].clone()));
            }
        }
        if !ls.is_empty() {
            out.push(ls);
        }
    }
    for s in inner {
        out.extend(parse_at(&s, depth + 1)?);
    }
    Ok(out)
}

/// What a firing rule knows, for `{…}` placeholders.
#[derive(Default)]
struct Ctx {
    path: Option<String>,
    program: Option<String>,
    args: Vec<String>,
    when_value: Option<String>,
    check_output: Option<String>,
    check_matched: Option<String>,
}

impl Ctx {
    fn get(&self, name: &str) -> Option<String> {
        match name {
            "path" => self.path.clone(),
            "program" => self.program.clone(),
            "when.value" => self.when_value.clone(),
            "check.output" => self.check_output.clone(),
            "check.matched" => self.check_matched.clone(),
            n => n.strip_prefix("arg.").and_then(|k| k.parse::<usize>().ok()).and_then(|k| self.args.get(k).cloned()),
        }
    }

    fn subst(&self, text: &str, strict: bool) -> Result<String, String> {
        let re = Regex::new(r"\{([A-Za-z_][A-Za-z0-9_.]*)\}").expect("static");
        let mut err = None;
        let out = re.replace_all(text, |c: &regex::Captures| match self.get(&c[1]) {
            Some(v) => v,
            None => {
                if strict {
                    err = Some(format!("không có giá trị cho `{{{}}}`", &c[1]));
                }
                String::new()
            }
        }).into_owned();
        err.map_or(Ok(out), Err)
    }
}

fn action_path(p: &str, places: &Places, cwd: &Path) -> Result<PathBuf, String> {
    paths::normalize(&paths::expand(p, places.home.as_deref(), cwd)?)
}

fn test_arg(t: &Test, a: &str, places: &Places, cwd: &Path) -> Result<bool, String> {
    Ok(match t {
        Test::Eq(v) => a == v,
        Test::OneOf(v) => v.iter().any(|x| x == a),
        Test::Regex(r) => r.is_match(a),
        Test::BasenameRegex(r) => r.is_match(paths::basename_of(a)),
        Test::Path(set) => {
            if a.contains('$') || a.contains('`') {
                return Err(format!("đối số `{a}` là biến, không biết trỏ đi đâu"));
            }
            paths::in_set(&action_path(a, places, cwd)?, set)
        }
    })
}

fn alt_matches(alt: &config::Alt, layer: &[String], places: &Places, cwd: &Path) -> Result<bool, String> {
    let name = prog_name(&layer[0]);
    if !alt.programs.iter().any(|g| g.is_match(name)) {
        return Ok(false);
    }
    let args = &layer[1..];
    // Cheap string tests first, so a path test (which may refuse a `$VAR`) runs only when the
    // rest already matched.
    let mut conds: Vec<&config::ArgCond> = alt.args.iter().collect();
    conds.sort_by_key(|c| matches!(c.test, Test::Path(_)));
    for c in conds {
        let ok = match c.sel {
            config::Sel::Pos(n) => match args.get(n) {
                Some(a) => test_arg(&c.test, a, places, cwd)?,
                None => false,
            },
            config::Sel::Any => {
                let mut any = false;
                for a in args {
                    if test_arg(&c.test, a, places, cwd)? {
                        any = true;
                        break;
                    }
                }
                any
            }
        };
        if !ok {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The role value, if a source exists (first existing source decides).
fn role_value(role: &config::Role) -> Result<Option<String>, String> {
    for s in &role.sources {
        match s {
            config::Source::File { path, trim } => match std::fs::read(path) {
                Ok(b) => {
                    // Strict: a lossy read would turn a damaged marker into a near-miss value.
                    let v = String::from_utf8(b).map_err(|_| format!("không đọc được {} (không phải UTF-8)", path.display()))?;
                    return Ok(Some(if *trim { v.trim().to_string() } else { v }));
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(format!("không đọc được {} ({e})", path.display())),
            },
            config::Source::FileExists { path, value } => match std::fs::symlink_metadata(path) {
                Ok(_) => return Ok(Some(value.clone())),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(format!("không đọc được {} ({e})", path.display())),
            },
        }
    }
    Ok(None)
}

const MAX_CONTENT: u64 = 32 << 20;

/// Conditions after the command/path match: role, then check. `Ok(Some(message))` = block.
fn finish(rule: &Rule, mut ctx: Ctx, places: &Places, runner: &dyn Runner, trace: &mut Vec<String>) -> Result<Option<String>, String> {
    if let Some(role) = &rule.role {
        match role_value(role)? {
            Some(v) if role.one_of.contains(&v) => {
                trace.push(format!("  vai «{v}» thuộc one_of"));
                ctx.when_value = Some(v);
            }
            Some(v) => {
                trace.push(format!("  vai «{v}» không thuộc one_of → không chặn"));
                return Ok(None);
            }
            None => {
                trace.push("  không có nguồn vai nào → không chặn".into());
                return Ok(None);
            }
        }
    }
    if let Some(c) = &rule.check {
        let args = c.args.iter().map(|a| ctx.subst(a, true)).collect::<Result<Vec<_>, _>>();
        // The substituted argv must read as the same programs as the template, with no shell `-c`:
        // a value must never become a program or code (review 1d round 4). Blocks even with
        // `on_error = allow`, since this is input shaping the check, not the check failing.
        if let Ok(a) = &args {
            check_shape(&c.program, &c.args, a)?;
        }
        let ran = args.and_then(|args| {
            let prog = check::resolve(&c.program, places.home.as_deref(), &rule.root)?;
            trace.push(format!("  chạy check: {} {}", prog.display(), args.join(" ")));
            runner.run(&prog, &args, &rule.root, c.timeout)
        });
        match ran {
            Err(e) if c.allow_on_error => {
                trace.push(format!("  check hỏng ({e}); on_error = allow → không chặn"));
                return Ok(None);
            }
            Err(e) => return Err(format!("lệnh kiểm không chạy được ({e}), nên chặn cho chắc (on_error = block)")),
            Ok(r) => {
                let fires = match &c.fires_on {
                    config::Fires::Success => r.success,
                    config::Fires::Failure => !r.success,
                    // Over the whole stdout (write `(?m)` for line anchors, as the architecture
                    // says); `{check.matched}` is the lines the regex matches on their own.
                    config::Fires::Output(re) => {
                        let m: Vec<&str> = r.stdout.lines().filter(|l| re.is_match(l)).collect();
                        ctx.check_matched = Some(m.join("\n"));
                        re.is_match(&r.stdout)
                    }
                };
                trace.push(format!("  check {} → {}", if r.success { "thoát 0" } else { "thoát khác 0" },
                                   if fires { "chặn" } else { "không chặn" }));
                ctx.check_output = Some(r.stdout.trim_end().to_string());
                if !fires {
                    return Ok(None);
                }
            }
        }
    }
    Ok(Some(ctx.subst(&rule.message, false)?))
}

fn event_of(p: &Map<String, Value>) -> Option<Ev> {
    match p.get("hook_event_name") {
        None => Some(Ev::Pre),
        Some(Value::String(s)) => match s.as_str() {
            "PreToolUse" => Some(Ev::Pre),
            "PostToolUse" | "PostToolUseFailure" => Some(Ev::Post),
            _ => None,
        },
        _ => None,
    }
}

/// Decide one hook payload.
/// See the call in the check step: same program list before and after substitution, no shell `-c`.
fn check_shape(program: &str, template: &[String], args: &[String]) -> Result<(), String> {
    let progs = |a: &[String]| -> Result<Vec<String>, String> {
        let words: Vec<String> = std::iter::once(program.to_string()).chain(a.iter().cloned()).collect();
        Ok(layers(&words)?.into_iter().map(|l| l[0].clone()).collect())
    };
    let words: Vec<String> = std::iter::once(program.to_string()).chain(args.iter().cloned()).collect();
    let after = layers(&words).map_err(|e| format!("giá trị thay vào làm lệnh kiểm không đọc được ({e})"))?;
    let shell_c = after.iter().any(|l| {
        shell::SHELLS.contains(&prog_name(&l[0]))
            && l[1..].iter().any(|a| a.starts_with('-') && !a.starts_with("--") && a.contains('c'))
    });
    let same = progs(template).is_ok_and(|p| p == after.iter().map(|l| l[0].clone()).collect::<Vec<_>>());
    if shell_c || !same {
        return Err("giá trị thay vào đổi chương trình hoặc thành mã shell trong lệnh kiểm".into());
    }
    Ok(())
}

pub fn decide(set: &RuleSet, payload: &Value, places: &Places, runner: &dyn Runner) -> Outcome {
    let mut out = Outcome::default();
    let Some(p) = payload.as_object() else {
        out.block("Chặn: payload hook không phải JSON object, không đọc được lệnh sắp chạy.".into());
        return out;
    };
    // Present but not a string (null included) is a malformed payload; only a missing field
    // means the agent sends no event (then it is a pre-tool hook).
    if let Some(v) = p.get("hook_event_name").filter(|v| !v.is_string()) {
        if !set.rules.is_empty() {
            out.block(format!("Chặn: hook_event_name = {v} không phải chuỗi, không biết sự kiện nào."));
            return out;
        }
    }
    let Some(ev) = event_of(p) else {
        out.trace.push(format!("sự kiện {} không thuộc luật nào → cho qua", p.get("hook_event_name").unwrap_or(&Value::Null)));
        return out;
    };
    out.post = ev == Ev::Post;
    let of_event: Vec<&Rule> = set.rules.iter().filter(|r| r.event == ev).collect();
    let Some(tool) = str_field(p, "tool_name") else {
        if !of_event.is_empty() {
            out.block("Chặn: payload hook thiếu tool_name, không biết công cụ nào sắp chạy.".into());
        }
        return out;
    };
    let rules: Vec<&Rule> = of_event.into_iter().filter(|r| rule_tools(r).contains(&tool)).collect();
    if rules.is_empty() {
        out.trace.push(format!("không luật nào áp cho {tool} ở sự kiện này → cho qua"));
        return out;
    }
    let asks = match actions(tool, p.get("tool_input")) {
        Ok(Some(a)) => a,
        Ok(None) => return out,
        Err(e) => {
            out.block(format!("Chặn: payload {tool} sai ({e}); luật {} không kiểm được.",
                              rules.iter().map(|r| format!("`{}`", r.id)).collect::<Vec<_>>().join(", ")));
            return out;
        }
    };
    let cwd = match str_field(p, "cwd").filter(|c| !c.is_empty()) {
        Some(c) => places.cwd.join(c),
        None => places.cwd.clone(),
    };
    for ask in &asks {
        match ask {
            Ask::Command(cmd) => command_ask(&rules, cmd, places, &cwd, runner, &mut out),
            Ask::Path(raw) => path_ask(&rules, raw, places, &cwd, runner, &mut out),
        }
    }
    out
}

fn command_ask(rules: &[&Rule], cmd: &str, places: &Places, cwd: &Path, runner: &dyn Runner, out: &mut Outcome) {
    let live: Vec<(&Rule, &config::CommandCond)> = rules.iter().filter_map(|r| match &r.kind {
        Kind::Command(c) => {
            let raw_ok = c.raw.as_ref().is_none_or(|re| re.is_match(cmd));
            let not_raw_ok = c.not_raw.as_ref().is_none_or(|re| !re.is_match(cmd));
            if !(raw_ok && not_raw_ok) {
                out.trace.push(format!("luật `{}`: raw/not_raw loại lệnh này", r.id));
            }
            (raw_ok && not_raw_ok).then_some((*r, c))
        }
        Kind::Paths(_) => None,
    }).collect();
    if live.is_empty() {
        return;
    }
    let parsed = match parse_command(cmd) {
        Ok(p) => p,
        Err(e) => {
            out.block(format!("Chặn: không phân tích được lệnh ({e}); luật {} cần đọc được lệnh. Viết lệnh đơn giản hơn \
                               (bỏ eval / biến làm tên chương trình / pipe vào shell).",
                              live.iter().map(|(r, _)| format!("`{}`", r.id)).collect::<Vec<_>>().join(", ")));
            return;
        }
    };
    for (rule, cond) in live {
        for layers in &parsed {
            'layer: for layer in layers {
                for alt in &cond.alts {
                    match alt_matches(alt, layer, places, cwd) {
                        Ok(false) => continue,
                        Ok(true) => {
                            out.trace.push(format!("luật `{}`: khớp `{}`", rule.id, layer.join(" ")));
                            let ctx = Ctx { program: Some(layer[0].clone()), args: layer[1..].to_vec(), ..Ctx::default() };
                            match finish(rule, ctx, places, runner, &mut out.trace) {
                                Ok(Some(m)) => {
                                    out.block(m);
                                    return;
                                }
                                // This program passed; later commands in the line still count.
                                Ok(None) => continue 'layer,
                                Err(e) => {
                                    out.block(format!("Chặn (luật `{}`): {e}", rule.id));
                                    return;
                                }
                            }
                        }
                        Err(e) => {
                            out.block(format!("Chặn (luật `{}`): {e}; viết đường dẫn thật thay cho biến.", rule.id));
                            return;
                        }
                    }
                }
            }
        }
        out.trace.push(format!("luật `{}`: không lệnh nào khớp", rule.id));
    }
}

fn path_ask(rules: &[&Rule], raw: &str, places: &Places, cwd: &Path, runner: &dyn Runner, out: &mut Outcome) {
    let live: Vec<(&Rule, &config::PathsCond)> = rules.iter().filter_map(|r| match &r.kind {
        Kind::Paths(c) => Some((*r, c)),
        Kind::Command(_) => None,
    }).collect();
    if live.is_empty() {
        return;
    }
    let p = match action_path(raw, places, cwd) {
        Ok(p) => p,
        Err(e) => {
            out.block(format!("Chặn: không đọc được đường dẫn `{raw}` ({e}); luật {} cần biết file thật.",
                              live.iter().map(|(r, _)| format!("`{}`", r.id)).collect::<Vec<_>>().join(", ")));
            return;
        }
    };
    for (rule, cond) in live {
        if !paths::in_set(&p, &cond.set) {
            out.trace.push(format!("luật `{}`: {} không khớp", rule.id, p.display()));
            continue;
        }
        if cond.exists && !p.is_file() {
            out.trace.push(format!("luật `{}`: {} chưa có → bỏ qua", rule.id, p.display()));
            continue;
        }
        if let Some(re) = &cond.content {
            let text = match std::fs::File::open(&p) {
                Ok(f) => {
                    use std::io::Read;
                    let mut b = Vec::new();
                    if let Err(e) = f.take(MAX_CONTENT + 1).read_to_end(&mut b) {
                        out.block(format!("Chặn (luật `{}`): không đọc được {} ({e})", rule.id, p.display()));
                        return;
                    }
                    if b.len() as u64 > MAX_CONTENT {
                        out.block(format!("Chặn (luật `{}`): {} lớn hơn {} MiB, không kiểm hết được `content`",
                                          rule.id, p.display(), MAX_CONTENT >> 20));
                        return;
                    }
                    String::from_utf8_lossy(&b).into_owned()
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
                Err(e) => {
                    out.block(format!("Chặn (luật `{}`): không đọc được {} ({e})", rule.id, p.display()));
                    return;
                }
            };
            if !re.is_match(&text) {
                out.trace.push(format!("luật `{}`: nội dung không khớp `content`", rule.id));
                continue;
            }
        }
        out.trace.push(format!("luật `{}`: khớp {}", rule.id, p.display()));
        let ctx = Ctx { path: Some(p.display().to_string()), ..Ctx::default() };
        match finish(rule, ctx, places, runner, &mut out.trace) {
            Ok(Some(m)) => {
                out.block(m);
                return;
            }
            Ok(None) => continue,
            Err(e) => {
                out.block(format!("Chặn (luật `{}`): {e}", rule.id));
                return;
            }
        }
    }
}

/// The block message for a rules file that cannot be loaded.
pub fn load_error_message(e: &str, files: &[PathBuf]) -> String {
    let f = files.first().map(|f| f.display().to_string()).unwrap_or_else(|| "<file>".into());
    format!("Chặn: file luật hỏng nên không kiểm được gì: {e}\nSửa file luật rồi kiểm: `sos hook rules --file {f} --lint`")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn progs(cmd: &str) -> Vec<String> {
        parse_command(cmd).unwrap().iter().flat_map(|ls| ls.iter().map(|l| l[0].clone())).collect()
    }

    #[test]
    fn parse_follows_wrappers_and_shells() {
        assert_eq!(progs("FOO=1 sudo -u x env A=b timeout 5 npm i"), ["sudo", "env", "timeout", "npm"]);
        assert_eq!(progs("bash -lc 'cd x && ssh h'"), ["bash", "cd", "ssh"]);
        assert_eq!(progs("echo $(gh pr merge 1)"), ["echo", "gh"]);
        assert_eq!(progs("bash <<EOF\nssh h\nEOF"), ["bash", "ssh"]);
        assert_eq!(progs("cat <<EOF\nssh h\nEOF"), ["cat"]);
        assert_eq!(progs("echo npm install"), ["echo"]);
        assert_eq!(progs("bash script.sh"), ["bash"]);
        assert_eq!(progs("command -p ssh h"), ["command", "ssh"]);
        assert_eq!(progs("exec -a audit ssh h"), ["exec", "ssh"]);
        assert!(progs("(( $(ssh h) + 1 ))").contains(&"ssh".to_string()));
        assert_eq!(progs("time -p ssh h"), ["time", "ssh"]);
        assert_eq!(progs("/usr/bin/nohup ssh h"), ["/usr/bin/nohup", "ssh"]);
        assert!(progs("bash /dev/stdin <<'EOF'\nssh h\nEOF").contains(&"ssh".to_string()));
        assert!(progs("bash - <<EOF\nssh h\nEOF").contains(&"ssh".to_string()));
        // Review 1d round 3: getopt clusters and abbreviations, fd scripts, stdin heredoc on fd 0.
        assert_eq!(progs("sudo -iu root ssh h"), ["sudo", "ssh"]);
        assert_eq!(progs("env --unse X ssh h"), ["env", "ssh"]);
        assert_eq!(progs("timeout -vk 5 10 ssh h"), ["timeout", "ssh"]);
        assert_eq!(progs("sudo --user root ssh h"), ["sudo", "ssh"]);
        assert!(progs("bash /dev/fd/3 3<<'EOF'\nssh h\nEOF").contains(&"ssh".to_string()));
        assert!(progs("bash 0<<'EOF'\nssh h\nEOF").contains(&"ssh".to_string()));
        assert_eq!(progs("cat 3<<'EOF'\nssh h\nEOF"), ["cat"]);
        // Review 1d round 4: `o` in a shell cluster takes a value; `-c` after it is still read.
        assert_eq!(progs("bash -eo pipefail -c 'ssh h'"), ["bash", "ssh"]);
        assert_eq!(progs("bash -o pipefail -ec 'ssh h'"), ["bash", "ssh"]);
        assert_eq!(progs("bash +o posix -c 'ssh h'"), ["bash", "ssh"]);
        assert!(progs("bash <<'EOF' 2>&1\nssh h\nEOF").contains(&"ssh".to_string()));
        // Review 1d round 5: `+c`, quoted heredoc keeps `$X` as data, `-c` with a single-quoted `$X`.
        assert_eq!(progs("bash +c 'ssh h'"), ["bash", "ssh"]);
        assert!(progs("bash <<'EOF'\n: $TAIL\nssh h\nEOF").contains(&"ssh".to_string()));
        assert_eq!(progs("bash -c ': $X; ssh h'"), ["bash", ":", "ssh"]);
    }

    #[test]
    fn parse_refuses_what_it_cannot_follow() {
        for c in ["echo 'x", "eval \"$X\"", "$GH pr merge 1", "echo x | sh", "env -S 'gh pr merge 1'", "source /dev/stdin", ". ./x.sh", "(( 1 + 2",
                  "builtin source /dev/stdin <<'EOF'\nssh h\nEOF", "bash /dev/stdin", "builtin eval x",
                  "env -vS 'ssh h'", "env --spl='ssh h'", "env -iS 'ssh h'",
                  "printf 'ssh h' | bash 3<<'EOF'\n:\nEOF", "printf 'ssh h' | bash {fd}<<'EOF'\n:\nEOF", "bash /dev/fd/3",
                  "bash <<'EOF' <<< 'ssh h'\n:\nEOF", "bash <<'EOF' < f\n:\nEOF",
                  "printf 'ssh h' | bash /dev/fd/3 3<&0 <<'EOF'\n:\nEOF", "bash /dev/fd/3 3<<'EOF' 3<f\n:\nEOF",
                  "bash /dev/fd/3 <<'EOF'\n:\nEOF",
                  "printf 'ssh h' | bash -s /dev/fd/3 3<<'EOF'\n:\nEOF", "zsh -oerrexit -c 'ssh h'", "bash -eO extglob -oc x",
                  "TAIL='; ssh h'\nbash <<EOF\n: $TAIL\nEOF", "bash <<EOF\n: `printf x`\nEOF", "bash <<EOF\ns\\\\\nsh h\nEOF",
                  "bash -c \": $TAIL\"", "bash -c \"$(printf 'ssh h')\"",
                  "bash -c \"bash -c \\\"bash -c 'bash -c \\\\\\\"ls\\\\\\\"'\\\"\""] {
            assert!(parse_command(c).is_err(), "{c}");
        }
        assert!(parse_command("\"$CLAUDE_PROJECT_DIR/scripts/x.sh\" a").is_ok());
    }

    #[test]
    fn patch_paths_reads_every_header() {
        let p = "*** Begin Patch\n*** Add File: a.txt\n+hi\n\u{a0}*** Update File: dir\u{2028}/.env\n*** Move to: b\n*** End Patch";
        assert_eq!(patch_paths(p), vec!["a.txt", "dir\u{2028}/.env", "b"]);
        assert!(patch_paths("garbage").is_empty());
    }

    #[test]
    fn prog_name_drops_exe() {
        assert_eq!(prog_name(r"C:\bin\gh.EXE"), "gh");
        assert_eq!(prog_name("/usr/bin/python3.12"), "python3.12");
    }
}
