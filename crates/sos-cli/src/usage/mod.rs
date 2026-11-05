//! `sos usage` — token use by role × model from the agents' local transcripts, read-only.
//! Design and the review behind it: docs/plans/RUST_ARCHITECTURE_2026-11-05.md §7.
//!
//! Privacy: nothing is sent, written or spawned. Only ids, numbers and short metadata (model,
//! role, project path, day) are kept while reading; prompt and tool text are measured and dropped.

pub mod claude;
pub mod codex;
pub mod prices;
pub mod table;

use prices::{Prices, Tokens};
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::BufRead;
use std::path::{Path, PathBuf};

#[derive(clap::Args, Debug)]
pub struct Args {
    /// First UTC day counted (YYYY-MM-DD); default 30 days ago.
    #[arg(long)]
    pub since: Option<String>,
    /// Last UTC day counted (YYYY-MM-DD).
    #[arg(long)]
    pub until: Option<String>,
    /// Only this project: a path (matches the session's cwd or below) or a folder name.
    #[arg(long)]
    pub project: Option<String>,
    /// Rows by: role, model, project, day, session.
    #[arg(long, default_value = "role", value_parser = ["role", "model", "project", "day", "session"])]
    pub by: String,
    /// claude, codex or all.
    #[arg(long, default_value = "claude", value_parser = ["claude", "codex", "all"])]
    pub agent: String,
    /// Prices table (TOML); default ~/.config/sos/prices.toml, else the embedded one.
    #[arg(long)]
    pub prices: Option<PathBuf>,
    #[arg(long)]
    pub json: bool,
    /// Also print how each number was found: role sources, Codex sources, heuristics.
    #[arg(long)]
    pub explain: bool,
    /// Claude projects folder (default ~/.claude/projects).
    #[arg(long)]
    pub dir: Option<PathBuf>,
    /// Codex sessions folder (default ~/.codex/sessions).
    #[arg(long)]
    pub codex_dir: Option<PathBuf>,
    /// Lines longer than this are skipped and reported, not parsed.
    #[arg(long, default_value_t = 16 * 1024 * 1024)]
    pub max_line_bytes: usize,
    /// `large_initial_context` heuristic: a subagent whose first call reads more than this.
    #[arg(long, default_value_t = 50_000)]
    pub large_context: u64,
}

/// One API call, counted once.
#[derive(Clone, Debug)]
pub struct Call {
    pub session: String,
    pub role: String,
    pub model: String,
    pub project: String,
    pub day: String,
    pub tokens: Tokens,
    /// Claude `output_tokens_details.thinking_tokens` / Codex `reasoning_output_tokens`; part of output.
    pub thinking: Option<u64>,
}

#[derive(Default, Serialize, Debug, Clone, PartialEq)]
pub struct Skipped {
    pub synthetic: u64,
    /// Calls with no message/response id: counted once per line (they cannot be deduplicated).
    pub no_id: u64,
    pub oversize_lines: u64,
    pub bad_json: u64,
    /// Sidechain entries in a main file whose subagent file holds the same calls.
    pub sidechain_owned: u64,
    /// Lines that are not valid UTF-8: skipped, never repaired (a repaired id could merge two calls).
    pub invalid_utf8: u64,
}

/// Text a subagent handed back to its parent, per subagent role: what was returned once, not
/// what the parent re-read across requests.
#[derive(Default, Serialize, Debug, Clone, PartialEq)]
pub struct ResultText {
    pub results: u64,
    pub subagent_result_text_chars: u64,
    pub spawn_acks: u64,
    pub errors: u64,
}

#[derive(Serialize, Debug, Clone)]
pub struct LargeContext {
    pub session: String,
    pub agent_id: String,
    pub role: String,
    pub first_call_input_tokens: u64,
    /// `toolUseId` of the spawning call, when the subagent's meta names it; else unknown.
    pub parent_tool_use: String,
}

#[derive(Default)]
pub struct Scan {
    pub calls: Vec<Call>,
    pub files: u64,
    pub dedup_dropped: u64,
    pub skipped: Skipped,
    pub coverage: Vec<String>,
    pub results: BTreeMap<String, ResultText>,
    pub large: Vec<LargeContext>,
    /// Agent tool calls of every file, per (session, tool_use_id): copies of the same history in
    /// resumed files merge here, so a result counts once.
    pub(crate) agents: BTreeMap<(String, String), claude::AgentCall>,
    /// --explain lines (no content, only ids and counts).
    pub explain: Vec<String>,
}

/// Filters every reader applies to a record before counting it.
pub struct Filter {
    pub since: String,
    pub until: Option<String>,
    pub project: Option<String>,
}

impl Filter {
    pub fn day_ok(&self, day: &str) -> bool {
        !day.is_empty() && day >= self.since.as_str() && self.until.as_deref().is_none_or(|u| day <= u)
    }
    /// `--project` is a path (the cwd or below it) or a folder name (the cwd's last part). `\` and
    /// `/` are the same separator, so Windows paths match either way.
    pub fn project_ok(&self, cwd: &str) -> bool {
        let Some(p) = &self.project else { return true };
        let norm = |s: &str| s.replace('\\', "/").trim_end_matches('/').to_string();
        let (p, cwd) = (norm(p), norm(cwd));
        if p.contains('/') || p.contains(':') {
            cwd == p || cwd.starts_with(&format!("{p}/"))
        } else {
            cwd.rsplit('/').next() == Some(p.as_str())
        }
    }
}

/// Reads one line into `buf` (without the newline). `Ok(None)` at EOF; `Ok(Some(false))` when
/// the line was longer than `max` and was skipped without being held in memory.
pub fn read_line(r: &mut impl BufRead, buf: &mut Vec<u8>, max: usize) -> std::io::Result<Option<bool>> {
    buf.clear();
    let mut over = false;
    let mut any = false;
    loop {
        let chunk = r.fill_buf()?;
        if chunk.is_empty() {
            return Ok(any.then_some(!over));
        }
        any = true;
        let (take, done) = match chunk.iter().position(|&b| b == b'\n') {
            Some(i) => (i, true),
            None => (chunk.len(), false),
        };
        if !over {
            if buf.len() + take > max {
                over = true;
                buf.clear();
            } else {
                buf.extend_from_slice(&chunk[..take]);
            }
        }
        r.consume(take + usize::from(done));
        if done {
            return Ok(Some(!over));
        }
    }
}

/// Lines of a file, streamed: `Some(None)` for a line over `max` bytes (skipped, never held
/// whole); a line that is not UTF-8 is counted in `bad_utf8` and yielded empty (so skipped); a read
/// error ends the stream and is kept in `error`.
pub struct Lines<R> {
    r: R,
    buf: Vec<u8>,
    max: usize,
    pub bad_utf8: u64,
    pub error: Option<String>,
}

impl<R: BufRead> Iterator for Lines<R> {
    type Item = Option<String>;
    fn next(&mut self) -> Option<Self::Item> {
        match read_line(&mut self.r, &mut self.buf, self.max) {
            Ok(Some(true)) => Some(Some(match String::from_utf8(std::mem::take(&mut self.buf)) {
                Ok(s) => s,
                Err(_) => {
                    self.bad_utf8 += 1;
                    String::new()
                }
            })),
            Ok(Some(false)) => Some(None),
            Ok(None) => None,
            Err(e) => {
                self.error = Some(e.kind().to_string());
                None
            }
        }
    }
}

pub fn lines_of(path: &Path, max: usize, scan: &mut Scan) -> Option<Lines<std::io::BufReader<std::fs::File>>> {
    match std::fs::File::open(path) {
        Ok(f) => Some(Lines { r: std::io::BufReader::with_capacity(1 << 16, f), buf: Vec::new(), max, bad_utf8: 0, error: None }),
        Err(e) => {
            scan.coverage.push(format!("unreadable, not counted: {} ({})", path.display(), e.kind()));
            None
        }
    }
}

/// After a file: its decode and read problems go into the scan.
pub fn close<R>(lines: Lines<R>, path: &Path, scan: &mut Scan) {
    scan.skipped.invalid_utf8 += lines.bad_utf8;
    if let Some(e) = lines.error {
        scan.coverage.push(format!("read error, the rest of the file is not counted: {} ({e})", path.display()));
    }
}

/// Directory entries, sorted, without symlinks (an alias of a folder would count its files twice,
/// a link to a parent would loop). Unreadable folders and skipped links go into coverage.
pub fn entries(dir: &Path, scan: &mut Scan) -> Vec<PathBuf> {
    let rd = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(e) => {
            scan.coverage.push(format!("unreadable folder, not counted: {} ({})", dir.display(), e.kind()));
            return Vec::new();
        }
    };
    let mut v = Vec::new();
    for e in rd {
        match e {
            Ok(e) if e.file_type().is_ok_and(|t| t.is_symlink()) => {
                scan.coverage.push(format!("symlink not followed: {}", e.path().display()));
            }
            Ok(e) => v.push(e.path()),
            Err(e) => scan.coverage.push(format!("unreadable entry in {} ({})", dir.display(), e.kind())),
        }
    }
    v.sort();
    v
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

/// An RFC 3339 timestamp as a UTC sort key `YYYY-MM-DDTHH:MM[:SS[.fff]]`: `…Z` or no offset as
/// written, `±HH:MM` converted. Comparable as a string across offsets. Anything unreadable gives ""
/// (never counted as some other day); slicing is by checked byte ranges, so no input can panic.
pub fn utc_key(ts: &str) -> String {
    let Some(day) = ts.get(..10).filter(|d| valid_day(d)) else { return String::new() };
    let num = |s: &str| s.parse::<i64>().ok().filter(|_| s.bytes().all(|b| b.is_ascii_digit()));
    let t = &ts[10..];
    let (Some(hh), Some(mm)) = (t.get(1..3).and_then(num), t.get(4..6).and_then(num)) else { return day.to_string() };
    if !t.starts_with(['T', 't', ' ']) || t.as_bytes().get(3) != Some(&b':') {
        return day.to_string();
    }
    // Offset: the last '+' or '-' after the time; else 'Z' or the end.
    let off = t.rfind(['+', '-']).filter(|&i| i >= 6 && t.len() == i + 6 && t.as_bytes()[i + 3] == b':');
    let end = off.or_else(|| t.find(['Z', 'z'])).unwrap_or(t.len());
    let secs = t.get(6..end).filter(|x| x.bytes().all(|b| b.is_ascii_digit() || b == b':' || b == b'.')).unwrap_or("");
    let shift = match off {
        Some(i) => {
            let (Some(oh), Some(om)) = (t.get(i + 1..i + 3).and_then(num), t.get(i + 4..i + 6).and_then(num)) else { return day.to_string() };
            if t.as_bytes()[i] == b'-' { oh * 60 + om } else { -(oh * 60 + om) }
        }
        None => 0,
    };
    let minutes = hh * 60 + mm + shift;
    let (Some(y), Some(m), Some(d)) = (num(&day[..4]), num(&day[5..7]), num(&day[8..10])) else { return String::new() };
    let mins = minutes.rem_euclid(1440);
    format!("{}T{:02}:{:02}{secs}", civil_from_days(days_from_civil(y, m, d) + minutes.div_euclid(1440)), mins / 60, mins % 60)
}

/// The UTC day of a timestamp (see `utc_key`).
pub fn utc_day(ts: &str) -> String {
    utc_key(ts).chars().take(10).collect()
}

/// The user's home: HOME, else USERPROFILE (Windows); never a relative fallback.
pub fn home() -> Option<PathBuf> {
    ["HOME", "USERPROFILE"].iter().filter_map(std::env::var_os).map(PathBuf::from).find(|p| p.is_absolute())
}

/// UTC date `n` days before today, as YYYY-MM-DD (civil-from-days, no date library).
pub fn days_ago(n: u64) -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    civil_from_days((secs / 86_400).saturating_sub(n) as i64)
}

fn valid_day(s: &str) -> bool {
    s.len() == 10 && s.bytes().enumerate().all(|(i, b)| if i == 4 || i == 7 { b == b'-' } else { b.is_ascii_digit() })
}

pub fn run(a: Args) -> anyhow::Result<i32> {
    let since = a.since.clone().unwrap_or_else(|| days_ago(30));
    for d in std::iter::once(&since).chain(a.until.iter()) {
        if !valid_day(d) {
            anyhow::bail!("dates are UTC days written YYYY-MM-DD: {d}");
        }
    }
    let prices = Prices::load(a.prices.as_deref())?;
    let filter = Filter { since, until: a.until.clone(), project: a.project.clone() };
    let need_home = (a.agent != "codex" && a.dir.is_none()) || (a.agent != "claude" && a.codex_dir.is_none());
    let home = match home() {
        Some(h) => h,
        None if need_home => anyhow::bail!("cannot tell the home folder (HOME/USERPROFILE unset): pass --dir and --codex-dir"),
        None => PathBuf::new(),
    };
    let mut scans: Vec<(&'static str, Scan)> = Vec::new();
    if a.agent != "codex" {
        let dir = a.dir.clone().unwrap_or_else(|| home.join(".claude/projects"));
        scans.push(("claude", claude::scan_dir(&dir, &filter, a.max_line_bytes, a.large_context)?));
    }
    if a.agent != "claude" {
        let dir = a.codex_dir.clone().unwrap_or_else(|| home.join(".codex/sessions"));
        scans.push(("codex", codex::scan_dir(&dir, &filter, a.max_line_bytes)?));
    }
    let out = table::render(&scans, &a, &filter, &prices);
    let mut stdout = std::io::stdout();
    let _ = std::io::Write::write_all(&mut stdout, out.as_bytes());
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_line_skips_oversize_without_holding_it() {
        let data = format!("short\n{}\nafter\nlast-no-newline", "x".repeat(5000));
        let mut r = std::io::BufReader::with_capacity(64, data.as_bytes());
        let mut buf = Vec::new();
        let mut got = Vec::new();
        while let Some(ok) = read_line(&mut r, &mut buf, 1000).unwrap() {
            got.push(if ok { String::from_utf8_lossy(&buf).into_owned() } else { "<oversize>".into() });
            assert!(buf.capacity() <= 4096, "the long line was not buffered whole");
        }
        assert_eq!(got, ["short", "<oversize>", "after", "last-no-newline"]);
    }

    #[test]
    fn days_and_filters() {
        assert!(valid_day(&days_ago(0)) && valid_day(&days_ago(30)));
        assert!(days_ago(30) < days_ago(0));
        let f = Filter { since: "2026-11-01".into(), until: Some("2026-11-03".into()), project: Some("app".into()) };
        assert!(f.day_ok("2026-11-01") && f.day_ok("2026-11-03") && !f.day_ok("2026-10-31") && !f.day_ok("2026-11-04") && !f.day_ok(""));
        assert!(f.project_ok("/a/app") && f.project_ok("/b/app/") && !f.project_ok("/a/app2"));
        let f = Filter { project: Some("/a/app".into()), ..f };
        assert!(f.project_ok("/a/app") && f.project_ok("/a/app/sub") && !f.project_ok("/b/app") && !f.project_ok("/a/apple"));
        let f = Filter { project: Some(r"C:\work\app".into()), ..f };
        assert!(f.project_ok(r"C:\work\app") && f.project_ok(r"C:\work\app\sub") && f.project_ok("C:/work/app") && !f.project_ok(r"C:\work\apple"));
        let f = Filter { project: Some("app".into()), ..f };
        assert!(f.project_ok(r"C:\work\app") && f.project_ok(r"C:\work\app\") && !f.project_ok(r"C:\work\apple"));
    }

    #[test]
    fn utc_days() {
        assert_eq!(utc_day("2026-11-03T00:30:00+07:00"), "2026-11-02");
        assert_eq!(utc_day("2026-11-03T23:30:00.123-02:00"), "2026-11-04");
        assert_eq!(utc_day("2026-12-31T22:00:00-05:00"), "2027-01-01");
        assert_eq!(utc_day("2024-03-01T01:00:00+02:00"), "2024-02-29");
        assert_eq!(utc_day("2026-11-03T00:30:00Z"), "2026-11-03");
        assert_eq!(utc_day("2026-11-03T00:30:00.5Z"), "2026-11-03");
        assert_eq!(utc_day("2026-11-03"), "2026-11-03");
        assert_eq!(utc_day("garbage"), "");
        assert_eq!(utc_day(""), "");
        assert_eq!(utc_day("123456789é"), "", "no panic on a char boundary");
        assert_eq!(utc_day("2026-11-03Té:00"), "2026-11-03");
        assert_eq!(utc_day("2026-11-03T0é:00:00+07:00"), "2026-11-03");
        assert!(utc_key("2026-11-03T00:30:00+07:00") < utc_key("2026-11-02T23:00:00Z"));
        assert_eq!(utc_key("2026-11-03T00:30:05.25+07:00"), "2026-11-02T17:30:05.25");
        for d in [0i64, 59, 60, 365, 10_957, 20_000, -1, -719_468] {
            assert_eq!(utc_day(&civil_from_days(d)), civil_from_days(d));
        }
    }

    #[test]
    fn lines_count_bad_utf8_and_keep_ids_apart() {
        let data: &[u8] = b"{\"a\":1}\n\xff\xfe\n{\"b\":2}\n";
        let mut l = Lines { r: std::io::BufReader::new(data), buf: Vec::new(), max: 100, bad_utf8: 0, error: None };
        let got: Vec<_> = (&mut l).collect();
        assert_eq!(got, [Some("{\"a\":1}".to_string()), Some(String::new()), Some("{\"b\":2}".to_string())]);
        assert_eq!(l.bad_utf8, 1);
    }
}
