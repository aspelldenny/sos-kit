//! `sos bite` — do the tests of a change catch a wrong version of that change? (Rust wave 2.2;
//! the same tool as `scripts/bite`, which stays beside it until the apps move over.)
//!
//! Two measurements, in a throwaway worktree at HEAD (the checkout is never touched; the result is
//! written to evidence/bite/): the new or edited tests run against the old source ("before"), and
//! up to N one-token mutants of the added code lines run against the tests. `--audit` mutates all
//! of --src at HEAD and lists the tests that add no kill. LONG_ABOUT has the why.
//!
//! Python's `re` has lookaround and backreferences, Rust's `regex` has not: the mutators that need
//! them, the JS test title and the TAP block are matched by hand below, each next to the pattern
//! it replaces.

use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Serialize;
use serde_json::{json, Map, Value};
use sos_rules::shell::py_space;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{mpsc, LazyLock};
use std::time::{Duration, Instant};

pub const LONG_ABOUT: &str = "\
Do the tests of a change catch a wrong version of that change?

  sos bite [--base REF] [--head REF] [--test CMD] [--cwd DIR] [--src REGEX] [--tests REGEX] [--mutants N]
  sos bite --audit [--head REF] [--test CMD] [--src REGEX] [--tests REGEX] [--mutants N]

Why (Reel, 18/10/2026): 21 of 578 agent-written tests could not catch any bug (copied constants,
asserting what the test itself set, duplicate proofs); the contract's test rule was words, and the
reviewer only read code. This turns the rule into numbers, in a throwaway worktree at HEAD (source
and tests in the checkout are never touched; the result is written to evidence/bite/):

  1. before: the changed source files go back to BASE, the tests stay at HEAD. A new or edited test
     that still passes there proves nothing about this change (fine for a test that guards old
     behaviour on purpose; say so).
  2. mutants: up to N one-token edits (== -> !=, < -> >=, and -> or, True -> False, n -> n+1 ...)
     on code lines the change added, outside comments, strings and docstrings. A mutant no test
     kills is a changed line no test checks (or an equivalent edit; say why). A mutant after which
     no test ran (it broke the build) counts as neither.

  --audit: no change, the tests that already exist. Up to N mutants (default 30) spread over all code
     lines of --src at HEAD; lists tests that killed none, and tests whose kills another test also made.
     Why: OpenClaw (10/2026) reported deleting ~400k lines of its own tests with little coverage change;
     coverage counts lines run, not lines checked. These are candidates for review, not deletions: narrow
     --src and --test to one module so the sample means something.

Settings, strongest first: flags, `.sos.toml` [bite] (test, cwd, src, tests, mutants), defaults
(base: the merge base with origin/HEAD, else main; mutants 8). --src and --tests are Rust regexes
(no lookaround or backreferences). Per-test results are read from `python -m unittest -v`,
`pytest -v`, XCTest, `node --test`, `vitest --reporter=verbose`, `jest --verbose`, TAP and
`cargo test` output; with another runner the \"before\" check sees only the suite verdict and a
mutant counts as killed whenever the run fails. The test command runs in sh (Git's bash on
Windows) with stdin closed and a 900 s limit per run.
Exit 0 measured, 1 not measurable (tests red at HEAD, nothing changed, git error), 2 not configured,
130 stopped by a signal (the worktree is still removed).";

#[derive(clap::Args)]
pub struct Args {
    /// The commit before the change (default: the merge base with origin/HEAD, else main).
    #[arg(long)]
    base: Option<String>,
    /// The change's last commit (default HEAD).
    #[arg(long)]
    head: Option<String>,
    /// Test command, run by sh in the worktree (default .sos.toml [bite] test).
    #[arg(long)]
    test: Option<String>,
    /// Folder inside the repo the test command runs in (default ".").
    #[arg(long)]
    cwd: Option<String>,
    /// Regex for source files to measure.
    #[arg(long)]
    src: Option<String>,
    /// Regex for test files.
    #[arg(long)]
    tests: Option<String>,
    /// Most mutants to run (default 8; 30 with --audit).
    #[arg(long, allow_negative_numbers = true)]
    mutants: Option<i64>,
    /// Mutate all of --src at HEAD and list tests that add no kill.
    #[arg(long)]
    audit: bool,
    /// Parser check for tests/bite_parsers.py: results | test-names | mutants, input on stdin.
    #[arg(long, hide = true)]
    internal: Option<String>,
    #[arg(long, hide = true)]
    internal_dir: Option<PathBuf>,
}

/// {file: [(line number at HEAD, text)]}, in diff (or ls-files) order.
type Lines = Vec<(String, Vec<(usize, String)>)>;
/// (file, line number, old text, new text).
type Mutant = (String, usize, String, String);

/// Set by SIGTERM/SIGHUP/SIGINT (a console close or Ctrl+C on Windows): stop the test run and
/// remove the worktree, then exit 130.
static STOP: AtomicBool = AtomicBool::new(false);
/// Set once the worktree is gone, so the Windows close handler may let the process end.
static CLEANED: AtomicBool = AtomicBool::new(false);
const TIMEOUT: Duration = Duration::from_secs(900);
const DIFF: [&str; 6] = ["diff", "--no-color", "--no-ext-diff", "--no-renames", "--src-prefix=a/", "--dst-prefix=b/"];
const CACHES: [&str; 5] = [".cache", ".vite", ".vitest", ".vite-temp", ".tmp"];

/// The run was stopped by a signal: no result, exit 130.
#[derive(Debug)]
struct Stopped;
impl std::fmt::Display for Stopped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("stopped")
    }
}
impl std::error::Error for Stopped {}

/// A built-in pattern with Python's `str` classes: `\w` is a letter, number or `_` (Python's
/// isalnum; Rust's own `\w` also takes combining marks and leaves out `²`), `\s` adds \x1c-\x1f
/// (Python's isspace). `\b` is not used: each boundary is written out by hand.
fn re(p: &str) -> Regex {
    let mut out = String::new();
    let mut cs = p.chars();
    while let Some(c) = cs.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match cs.next() {
            Some('w') => out.push_str(r"[\p{L}\p{N}_]"),
            Some('W') => out.push_str(r"[^\p{L}\p{N}_]"),
            Some('s') => out.push_str(r"[\s\x1c-\x1f]"),
            Some('S') => out.push_str(r"[^\s\x1c-\x1f]"),
            Some('b') => panic!("built-in pattern uses \\b: {p}"),
            Some(x) => {
                out.push('\\');
                out.push(x);
            }
            None => out.push('\\'),
        }
    }
    Regex::new(&out).expect("built-in pattern")
}

/// Python's `\w` for one character (see [`re`]).
fn py_word(c: char) -> bool {
    static W: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\A[\p{L}\p{N}_]\z").expect("built-in pattern"));
    c.is_ascii_alphanumeric() || c == '_' || (!c.is_ascii() && W.is_match(c.encode_utf8(&mut [0; 4])))
}

/// The first `w` with no word character before it and, when `end`, none after it
/// (Python: `\bw\b`, or `\bw` when not `end`).
fn word_at(code: &str, w: &str, end: bool) -> Option<(usize, usize)> {
    code.match_indices(w).map(|(p, _)| (p, p + w.len())).find(|&(s, e)| {
        let joined_before = code[..s].chars().next_back().is_some_and(py_word);
        let joined_after = end && code[e..].chars().next().is_some_and(py_word);
        !joined_before && !joined_after
    })
}

static SKIP: LazyLock<Regex> = LazyLock::new(|| {
    re(r#"^\s*(#|//|/\*|\*(?:\s|/|$)|import |from \S+ import|use |"""|'''|@|print\(|(?:log|logger|logging|os_log|NSLog|console)[.(]|(?:e?println|dbg)!)"#)
});
/// (word, whole word, replacement): Python `\band\b` → or, …, `\bnot ` → "".
const WORDS: [(&str, bool, &str); 7] = [("and", true, "or"), ("or", true, "and"), ("True", true, "False"),
    ("False", true, "True"), ("true", true, "false"), ("false", true, "true"), ("not ", false, "")];
static CLASS: LazyLock<Regex> = LazyLock::new(|| {
    re(r"^\s*(?:@\w+(?:\([^)]*\))?\s+)*(?:(?:public|internal|private|fileprivate|open|final)\s+)*class\s+(\w+)")
});
static TEST_FN: LazyLock<Regex> = LazyLock::new(|| {
    re(r"^\s*(?:@\w+(?:\([^)]*\))?\s+)*(?:(?:public|internal|override|final)\s+)*(?:async\s+)?(?:def|func)\s+(test\w*)")
});
// node:test / vitest / jest: it('title', …) or test("title", …); the title's end is found by hand
// (Python: `(.*?)(?<!\\)\1`). Rust: fn after #[test] or #[tokio::test].
static JS_TEST: LazyLock<Regex> =
    LazyLock::new(|| re(r#"^\s*(?:it|test)(?:\.(?:only|skip|todo|concurrent|fails))?\(\s*(['"`])"#));
static RUST_FN: LazyLock<Regex> = LazyLock::new(|| re(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(\w+)"));
static RUST_TEST: LazyLock<Regex> = LazyLock::new(|| re(r"^\s*#\[(?:\w+::)*test(?:\W|$)"));
static FAILING: LazyLock<Regex> = LazyLock::new(|| re(r"(?m)^✖ failing tests:$"));
static SUITE: LazyLock<Regex> = LazyLock::new(|| re(r"(?m)^\s*▶ (.+)$"));
static NAMED: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?m)^(\s*)([✓✔√]|[×✕✖]) (.+?)(?: \(?\d+(?:\.\d+)? ?m?s\)?)?[ \t]*$"));
// The TAP header; its indented block (Python: `((?:\1 +.*\n)*)`) is read by hand.
static TAP: LazyLock<Regex> = LazyLock::new(|| re(r"(?m)^(\s*)(not ok|ok) \d+ - (.+?)(\s+#\s*(?:SKIP|TODO)(?:[^\p{L}\p{N}_\n].*)?)?\n"));
static CARGO: LazyLock<Regex> = LazyLock::new(|| re(r"(?m)^test (\S+) \.\.\. (ok|FAILED)$"));
static LOAD_FAIL: LazyLock<Regex> = LazyLock::new(|| re(r"\A(?:\S+|.*/.*|.*\.(?:test|spec))\.[cm]?[jt]sx?\z"));
static UNITTEST: LazyLock<Regex> = LazyLock::new(|| {
    re(r"(?m)^(test\w+) \(([^)]*)\)(?:\n[^\n]*?)? \.\.\. (ok|FAIL|ERROR|skipped|expected failure)")
});
static UNITTEST_FAIL: LazyLock<Regex> = LazyLock::new(|| re(r"(?m)^(?:FAIL|ERROR): (test\w+) \(([^)]*)\)"));
static PYTEST: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?m)^\S+?::(?:(\w+)::)?(test\w*)\S* (PASSED|FAILED|ERROR|XFAIL|XPASS)"));
static XCTEST: LazyLock<Regex> =
    LazyLock::new(|| re(r"Test [Cc]ase '-?\[?(?:\w+\.)?(\w+)[ .](test\w*)(?:\(\))?\]?' (passed|failed)"));

/// Output as Python's text mode reads it: `\r\n` and a lone `\r` are line ends.
fn nl(s: String) -> String {
    if s.contains('\r') {
        s.replace("\r\n", "\n").replace('\r', "\n")
    } else {
        s
    }
}

fn blank(l: &str) -> bool {
    l.trim_matches(py_space).is_empty()
}

fn indent(l: &str) -> usize {
    l.chars().take_while(|&c| py_space(c)).count()
}

/// Is `pos` (a byte index) inside a '…', "…" or `…` string that starts earlier on the line?
fn in_string(line: &str, pos: usize) -> bool {
    let (mut q, mut prev) = (None, None);
    for c in line[..pos].chars() {
        if matches!(c, '"' | '\'' | '`') && prev != Some('\\') {
            q = match q {
                Some(x) if x == c => None,
                None => Some(c),
                other => other,
            };
        }
        prev = Some(c);
    }
    q.is_some()
}

/// Byte index where a trailing # or // comment starts (outside strings), else the line's length.
fn code_end(line: &str) -> usize {
    let b = line.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let w = if b[i] == b'#' {
            1
        } else if b[i] == b'/' && b.get(i + 1) == Some(&b'/') {
            2
        } else {
            0
        };
        if w > 0 && !in_string(line, i) {
            return i;
        }
        i += w.max(1);
    }
    b.len()
}

/// 1-based line numbers inside triple-quoted blocks (Python docstrings, Swift multi-line strings).
fn docstring_lines(text: &str) -> HashSet<usize> {
    let (mut inside, mut out) = (false, HashSet::new());
    for (i, l) in text.split('\n').enumerate() {
        let k = l.matches("\"\"\"").count() + l.matches("'''").count();
        if inside || k > 0 {
            out.insert(i + 1);
        }
        if k % 2 == 1 {
            inside = !inside;
        }
    }
    out
}

/// The first match of a 2-byte operator whose neighbours pass `ok(before, after)`.
fn op_at(code: &str, op: &[u8; 2], ok: impl Fn(Option<u8>, Option<u8>) -> bool) -> Option<(usize, usize)> {
    let b = code.as_bytes();
    (0..b.len().saturating_sub(1))
        .find(|&p| b[p] == op[0] && b[p + 1] == op[1] && ok(p.checked_sub(1).map(|q| b[q]), b.get(p + 2).copied()))
        .map(|p| (p, p + 2))
}

/// The first `c` with whitespace on both sides (Python: `(?<=\s)c(?=\s)`).
fn spaced(code: &str, c: char) -> Option<(usize, usize)> {
    code.char_indices()
        .find(|&(p, x)| {
            x == c
                && code[..p].chars().next_back().is_some_and(py_space)
                && code[p + 1..].chars().next().is_some_and(py_space)
        })
        .map(|(p, _)| (p, p + 1))
}

/// The first whole number (Python: `(?<![\w.])(\d+)(?![\w.])`) and the number plus one.
fn number(code: &str) -> Option<(usize, usize, String)> {
    let word = |c: char| py_word(c) || c == '.';
    let cs: Vec<(usize, char)> = code.char_indices().collect();
    let mut i = 0;
    while i < cs.len() {
        if !cs[i].1.is_ascii_digit() {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < cs.len() && cs[j].1.is_ascii_digit() {
            j += 1;
        }
        let before = i.checked_sub(1).map(|k| cs[k].1);
        let after = cs.get(j).map(|x| x.1);
        let (s, e) = (cs[i].0, cs.get(j).map_or(code.len(), |x| x.0));
        if !before.is_some_and(word) && !after.is_some_and(word) {
            return Some((s, e, plus_one(&code[s..e])));
        }
        i = j;
    }
    None
}

/// Decimal digits plus one, as Python's str(int(s) + 1) prints it ("007" -> "8").
fn plus_one(s: &str) -> String {
    let t = s.trim_start_matches('0');
    let mut d: Vec<u8> = if t.is_empty() { vec![b'0'] } else { t.bytes().collect() };
    let mut i = d.len();
    loop {
        if i == 0 {
            d.insert(0, b'1');
            break;
        }
        i -= 1;
        if d[i] == b'9' {
            d[i] = b'0';
        } else {
            d[i] += 1;
            break;
        }
    }
    String::from_utf8(d).unwrap_or_default()
}

/// The first mutator (in Python's MUTATORS order) whose first match is outside a string:
/// (start, end, replacement).
fn mutate(code: &str) -> Option<(usize, usize, String)> {
    let lit = |s: &str| code.find(s).map(|p| (p, p + s.len()));
    let fixed: [(Option<(usize, usize)>, &str); 8] = [
        (lit("==="), "!=="),
        (lit("!=="), "==="),
        (op_at(code, b"==", |a, z| !matches!(a, Some(b'!' | b'=')) && z != Some(b'=')), "!="),
        (op_at(code, b"!=", |_, z| z != Some(b'=')), "=="),
        (lit("<="), ">"),
        (lit(">="), "<"),
        (spaced(code, '<'), ">="),
        (spaced(code, '>'), "<="),
    ];
    let words = |k: usize| word_at(code, WORDS[k].0, WORDS[k].1).map(|m| (m, WORDS[k].2));
    let tries = fixed.into_iter().map(|(m, r)| m.map(|m| (m, r)))
        .chain([words(0), words(1), lit("&&").map(|m| (m, "||")), lit("||").map(|m| (m, "&&"))])
        .chain((2..7).map(words))
        .chain([spaced(code, '+').map(|m| (m, "-")), spaced(code, '-').map(|m| (m, "+"))]);
    for ((s, e), rep) in tries.flatten() {
        if !in_string(code, s) {
            return Some((s, e, rep.to_string()));
        }
    }
    number(code).filter(|&(s, _, _)| !in_string(code, s))
}

/// File text, or None when it is not UTF-8 (such files are left alone).
fn read(p: &Path) -> Option<String> {
    String::from_utf8(fs::read(p).ok()?).ok()
}

fn is_symlink(p: &Path) -> bool {
    fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink())
}

fn mutants(lines: &Lines, limit: i64, root: &Path) -> Vec<Mutant> {
    let rootc = fs::canonicalize(root).ok();
    let mut cands = Vec::new();
    for (f, ls) in lines {
        let p = root.join(f);
        // Writing a link, or a file whose path leaves the worktree, would change a file outside it.
        let below = match (&rootc, fs::canonicalize(&p)) {
            (Some(r), Ok(c)) => c != *r && c.starts_with(r),
            _ => false,
        };
        if is_symlink(&p) || !below {
            continue;
        }
        let Some(all) = read(&p) else { continue };
        let doc = docstring_lines(&all);
        for (n, text) in ls {
            if blank(text) || SKIP.is_match(text) || doc.contains(n) {
                continue;
            }
            let code = &text[..code_end(text)];
            if let Some((s, e, new)) = mutate(code) {
                cands.push((f.clone(), *n, text.clone(), format!("{}{new}{}", &text[..s], &text[e..])));
            }
        }
    }
    if limit <= 0 {
        return Vec::new();
    }
    let limit = limit as usize;
    if cands.len() <= limit {
        return cands;
    }
    let step = cands.len() as f64 / limit as f64; // spread over the change, deterministic
    (0..limit).map(|i| cands[(i as f64 * step) as usize].clone()).collect()
}

/// The JS test title after the opening quote at `open` (Python: `(.*?)(?<!\\)\1`).
fn js_title(line: &str, open: usize) -> Option<&str> {
    let q = line[open..].chars().next()?;
    let body = open + q.len_utf8();
    let mut prev = q;
    for (k, c) in line[body..].char_indices() {
        if c == q && prev != '\\' {
            return Some(&line[body..body + k]);
        }
        prev = c;
    }
    None
}

/// Class.test for every test function the change added or edited (".test" when not in a class).
fn test_names(added: &Lines, root: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (f, ls) in added {
        let Some(text) = read(&root.join(f)) else { continue };
        let src: Vec<&str> = text.split('\n').collect();
        let touched: HashSet<usize> = ls.iter().map(|(n, _)| *n).collect();
        for (i0, l) in src.iter().enumerate() {
            let i = i0 + 1;
            let m = TEST_FN.captures(l);
            let mut key = None;
            if m.is_none() {
                if let Some(j) = JS_TEST.captures(l) {
                    key = js_title(l, j.get(1).map_or(0, |g| g.start())).map(|t| format!(".{t}"));
                }
                if key.is_none() {
                    if let Some(r) = RUST_FN.captures(l) {
                        if src[i.saturating_sub(4)..i - 1].iter().any(|x| RUST_TEST.is_match(x)) {
                            key = Some(format!(".{}", &r[1]));
                        }
                    }
                }
            }
            if m.is_none() && key.is_none() {
                continue;
            }
            // The test ends at the next non-blank line indented no deeper than its own header.
            let mut end = (i + 1..=src.len())
                .find(|&j| {
                    let s = src[j - 1];
                    let t = s.trim_matches(py_space);
                    !t.is_empty() && indent(s) <= indent(l) && !t.starts_with(['}', ')'])
                })
                .unwrap_or(src.len() + 1);
            while end - 1 > i && blank(src[end - 2]) {
                end -= 1; // trailing blank lines are not the test
            }
            if !(i..end).any(|n| touched.contains(&n)) {
                continue;
            }
            if let Some(k) = key {
                out.insert(k);
                continue;
            }
            let mut cls = "";
            for j in (0..i - 1).rev() {
                let c = CLASS.captures(src[j]);
                if let Some(c) = &c {
                    if indent(src[j]) < indent(l) {
                        cls = c.get(1).map_or("", |g| g.as_str());
                        break;
                    }
                }
                if c.is_none() && !blank(src[j]) && indent(src[j]) == 0 {
                    break; // left the enclosing block: a top-level test function
                }
            }
            out.insert(format!("{cls}.{}", &m.expect("test fn")[1]));
        }
    }
    out
}

/// node:test, vitest, jest, TAP and cargo test output -> {title: [passed?, …]}, one entry per run of
/// that title. node prints a failing test again under "failing tests:"; a suite ("▶ name", TAP type
/// 'suite') and a file that did not load ("✖ test/a.test.mjs") are not tests. results() gives no
/// verdict to a title run twice.
fn leaf_runs(out: &str) -> HashMap<String, Vec<bool>> {
    let once = FAILING.find(out).map_or(out, |m| &out[..m.start()]);
    let suites: HashSet<&str> = SUITE.captures_iter(once).filter_map(|c| c.get(1)).map(|g| g.as_str()).collect();
    let mut named: Vec<(String, bool)> = Vec::new();
    for c in NAMED.captures_iter(once) {
        if !suites.contains(&c[3]) {
            let title = c[3].rsplit(" > ").next().unwrap_or("").to_string();
            named.push((title, "✓✔√".contains(&c[2])));
        }
    }
    let text = format!("{out}\n");
    let mut pos = 0;
    while let Some(c) = TAP.captures_at(&text, pos) {
        let whole = c.get(0).expect("match");
        let ind = c.get(1).map_or("", |g| g.as_str());
        let mut end = whole.end();
        loop {
            let rest = &text[end..];
            match rest.strip_prefix(ind).filter(|r| r.starts_with(' ')).and_then(|r| r.find('\n')) {
                Some(k) => end += ind.len() + k + 1,
                None => break,
            }
        }
        if c.get(4).is_none() && !text[whole.end()..end].contains("type: 'suite'") {
            named.push((c[3].to_string(), &c[2] == "ok"));
        }
        pos = end.max(whole.start() + 1);
    }
    for c in CARGO.captures_iter(out) {
        named.push((c[1].rsplit("::").next().unwrap_or("").to_string(), &c[2] == "ok"));
    }
    let mut seen: HashMap<String, Vec<bool>> = HashMap::new();
    for (title, ok) in named {
        if !LOAD_FAIL.is_match(&title) {
            seen.entry(title).or_default().push(ok); // else a file that did not load
        }
    }
    seen
}

fn cls(group: &str, name: &str) -> String {
    let parts: Vec<&str> = group.split('.').collect();
    let last = parts.len() - 1;
    if parts[last] == name && parts.len() > 1 { parts[last - 1] } else { parts[last] }.to_string()
}

/// What a run says: per-test verdicts, whether it has any, and whether some test failed (a kill)
/// as opposed to a red run in which no test failed (a broken build).
struct Verdicts {
    results: HashMap<String, bool>,
    has: bool,
    any_failed: bool,
}

/// unittest -v / pytest -v / XCTest lines -> {Class.test: passed?}; node:test, vitest, jest, TAP and
/// cargo test lines -> {.title: passed?} (a title seen twice gets no verdict).
fn verdicts(out: &str) -> Verdicts {
    let mut r: HashMap<String, bool> = HashMap::new();
    // unittest: "test_x (mod.Class.test_x) ... ok" (3.11+) or "(mod.Class)"; a docstring puts a line between.
    for m in UNITTEST.captures_iter(out) {
        r.insert(format!("{}.{}", cls(&m[2], &m[1]), &m[1]), &m[3] == "ok");
    }
    for m in UNITTEST_FAIL.captures_iter(out) {
        r.insert(format!("{}.{}", cls(&m[2], &m[1]), &m[1]), false); // subTest failures
    }
    // pytest -v: "tests/test_a.py::TestC::test_x PASSED" or "tests/test_a.py::test_x[param] FAILED"
    for m in PYTEST.captures_iter(out) {
        let key = format!("{}.{}", m.get(1).map_or("", |g| g.as_str()), &m[2]);
        let prev = *r.get(&key).unwrap_or(&true);
        r.insert(key, prev && &m[3] == "PASSED");
    }
    // XCTest: "Test Case '-[Mod.Class testA]' passed", "Test case '-[Mod.Class testA()]' passed on …", Linux "'Class.testA'"
    for m in XCTEST.captures_iter(out) {
        r.insert(format!("{}.{}", &m[1], &m[2]), &m[3] == "passed");
    }
    let runs = leaf_runs(out);
    for (t, oks) in &runs {
        if oks.len() == 1 {
            r.insert(format!(".{t}"), oks[0]);
        }
    }
    let has = !r.is_empty() || !runs.is_empty();
    let any_failed = r.values().any(|ok| !ok) || runs.values().any(|oks| oks.contains(&false));
    Verdicts { results: r, has, any_failed }
}

#[derive(Debug)]
struct Settings {
    test: String,
    cwd: String,
    src: String,
    tests: String,
    mutants: i64,
}

/// "message (at line L, column C)", the way Python's tomllib reports where.
fn toml_where(text: &str, e: &toml::de::Error) -> String {
    let msg = e.message().trim().to_string();
    match e.span() {
        Some(s) => {
            let before = &text[..s.start.min(text.len())];
            let line = before.matches('\n').count() + 1;
            let col = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
            format!("{msg} (at line {line}, column {col})")
        }
        None => msg,
    }
}

/// .sos.toml [bite] over the defaults. Err((exit code, message)).
fn settings(root: &Path) -> std::result::Result<Settings, (i32, String)> {
    let mut s = Settings { test: String::new(), cwd: ".".into(), src: String::new(), tests: String::new(), mutants: 8 };
    let cfg = root.join(".sos.toml");
    if !cfg.exists() {
        return Ok(s);
    }
    let bytes = fs::read(&cfg).map_err(|e| (1, format!("bite: cannot read .sos.toml: {e}")))?;
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let doc: toml::Table = text.parse().map_err(|e: toml::de::Error| {
        (1, format!("bite: .sos.toml is not valid TOML ({}); `sos check` shows where", toml_where(&text, &e)))
    })?;
    let Some(bite) = doc.get("bite") else { return Ok(s) };
    let Some(bite) = bite.as_table() else { return Err((2, "bite: .sos.toml [bite] must be a table".into())) };
    for (k, v) in bite {
        let slot = match k.as_str() {
            "test" => &mut s.test,
            "cwd" => &mut s.cwd,
            "src" => &mut s.src,
            "tests" => &mut s.tests,
            "mutants" => {
                s.mutants = match v {
                    toml::Value::Integer(n) => *n,
                    toml::Value::Float(x) if x.is_finite() => x.trunc() as i64,
                    toml::Value::String(t) if t.trim().parse::<i64>().is_ok() => t.trim().parse().unwrap_or(8),
                    _ => return Err((2, format!("bite: .sos.toml [bite] mutants must be a number, not {v}"))),
                };
                continue;
            }
            _ => continue,
        };
        match v.as_str() {
            Some(t) => *slot = t.to_string(),
            None => return Err((2, format!("bite: .sos.toml [bite] {k} must be a string, not {v}"))),
        }
    }
    Ok(s)
}

/// Python's repr() of a str, for messages that quote a setting.
fn py_repr(s: &str) -> String {
    let q = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
    let mut out = String::from(q);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == q => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push(q);
    out
}

/// `git args` in `cwd`; its stdout, line ends as Python's text mode reads them.
fn git(args: &[&str], cwd: &Path) -> Result<String> {
    let o = Command::new("git").args(args).current_dir(cwd).stdin(Stdio::null()).output()
        .context("starting git")?;
    if !o.status.success() {
        let err = nl(String::from_utf8_lossy(&o.stderr).into_owned());
        let err: String = err.trim_matches(py_space).chars().take(300).collect();
        bail!("git failed: git {}: {err}", args.join(" "));
    }
    if STOP.load(Ordering::SeqCst) {
        return Err(Stopped.into());
    }
    Ok(nl(String::from_utf8_lossy(&o.stdout).into_owned()))
}

/// {file: [(line number at HEAD, text)]} for lines the change added in files matching `pattern`.
fn added_lines(repo: &Path, base: &str, head: &str, pattern: &Regex) -> Result<Lines> {
    let range = format!("{base}..{head}");
    let mut args = DIFF.to_vec();
    args.extend(["-U0", range.as_str()]);
    let diff = git(&args, repo)?;
    let (mut out, mut f, mut prev, mut n): (Lines, Option<String>, &str, usize) = (Vec::new(), None, "", 0);
    for l in diff.split('\n') {
        if l.starts_with("+++ ") && prev.starts_with("--- ") {
            f = l.strip_prefix("+++ b/").filter(|x| !x.is_empty() && pattern.is_match(x)).map(String::from);
        } else if l.starts_with("@@") {
            if let Some(p) = l.find('+') {
                let digits: String = l[p + 1..].chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(v) = digits.parse() {
                    n = v;
                }
            }
        } else if let (Some(file), Some(text)) = (&f, l.strip_prefix('+')) {
            let text = text.trim_end_matches('\r').to_string();
            match out.iter_mut().find(|(g, _)| g == file) {
                Some((_, ls)) => ls.push((n, text)),
                None => out.push((file.clone(), vec![(n, text)])),
            }
            n += 1;
        }
        prev = l;
    }
    Ok(out)
}

#[cfg(windows)]
/// Lexical normpath, as Python's os.path.normpath for a path that has no links.
fn normpath(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            c => out.push(c),
        }
    }
    out
}

/// The shell a test command runs in: /bin/sh; on Windows Git's own bash, so a test command reads
/// the same as on macOS/Linux (`./run.sh`, `A=1 make test`). A bare "bash" may be WSL's System32/bash.exe.
fn shell(cmd: &str) -> Command {
    #[cfg(windows)]
    let program = {
        static BASH: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
        BASH.get_or_init(|| {
            let core = Command::new("git").arg("--exec-path").stdin(Stdio::null()).output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
            for rel in ["../../../bin/bash.exe", "../../../usr/bin/bash.exe"] {
                let p = normpath(&Path::new(&core).join(rel));
                if !core.is_empty() && p.is_file() {
                    return p;
                }
            }
            crate::kit::find_program("bash").unwrap_or_else(|| PathBuf::from("bash"))
        }).clone()
    };
    #[cfg(not(windows))]
    let program = PathBuf::from("/bin/sh");
    let mut c = Command::new(program);
    c.arg("-c").arg(cmd);
    c
}

/// One run of the test command: exit code (124 at the time limit), stdout then stderr, seconds.
struct Run {
    rc: i32,
    out: String,
    secs: f64,
}

/// Run `cmd` in `cwd` with stdin closed; the whole process group is stopped at the time limit or
/// on a signal (then Err(Stopped)).
fn sh(cmd: &str, cwd: &Path) -> Result<Run> {
    if STOP.load(Ordering::SeqCst) {
        return Err(Stopped.into());
    }
    let t = Instant::now();
    let mut c = shell(cmd);
    c.current_dir(cwd).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut c, 0);
    let failed = |e: &dyn std::fmt::Display| {
        let out = format!("bite: could not start the test command in {}: {e}", crate::kit::plain(cwd).display());
        Ok(Run { rc: 127, out, secs: t.elapsed().as_secs_f64() })
    };
    // Windows: the shell starts suspended inside a job object that kills every process in it when
    // closed, so a grandchild dies at the time limit or on Ctrl+C even after the shell has exited
    // (`taskkill /T` only finds a tree through a live parent; review 2.2 round 1, as in 1d).
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(&mut c, 0x0000_0004); // CREATE_SUSPENDED
    #[cfg(windows)]
    let job = match sos_rules::check::job::Job::new() {
        Ok(j) => j,
        Err(e) => return failed(&e),
    };
    let mut child = match c.spawn() {
        Ok(ch) => ch,
        Err(e) => return failed(&e),
    };
    #[cfg(windows)]
    if let Err(e) = job.assign(&child).and_then(|()| sos_rules::check::job::Job::resume(child.id())) {
        job.kill();
        let _ = child.kill();
        let _ = child.wait();
        return failed(&e);
    }
    // Whatever the test command left running is killed with it (its folder is about to go).
    #[cfg(windows)]
    let kill_tree = || job.kill();
    #[cfg(unix)]
    let kill_tree = {
        let pid = child.id();
        move || crate::role::kill_group(pid)
    };
    let (tx, rx) = mpsc::channel();
    let pipes: [Option<Box<dyn Read + Send>>; 2] =
        [child.stdout.take().map(|p| Box::new(p) as _), child.stderr.take().map(|p| Box::new(p) as _)];
    for (i, p) in pipes.into_iter().enumerate() {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(mut p) = p {
                let _ = p.read_to_end(&mut buf);
            }
            let _ = tx.send((i, buf));
        });
    }
    drop(tx);
    let mut bufs: [Option<Vec<u8>>; 2] = [None, None];
    let mut status = None;
    loop {
        if status.is_none() {
            status = child.try_wait().context("waiting for the test command")?;
            if status.is_some() {
                // A background process still holding the pipes would otherwise keep the run going
                // until the time limit (review 2.2 round 2; Python waited the same way).
                kill_tree();
            }
        }
        if status.is_some() && bufs.iter().all(Option::is_some) {
            break;
        }
        let stop = STOP.load(Ordering::SeqCst);
        if stop || t.elapsed() >= TIMEOUT {
            kill_tree();
            let _ = child.kill();
            let _ = child.wait();
            if stop {
                return Err(Stopped.into());
            }
            return Ok(Run { rc: 124, out: "timeout".into(), secs: t.elapsed().as_secs_f64() });
        }
        if let Ok((i, b)) = rx.recv_timeout(Duration::from_millis(20)) {
            bufs[i] = Some(b);
        }
    }
    let status = status.expect("exited");
    #[cfg(unix)]
    let rc = status.code().unwrap_or_else(|| -std::os::unix::process::ExitStatusExt::signal(&status).unwrap_or(1));
    #[cfg(not(unix))]
    let rc = status.code().unwrap_or(1);
    let [o, e] = bufs.map(|b| String::from_utf8_lossy(&b.unwrap_or_default()).into_owned());
    Ok(Run { rc, out: nl(o + &e), secs: t.elapsed().as_secs_f64() })
}

/// Python's round(x, 1): the exact value rounded half to even (0.25 -> 0.2), as `{:.1}` prints it.
fn round1(x: f64) -> f64 {
    format!("{x:.1}").parse().unwrap_or(x)
}

/// `p` with every symlink and junction resolved as far as it exists (Python's Path.resolve()).
fn resolve(p: &Path, depth: u32) -> PathBuf {
    if let Ok(c) = fs::canonicalize(p) {
        return c;
    }
    if depth > 0 {
        if let Ok(t) = fs::read_link(p) {
            return resolve(&p.parent().unwrap_or(Path::new("")).join(t), depth - 1);
        }
    }
    match (p.parent(), p.file_name()) {
        (Some(d), Some(name)) if !d.as_os_str().is_empty() => resolve(d, depth).join(name),
        _ => p.to_path_buf(),
    }
}

/// p, with every symlink and junction resolved, is in the throwaway worktree.
fn inside(wt: &Path, p: &Path) -> bool {
    resolve(p, 40).starts_with(resolve(wt, 40))
}

/// A symlink or a Windows junction.
fn is_link(p: &Path) -> bool {
    let Ok(m) = fs::symlink_metadata(p) else { return false };
    #[cfg(windows)]
    if std::os::windows::fs::MetadataExt::file_attributes(&m) & 0x400 != 0 {
        return true; // FILE_ATTRIBUTE_REPARSE_POINT
    }
    m.file_type().is_symlink()
}

/// Remove the link at `p`, not what it points to.
fn unlink(p: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        let m = fs::symlink_metadata(p)?;
        if std::os::windows::fs::MetadataExt::file_attributes(&m) & 0x10 != 0 {
            return fs::remove_dir(p); // a junction or a folder symlink (FILE_ATTRIBUTE_DIRECTORY)
        }
    }
    fs::remove_file(p)
}

/// dst -> src. A Windows symlink needs Developer Mode or admin rights; without them a folder gets
/// a junction and a file a copy (no node_modules reached the worktree there, 23/10/2026).
fn link(src: &Path, dst: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    return std::os::unix::fs::symlink(src, dst);
    #[cfg(windows)]
    {
        let dir = src.is_dir();
        let made = if dir { std::os::windows::fs::symlink_dir(src, dst) } else { std::os::windows::fs::symlink_file(src, dst) };
        match made {
            Ok(()) => Ok(()),
            Err(_) if dir => win::junction(src, dst),
            Err(_) => fs::copy(src, dst).map(|_| ()),
        }
    }
}

#[cfg(windows)]
mod win {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use std::sync::atomic::Ordering;

    type Handle = *mut c_void;
    #[link(name = "kernel32")]
    extern "system" {
        fn CreateFileW(name: *const u16, access: u32, share: u32, sa: *const c_void, disposition: u32, flags: u32,
                       template: Handle) -> Handle;
        fn DeviceIoControl(h: Handle, code: u32, inb: *const c_void, insize: u32, outb: *mut c_void, outsize: u32,
                           returned: *mut u32, overlapped: *mut c_void) -> i32;
        fn CloseHandle(h: Handle) -> i32;
        fn SetConsoleCtrlHandler(handler: Option<unsafe extern "system" fn(u32) -> i32>, add: i32) -> i32;
    }

    /// A directory junction dst -> src (what Python's _winapi.CreateJunction makes): an empty
    /// folder with a mount-point reparse point.
    pub fn junction(src: &Path, dst: &Path) -> std::io::Result<()> {
        let target = crate::kit::plain(&std::path::absolute(src)?);
        let print: Vec<u16> = target.as_os_str().encode_wide().collect();
        let subst: Vec<u16> = r"\??\".encode_utf16().chain(print.iter().copied()).collect();
        let (sb, pb) = (subst.len() * 2, print.len() * 2);
        let data = 8 + sb + 2 + pb + 2;
        let too_long = || std::io::Error::new(std::io::ErrorKind::InvalidInput, "junction target too long");
        let u16of = |n: usize| u16::try_from(n).map_err(|_| too_long());
        let mut buf: Vec<u8> = Vec::with_capacity(8 + data);
        buf.extend(0xA000_0003u32.to_le_bytes()); // IO_REPARSE_TAG_MOUNT_POINT
        for v in [u16of(data)?, 0, 0, u16of(sb)?, u16of(sb + 2)?, u16of(pb)?] {
            buf.extend(v.to_le_bytes());
        }
        for u in subst.iter().chain(&[0]).chain(print.iter()).chain(&[0]) {
            buf.extend(u.to_le_bytes());
        }
        std::fs::create_dir(dst)?;
        let name: Vec<u16> = dst.as_os_str().encode_wide().chain([0]).collect();
        // SAFETY: plain Win32 calls on a NUL-terminated name and a buffer that outlives them; the
        // handle is closed on every path.
        let res = unsafe {
            // GENERIC_WRITE, OPEN_EXISTING, FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS
            let h = CreateFileW(name.as_ptr(), 0x4000_0000, 0, std::ptr::null(), 3, 0x0020_0000 | 0x0200_0000,
                                std::ptr::null_mut());
            if h as isize == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                let mut got = 0u32;
                // FSCTL_SET_REPARSE_POINT
                let ok = DeviceIoControl(h, 0x0009_00A4, buf.as_ptr().cast(), buf.len() as u32, std::ptr::null_mut(), 0,
                                         &mut got, std::ptr::null_mut());
                let err = std::io::Error::last_os_error();
                CloseHandle(h);
                if ok == 0 { Err(err) } else { Ok(()) }
            }
        };
        if res.is_err() {
            let _ = std::fs::remove_dir(dst);
        }
        res
    }

    unsafe extern "system" fn on_ctrl(kind: u32) -> i32 {
        super::STOP.store(true, Ordering::SeqCst);
        // CTRL_CLOSE / LOGOFF / SHUTDOWN end the process when the handler returns: give the main
        // thread up to 4.5 s to remove the worktree first.
        if matches!(kind, 2 | 5 | 6) {
            for _ in 0..90 {
                if super::CLEANED.load(Ordering::SeqCst) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
        1
    }

    pub fn on_signals() {
        // SAFETY: registers a handler that only touches atomics and sleeps.
        unsafe {
            SetConsoleCtrlHandler(Some(on_ctrl), 1);
        }
    }
}

/// SIGTERM/SIGHUP/SIGINT (an agent's timeout, a closed terminal, Ctrl+C) must still remove the worktree.
#[cfg(unix)]
fn on_signals() {
    extern "C" fn stop(_: libc::c_int) {
        STOP.store(true, Ordering::SeqCst);
    }
    for s in [libc::SIGTERM, libc::SIGHUP, libc::SIGINT] {
        // SAFETY: the handler only stores to an atomic, which is async-signal-safe.
        unsafe {
            libc::signal(s, stop as *const () as libc::sighandler_t);
        }
    }
}
#[cfg(windows)]
use win::on_signals;

/// The throwaway worktree; dropping it removes it (links first).
struct Worktree {
    root: PathBuf,
    wt: PathBuf,
    links: Vec<PathBuf>,
}

impl Worktree {
    fn add(root: &Path, head: &str) -> Result<Worktree> {
        // A run killed outright (SIGKILL) leaves a registered worktree whose folder is gone.
        let _ = Command::new("git").args(["worktree", "prune"]).current_dir(root)
            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
        // Owned before `worktree add` runs: a signal that lands while git works (git() then returns
        // Stopped even though the worktree exists) still removes it in Drop (review 2.2 round 1).
        let w = Worktree { root: root.to_path_buf(), wt: mkdtemp()?, links: Vec::new() };
        git(&["worktree", "add", "-q", "--detach", &w.wt.to_string_lossy(), head], root)?;
        Ok(w)
    }

    /// Installed dependencies are not in git. The worktree gets its own node_modules folder whose
    /// entries link to the checkout's (or the main checkout's, when this one is a worktree without
    /// them); the runners' cache folders stay real folders in the worktree, so a run does not write
    /// into the checkout's.
    fn link_deps(&mut self, cwd: &str) -> Result<()> {
        let common = git(&["rev-parse", "--path-format=absolute", "--git-common-dir"], &self.root)?;
        let main = Path::new(common.trim()).parent().map(Path::to_path_buf).unwrap_or_default();
        let mut ds = vec![PathBuf::from("node_modules")];
        let under: PathBuf = Path::new(cwd).components().filter(|c| matches!(c, Component::Normal(_))).collect();
        let d2 = under.join("node_modules");
        if !ds.contains(&d2) {
            ds.push(d2);
        }
        ds.sort_by_key(|d| d.components().count());
        for d in ds {
            let Some(src) = [&self.root, &main].iter().map(|r| r.join(&d)).find(|p| p.is_dir()) else { continue };
            let dst = self.wt.join(&d);
            // `cwd` under a linked package resolves into the checkout: never create anything there.
            if dst.exists() || !dst.parent().is_some_and(|p| inside(&self.wt, p)) {
                continue;
            }
            let _ = self.fill(&src, &dst); // on failure the test command fails and says why
        }
        Ok(())
    }

    fn fill(&mut self, src: &Path, dst: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dst)?;
        for e in fs::read_dir(src)? {
            let e = e?;
            let name = e.file_name();
            if CACHES.iter().any(|c| name == *c) {
                continue;
            }
            link(&e.path(), &dst.join(&name))?;
            self.links.push(dst.join(&name));
        }
        Ok(())
    }
}

/// Git for Windows' `worktree remove` deletes through a junction: it emptied the checkout's own
/// node_modules (23/10/2026). The links link_deps() made are removed first, the link only, and only
/// where the folder holding it is still in the worktree. A run killed outright (TerminateProcess,
/// SIGKILL) skips this: remove a leftover sos-bite-* worktree by hand, links first.
impl Drop for Worktree {
    fn drop(&mut self) {
        for e in self.links.iter().rev() {
            if is_link(e) && e.parent().is_some_and(|d| inside(&self.wt, d)) {
                if let Err(err) = unlink(e) {
                    eprintln!("bite: could not remove the link {} ({err}): the worktree {} is left; remove it by hand, links first",
                              e.display(), self.wt.display());
                    CLEANED.store(true, Ordering::SeqCst);
                    return;
                }
            }
        }
        let _ = Command::new("git").args(["worktree", "remove", "--force"]).arg(&self.wt).current_dir(&self.root)
            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
        let _ = fs::remove_dir(&self.wt); // still there (empty) when `worktree add` failed
        CLEANED.store(true, Ordering::SeqCst);
    }
}

/// A new empty folder sos-bite-* in the system temp folder.
fn mkdtemp() -> Result<PathBuf> {
    static N: AtomicU32 = AtomicU32::new(0);
    let mut last = None;
    for _ in 0..100 {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
        let p = std::env::temp_dir().join(format!("sos-bite-{}-{nanos:x}{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        #[cfg_attr(not(unix), allow(unused_mut))]
        let mut b = fs::DirBuilder::new();
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut b, 0o700);
        match b.create(&p) {
            Ok(()) => return Ok(p),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => last = Some(e),
            Err(e) => return Err(e).context("creating the temporary worktree folder"),
        }
    }
    Err(last.map_or_else(|| anyhow::anyhow!("no temporary folder"), anyhow::Error::from))
}

fn default_base(root: &Path) -> String {
    for r in ["origin/HEAD", "origin/main", "main", "master"] {
        if let Ok(o) = Command::new("git").args(["merge-base", "HEAD", r]).current_dir(root)
            .stdin(Stdio::null()).stderr(Stdio::null()).output()
        {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if o.status.success() && !s.is_empty() {
                return s;
            }
        }
    }
    "HEAD~1".into()
}

struct Patterns {
    src: Regex,
    tests: Regex,
}

impl Patterns {
    fn source(&self, f: &str) -> bool {
        !f.is_empty() && self.src.is_match(f) && !self.tests.is_match(f)
    }
}

fn head10(s: &str) -> String {
    s.chars().take(10).collect()
}

/// The last `n` characters of `s` with surrounding whitespace removed.
fn tail(s: &str, n: usize) -> String {
    let s = s.trim_matches(py_space);
    let k = s.chars().count();
    s.chars().skip(k.saturating_sub(n)).collect()
}

/// Up to 40 characters either side of the first difference between `old` and `new`, of `s`.
fn around(old: &str, new: &str, s: &str) -> String {
    let i = old.chars().zip(new.chars()).position(|(x, y)| x != y).unwrap_or(0);
    let cs: Vec<char> = s.chars().collect();
    let a = i.saturating_sub(40);
    if a >= cs.len() {
        return String::new();
    }
    cs[a..(i + 40).min(cs.len())].iter().collect::<String>().trim_matches(py_space).to_string()
}

/// Write mutant `new` over line `n` of `p` (keeping its \r), run the tests, put the file back.
fn try_mutant(p: &Path, n: usize, old: Option<&str>, new: &str, test: &str, run: &Path) -> Result<Option<Run>> {
    let orig = fs::read(p).with_context(|| format!("reading {}", p.display()))?;
    let text = String::from_utf8_lossy(&orig).into_owned();
    let mut ls: Vec<&str> = text.split('\n').collect();
    if n == 0 || n > ls.len() {
        return Ok(None);
    }
    let cur = ls[n - 1];
    if old.is_some_and(|o| cur.trim_end_matches('\r') != o) {
        return Ok(None);
    }
    let line = format!("{new}{}", if cur.ends_with('\r') { "\r" } else { "" });
    ls[n - 1] = &line;
    fs::write(p, ls.join("\n"))?;
    let r = sh(test, run);
    fs::write(p, &orig)?;
    r.map(Some)
}

fn measure(root: &Path, base: &str, head: &str, s: &Settings, pat: &Patterns) -> Result<Map<String, Value>> {
    let head = git(&["rev-parse", head], root)?.trim().to_string();
    let base = git(&["rev-parse", base], root)?.trim().to_string();
    let mut rep = Map::new();
    rep.insert("base".into(), json!(head10(&base)));
    rep.insert("head".into(), json!(head10(&head)));
    rep.insert("test".into(), json!(s.test));
    if base == head {
        rep.insert("error".into(), json!(format!(
            "base and head are the same commit ({}): pass --base <the commit before the change>", head10(&head))));
        return Ok(rep);
    }
    let mut w = Worktree::add(root, &head)?;
    w.link_deps(&s.cwd)?;
    let wt = w.wt.clone();
    let run = wt.join(&s.cwd);
    if !inside(&wt, &run) {
        rep.insert("error".into(), json!(format!(
            "cwd {} resolves outside the throwaway worktree (through a link): not run", py_repr(&s.cwd))));
        return Ok(rep);
    }
    let first = sh(&s.test, &run)?;
    rep.insert("suite_seconds".into(), json!(round1(first.secs)));
    if first.rc != 0 {
        rep.insert("error".into(), json!(format!("tests are red at HEAD: {}", tail(&first.out, 600))));
        return Ok(rep);
    }
    let per_test = verdicts(&first.out).has;
    let range = format!("{base}..{head}");
    let mut args = DIFF.to_vec();
    args.extend(["--name-only", "-z", range.as_str()]);
    let changed = git(&args, root)?;
    let src: Vec<String> = changed.split('\0').filter(|f| pat.source(f)).map(String::from).collect();
    let tadded: Lines = added_lines(root, &base, &head, &pat.tests)?.into_iter().filter(|(f, _)| wt.join(f).is_file()).collect();
    let new_tests: Vec<String> = test_names(&tadded, &wt).into_iter().collect();
    rep.insert("src_files".into(), json!(src));
    rep.insert("tests_touched".into(), json!(new_tests));
    // 1. before: source at BASE, tests at HEAD
    let existed: HashSet<String> = git(&["ls-tree", "-r", "-z", "--name-only", &base], root)?.split('\0').map(String::from).collect();
    for f in &src {
        if existed.contains(f) {
            git(&["checkout", &base, "--", f], &wt)?;
        } else {
            let _ = fs::remove_file(wt.join(f));
        }
    }
    let before = sh(&s.test, &run)?;
    let res = verdicts(&before.out).results;
    rep.insert("before".into(), json!({
        "suite_failed": before.rc != 0,
        "pass_on_old_code": new_tests.iter().filter(|t| res.get(*t) == Some(&true)).collect::<Vec<_>>(),
        "no_verdict": new_tests.iter().filter(|t| !res.contains_key(*t)).collect::<Vec<_>>(),
    }));
    git(&["reset", "-q", "--hard", &head], &wt)?; // also drops base-only files put back above
    // 2. mutants on added source lines
    let lines: Lines = added_lines(root, &base, &head, &pat.src)?.into_iter().filter(|(f, _)| src.contains(f)).collect();
    let mut ms = Vec::new();
    for (f, n, old, new) in mutants(&lines, s.mutants, &wt) {
        let Some(r) = try_mutant(&wt.join(&f), n, Some(&old), &new, &s.test, &run)? else { continue };
        // With per-test verdicts a kill is a test that failed; a red run without one did not reach
        // the tests (the mutant broke the build, or a test file no longer loads).
        let ran = r.rc == 0 || !per_test || verdicts(&r.out).any_failed;
        ms.push(json!({"at": format!("{f}:{n}"), "from": around(&old, &new, &old), "to": around(&old, &new, &new),
                       "killed": ran && r.rc != 0, "no_tests_ran": !ran, "seconds": round1(r.secs)}));
    }
    rep.insert("mutants".into(), json!(ms));
    Ok(rep)
}

/// Which existing tests kill no mutant, or only mutants another test also kills? Candidates for removal.
fn audit(root: &Path, head: &str, s: &Settings, pat: &Patterns) -> Result<Map<String, Value>> {
    let head = git(&["rev-parse", head], root)?.trim().to_string();
    let mut rep = Map::new();
    rep.insert("head".into(), json!(head10(&head)));
    rep.insert("test".into(), json!(s.test));
    rep.insert("audit".into(), json!(true));
    let mut w = Worktree::add(root, &head)?;
    w.link_deps(&s.cwd)?;
    let wt = w.wt.clone();
    let run = wt.join(&s.cwd);
    if !inside(&wt, &run) {
        rep.insert("error".into(), json!(format!(
            "cwd {} resolves outside the throwaway worktree (through a link): not run", py_repr(&s.cwd))));
        return Ok(rep);
    }
    let first = sh(&s.test, &run)?;
    rep.insert("suite_seconds".into(), json!(round1(first.secs)));
    if first.rc != 0 {
        rep.insert("error".into(), json!(format!("tests are red at HEAD: {}", tail(&first.out, 600))));
        return Ok(rep);
    }
    let v = verdicts(&first.out);
    let green: Vec<String> = v.results.iter().filter(|(_, ok)| **ok).map(|(t, _)| t.clone()).collect::<BTreeSet<_>>().into_iter().collect();
    if !v.has {
        rep.insert("error".into(), json!(
            "no per-test verdicts in the test output: the audit needs one of the runners listed in `sos bite --help`"));
        return Ok(rep);
    }
    let mut lines: Lines = Vec::new();
    for f in git(&["ls-files", "-z"], &wt)?.split('\0').filter(|f| pat.source(f)) {
        if let Some(text) = read(&wt.join(f)) {
            lines.push((f.to_string(), text.split('\n').enumerate().map(|(i, l)| (i + 1, l.trim_end_matches('\r').to_string())).collect()));
        }
    }
    let mut kills: BTreeMap<String, Vec<String>> = green.iter().map(|t| (t.clone(), Vec::new())).collect();
    let mut ms = Vec::new();
    for (f, n, old, new) in mutants(&lines, s.mutants, &wt) {
        let Some(r) = try_mutant(&wt.join(&f), n, None, &new, &s.test, &run)? else { continue };
        let v = verdicts(&r.out);
        let by: Vec<&String> = green.iter().filter(|t| v.results.get(*t) == Some(&false)).collect();
        let killed = r.rc != 0 && v.any_failed;
        ms.push(json!({"at": format!("{f}:{n}"), "to": around(&old, &new, &new), "killed_by": by, "killed": killed,
                       "no_tests_ran": r.rc != 0 && !killed, "seconds": round1(r.secs)}));
        for t in by {
            kills.get_mut(t).expect("green").push(format!("{f}:{n}"));
        }
    }
    rep.insert("mutants".into(), json!(ms));
    // A test whose kills are all killed by one other test adds no proof on this sample (the last such test is named).
    let mut covered: BTreeMap<&String, &String> = BTreeMap::new();
    for t in &green {
        let kt: BTreeSet<&String> = kills[t].iter().collect();
        if kt.is_empty() {
            continue;
        }
        for o in &green {
            let ko: BTreeSet<&String> = kills[o].iter().collect();
            if o != t && kt.is_subset(&ko) && (kt.len() < ko.len() || o < t) {
                covered.insert(t, o);
            }
        }
    }
    let nothing: Vec<&String> = green.iter().filter(|t| kills[*t].is_empty()).collect();
    let covered = json!(covered);
    rep.insert("kills".into(), json!(kills));
    rep.insert("killed_nothing".into(), json!(nothing));
    rep.insert("covered_by".into(), covered);
    Ok(rep)
}

/// json.dumps(rep, indent=1, ensure_ascii=False) + "\n".
fn dump(rep: &Map<String, Value>) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, serde_json::ser::PrettyFormatter::with_indent(b" "));
    rep.serialize(&mut ser)?;
    buf.push(b'\n');
    Ok(buf)
}

fn strs(v: &Value) -> Vec<&str> {
    v.as_array().map(|a| a.iter().filter_map(Value::as_str).collect()).unwrap_or_default()
}

fn secs(v: &Value) -> String {
    format!("{:?}", v.as_f64().unwrap_or(0.0))
}

pub fn run(a: Args) -> Result<i32> {
    if let Some(kind) = &a.internal {
        return internal(kind, &a);
    }
    let here = std::env::current_dir()?;
    let root = match git(&["rev-parse", "--show-toplevel"], &here) {
        Ok(r) => PathBuf::from(r.trim()),
        Err(_) => {
            eprintln!("bite: not inside a git repository");
            return Ok(1);
        }
    };
    let mut s = match settings(&root) {
        Ok(s) => s,
        Err((code, msg)) => {
            eprintln!("{msg}");
            return Ok(code);
        }
    };
    for (slot, v) in [(&mut s.test, &a.test), (&mut s.cwd, &a.cwd), (&mut s.src, &a.src), (&mut s.tests, &a.tests)] {
        if let Some(v) = v {
            *slot = v.clone();
        }
    }
    if let Some(n) = a.mutants {
        s.mutants = n;
    }
    let missing: Vec<&str> = [("test", &s.test), ("src", &s.src), ("tests", &s.tests)]
        .iter().filter(|(_, v)| v.is_empty()).map(|(k, _)| *k).collect();
    if !missing.is_empty() {
        eprintln!("bite: set {} in .sos.toml [bite] (or --{}); see harness-lite/templates/sos.toml", missing.join(", "), missing[0]);
        return Ok(2);
    }
    let c = Path::new(&s.cwd);
    if c.is_absolute() || c.has_root() || c.components().any(|x| matches!(x, Component::Prefix(_) | Component::ParentDir)) {
        eprintln!("bite: cwd must be a path inside the repo ({}): the tests would run outside the throwaway worktree", py_repr(&s.cwd));
        return Ok(2);
    }
    let pat = match (Regex::new(&s.src), Regex::new(&s.tests)) {
        (Ok(src), Ok(tests)) => Patterns { src, tests },
        (Err(e), _) | (_, Err(e)) => {
            eprintln!("bite: --src / --tests must be a regex Rust's regex crate reads (no lookaround or backreferences): {e}");
            return Ok(2);
        }
    };
    // bite's test runs bypass the agent hooks: the simulator guard is checked here.
    if let Some(msg) = crate::guards::sim::check(&s.test) {
        eprintln!("bite: {}", msg.trim());
        return Ok(2);
    }
    on_signals();
    if a.audit && a.mutants.is_none() {
        s.mutants = s.mutants.max(30);
    }
    let head = a.head.clone().unwrap_or_else(|| "HEAD".into());
    let got = if a.audit {
        audit(&root, &head, &s, &pat)
    } else {
        let base = a.base.clone().unwrap_or_else(|| default_base(&root));
        measure(&root, &base, &head, &s, &pat)
    };
    let rep = match got {
        Ok(r) => r,
        Err(e) if e.is::<Stopped>() => return Ok(130),
        Err(e) => {
            eprintln!("bite: {e:#}");
            return Ok(1);
        }
    };
    if STOP.load(Ordering::SeqCst) {
        return Ok(130);
    }
    let rel = Path::new("evidence").join("bite").join(format!(
        "{}-{}.json", crate::role::local_time("%Y%m%d-%H%M%S"), rep["head"].as_str().unwrap_or("")));
    fs::create_dir_all(root.join("evidence").join("bite"))?;
    fs::write(root.join(&rel), dump(&rep)?)?;
    if let Some(e) = rep.get("error") {
        eprintln!("bite: not measured, {}", e.as_str().unwrap_or(""));
        return Ok(1);
    }
    let ms = rep["mutants"].as_array().cloned().unwrap_or_default();
    let flag = |m: &Value, k: &str| m[k].as_bool().unwrap_or(false);
    let txt = |m: &Value, k: &str| m[k].as_str().unwrap_or("").to_string();
    let counted: Vec<&Value> = ms.iter().filter(|m| !flag(m, "no_tests_ran")).collect();
    let killed = counted.iter().filter(|m| flag(m, "killed")).count();
    if a.audit {
        let nothing = strs(&rep["killed_nothing"]);
        println!("bite --audit {}: {} tests, {killed}/{} mutants killed; {} tests killed none; {} killed only \
                  what one other test also killed (listed in the result file) (suite {} s)",
                 txt(&Value::Object(rep.clone()), "head"), rep["kills"].as_object().map_or(0, Map::len), counted.len(),
                 nothing.len(), rep["covered_by"].as_object().map_or(0, Map::len), secs(&rep["suite_seconds"]));
        for t in nothing {
            println!("  killed none: {t}");
        }
        for m in &counted {
            if !flag(m, "killed") {
                println!("  survived: {}  ->  {}", txt(m, "at"), txt(m, "to"));
            }
        }
        for m in &ms {
            if flag(m, "no_tests_ran") {
                println!("  not counted (no test failed, build broke?): {}  {}", txt(m, "at"), txt(m, "to"));
            }
        }
        println!("  (candidates, not verdicts: a test may guard lines this sample missed or a contract no mutant reaches; \
                  full result: {})", rel.display());
        return Ok(0);
    }
    let b = &rep["before"];
    let pass_old = strs(&b["pass_on_old_code"]);
    println!("bite {}..{}: {killed}/{} mutants killed; {}/{} new or edited tests pass on the old code (suite {} s)",
             rep["base"].as_str().unwrap_or(""), rep["head"].as_str().unwrap_or(""), counted.len(), pass_old.len(),
             strs(&rep["tests_touched"]).len(), secs(&rep["suite_seconds"]));
    for t in pass_old {
        println!("  passes on old code: {t}");
    }
    let suite_failed = b["suite_failed"].as_bool().unwrap_or(false);
    for t in strs(&b["no_verdict"]) {
        println!("  no per-test verdict: {t}{}", if suite_failed { "" } else { " (the old-code suite passed)" });
    }
    for m in &counted {
        if !flag(m, "killed") {
            println!("  survived: {}  {}  ->  {}", txt(m, "at"), txt(m, "from"), txt(m, "to"));
        }
    }
    for m in &ms {
        if flag(m, "no_tests_ran") {
            println!("  not counted (no test ran, build broke?): {}  {}", txt(m, "at"), txt(m, "to"));
        }
    }
    println!("  (full result: {})", rel.display());
    Ok(0)
}

/// {"file": [[n, "text"], …]} (key order kept) as Lines.
fn lines_from(input: &str) -> Result<Lines> {
    let v: Map<String, Value> = serde_json::from_str(input).context("--internal: stdin must be {file: [[n, text], …]}")?;
    let mut out = Vec::new();
    for (f, ls) in v {
        let ls = ls.as_array().cloned().unwrap_or_default().iter()
            .map(|x| (x[0].as_u64().unwrap_or(0) as usize, x[1].as_str().unwrap_or("").to_string())).collect();
        out.push((f, ls));
    }
    Ok(out)
}

/// The parsers on their own, for tests/bite_parsers.py: it runs the same samples on both versions.
fn internal(kind: &str, a: &Args) -> Result<i32> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let dir = a.internal_dir.clone().unwrap_or_default();
    let out = match kind {
        "results" => {
            let v = verdicts(&nl(input));
            json!({"results": v.results.into_iter().collect::<BTreeMap<_, _>>(), "has_verdicts": v.has, "any_failed": v.any_failed})
        }
        "test-names" => json!(test_names(&lines_from(&input)?, &dir)),
        "mutants" => json!(mutants(&lines_from(&input)?, a.mutants.unwrap_or(8), &dir)),
        _ => bail!("--internal takes results, test-names or mutants"),
    };
    println!("{out}");
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn res(out: &str) -> BTreeMap<String, bool> {
        verdicts(out).results.into_iter().collect()
    }

    fn one(line: &str) -> Option<String> {
        let code = &line[..code_end(line)];
        mutate(code).map(|(s, e, new)| format!("{}{new}{}", &line[..s], &line[e..]))
    }

    #[test]
    fn mutators_follow_python_order_and_lookarounds() {
        for (line, want) in [
            ("if (a !== 'x') f()", Some("if (a === 'x') f()")),
            ("if (a === 1) f()", Some("if (a !== 1) f()")),
            ("if (a == b) f()", Some("if (a != b) f()")),
            ("if (a != b) f()", Some("if (a == b) f()")),
            ("if x <= 3:", Some("if x > 3:")),
            ("if x < lo:", Some("if x >= lo:")),
            ("const s = new Set<string>(); let f = (a) => a", None),
            ("return a and b", Some("return a or b")),
            ("ok = a && b", Some("ok = a || b")),
            ("flag = True", Some("flag = False")),
            ("if not done:", Some("if done:")),
            ("y = x + 1", Some("y = x - 1")),
            ("LIMIT = 10", Some("LIMIT = 11")),
            ("v = 099 + x.5", Some("v = 099 - x.5")),
            ("v = 099", Some("v = 100")),
            ("v = a1 + 2.5", Some("v = a1 - 2.5")),
            ("v = 'a == b'", None),
            ("return x  # never x + 1", None),
            ("s = \"a == b\" == c", None), // each mutator tries its first match only, as in Python
            ("x = \"=== \" + y", Some("x = \"=== \" - y")),
            ("++ i == 3", Some("++ i != 3")),
        ] {
            assert_eq!(one(line).as_deref(), want, "{line}");
        }
    }

    #[test]
    fn round1_is_python_round() {
        for (x, want) in [(0.25, 0.2), (0.35, 0.3), (0.05, 0.1), (0.15, 0.1), (1.25, 1.2), (0.45, 0.5), (12.04, 12.0)] {
            assert_eq!(round1(x), want, "{x}");
        }
    }

    #[test]
    fn python_word_class() {
        assert!(py_word('\u{b2}') && py_word('é') && py_word('_') && !py_word('\u{301}') && !py_word('.'));
        assert_eq!(word_at("x = trúe or true", "true", true), Some((13, 17)));
        assert_eq!(word_at("tru\u{65}\u{301}", "true", true), Some((0, 4)));
    }

    #[test]
    fn number_plus_one_like_python_int() {
        assert_eq!(plus_one("9"), "10");
        assert_eq!(plus_one("007"), "8");
        assert_eq!(plus_one("000"), "1");
        assert_eq!(plus_one("1999999999999999999999"), "2000000000000000000000");
    }

    #[test]
    fn result_parsers_match_bite_parsers_py() {
        let out = "test_a (tests.t.C.test_a) ... ok\ntest_b (tests.t.C)\nDocstring line ... FAIL\n\
                   tests/test_p.py::TestP::test_x PASSED\ntests/test_p.py::test_top[1] PASSED\ntests/test_p.py::test_top[2] FAILED\n\
                   Test Case '-[App.Feed testLoads]' passed (0.1 seconds).\n\
                   Test case '-[AppTests.Sync testRetry()]' failed on 'iPhone' (0.2 seconds)\n\
                   Test Case 'Store.testSave' passed (0.0 seconds)\n";
        let want: BTreeMap<String, bool> = [(".test_top", false), ("C.test_a", true), ("C.test_b", false), ("Feed.testLoads", true),
            ("Store.testSave", true), ("Sync.testRetry", false), ("TestP.test_x", true)]
            .into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        assert_eq!(res(out), want);
        let tap = "TAP version 13\n# Subtest: group\n    # Subtest: works\n    ok 1 - works\n      ---\n      type: 'test'\n      ...\n\
                   \x20   ok 2 - tap skipped # SKIP no db\n    not ok 3 - tap fails\n      ---\n      type: 'test'\n      ...\n    1..3\n\
                   not ok 1 - group\n  ---\n  type: 'suite'\n  ...\n# Subtest: t/bad.test.mjs\nnot ok 2 - t/bad.test.mjs\n  ---\n  type: 'test'\n  ...\n";
        assert_eq!(res(tap), [(".tap fails".to_string(), false), (".works".to_string(), true)].into_iter().collect());
        let spec = "▶ group\n  ✔ works (0.296917ms)\n  ✔ same (0.056ms)\n✔ group (0.884208ms)\n▶ other\n  ✖ same (0.101958ms)\n\
                    \x20 ✖ fails here (1.0ms)\n✖ other (0.5385ms)\n✖ t/bad.test.mjs (116.112ms)\nℹ tests 4\n\n✖ failing tests:\n\n\
                    test at t/a.test.mjs:3:27\n✖ fails here (1.0ms)\n";
        assert_eq!(res(spec), [(".fails here".to_string(), false), (".works".to_string(), true)].into_iter().collect());
        let dup = "▶ a\n  ✔ same (1ms)\n▶ b\n  ✖ same (1ms)\n";
        let v = verdicts(dup);
        assert!(v.results.is_empty() && v.has && v.any_failed);
        let cargo = "test tests::rounds_down ... ok\ntest store::tests::rejects_empty ... FAILED\n";
        assert_eq!(res(cargo), [(".rejects_empty".to_string(), false), (".rounds_down".to_string(), true)].into_iter().collect());
        assert_eq!(res("\u{2714} works (1ms)\n\u{2716} test/my file.test.mjs (1ms)\n"), [(".works".to_string(), true)].into_iter().collect());
    }

    #[test]
    fn test_bounds_and_classes() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("t.swift"), "import XCTest\n\n@MainActor\nfinal class FeedTests: XCTestCase {\n    func testLoads() {\n        XCTAssertTrue(true)\n    }\n\n    override func setUp() {\n        super.setUp()\n    }\n}\n\ndef test_top():\n    assert 1\n").unwrap();
        for (line, want) in [(6, vec!["FeedTests.testLoads"]), (10, vec![]), (15, vec![".test_top"])] {
            let got: Vec<String> = test_names(&vec![("t.swift".into(), vec![(line, String::new())])], d.path()).into_iter().collect();
            assert_eq!(got, want, "line {line}");
        }
        fs::write(d.path().join("a.test.mjs"), "import test from 'node:test';\ndescribe('clamp', () => {\n  it('keeps \"low\" end', () => {\n    assert.equal(clamp(-1), 0);\n  });\n\n  test.only(`one liner`, () => assert.ok(1));\n});\n").unwrap();
        for (line, want) in [(4, vec![".keeps \"low\" end"]), (7, vec![".one liner"]), (2, vec![])] {
            let got: Vec<String> = test_names(&vec![("a.test.mjs".into(), vec![(line, String::new())])], d.path()).into_iter().collect();
            assert_eq!(got, want, "js line {line}");
        }
        assert_eq!(js_title(r#"it('a\'b', x)"#, 3), Some(r"a\'b"));
    }

    #[test]
    fn comments_strings_and_docstrings() {
        assert_eq!(code_end("x = '#' # c"), 8);
        assert_eq!(code_end("a // b"), 2);
        assert!(in_string("x = 'a == b'", 7));
        assert!(!in_string("x = 'a' == b", 8));
        assert_eq!(docstring_lines("a\n\"\"\"doc\nx == y\n\"\"\"\nb"), [2, 3, 4].into_iter().collect());
    }

    #[test]
    fn settings_read_and_refuse() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join(".sos.toml"), "[bite]\ntest = 'make t'\nsrc = '\\.py$'\nmutants = 3\n").unwrap();
        let s = settings(d.path()).unwrap();
        assert_eq!((s.test.as_str(), s.src.as_str(), s.cwd.as_str(), s.mutants), ("make t", "\\.py$", ".", 3));
        fs::write(d.path().join(".sos.toml"), "[bite]\ntest = \n").unwrap();
        let (code, msg) = settings(d.path()).unwrap_err();
        assert_eq!(code, 1);
        assert!(msg.contains("not valid TOML") && msg.contains("line 2"), "{msg}");
        assert_eq!(py_repr("node_modules/dep"), "'node_modules/dep'");
        assert_eq!(py_repr("\\js"), "'\\\\js'");
        assert_eq!(py_repr("it's"), "\"it's\"");
    }

    #[test]
    fn evidence_json_is_python_shaped() {
        let mut m = Map::new();
        m.insert("head".into(), json!("abc"));
        m.insert("suite_seconds".into(), json!(round1(12.04)));
        m.insert("mutants".into(), json!([]));
        m.insert("before".into(), json!({"pass_on_old_code": ["Ä.x"]}));
        assert_eq!(String::from_utf8(dump(&m).unwrap()).unwrap(),
                   "{\n \"head\": \"abc\",\n \"suite_seconds\": 12.0,\n \"mutants\": [],\n \"before\": {\n  \"pass_on_old_code\": [\n   \"Ä.x\"\n  ]\n }\n}\n");
    }
}
