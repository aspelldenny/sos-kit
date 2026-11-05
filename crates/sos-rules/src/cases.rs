//! `sos hook rules --file R --cases C.jsonl`: run a rules file against recorded cases.
//!
//! One JSON object per line (blank lines and lines starting with `//` are skipped):
//!
//! ```text
//! {"name": "...", "payload": {...}, "want": "allow"|"block",
//!  "old": "allow"|"block",          // what the script this rule replaces did (recorded)
//!  "note": "...",                   // required when old != want: why the difference is wanted
//!  "stderr_has": "..." | ["..."],   // substrings the block message must contain
//!  "files": {"path": "content"}, "symlinks": {"path": "target"},
//!  "fake": {"ssh": {"exit": 0, "stdout": ""} | {"error": "..."} | [{"args_has": ["x"], "exit": 1, "stdout": ""}]},
//!  "rule_files": ["other.toml"], "unix_only": true}
//! ```
//!
//! Each case gets a fresh temp dir T: `T/home` is HOME, `T/work` the working directory. The
//! tokens `{HOME}`, `{WORK}` and `{T}` are replaced (JSON-escaped) in the payload, in file and
//! symlink paths and in symlink targets; relative file paths are under T. A check program is
//! never run: it is answered from `fake` by its basename, and a check with no fake fails the case.

use crate::check::{Ran, Runner};
use crate::config::{self, Places};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

struct FakeRunner {
    fakes: BTreeMap<String, Value>,
    missing: RefCell<Vec<String>>,
}

impl Runner for FakeRunner {
    fn run(&self, program: &Path, args: &[String], _cwd: &Path, _t: Duration) -> Result<Ran, String> {
        let name = program.to_string_lossy();
        let base = crate::paths::basename_of(&name);
        let base = base.strip_suffix(".exe").unwrap_or(base);
        let Some(spec) = self.fakes.get(base) else {
            self.missing.borrow_mut().push(format!("{base} {}", args.join(" ")));
            return Err(format!("ca thử không có fake cho `{base}`"));
        };
        let pick = match spec {
            Value::Array(list) => list.iter().find(|e| {
                e.get("args_has").and_then(Value::as_array).is_none_or(|need| {
                    need.iter().filter_map(Value::as_str).all(|n| args.iter().any(|a| a.contains(n)))
                })
            }),
            v => Some(v),
        };
        let Some(e) = pick else {
            self.missing.borrow_mut().push(format!("{base} {}", args.join(" ")));
            return Err(format!("không fake nào của `{base}` khớp đối số"));
        };
        if let Some(err) = e.get("error").and_then(Value::as_str) {
            return Err(err.to_string());
        }
        let exit = e.get("exit").and_then(Value::as_i64).unwrap_or(0);
        let stdout = e.get("stdout").and_then(Value::as_str).unwrap_or("").to_string();
        Ok(Ran { success: exit == 0, stdout })
    }
}

fn json_escaped(s: &str) -> String {
    let q = serde_json::to_string(s).expect("string");
    q[1..q.len() - 1].to_string()
}

fn tokens(s: &str, t: &Path, escape: bool) -> String {
    let f = |p: PathBuf| {
        let s = p.to_string_lossy().into_owned();
        if escape { json_escaped(&s) } else { s }
    };
    s.replace("{HOME}", &f(t.join("home"))).replace("{WORK}", &f(t.join("work"))).replace("{T}", &f(t.to_path_buf()))
}

fn place(key: &str, t: &Path) -> PathBuf {
    let k = tokens(key, t, false);
    let p = Path::new(&k);
    if p.is_absolute() { p.to_path_buf() } else { t.join(p) }
}

fn word(v: Option<&Value>) -> Option<&str> {
    v.and_then(Value::as_str).filter(|s| *s == "allow" || *s == "block")
}

/// Run one case; Ok(None) = skipped on this platform; Err = why the case failed.
fn run_case(case: &Value, rule_files: &[PathBuf], dir: &Path) -> Result<Option<()>, String> {
    if case.get("unix_only").and_then(Value::as_bool) == Some(true) && !cfg!(unix) {
        return Ok(None);
    }
    let want = word(case.get("want")).ok_or("thiếu `want` (allow|block)")?;
    let old = case.get("old").map(|_| word(case.get("old")).ok_or("`old` phải là allow|block")).transpose()?;
    if let Some(o) = old {
        if o != want && case.get("note").and_then(Value::as_str).is_none_or(str::is_empty) {
            return Err(format!("old = {o} khác want = {want} mà không có `note` giải thích"));
        }
    }
    let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
    // Canonical, so symlinked temp roots (macOS /var → /private/var) do not change match results;
    // without the Windows `\\?\` prefix, which rejects the `/` in case file names.
    let t = crate::paths::strip_verbatim(std::fs::canonicalize(tmp.path()).map_err(|e| e.to_string())?);
    for d in ["home", "work"] {
        std::fs::create_dir_all(t.join(d)).map_err(|e| e.to_string())?;
    }
    if let Some(files) = case.get("files").and_then(Value::as_object) {
        for (k, v) in files {
            let p = place(k, &t);
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&p, v.as_str().unwrap_or_default()).map_err(|e| format!("{}: {e}", p.display()))?;
        }
    }
    if let Some(links) = case.get("symlinks").and_then(Value::as_object) {
        for (k, v) in links {
            let p = place(k, &t);
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let target = place(v.as_str().unwrap_or_default(), &t);
            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, &p).map_err(|e| format!("{}: {e}", p.display()))?;
            #[cfg(windows)]
            {
                let r = if target.is_dir() {
                    std::os::windows::fs::symlink_dir(&target, &p)
                } else {
                    std::os::windows::fs::symlink_file(&target, &p)
                };
                r.map_err(|e| format!("{}: {e} (symlink trên Windows cần Developer Mode; đánh dấu ca là unix_only)", p.display()))?;
            }
        }
    }
    let payload_text = tokens(&serde_json::to_string(case.get("payload").ok_or("thiếu `payload`")?).expect("json"), &t, true);
    let payload: Value = serde_json::from_str(&payload_text).map_err(|e| e.to_string())?;
    let places = Places { home: Some(t.join("home")), cwd: t.join("work") };
    let files: Vec<PathBuf> = match case.get("rule_files").and_then(Value::as_array) {
        Some(list) => list.iter().filter_map(Value::as_str).map(|f| dir.join(f)).collect(),
        None => rule_files.to_vec(),
    };
    let fakes = case.get("fake").and_then(Value::as_object).map(|m| m.clone().into_iter().collect()).unwrap_or_default();
    let runner = FakeRunner { fakes, missing: RefCell::new(Vec::new()) };
    let blocks = match config::load(&files, &places) {
        Ok(set) => crate::decide(&set, &payload, &places, &runner).blocks,
        Err(e) => vec![format!("sửa file luật: {e}")],
    };
    if let Some(m) = runner.missing.borrow().first() {
        return Err(format!("check gọi `{m}` mà ca thử không có `fake` cho nó"));
    }
    let got = if blocks.is_empty() { "allow" } else { "block" };
    let text = blocks.join("\n");
    if got != want {
        return Err(format!("muốn {want}, ra {got}{}", if text.is_empty() { String::new() } else { format!(": {text}") }));
    }
    let needs: Vec<&str> = match case.get("stderr_has") {
        Some(Value::String(s)) => vec![s.as_str()],
        Some(Value::Array(a)) => a.iter().filter_map(Value::as_str).collect(),
        _ => vec![],
    };
    for n in needs {
        let n = tokens(n, &t, false);
        if !text.contains(&n) {
            return Err(format!("thông báo thiếu «{n}»; thông báo là: {text}"));
        }
    }
    Ok(Some(()))
}

/// Run every case of `cases`; prints one line per case and a summary. Returns the failure count
/// (a file that cannot be read counts as one failure).
pub fn run(rule_files: &[PathBuf], cases: &Path) -> usize {
    let text = match std::fs::read_to_string(cases) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("FAIL {}: không đọc được ({e})", cases.display());
            return 1;
        }
    };
    let dir = cases.parent().unwrap_or(Path::new("."));
    let (mut ok, mut bad, mut skip) = (0, 0, 0);
    for (i, line) in text.lines().enumerate() {
        let l = line.trim();
        if l.is_empty() || l.starts_with("//") {
            continue;
        }
        let (name, res) = match serde_json::from_str::<Value>(l) {
            Ok(case) => {
                let name = case.get("name").and_then(Value::as_str).unwrap_or("").to_string();
                (name, run_case(&case, rule_files, dir))
            }
            Err(e) => (String::new(), Err(format!("JSON hỏng: {e}"))),
        };
        let label = if name.is_empty() { format!("dòng {}", i + 1) } else { name };
        match res {
            Ok(Some(())) => {
                ok += 1;
                println!("ok   {label}");
            }
            Ok(None) => {
                skip += 1;
                println!("skip {label} (unix_only)");
            }
            Err(e) => {
                bad += 1;
                println!("FAIL {label}: {e}");
            }
        }
    }
    if ok + bad == 0 && skip == 0 {
        println!("FAIL {}: không có ca nào", cases.display());
        return 1;
    }
    println!("{}: {ok} đạt, {bad} hỏng, {skip} bỏ qua", cases.display());
    bad
}
