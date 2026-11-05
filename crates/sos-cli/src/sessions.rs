//! `sos sessions idle|count` — the Claude desktop app's session files, read-only.
//!
//! The app keeps one `local_<id>.json` per Code session under
//! `<app data>/Claude/claude-code-sessions/<account>/<org>/`, with `lastActivityAt` (ms),
//! `isArchived`, `cwd`, `title` and `cliSessionId`. Reading them lets a script answer "which
//! sessions have gone quiet" and "how many are running" without asking a model to call
//! list_sessions (plan docs/plans/RUST_ALL_TOOLS_2026-11-04.md, wave 4.3 and 4.4).
//!
//!   sos sessions idle [--over 24h] [--json] [ID…]   quiet sessions (all unarchived, or the IDs)
//!   sos sessions count [--within 2h] [--max N] [--exclude ID] [--hook]
//!
//! `count --max N` is a gate: N or more active sessions → exit 2 with the reason on stderr
//! ("xếp hàng chờ"). With `--hook` it reads the Claude Code PreToolUse payload on stdin and does
//! not count the session that is asking. Anything it cannot read (folder, a session file, the
//! payload) also exits 2: a gate that cannot count does not let the action through.

use clap::Subcommand;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Subcommand)]
pub enum SessionsCmd {
    /// Sessions with no activity for longer than `--over`: every unarchived session, or the IDs
    /// given (an ID that is archived or has no file is reported as such). Exit 0; 2 when the
    /// session folder or a session file cannot be read.
    Idle {
        /// Report only sessions quiet for longer than this (e.g. 90m, 24h, 3d). Default: all.
        #[arg(long, value_parser = parse_age)]
        over: Option<Duration>,
        /// One JSON array on stdout (every session looked at, with idle_secs and found/archived).
        #[arg(long)]
        json: bool,
        /// Session folder (default: the app's, or $SOS_SESSIONS_DIR).
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Session IDs (`local_…`) to look at; none = every unarchived session.
        ids: Vec<String>,
    },
    /// Count unarchived sessions active within `--within`. With `--max N`: exit 2 ("xếp hàng
    /// chờ") when the count is N or more, else 0. Prints the count when there is no `--max`.
    Count {
        /// Active = last activity within this (default 2h).
        #[arg(long, value_parser = parse_age, default_value = "2h")]
        within: Duration,
        /// Block (exit 2) at this many active sessions.
        #[arg(long)]
        max: Option<usize>,
        /// Do not count this session (`local_…` or the CLI session id). Repeatable.
        #[arg(long)]
        exclude: Vec<String>,
        /// Read a Claude Code hook payload on stdin and do not count the session it comes from.
        #[arg(long)]
        hook: bool,
        /// Text after the count when blocking (default: queue it and open it when one finishes).
        #[arg(long)]
        message: Option<String>,
        #[arg(long)]
        dir: Option<PathBuf>,
    },
}

/// The fields this module reads; the app writes many more, which are ignored.
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub session_id: String,
    #[serde(default)]
    pub cli_session_id: Option<String>,
    #[serde(default)]
    pub last_assistant_cli_session_id: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub last_activity_at: Option<f64>,
    #[serde(default)]
    pub created_at: Option<f64>,
    #[serde(default)]
    pub is_archived: bool,
}

impl Session {
    /// Last activity in ms since the epoch: `lastActivityAt`, else `createdAt`, else 0 (oldest).
    fn last_ms(&self) -> f64 {
        self.last_activity_at.or(self.created_at).unwrap_or(0.0)
    }
    fn idle_secs(&self, now_ms: f64) -> u64 {
        ((now_ms - self.last_ms()).max(0.0) / 1000.0) as u64
    }
    fn is(&self, id: &str) -> bool {
        self.session_id == id
            || self.cli_session_id.as_deref() == Some(id)
            || self.last_assistant_cli_session_id.as_deref() == Some(id)
    }
}

/// `90s`, `45m`, `24h`, `3d` (a unit is required: "3" alone is ambiguous).
pub fn parse_age(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    let (num, unit) = s.split_at(s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len()));
    let n: u64 = num.parse().map_err(|_| format!("`{s}`: expected a number and a unit, e.g. 90m, 24h, 3d"))?;
    let mult = match unit {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86_400,
        _ => return Err(format!("`{s}`: unit must be s, m, h or d (e.g. 24h)")),
    };
    n.checked_mul(mult).map(Duration::from_secs).ok_or_else(|| format!("`{s}`: too large"))
}

/// The app's session folder: $SOS_SESSIONS_DIR, else per OS
/// (macOS ~/Library/Application Support, Windows %APPDATA%, else $XDG_CONFIG_HOME or ~/.config).
pub fn default_dir() -> Result<PathBuf, String> {
    if let Some(d) = std::env::var_os("SOS_SESSIONS_DIR").filter(|d| !d.is_empty()) {
        return Ok(PathBuf::from(d));
    }
    let home = || std::env::var_os("HOME").filter(|h| !h.is_empty()).map(PathBuf::from)
        .ok_or_else(|| "HOME is not set; pass --dir".to_string());
    let base = if cfg!(target_os = "macos") {
        home()?.join("Library/Application Support")
    } else if cfg!(windows) {
        std::env::var_os("APPDATA").filter(|d| !d.is_empty()).map(PathBuf::from)
            .ok_or_else(|| "APPDATA is not set; pass --dir".to_string())?
    } else {
        match std::env::var_os("XDG_CONFIG_HOME").filter(|d| !d.is_empty()) {
            Some(d) => PathBuf::from(d),
            None => home()?.join(".config"),
        }
    };
    Ok(base.join("Claude").join("claude-code-sessions"))
}

/// Every `local_*.json` under `dir` (it and up to three levels below; symlinks are not followed).
/// A file that does not parse is read once more after a short pause (the app may be mid-write),
/// then it is an error: a session we cannot read could be one that is running.
pub fn load(dir: &Path) -> Result<Vec<Session>, String> {
    let meta = std::fs::metadata(dir).map_err(|e| format!("session folder {}: {e}", dir.display()))?;
    if !meta.is_dir() {
        return Err(format!("session folder {} is not a folder", dir.display()));
    }
    let mut files = Vec::new();
    walk(dir, 0, &mut files)?;
    files.sort();
    let mut out = Vec::with_capacity(files.len());
    for f in files {
        out.push(read_one(&f).or_else(|_| {
            std::thread::sleep(Duration::from_millis(150));
            read_one(&f)
        })?);
    }
    Ok(out)
}

fn walk(dir: &Path, depth: usize, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let rd = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for ent in rd {
        let ent = ent.map_err(|e| format!("{}: {e}", dir.display()))?;
        let ft = ent.file_type().map_err(|e| format!("{}: {e}", ent.path().display()))?;
        let name = ent.file_name();
        let name = name.to_string_lossy();
        if ft.is_dir() && depth < 3 {
            walk(&ent.path(), depth + 1, files)?;
        } else if ft.is_file() && name.starts_with("local_") && name.ends_with(".json") {
            files.push(ent.path());
        }
    }
    Ok(())
}

fn read_one(f: &Path) -> Result<Session, String> {
    let bytes = std::fs::read(f).map_err(|e| format!("{}: {e}", f.display()))?;
    let s: Session = serde_json::from_slice(&bytes).map_err(|e| format!("{}: not a session file ({e})", f.display()))?;
    if s.session_id.is_empty() {
        return Err(format!("{}: sessionId is empty", f.display()));
    }
    Ok(s)
}

fn now_ms() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as f64).unwrap_or(0.0)
}

/// "26h", "3d 4h", "45m": what a person reads in a report.
pub fn human(secs: u64) -> String {
    let (d, h, m) = (secs / 86_400, secs % 86_400 / 3600, secs % 3600 / 60);
    match (d, h) {
        (0, 0) => format!("{m}m"),
        (0, _) => format!("{h}h"),
        _ if h == 0 => format!("{d}d"),
        _ => format!("{d}d {h}h"),
    }
}

/// One row of `idle`: a session looked at, or an ID with no file.
#[derive(Debug, PartialEq)]
pub enum Row {
    Quiet { id: String, idle_secs: u64, archived: bool, title: String, cwd: String },
    Missing { id: String },
}

pub fn idle_rows(all: &[Session], ids: &[String], over: Option<Duration>, now_ms: f64) -> Vec<Row> {
    // No --over: every unarchived session, including one active this very second (idle 0).
    let quiet = |s: &Session| over.is_none_or(|d| s.idle_secs(now_ms) > d.as_secs());
    let row = |s: &Session, id: &str| Row::Quiet {
        id: id.to_string(),
        idle_secs: s.idle_secs(now_ms),
        archived: s.is_archived,
        title: s.title.clone().unwrap_or_default(),
        cwd: s.cwd.clone().unwrap_or_default(),
    };
    if ids.is_empty() {
        let mut v: Vec<&Session> = all.iter().filter(|s| !s.is_archived && quiet(s)).collect();
        v.sort_by(|a, b| b.idle_secs(now_ms).cmp(&a.idle_secs(now_ms)).then(a.session_id.cmp(&b.session_id)));
        return v.into_iter().map(|s| row(s, &s.session_id)).collect();
    }
    ids.iter().filter_map(|id| match all.iter().find(|s| s.session_id == *id) {
        None => Some(Row::Missing { id: id.clone() }),
        // An archived session is finished, not quiet: always reported, so a board that still
        // names it can be corrected.
        Some(s) if s.is_archived || quiet(s) => Some(row(s, id)),
        Some(_) => None,
    }).collect()
}

/// Unarchived sessions active within `within`, minus the excluded ones. An unarchived session
/// with no `lastActivityAt` is an error, not an old session: a gate must not guess it is idle.
pub fn count_active(all: &[Session], within: Duration, exclude: &[String], now_ms: f64) -> Result<usize, String> {
    let mut n = 0;
    for s in all.iter().filter(|s| !s.is_archived && !exclude.iter().any(|x| s.is(x))) {
        if s.last_activity_at.is_none() {
            return Err(format!("session {} has no lastActivityAt, so it cannot be told apart from a running one", s.session_id));
        }
        if s.idle_secs(now_ms) < within.as_secs().max(1) {
            n += 1;
        }
    }
    Ok(n)
}

pub fn run(cmd: SessionsCmd) -> anyhow::Result<i32> {
    Ok(match cmd {
        SessionsCmd::Idle { over, json, dir, ids } => idle(over, json, dir, &ids),
        SessionsCmd::Count { within, max, exclude, hook, message, dir } => count(within, max, exclude, hook, message, dir),
    })
}

fn sessions_or_err(dir: Option<PathBuf>, what: &str) -> Result<Vec<Session>, i32> {
    let dir = match dir.map(Ok).unwrap_or_else(default_dir) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{what}: {e}");
            return Err(2);
        }
    };
    load(&dir).map_err(|e| {
        eprintln!("{what}: {e}\nHow to fix: pass --dir (or SOS_SESSIONS_DIR) pointing at the app's claude-code-sessions folder, \
                   or move the unreadable file aside.");
        2
    })
}

fn idle(over: Option<Duration>, json: bool, dir: Option<PathBuf>, ids: &[String]) -> i32 {
    if let Some(bad) = ids.iter().find(|i| !i.starts_with("local_")) {
        eprintln!("sos sessions idle: `{bad}` is not a session id (local_…)");
        return 2;
    }
    let all = match sessions_or_err(dir, "sos sessions idle") {
        Ok(a) => a,
        Err(c) => return c,
    };
    let rows = idle_rows(&all, ids, over, now_ms());
    if json {
        let v: Vec<Value> = rows.iter().map(|r| match r {
            Row::Quiet { id, idle_secs, archived, title, cwd } => json!({
                "id": id, "found": true, "archived": archived, "idle_secs": idle_secs, "title": title, "cwd": cwd,
            }),
            Row::Missing { id } => json!({ "id": id, "found": false }),
        }).collect();
        println!("{}", Value::Array(v));
        return 0;
    }
    for r in &rows {
        match r {
            Row::Quiet { id, archived: true, title, .. } => println!("{id}\tarchived\t{title}"),
            Row::Quiet { id, idle_secs, title, cwd, .. } => println!("{id}\t{}\t{title}\t{cwd}", human(*idle_secs)),
            Row::Missing { id } => println!("{id}\tno session file"),
        }
    }
    0
}

fn count(within: Duration, max: Option<usize>, mut exclude: Vec<String>, hook: bool, message: Option<String>, dir: Option<PathBuf>) -> i32 {
    let what = "sos sessions count";
    if hook {
        let Some(p) = crate::hook::payload() else {
            eprintln!("BLOCKED: {what} --hook: the hook payload on stdin is not a JSON object, so the sessions cannot be counted.");
            return 2;
        };
        match p.get("session_id").and_then(Value::as_str).filter(|s| !s.is_empty()) {
            Some(id) => exclude.push(id.to_string()),
            None => {
                eprintln!("BLOCKED: {what} --hook: the payload has no session_id, so the asking session would count itself.");
                return 2;
            }
        }
    }
    let all = match sessions_or_err(dir, if max.is_some() { "BLOCKED: sos sessions count" } else { what }) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let n = match count_active(&all, within, &exclude, now_ms()) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{}{what}: {e}.\nHow to fix: open or archive that session in the app (or move its file aside if it is broken).",
                      if max.is_some() { "BLOCKED: " } else { "" });
            return 2;
        }
    };
    let Some(max) = max else {
        println!("{n}");
        return 0;
    };
    if n < max {
        return 0;
    }
    let tail = message.unwrap_or_else(|| "Không mở thêm: xếp hàng chờ, khi một session xong thì mở việc đầu hàng.".to_string());
    eprintln!("BLOCKED: đang có {n} session chạy (tối đa {max}; tính session chưa lưu trữ có hoạt động trong {}, \
               trừ session đang hỏi). {tail}", human(within.as_secs()));
    2
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: f64 = 1_800_000_000_000.0;

    fn s(id: &str, idle_secs: u64, archived: bool) -> Session {
        Session {
            session_id: id.into(),
            cli_session_id: Some(format!("cli-{id}")),
            last_activity_at: Some(NOW - idle_secs as f64 * 1000.0),
            is_archived: archived,
            title: Some(format!("t {id}")),
            ..Default::default()
        }
    }

    #[test]
    fn ages() {
        assert_eq!(parse_age("90m").unwrap(), Duration::from_secs(5400));
        assert_eq!(parse_age("3d").unwrap(), Duration::from_secs(259_200));
        assert!(parse_age("3").is_err());
        assert!(parse_age("h").is_err());
        assert!(parse_age("2w").is_err());
        assert_eq!(human(26 * 3600), "1d 2h");
        assert_eq!(human(5 * 3600), "5h");
        assert_eq!(human(3 * 86_400 + 4 * 3600), "3d 4h");
        assert_eq!(human(45 * 60), "45m");
    }

    #[test]
    fn count_two_allows_three_blocks() {
        let h = 3600;
        let mut all = vec![s("local_a", 60, false), s("local_b", h, false), s("local_old", 3 * h, false),
                           s("local_arch", 10, true), s("local_me", 5, false)];
        let me = vec!["cli-local_me".to_string()];
        let within = Duration::from_secs(2 * h);
        assert_eq!(count_active(&all, within, &me, NOW), Ok(2));
        all.push(s("local_c", 30, false));
        assert_eq!(count_active(&all, within, &me, NOW), Ok(3));
        // Excluded by its local id too.
        assert_eq!(count_active(&all, within, &["local_c".into(), "local_me".into()], NOW), Ok(2));
        // No lastActivityAt: an error, even with an old createdAt; archived ones do not matter.
        let mut blind = s("local_blind", 0, false);
        blind.last_activity_at = None;
        blind.created_at = Some(1.0);
        let mut gone = blind.clone();
        gone.session_id = "local_gone".into();
        gone.is_archived = true;
        all.push(gone);
        assert_eq!(count_active(&all, within, &me, NOW), Ok(3));
        all.push(blind);
        assert!(count_active(&all, within, &me, NOW).unwrap_err().contains("local_blind"));
    }

    #[test]
    fn idle_listed_and_missing() {
        let all = vec![s("local_a", 30 * 3600, false), s("local_b", 3600, false), s("local_c", 99 * 3600, true)];
        let over = Some(Duration::from_secs(24 * 3600));
        let rows = idle_rows(&all, &["local_a".into(), "local_b".into(), "local_c".into(), "local_x".into()], over, NOW);
        assert_eq!(rows.len(), 3);
        assert!(matches!(&rows[0], Row::Quiet { id, archived: false, idle_secs, .. } if id == "local_a" && *idle_secs == 30 * 3600));
        assert!(matches!(&rows[1], Row::Quiet { id, archived: true, .. } if id == "local_c"));
        assert_eq!(rows[2], Row::Missing { id: "local_x".into() });
        // No IDs: unarchived only, quietest first.
        let rows = idle_rows(&all, &[], None, NOW);
        let ids: Vec<_> = rows.iter().map(|r| match r { Row::Quiet { id, .. } | Row::Missing { id } => id.as_str() }).collect();
        assert_eq!(ids, ["local_a", "local_b"]);
        // Without --over a session active this second (idle 0) is listed too.
        let all = vec![s("local_now", 0, false)];
        assert_eq!(idle_rows(&all, &[], None, NOW).len(), 1);
        assert_eq!(idle_rows(&all, &["local_now".into()], None, NOW).len(), 1);
    }

    #[test]
    fn load_reads_nested_and_refuses_broken() {
        let d = tempfile::tempdir().unwrap();
        let org = d.path().join("acct/org");
        std::fs::create_dir_all(org.join("backlog")).unwrap();
        std::fs::write(org.join("local_a.json"), r#"{"sessionId":"local_a","lastActivityAt":1,"isArchived":false,"extra":{"x":1}}"#).unwrap();
        std::fs::write(org.join("scheduled-tasks.json"), "not json").unwrap();
        std::fs::write(org.join("backlog/tasks.json"), "[]").unwrap();
        let all = load(d.path()).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].session_id, "local_a");
        std::fs::write(org.join("local_b.json"), "{\"sessionId\":").unwrap();
        assert!(load(d.path()).unwrap_err().contains("local_b.json"));
        assert!(load(&d.path().join("nope")).is_err());
    }
}
