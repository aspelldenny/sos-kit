//! The rules file (TOML, `version = 1`) and its lint. Everything that can be wrong in a file is
//! found here, at load, so a broken file blocks every action instead of half-matching
//! (docs/plans/RUST_ARCHITECTURE_2026-11-05.md §2, «Đóng khi hỏng»).

use crate::paths;
use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Tools whose input the engine knows how to read. A rule naming another tool would never match,
/// which reads like protection that is not there, so lint refuses it.
pub const COMMAND_TOOLS: &[&str] = &["Bash"];
pub const PATH_TOOLS: &[&str] = &["Edit", "Write", "MultiEdit", "NotebookEdit", "Read", "Glob", "apply_patch"];
/// A check must end well inside the hook's 30 s deadline (sos-cli `GUARD_DEADLINE`).
pub const MAX_CHECK_SECS: u64 = 25;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    version: u32,
    root: Option<String>,
    #[serde(default)]
    rule: Vec<RawRule>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRule {
    id: String,
    event: Option<String>,
    #[serde(default)]
    tools: Vec<String>,
    command: Option<RawCommand>,
    paths: Option<RawPaths>,
    when: Option<RawWhen>,
    check: Option<RawCheck>,
    decision: String,
    message: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCommand {
    #[serde(default)]
    program: Vec<String>,
    #[serde(default)]
    args: Vec<RawArg>,
    #[serde(default)]
    or: Vec<RawAlt>,
    raw: Option<String>,
    not_raw: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAlt {
    program: Vec<String>,
    #[serde(default)]
    args: Vec<RawArg>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawTest {
    eq: Option<String>,
    one_of: Option<Vec<String>>,
    regex: Option<String>,
    basename_regex: Option<String>,
    path: Option<RawPathSet>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawArg {
    pos: Option<usize>,
    any: Option<RawTest>,
    eq: Option<String>,
    one_of: Option<Vec<String>>,
    regex: Option<String>,
    basename_regex: Option<String>,
    path: Option<RawPathSet>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPathSet {
    #[serde(default)]
    outside: Vec<String>,
    #[serde(default)]
    glob: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPaths {
    #[serde(default)]
    outside: Vec<String>,
    #[serde(default)]
    glob: Vec<String>,
    exists: Option<bool>,
    content: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWhen {
    role: Option<RawRole>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRole {
    sources: Vec<RawSource>,
    one_of: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSource {
    file: Option<String>,
    trim: Option<bool>,
    file_exists: Option<String>,
    value: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCheck {
    program: String,
    #[serde(default)]
    args: Vec<String>,
    timeout_secs: Option<u64>,
    fires_on: RawFires,
    on_error: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawFires {
    Word(String),
    Output(RawOutput),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawOutput {
    output: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ev {
    Pre,
    Post,
}

pub struct Rule {
    pub id: String,
    pub event: Ev,
    pub tools: Vec<String>,
    pub kind: Kind,
    pub role: Option<Role>,
    pub check: Option<Check>,
    pub message: String,
    /// Base for relative paths in this rule (the file's `root`, else the hook's working directory).
    pub root: PathBuf,
}

pub enum Kind {
    Command(CommandCond),
    Paths(PathsCond),
}

pub struct CommandCond {
    pub alts: Vec<Alt>,
    pub raw: Option<Regex>,
    pub not_raw: Option<Regex>,
}

pub struct Alt {
    /// Basename globs (`python*`).
    pub programs: Vec<Regex>,
    pub args: Vec<ArgCond>,
}

pub enum Sel {
    Pos(usize),
    Any,
}

pub struct ArgCond {
    pub sel: Sel,
    pub test: Test,
}

pub enum Test {
    Eq(String),
    OneOf(Vec<String>),
    Regex(Regex),
    BasenameRegex(Regex),
    Path(PathSet),
}

/// A path matches when it is outside every `outside` root (if any are given) and matches some
/// glob (if any are given).
pub struct PathSet {
    pub outside: Vec<PathBuf>,
    pub globs: Vec<Regex>,
}

pub struct PathsCond {
    pub set: PathSet,
    pub exists: bool,
    pub content: Option<Regex>,
}

pub enum Source {
    File { path: PathBuf, trim: bool },
    FileExists { path: PathBuf, value: String },
}

pub struct Role {
    pub sources: Vec<Source>,
    pub one_of: Vec<String>,
}

pub enum Fires {
    Success,
    Failure,
    Output(Regex),
}

pub struct Check {
    pub program: String,
    pub args: Vec<String>,
    pub timeout: Duration,
    pub fires_on: Fires,
    /// `on_error = "allow"`: a check that could not run lets the action through.
    pub allow_on_error: bool,
}

pub struct RuleSet {
    pub rules: Vec<Rule>,
    pub warnings: Vec<String>,
}

/// Where `~` and relative paths point while loading and matching.
#[derive(Clone)]
pub struct Places {
    pub home: Option<PathBuf>,
    /// The hook's working directory: base for a file without `root`, and for relative action
    /// paths when the payload has no `cwd`.
    pub cwd: PathBuf,
}

impl Places {
    /// HOME (USERPROFILE on Windows) and the current directory.
    pub fn from_env() -> Places {
        let home = std::env::var_os("HOME").filter(|h| !h.is_empty())
            .or_else(|| std::env::var_os("USERPROFILE").filter(|h| !h.is_empty()))
            .map(PathBuf::from);
        Places { home, cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")) }
    }
}

/// Load the files in order (rules of later files come after earlier ones). Any error names the
/// file and the rule.
pub fn load(files: &[PathBuf], places: &Places) -> Result<RuleSet, String> {
    let mut set = RuleSet { rules: Vec::new(), warnings: Vec::new() };
    for f in files {
        let text = std::fs::read_to_string(f).map_err(|e| format!("{}: không đọc được ({e})", f.display()))?;
        load_text(&text, f, places, &mut set).map_err(|e| format!("{}: {e}", f.display()))?;
    }
    let mut seen = std::collections::HashSet::new();
    for r in &set.rules {
        if !seen.insert(r.id.as_str()) {
            return Err(format!("id `{}` trùng (id phải khác nhau trên mọi --file)", r.id));
        }
    }
    Ok(set)
}

fn load_text(text: &str, file: &Path, places: &Places, set: &mut RuleSet) -> Result<(), String> {
    let raw: RawFile = toml::from_str(text).map_err(|e| e.to_string().trim().replace('\n', " "))?;
    if raw.version != 1 {
        return Err(format!("version = {} chưa hỗ trợ (chỉ có version = 1)", raw.version));
    }
    let root = match &raw.root {
        Some(r) => paths::expand(r, places.home.as_deref(), &places.cwd)?,
        None => places.cwd.clone(),
    };
    if raw.rule.is_empty() {
        set.warnings.push(format!("{}: không có [[rule]] nào", file.display()));
    }
    for r in raw.rule {
        let id = r.id.clone();
        let rule = compile(r, &root, places, &mut set.warnings).map_err(|e| format!("luật `{id}`: {e}"))?;
        set.rules.push(rule);
    }
    Ok(())
}

fn re(s: &str, what: &str) -> Result<Regex, String> {
    Regex::new(s).map_err(|e| format!("{what}: regex sai ({})", e.to_string().lines().last().unwrap_or("")))
}

fn path_set(outside: &[String], glob: &[String], root: &Path, places: &Places) -> Result<PathSet, String> {
    let outside = outside.iter()
        .map(|o| paths::expand(o, places.home.as_deref(), root).and_then(|p| paths::normalize(&p)))
        .collect::<Result<Vec<_>, _>>()?;
    let globs = glob.iter().map(|g| paths::glob(g, places.home.as_deref(), root)).collect::<Result<Vec<_>, _>>()?;
    if outside.is_empty() && globs.is_empty() {
        return Err("điều kiện đường dẫn cần `outside` hoặc `glob`".into());
    }
    Ok(PathSet { outside, globs })
}

fn test(t: RawTest, root: &Path, places: &Places) -> Result<Test, String> {
    let n = t.eq.is_some() as u8 + t.one_of.is_some() as u8 + t.regex.is_some() as u8
        + t.basename_regex.is_some() as u8 + t.path.is_some() as u8;
    if n != 1 {
        return Err("mỗi điều kiện đối số cần đúng một trong eq / one_of / regex / basename_regex / path".into());
    }
    Ok(if let Some(v) = t.eq {
        Test::Eq(v)
    } else if let Some(v) = t.one_of {
        Test::OneOf(v)
    } else if let Some(v) = t.regex {
        Test::Regex(re(&v, "args.regex")?)
    } else if let Some(v) = t.basename_regex {
        Test::BasenameRegex(re(&v, "args.basename_regex")?)
    } else {
        let p = t.path.expect("counted");
        Test::Path(path_set(&p.outside, &p.glob, root, places)?)
    })
}

fn args(raw: Vec<RawArg>, root: &Path, places: &Places) -> Result<Vec<ArgCond>, String> {
    raw.into_iter().map(|a| {
        let inline = RawTest { eq: a.eq, one_of: a.one_of, regex: a.regex, basename_regex: a.basename_regex, path: a.path };
        match (a.pos, a.any) {
            (Some(p), None) => Ok(ArgCond { sel: Sel::Pos(p), test: test(inline, root, places)? }),
            (None, Some(t)) => {
                if inline.eq.is_some() || inline.one_of.is_some() || inline.regex.is_some()
                    || inline.basename_regex.is_some() || inline.path.is_some() {
                    return Err("`any = {…}` mang điều kiện bên trong, không kèm điều kiện ngoài".into());
                }
                Ok(ArgCond { sel: Sel::Any, test: test(t, root, places)? })
            }
            _ => Err("mỗi điều kiện đối số cần đúng một trong `pos = N` hoặc `any = {…}`".into()),
        }
    }).collect()
}

fn programs(p: &[String]) -> Result<Vec<Regex>, String> {
    if p.is_empty() {
        return Err("`program` rỗng".into());
    }
    p.iter().map(|g| {
        if g.contains('/') || g.contains('\\') {
            return Err(format!("program `{g}`: chỉ ghi tên chương trình (so với basename), không ghi đường dẫn"));
        }
        Ok(sos_gates::glob::to_regex(g))
    }).collect()
}

/// `{…}` placeholders a rule may use, by place.
const PLACEHOLDER: &str = r"\{([A-Za-z_][A-Za-z0-9_.]*)\}";

fn check_placeholders(text: &str, what: &str, ok: &dyn Fn(&str) -> bool) -> Result<(), String> {
    for c in Regex::new(PLACEHOLDER).expect("static").captures_iter(text) {
        let name = &c[1];
        if !ok(name) {
            return Err(format!("{what}: chỗ thay `{{{name}}}` không dùng được ở luật này"));
        }
    }
    Ok(())
}

fn compile(r: RawRule, root: &Path, places: &Places, warnings: &mut Vec<String>) -> Result<Rule, String> {
    if r.id.trim().is_empty() {
        return Err("`id` rỗng".into());
    }
    let event = match r.event.as_deref() {
        None | Some("pre") => Ev::Pre,
        Some("post") => Ev::Post,
        Some(e) => return Err(format!("event = \"{e}\" (chỉ có \"pre\" hoặc \"post\")")),
    };
    match r.decision.as_str() {
        "block" => {}
        "allow" | "ask" => return Err(format!("decision = \"{}\" chưa hỗ trợ ở bản này (chỉ có \"block\")", r.decision)),
        d => return Err(format!("decision = \"{d}\" (chỉ có \"block\")")),
    }
    let message = match r.message {
        Some(m) if !m.trim().is_empty() => m,
        _ => return Err("thiếu `message` (luật chặn phải nói vì sao và làm gì thay)".into()),
    };
    let kind = match (r.command, r.paths) {
        (Some(c), None) => {
            let mut alts = Vec::new();
            if !c.program.is_empty() {
                alts.push(Alt { programs: programs(&c.program)?, args: args(c.args, root, places)? });
            } else if !c.args.is_empty() {
                return Err("`command.args` cần `command.program`".into());
            }
            for a in c.or {
                alts.push(Alt { programs: programs(&a.program)?, args: args(a.args, root, places)? });
            }
            if alts.is_empty() {
                return Err("`command` cần `program` hoặc `or`".into());
            }
            Kind::Command(CommandCond {
                alts,
                raw: c.raw.as_deref().map(|s| re(s, "command.raw")).transpose()?,
                not_raw: c.not_raw.as_deref().map(|s| re(s, "command.not_raw")).transpose()?,
            })
        }
        (None, Some(p)) => Kind::Paths(PathsCond {
            set: path_set(&p.outside, &p.glob, root, places)?,
            exists: p.exists.unwrap_or(false),
            content: p.content.as_deref().map(|s| re(s, "paths.content")).transpose()?,
        }),
        _ => return Err("cần đúng một trong `command` hoặc `paths`".into()),
    };
    let allowed: &[&str] = match kind {
        Kind::Command(_) => COMMAND_TOOLS,
        Kind::Paths(_) => PATH_TOOLS,
    };
    for t in &r.tools {
        if !allowed.contains(&t.as_str()) {
            return Err(format!("tool `{t}` không dùng được với luật {} (dùng được: {})",
                               if matches!(kind, Kind::Command(_)) { "command" } else { "paths" }, allowed.join(", ")));
        }
    }
    let role = match r.when {
        Some(RawWhen { role: Some(w) }) => {
            if w.sources.is_empty() || w.one_of.is_empty() {
                return Err("`when.role` cần `sources` và `one_of`".into());
            }
            let sources = w.sources.into_iter().map(|s| {
                let p = |f: &str| paths::expand(f, places.home.as_deref(), root);
                match (s.file, s.file_exists) {
                    (Some(f), None) if s.value.is_none() => Ok(Source::File { path: p(&f)?, trim: s.trim.unwrap_or(false) }),
                    (None, Some(f)) if s.trim.is_none() => match s.value {
                        Some(v) => Ok(Source::FileExists { path: p(&f)?, value: v }),
                        None => Err("nguồn `file_exists` cần `value`".to_string()),
                    },
                    _ => Err("mỗi nguồn là `{ file, trim }` hoặc `{ file_exists, value }`".to_string()),
                }
            }).collect::<Result<Vec<_>, _>>()?;
            Some(Role { sources, one_of: w.one_of })
        }
        Some(RawWhen { role: None }) => return Err("`when` rỗng (có `when.role`)".into()),
        None => None,
    };
    let check = match r.check {
        Some(c) => {
            let secs = c.timeout_secs.ok_or("`check` thiếu `timeout_secs`")?;
            if secs == 0 || secs > MAX_CHECK_SECS {
                return Err(format!("check.timeout_secs = {secs}: phải từ 1 đến {MAX_CHECK_SECS} (hook tự chặn ở 30 giây)"));
            }
            let fires_on = match c.fires_on {
                RawFires::Word(w) if w == "success" => Fires::Success,
                RawFires::Word(w) if w == "failure" => Fires::Failure,
                RawFires::Word(w) => return Err(format!("check.fires_on = \"{w}\" (\"success\", \"failure\" hoặc {{ output = \"regex\" }})")),
                RawFires::Output(o) => Fires::Output(re(&o.output, "check.fires_on.output")?),
            };
            let allow_on_error = match c.on_error.as_deref() {
                None | Some("block") => false,
                Some("allow") => {
                    warnings.push(format!("luật `{}`: check.on_error = \"allow\" — lệnh kiểm hỏng (thiếu chương trình, hết giờ) sẽ cho qua", r.id));
                    true
                }
                Some(o) => return Err(format!("check.on_error = \"{o}\" (\"block\" hoặc \"allow\")")),
            };
            // Look through wrappers (`env sh -c`, `timeout 5 bash.exe -c`) for a shell given `-c`.
            let words: Vec<String> = std::iter::once(c.program.clone()).chain(c.args.iter().cloned()).collect();
            // A program line the reader cannot follow (`env -S '…'`, `eval`) could hide a shell: refuse.
            let ls = crate::layers(&words)
                .map_err(|e| format!("check.program/args không phân tích được ({e}); gọi thẳng chương trình kiểm"))?;
            // Any shell with a `{…}` argument: the value could turn into `-c CODE` (`sh {arg.0} {arg.1}`
            // with `-c 'code'`), review 1d round 4. The engine also re-reads the argv after substitution.
            let shell = ls.iter().any(|l| crate::shell::SHELLS.contains(&crate::prog_name(&l[0])));
            if shell && c.args.iter().any(|a| Regex::new(PLACEHOLDER).expect("static").is_match(a)) {
                return Err("check chạy shell với chỗ thay `{…}`: giá trị có thể thành mã (`-c …`); gọi thẳng chương trình".into());
            }
            Some(Check { program: c.program, args: c.args, timeout: Duration::from_secs(secs), fires_on, allow_on_error })
        }
        None => None,
    };
    let is_cmd = matches!(kind, Kind::Command(_));
    let in_args = |n: &str| match n {
        "path" => !is_cmd,
        "program" => is_cmd,
        "when.value" => role.is_some(),
        _ => is_cmd && n.strip_prefix("arg.").is_some_and(|k| k.parse::<usize>().is_ok()),
    };
    if let Some(c) = &check {
        for a in &c.args {
            check_placeholders(a, "check.args", &in_args)?;
        }
    }
    let has_check = check.is_some();
    check_placeholders(&message, "message", &|n| in_args(n) || (has_check && (n == "check.output" || n == "check.matched")))?;
    Ok(Rule { id: r.id, event, tools: r.tools, kind, role, check, message, root: root.to_path_buf() })
}
