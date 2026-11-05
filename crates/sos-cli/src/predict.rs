//! `sos predict` — list written predictions and pass thresholds in a repo, with their age (wave 3.1;
//! port of `skills/depth-check/scripts/predictions.py`, which stays beside it as the reference).
//!
//! The depth-check skill asks whether each threshold written before a result was later graded
//! against the actual result. This finds candidate lines in Markdown and text files and dates them
//! (git blame, else a date in the path, else the file's mtime); the agent reading the output decides
//! which are real predictions and whether they were graded. Read-only; always exits 0.
//!
//! Text rules follow Python's `re`/`str` rules, so both versions score the same lines: `\s`
//! includes U+001C..U+001F, `\b` and `\w` count only letters, numbers and `_` as word characters,
//! `ı`/`İ` match `i` when case is ignored, lines split like `str.splitlines`, lengths count
//! characters. Intended differences: files whose names git would quote (non-ASCII) are read (the
//! script skipped them); dates are written with ASCII digits only; on Windows the dates are UTC
//! days, output lines end in LF, and outside git paths use `/` with only the file name checked for
//! history words (the script used `\`); `git blame` is not run on a file with a `filter`
//! attribute or in a partial clone, and git runs with `core.fsmonitor` off and without lazy
//! fetch, so no command from git config runs and nothing is fetched (those lines are dated by path
//! or mtime); numbers are 64-bit without `_`; exit 1 when stdout cannot be written (the script
//! raised).

use anyhow::Result;
use regex::Regex;
use serde_json::{json, Value};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

#[derive(clap::Args)]
pub struct Args {
    /// The repo to scan (default: the current folder).
    #[arg(default_value = ".")]
    repo: String,
    /// Only lines at least this many days old.
    #[arg(long, default_value_t = 0, allow_negative_numbers = true)]
    days: i64,
    /// Show at most this many lines (strongest first).
    #[arg(long, default_value_t = 60, allow_negative_numbers = true)]
    limit: i64,
    /// 4 (default) keeps bet word + outcome; 3 also catches bare 'kỳ vọng < N%' lines, with much more noise.
    #[arg(long = "min-score", default_value_t = 4, allow_negative_numbers = true)]
    min_score: i64,
    /// Print JSON instead of a Markdown table.
    #[arg(long)]
    json: bool,
}

/// Python's `\s` (str patterns): Unicode white space plus the separators U+001C..U+001F.
const S: &str = r"[\s\x1c-\x1f]";

struct Pats {
    keywords: Regex,
    follow_up: Regex,
    weak: Regex,
    outcome: Regex,
    window: Regex,
    comparator: Regex,
    number_unit: Regex,
    result_words: Regex,
    date: Regex,
    path_date: Regex,
    spaces: Regex,
    skip_files: Regex,
    py_word: Regex,
    rs_word: Regex,
}

impl Pats {
    fn new() -> Pats {
        let re = |p: &str| Regex::new(&p.replace(r"\s", S)).expect("pattern");
        Pats {
            // Words that mark a bet on a future result. "mục tiêu", "target" and "threshold" matched
            // mostly engineering specs in trial runs, and "kỳ vọng" / "expect" mostly test
            // expectations, so those count for little on their own.
            keywords: re(r"(?i)dự đoán|du-doan|giả thuyết|thước đo|đạt nếu|trượt nếu|nếu trượt|predict|hypothes|pass if|fail if|success metric|kill criteria|\b(build|làm|giữ|bỏ|dừng|ship|bật|tắt|mở) nếu"),
            follow_up: re(r"(?i)đo lại|đếm lại|đo hiệu quả|đọc lại số|re-?measure|follow-?up"),
            weak: re(r"(?i)kỳ vọng|expect|ngưỡng|threshold|KPI"),
            // What real users or markets do: the results a product prediction is about.
            outcome: re(r"(?i)view|xem TB|xem hết|lượt (tải|xem|mua)|tải về|download|mua|trial|retention|giữ chân|quay lại|signup|đăng ký|conversion|chuyển đổi|doanh thu|revenue|refund|hoàn tiền|win rate|expectancy|tỉ lệ|tỷ lệ|người dùng|users?\b|khách|comment|bình luận|lưu|chia sẻ|share|click|phiên|session"),
            window: re(r"(?i)@\s?\d+\s?h|\btuần \d|\d+\s?ngày|\bD\d{1,2}\b|\bweek|\d+\s?days"),
            // The script's `(?<![-=])>(?!=)` and `<(?![-!/a-z])`: only the look-behind can change a
            // match (what follows must be a space, ~, $ or digit anyway), and a match is only tested
            // for existence, so "a '>' at the start or after anything but - or =" is the same test.
            comparator: re(r"(?i)(≥|≤|>=|<=|(?:^|[^-=])>|<|tối thiểu|tối đa|ít nhất|không quá|trên|dưới|at least|at most|more than|less than|under|over)\s*~?\s*\$?\d"),
            number_unit: re(r"(?i)\d+([.,]\d+)?\s*(%|s\b|giây|ngày|tuần|views?|lượt|R\b|x\b)"),
            result_words: re(r"(?i)thực tế|kết quả|đã chấm|đạt|trượt|bị bác|đứng vững|actual|result|graded|passed|failed|\bPASS\b|\bFAIL\b|✅|❌"),
            date: re(r"\b(20[0-9][0-9])-([0-9][0-9])-([0-9][0-9])\b|\b([0-9]{1,2})/([0-9]{1,2})/(20[0-9][0-9])\b"),
            path_date: re(r"(20[0-9][0-9])-([0-9][0-9])-([0-9][0-9])|(20[0-9][0-9])-([0-9][0-9])-W([0-9])"),
            spaces: re(r"\s+"),
            skip_files: re(r"(?i)(changelog|history|lich-su|discoveries|depth-check)[^/]*$"),
            // Python's `\w`: `isalnum()` (letters and numbers) or `_`. The regex crate's `\w` also
            // counts marks, joiners and other connectors, which moves `\b`.
            py_word: Regex::new(r"^[\p{L}\p{N}_]$").unwrap(),
            rs_word: Regex::new(r"^\w$").unwrap(),
        }
    }

    /// The text the patterns run on: a character Python treats differently is swapped for one the
    /// regex crate treats the way Python does. Same characters count, digits untouched, so
    /// captured numbers and every yes/no answer are Python's.
    fn view<'a>(&self, s: &'a str) -> std::borrow::Cow<'a, str> {
        if s.is_ascii() {
            return s.into();
        }
        let mut buf = [0u8; 4];
        s.chars().map(|c| {
            if c.is_ascii() {
                return c;
            }
            if c == 'ı' || c == 'İ' {
                return 'i'; // Python's case-insensitive match pairs both with i
            }
            let cs: &str = c.encode_utf8(&mut buf);
            match (self.py_word.is_match(cs), self.rs_word.is_match(cs)) {
                (true, false) => 'ꙮ', // a letter both treat as a word character, in no pattern
                (false, true) => '¤', // a symbol neither treats as a word character or a space
                _ => c,
            }
        }).collect::<String>().into()
    }
}

const SKIP_DIRS: &[&str] = &["node_modules", ".git", "build", "dist", "target", "archive", "vendor",
    "deriveddata", ".venv", "venv", "__pycache__", ".next", "pods", "done",
    "references", "cold-storage", "runs", "evidence", "tmp", "scratch"];
const EXTS: &[&str] = &[".md", ".markdown", ".txt"];

fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\x1c'..='\x1f').contains(&c)
}

/// Python's `str.splitlines()`.
pub(crate) fn splitlines(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut it = s.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        if matches!(c, '\n' | '\r' | '\x0b' | '\x0c' | '\x1c' | '\x1d' | '\x1e' | '\u{85}' | '\u{2028}' | '\u{2029}') {
            out.push(&s[start..i]);
            let mut end = i + c.len_utf8();
            if c == '\r' && it.peek().is_some_and(|&(_, n)| n == '\n') {
                it.next();
                end += 1;
            }
            start = end;
        }
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
}

fn first_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

// ── dates as day numbers (days since 1970-01-01) ──

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn iso(days: i64) -> String {
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

/// `datetime.date(y, m, d)`: None where Python raises ValueError.
fn date(y: i64, m: i64, d: i64) -> Option<i64> {
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let dim = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    ((1..=9999).contains(&y) && (1..=dim).contains(&d)).then(|| days_from_civil(y, m, d))
}

/// The local calendar day of a Unix time (`date.fromtimestamp`). UTC where there is no localtime.
fn local_day(secs: i64) -> i64 {
    #[cfg(unix)]
    {
        let t = secs as libc::time_t;
        // SAFETY: localtime_r writes only into `tm`, which lives on this stack frame.
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        if !unsafe { libc::localtime_r(&t, &mut tm) }.is_null() {
            return days_from_civil(i64::from(tm.tm_year) + 1900, i64::from(tm.tm_mon) + 1, i64::from(tm.tm_mday));
        }
    }
    secs.div_euclid(86_400)
}

fn unix_secs(t: std::time::SystemTime) -> i64 {
    match t.duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => {
            let d = e.duration();
            -(d.as_secs() as i64) - i64::from(d.subsec_nanos() > 0)
        }
    }
}

// ── files ──

fn skipped(p: &Pats, path: &str) -> bool {
    let parts: Vec<&str> = path.split('/').collect();
    let in_skip_dir = parts[..parts.len() - 1].iter().any(|p| SKIP_DIRS.contains(&p.to_lowercase().as_str()));
    // History files and earlier depth-check reports restate predictions made elsewhere; a
    // prediction is graded where it was made. (The script: `(changelog|…)[^/]*$`, ignoring case.)
    in_skip_dir || p.skip_files.is_match(&p.view(parts[parts.len() - 1]))
}

/// Markdown and text files: git's list (tracked + untracked, not ignored), or a walk outside git.
fn list_files(p: &Pats, repo: &Path) -> Vec<String> {
    let out = git(repo)
        .args(["ls-files", "-z", "--cached", "--others", "--exclude-standard"]).output();
    let files: Vec<String> = match out {
        Ok(o) if o.status.success() => o.stdout.split(|&b| b == 0).filter(|b| !b.is_empty())
            .map(|b| String::from_utf8_lossy(b).into_owned()).collect(),
        _ => {
            let mut v = Vec::new();
            walk(repo, "", &mut v);
            v
        }
    };
    files.into_iter().filter(|f| EXTS.iter().any(|e| f.ends_with(e)) && !skipped(p, f)).collect()
}

/// `os.walk` order: a folder's files in directory order, then each subfolder in turn.
fn walk(root: &Path, rel: &str, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(root.join(rel)) else { return };
    let mut dirs = Vec::new();
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let r = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
        // Like os.walk: a link to a folder is listed as a folder but not entered.
        if e.path().is_dir() {
            let is_link = e.file_type().is_ok_and(|t| t.is_symlink());
            if !is_link && !name.starts_with('.') && !SKIP_DIRS.contains(&name.to_lowercase().as_str()) {
                dirs.push(r);
            }
        } else {
            out.push(r);
        }
    }
    for d in dirs {
        walk(root, &d, out);
    }
}

/// A read-only git call: no fsmonitor hook (a command from git config), no index refresh write, and
/// in a partial clone no lazy fetch of missing objects (git 2.44+; blame then fails and the line is
/// dated by path or mtime). `GIT_CONFIG` is dropped: it redirects only `git config`, so the checks
/// below would read another file than the one check-attr and blame use.
fn git(repo: &Path) -> Command {
    let mut c = Command::new("git");
    c.arg("-c").arg("core.fsmonitor=false").arg("-C").arg(repo)
        .env("GIT_OPTIONAL_LOCKS", "0").env("GIT_NO_LAZY_FETCH", "1").env_remove("GIT_CONFIG");
    c
}

/// Line number -> author day from one `git blame`; empty when untracked. Uncommitted lines are left out.
/// False in a partial clone: there blame, and check-attr reading a `.gitattributes` that is not
/// checked out, would fetch missing objects from the promisor remote (a network call and a write to
/// .git). GIT_NO_LAZY_FETCH in `git` stops that on git 2.44+; this check covers older git.
fn may_blame(repo: &Path) -> bool {
    let keys = r"^(extensions\.partialclone|remote\..*\.promisor)$";
    git(repo).args(["config", "-z", "--get-regexp", keys]).output()
        .is_ok_and(|o| o.status.code() == Some(1) && o.stdout.is_empty())
}

fn line_dates(repo: &Path, path: &str, may_blame: bool) -> std::collections::HashMap<usize, i64> {
    let mut dates = std::collections::HashMap::new();
    if !may_blame {
        return dates;
    }
    // Blame reads the working file through its clean filter, a command from git config. A
    // read-only scan runs none: a file with a filter is dated by path or mtime instead.
    // check-attr prints a driver named "unset" or "unspecified" like the state, so such a name
    // counts as a filter when git config defines that driver.
    let Ok(attr) = git(repo).args(["check-attr", "-z", "filter", "--", path]).output() else { return dates };
    let value = attr.stdout.split(|&b| b == 0).nth(2).unwrap_or(b"");
    if !attr.status.success() || !matches!(value, b"unspecified" | b"unset") {
        return dates;
    }
    let name = String::from_utf8_lossy(value);
    let Ok(cfg) = git(repo).args(["config", "-z", "--get-regexp", &format!(r"^filter\.{name}\.")]).output() else { return dates };
    if cfg.status.code() != Some(1) || !cfg.stdout.is_empty() {
        return dates; // defined (exit 0), or config could not be read
    }
    let Ok(o) = git(repo).args(["blame", "--no-textconv", "--line-porcelain", "--", path]).output() else { return dates };
    if !o.status.success() {
        return dates;
    }
    let out = String::from_utf8_lossy(&o.stdout);
    let (mut current, mut lineno, mut uncommitted) = (None, None, false);
    for row in splitlines(&out) {
        let b = row.as_bytes();
        if b.len() > 40 && b[40] == b' ' && b[..40].iter().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(c)) {
            lineno = row.split(' ').nth(2).and_then(|n| n.parse::<usize>().ok());
            uncommitted = b[..40].iter().all(|&c| c == b'0');
        } else if let Some(t) = row.strip_prefix("author-time ") {
            current = t.split(' ').next().and_then(|s| s.parse::<i64>().ok()).map(local_day);
        } else if row.starts_with('\t') && !uncommitted {
            if let (Some(n), Some(d)) = (lineno, current) {
                dates.insert(n, d);
            }
        }
    }
    dates
}

/// A date written in the path (2026-10-15, or a week like 2026-10-W3).
fn path_date(p: &Pats, path: &str) -> Option<i64> {
    let path = p.view(path);
    let c = p.path_date.captures(&path)?;
    let n = |i: usize| c.get(i).and_then(|m| m.as_str().parse::<i64>().ok());
    if c.get(1).is_some() {
        return date(n(1)?, n(2)?, n(3)?);
    }
    let w = n(6)?;
    date(n(4)?, n(5)?, 1).map(|d| d + 7 * (w - 1))
}

fn stated_dates(p: &Pats, text: &str) -> Vec<String> {
    let mut v = Vec::new();
    for c in p.date.captures_iter(&p.view(text)) {
        let n = |i: usize| c.get(i).and_then(|m| m.as_str().parse::<i64>().ok());
        let d = if c.get(1).is_some() { n(1).zip(n(2)).zip(n(3)).and_then(|((y, m), d)| date(y, m, d)) }
            else { n(6).zip(n(5)).zip(n(4)).and_then(|((y, m), d)| date(y, m, d)) };
        if let Some(d) = d {
            v.push(iso(d));
        }
    }
    v
}

struct Row {
    file: String,
    line: usize,
    heading: String,
    text: String,
    score: i64,
    result_nearby: bool,
    stated: Vec<String>,
    source: &'static str,
    written: i64,
    age: i64,
}

fn scan_file(p: &Pats, repo: &Path, path: &str, min_score: i64) -> Vec<Row> {
    let Ok(bytes) = std::fs::read(repo.join(path)) else { return Vec::new() };
    let Ok(body) = String::from_utf8(bytes) else { return Vec::new() };
    let lines = splitlines(&body);
    let (mut found, mut heading) = (Vec::new(), String::new());
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start_matches(py_space).starts_with('#') {
            heading = line.trim_matches(['#', ' ']).trim_matches(py_space).to_string();
            continue;
        }
        let text = line.trim_matches(py_space);
        if text.chars().count() < 8 || text.starts_with("```") || text.starts_with("<!--") {
            continue;
        }
        let (tv, hv) = (p.view(text), p.view(&heading));
        let cmp = p.comparator.is_match(&tv);
        if !(cmp || p.number_unit.is_match(&tv)) {
            continue;
        }
        let weak = p.weak.is_match(&tv);
        let score = 3 * i64::from(p.keywords.is_match(&tv)) + 2 * i64::from(p.keywords.is_match(&hv))
            + if weak && cmp { 2 } else { i64::from(weak) }
            + 2 * i64::from(p.outcome.is_match(&tv)) + i64::from(p.window.is_match(&tv))
            + i64::from(p.follow_up.is_match(&tv));
        if score < min_score {
            continue;
        }
        let window = lines.iter().skip(i + 1).take(15).copied().collect::<Vec<_>>().join(" ");
        found.push(Row {
            file: path.to_string(),
            line: i + 1,
            heading: first_chars(&heading, 60),
            text: first_chars(&p.spaces.replace_all(text, " "), 180),
            score,
            result_nearby: p.result_words.is_match(&p.view(&window)),
            stated: stated_dates(p, text),
            source: "git",
            written: 0,
            age: 0,
        });
    }
    found
}

/// `os.path.abspath`: absolute, with `.` and `..` folded without following links.
pub(crate) fn abspath(p: &Path) -> PathBuf {
    let abs = std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf());
    // POSIX keeps exactly two leading slashes (their meaning is up to the system); so does Python.
    let s = abs.to_string_lossy();
    let mut out = if cfg!(unix) && s.starts_with("//") && !s.starts_with("///") { PathBuf::from("//") } else { PathBuf::new() };
    for c in abs.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(out.components().next_back(), Some(Component::Normal(_))) {
                    out.pop();
                }
            }
            Component::RootDir if out.as_os_str() == "//" => {}
            other => out.push(other),
        }
    }
    out
}

/// Write all of stdout at once; a failed write (closed pipe, full disk) is exit 1, not a panic.
pub(crate) fn emit(out: &str, code: i32) -> i32 {
    use std::io::Write;
    let mut so = std::io::stdout().lock();
    match so.write_all(out.as_bytes()).and_then(|_| so.flush()) {
        Ok(()) => code,
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "sos: cannot write the output: {e}");
            1
        }
    }
}

pub fn run(a: Args) -> Result<i32> {
    let p = Pats::new();
    // Python's abspath('') is the current folder.
    let repo = abspath(Path::new(if a.repo.is_empty() { "." } else { &a.repo }));
    let today = local_day(unix_secs(std::time::SystemTime::now()));

    let blame_ok = may_blame(&repo);
    let mut rows = Vec::new();
    for path in list_files(&p, &repo) {
        let found = scan_file(&p, &repo, &path, a.min_score);
        if found.is_empty() {
            continue;
        }
        let blame = line_dates(&repo, &path, blame_ok);
        let mtime = std::fs::metadata(repo.join(&path)).and_then(|m| m.modified()).map(|t| local_day(unix_secs(t))).unwrap_or(today);
        let in_path = path_date(&p, &path);
        for mut row in found {
            // Uncommitted lines have no blame date; an edit refreshes mtime, so a date in the path
            // (content/2026-10-W1/...) is the better guess.
            (row.written, row.source) = match (blame.get(&row.line), in_path) {
                (Some(&d), _) => (d, "git"),
                (None, Some(d)) => (d, "path"),
                (None, None) => (mtime, "mtime"),
            };
            row.age = today - row.written;
            if row.age >= a.days {
                rows.push(row);
            }
        }
    }
    rows.sort_by_key(|r| (-r.score, r.result_nearby, -r.age));
    let n = if a.limit >= 0 { (a.limit as usize).min(rows.len()) } else { rows.len().saturating_sub(a.limit.unsigned_abs() as usize) };
    let shown = &rows[..n];
    let repo_s = repo.display().to_string();

    if a.json {
        let list: Vec<Value> = shown.iter().map(|r| json!({
            "file": r.file, "line": r.line, "heading": r.heading, "text": r.text, "score": r.score,
            "result_nearby": r.result_nearby, "stated_dates": r.stated, "date_source": r.source,
            "written": iso(r.written), "age_days": r.age,
        })).collect();
        let doc = json!({"repo": repo_s, "today": iso(today), "total": rows.len(), "rows": list});
        let mut buf = Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, serde_json::ser::PrettyFormatter::with_indent(b" "));
        serde::Serialize::serialize(&doc, &mut ser)?;
        return Ok(emit(&format!("{}\n", String::from_utf8_lossy(&buf)), 0));
    }
    use std::fmt::Write;
    let mut out = String::new();
    writeln!(out, "# Candidate predictions and thresholds — {repo_s}")?;
    writeln!(out, "{} candidate lines at least {} days old; showing {}, strongest first (bet word + user/market outcome + time window), \
then no result words nearby, then oldest. Candidates only: open each line and judge whether it is a prediction and whether it was graded.\n",
        rows.len(), a.days, shown.len())?;
    writeln!(out, "| File:line | Score | Written | Age (d) | Result words nearby | Heading | Text |")?;
    writeln!(out, "|---|---|---|---|---|---|---|")?;
    for r in shown {
        let src = if r.source == "git" { String::new() } else { format!(" ({})", r.source) };
        writeln!(out, "| {}:{} | {} | {}{src} | {} | {} | {} | {} |", r.file, r.line, r.score, iso(r.written), r.age,
            if r.result_nearby { "yes" } else { "no" }, r.heading.replace('|', "/"), r.text.replace('|', "\\|"))?;
    }
    Ok(emit(&out, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitlines_like_python() {
        assert_eq!(splitlines("a\r\nb\rc\x1cd\u{2028}e\n"), ["a", "b", "c", "d", "e"]);
        assert_eq!(splitlines(""), Vec::<&str>::new());
        assert_eq!(splitlines("\n\n"), ["", ""]);
        assert_eq!(splitlines("x\x1fy"), ["x\x1fy"]); // U+001F is a space for strip(), not a line break
    }

    #[test]
    fn dates() {
        assert_eq!(iso(days_from_civil(2026, 11, 5)), "2026-11-05");
        assert_eq!(date(2026, 2, 29), None);
        assert_eq!(date(2028, 2, 29).map(iso).as_deref(), Some("2028-02-29"));
        let p = Pats::new();
        assert_eq!(path_date(&p, "content/2026-10-W3/a.md").map(iso).as_deref(), Some("2026-10-15"));
        assert_eq!(path_date(&p, "x/2026-13-01-y/2026-10-01.md"), None); // first match only, as the script
        assert_eq!(stated_dates(&p, "due 2026-10-05 or 7/11/2026, not 2026-02-30"), ["2026-10-05", "2026-11-07"]);
    }

    #[test]
    fn comparator_look_behind() {
        let p = Pats::new();
        assert!(p.comparator.is_match("> 5"));
        assert!(p.comparator.is_match("x >5"));
        assert!(!p.comparator.is_match("a -> 5"));
        assert!(!p.comparator.is_match("a => 5"));
        assert!(p.comparator.is_match("a ->> 5")); // the second '>' follows '>'
        assert!(p.comparator.is_match("at least\x1f5")); // \s includes U+001F, as in Python
    }

    #[test]
    fn skip_rules() {
        let p = Pats::new();
        assert!(skipped(&p, "docs/archive/a.md"));
        assert!(skipped(&p, "Node_Modules/x/readme.md"));
        assert!(skipped(&p, "docs/CHANGELOG.md"));
        assert!(skipped(&p, "notes/depth-check-2026.md"));
        assert!(!skipped(&p, "history/plan.md")); // only the file name is checked for history words
        assert!(!skipped(&p, "docs/plan.md"));
        assert!(skipped(&p, "notes/hiſtory.md")); // long s matches s, as in Python's re
        assert!(skipped(&p, "notes/HİSTORY.md"));
    }

    #[test]
    fn word_and_case_rules_like_python() {
        let p = Pats::new();
        assert!(p.keywords.is_match(&p.view("predıct > 5 users")));
        assert!(p.outcome.is_match(&p.view("predict > 5 users\u{200D}"))); // a joiner ends a word in Python
        assert!(p.outcome.is_match(&p.view("5 users\u{301} x")) ); // so does a combining mark
        assert!(!p.window.is_match(&p.view("D12²"))); // ² is a word character in Python
        assert_eq!(stated_dates(&p, "hạn 2026-10-05"), ["2026-10-05"]);
    }

    #[cfg(unix)]
    #[test]
    fn abspath_keeps_two_leading_slashes() {
        assert_eq!(abspath(Path::new("//tmp/a/../b")), PathBuf::from("//tmp/b"));
        assert_eq!(abspath(Path::new("///tmp/b")), PathBuf::from("/tmp/b"));
    }
}
