//! `sos-dev trust-gate` — the kit's content-integrity gate (port of `scripts/trust-gate.sh`).
//!
//! sos-kit ships auto-exec surfaces as its product (git hooks, scripts and adapters that
//! `sos install` copies into apps, agent hook configs, install.sh, the npm wrapper, CI). A silent
//! change to them reaches every user. Two checks, both fail closed:
//!   1. baseline: the SHA-256 of every tracked surface equals its line in `.sos-trust-baseline`;
//!      a reviewed change is accepted by `sos-dev trust-gate rebaseline` (the diff shows in the PR);
//!   2. hidden Unicode: no BOM, zero-width, bidi, word-joiner, Mongolian vowel separator or tag
//!      characters in the files agents load as instructions.
//!
//! Threat model: SECURITY.md (INV-TRUST-01, -02).

use crate::git;
use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub const BASELINE: &str = ".sos-trust-baseline";

/// Auto-exec surfaces, as shell globs read from the repo root. Must stay equal to `SURFACE_GLOBS`
/// in scripts/trust-gate.sh while that script exists, and to the inventory in SECURITY.md.
/// `.claude/settings.local.json` is left out: globally gitignored, per machine, never shipped.
pub const SURFACES: &[&str] = &[
    ".claude/settings.json",
    ".codex/hooks.json",
    ".mcp.json",
    "adapters/*/*",
    "bin/sos-npm",
    "hooks/pre-commit",
    "hooks/pre-push",
    "hooks/commit-msg",
    "templates/app/hooks/*",
    "scripts/*",
    "install.sh",
    ".github/workflows/*",
    "tests/smoke.sh",
    "crates/sos-dev/*",
];

/// Instruction and doc files agents load into context, above all the prompts `sos install` ships
/// into every app. archive/ is history and not loaded.
pub const UNICODE_SCAN: &[&str] = &[
    "CLAUDE.md",
    "README.md",
    "harness-lite/",
    "templates/",
    "adapters/",
    "skills/",
    "recipes/",
    "docs/",
    ".claude/",
    "SECURITY.md",
];

/// One shell-glob segment against one path segment: `*` is any run of characters, `?` one
/// character, everything else literal; a wildcard never matches a leading dot (bash without
/// dotglob, which is how the old script expanded its unquoted globs).
fn segment_matches(pat: &str, name: &str) -> bool {
    if name.starts_with('.') && !pat.starts_with('.') {
        return false;
    }
    let p: Vec<char> = pat.chars().collect();
    let s: Vec<char> = name.chars().collect();
    let (mut pi, mut si, mut star, mut mark) = (0usize, 0usize, None::<usize>, 0usize);
    while si < s.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == s[si]) {
            pi += 1;
            si += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            mark = si;
            pi += 1;
        } else if let Some(st) = star {
            pi = st + 1;
            mark += 1;
            si = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

fn has_wildcard(g: &str) -> bool {
    g.contains(['*', '?'])
}

/// Bash's expansion of one glob, then git's listing of the words: the first segments of the path
/// match the glob segment by segment; a glob that names a directory covers everything under it
/// (git lists a directory pathspec recursively).
fn shell_glob_matches(g: &str, path: &str) -> bool {
    let segs: Vec<&str> = path.split('/').collect();
    let gs: Vec<&str> = g.split('/').collect();
    gs.len() <= segs.len() && gs.iter().zip(&segs).all(|(p, s)| segment_matches(p, s))
}

/// Bash's expansion of one glob into words, as the old script's unquoted `$SURFACE_GLOBS` did:
/// the paths on disk that match it segment by segment, sorted; the glob itself when none match
/// (no nullglob). A name that is not UTF-8 in a directory a wildcard reads is an error: bash would
/// hand it to git as a word, and leaving it out would hide a new surface.
fn bash_words(root: &Path, g: &str) -> Result<Vec<String>> {
    if !has_wildcard(g) {
        return Ok(vec![g.to_string()]);
    }
    let mut found = vec![String::new()];
    for seg in g.split('/') {
        let mut next = Vec::new();
        for d in &found {
            let join = |n: &str| if d.is_empty() { n.to_string() } else { format!("{d}/{n}") };
            if has_wildcard(seg) {
                let Ok(rd) = fs::read_dir(root.join(d)) else { continue };
                for e in rd.flatten() {
                    let os = e.file_name();
                    let Some(n) = os.to_str() else {
                        bail!(
                            "file name is not UTF-8: {} (rename it; sos-dev cannot check it safely)",
                            root.join(d).join(&os).display()
                        );
                    };
                    if segment_matches(seg, n) {
                        next.push(join(n));
                    }
                }
            } else if fs::symlink_metadata(root.join(d).join(seg)).is_ok() {
                next.push(join(seg));
            }
        }
        found = next;
    }
    if found.is_empty() {
        return Ok(vec![g.to_string()]);
    }
    found.sort();
    Ok(found)
}

/// `is_surface` for a tree where every wildcard glob matches something on disk (the usual case).
pub fn is_surface(path: &str) -> bool {
    SURFACES.iter().any(|g| shell_glob_matches(g, path))
}

/// The tracked files git lists for the old script's `git ls-files -- $SURFACE_GLOBS`: bash turns
/// each glob into words, then git itself reads each word as a pathspec (so a file name with a `*`
/// or a `[[:punct:]]` in it widens the match exactly as it did there).
fn tracked_surfaces(root: &Path) -> Result<Vec<String>> {
    let mut words = Vec::new();
    for g in SURFACES {
        words.extend(bash_words(root, g)?);
    }
    let mut args = vec!["--"];
    args.extend(words.iter().map(String::as_str));
    let mut v = git::ls_files(root, &args)?;
    v.sort();
    v.dedup();
    Ok(v)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// SHA-256 of the file as it is in the working tree, or None when it is not a readable file. A
/// symlink is followed and its target's content hashed, as `sha256sum` does: what runs is the
/// target. A dangling link is None, so the check blocks on it.
fn hash_file(path: &Path) -> Option<String> {
    if !fs::metadata(path).ok()?.is_file() {
        return None;
    }
    fs::read(path).ok().map(|b| hex(&Sha256::digest(&b)))
}

/// Baseline lines are `<sha256>  <path>` (two spaces). A `*` before the path (binary-mode
/// sha256sum on Windows) is dropped, as the old script's normaliser did.
fn read_baseline(root: &Path) -> Result<BTreeMap<String, String>> {
    let text = fs::read_to_string(root.join(BASELINE)).with_context(|| format!("cannot read {BASELINE}"))?;
    let mut map = BTreeMap::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }
        let Some((hash, path)) = line.split_once(' ') else {
            bail!("{BASELINE}:{}: not `<sha256>  <path>`: {line}", n + 1);
        };
        let path = path.trim_start_matches(' ').trim_start_matches('*');
        if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) || path.is_empty() {
            bail!("{BASELINE}:{}: not `<sha256>  <path>`: {line}", n + 1);
        }
        if map.insert(path.to_string(), hash.to_ascii_lowercase()).is_some() {
            bail!("{BASELINE}:{}: {path} is listed twice; regenerate with `sos-dev trust-gate rebaseline`", n + 1);
        }
    }
    Ok(map)
}

pub fn rebaseline(root: &Path) -> Result<()> {
    println!("trust-gate: regenerating {BASELINE} ...");
    let untracked: Vec<String> = git::ls_files(root, &["--others", "--exclude-standard"])?
        .into_iter()
        .filter(|p| is_surface(p))
        .collect();
    if !untracked.is_empty() {
        println!();
        eprintln!("WARNING: trust-gate: untracked auto-exec-matching file(s) found:");
        for f in &untracked {
            eprintln!("  {f}");
        }
        eprintln!("  These files are NOT in the baseline (git ls-files skips untracked).");
        eprintln!("  Run: git add <file> THEN rebaseline to include them.");
        println!();
    }
    let surfaces = tracked_surfaces(root)?;
    if surfaces.is_empty() {
        eprintln!("WARNING: trust-gate: no surfaces matched. Is this a sos-kit repo?");
    }
    let mut out = String::new();
    let mut count = 0;
    for f in &surfaces {
        match hash_file(&root.join(f)) {
            Some(h) => {
                out.push_str(&format!("{h}  {f}\n"));
                count += 1;
            }
            None => eprintln!("WARNING: trust-gate: surface listed by git ls-files but not found on disk: {f}"),
        }
    }
    write_replace(root, out.as_bytes())?;
    println!("trust-gate: baseline written → {BASELINE} ({count} surfaces)");
    println!();
    println!("REMINDER: run `git add {BASELINE}` to stage the updated baseline.");
    println!("REMINDER: new auto-exec file must be `git add`ed BEFORE rebaseline — git ls-files won't see untracked files.");
    Ok(())
}

/// Write the baseline through a temp file created new (never an existing file or a symlink that
/// someone left at that name), then rename it over the baseline.
fn write_replace(root: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    for n in 0..100u32 {
        let tmp = root.join(format!("{BASELINE}.{}.{n}.tmp", std::process::id()));
        let mut f = match fs::OpenOptions::new().write(true).create_new(true).open(&tmp) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e).with_context(|| format!("cannot create {}", tmp.display())),
        };
        let done = f.write_all(bytes).and_then(|_| f.sync_all()).and_then(|_| {
            drop(f);
            fs::rename(&tmp, root.join(BASELINE))
        });
        if let Err(e) = done {
            let _ = fs::remove_file(&tmp);
            return Err(e).with_context(|| format!("cannot replace {BASELINE}"));
        }
        return Ok(());
    }
    bail!("cannot create a temp file for {BASELINE} (100 names taken)")
}

/// Check 1. Returns true when the surfaces equal the baseline.
pub fn check_baseline(root: &Path) -> Result<bool> {
    if !root.join(BASELINE).is_file() {
        eprintln!("BLOCKED: trust-gate: {BASELINE} not found.");
        eprintln!("  Generate it: sos-dev trust-gate rebaseline");
        eprintln!("  Then: git add {BASELINE} && git commit");
        return Ok(false);
    }
    let base = read_baseline(root)?;
    let surfaces = tracked_surfaces(root)?;
    let mut changed = Vec::new();
    let mut added = Vec::new();
    for f in &surfaces {
        let now = hash_file(&root.join(f));
        if now.is_none() {
            eprintln!("WARNING: trust-gate: tracked surface missing on disk: {f}");
        }
        match base.get(f) {
            None => added.push(f.clone()),
            Some(h) if now.as_deref() != Some(h.as_str()) => changed.push(f.clone()),
            Some(_) => {}
        }
    }
    let current: BTreeSet<&String> = surfaces.iter().collect();
    let removed: Vec<&String> = base.keys().filter(|k| !current.contains(k)).collect();
    if changed.is_empty() && added.is_empty() && removed.is_empty() {
        return Ok(true);
    }
    eprintln!();
    eprintln!("BLOCKED: trust-gate: auto-exec surface(s) changed vs baseline.");
    let list = |title: &str, items: Vec<&String>| {
        if !items.is_empty() {
            eprintln!("  {title}:");
            for i in items {
                eprintln!("    {i}");
            }
        }
    };
    list("Changed", changed.iter().collect());
    list("Added (not in baseline)", added.iter().collect());
    list("Removed (was in baseline)", removed);
    eprintln!();
    eprintln!("  Review the diff in this PR, then rebaseline:");
    eprintln!("    sos-dev trust-gate rebaseline   (cargo run -q -p sos-dev -- trust-gate rebaseline)");
    eprintln!("    git add {BASELINE}");
    eprintln!("  (A new auto-exec file must be `git add`ed BEFORE rebaseline.)");
    Ok(false)
}

/// The hidden code points, by their UTF-8 bytes. The tag block U+E0000–U+E007F is matched by its
/// two 3-byte prefixes, so a truncated sequence is caught too.
const HIDDEN: &[(&[u8], &str)] = &[
    (b"\xef\xbb\xbf", "U+FEFF byte order mark"),
    (b"\xe2\x80\x8b", "U+200B zero width space"),
    (b"\xe2\x80\x8c", "U+200C zero width non-joiner"),
    (b"\xe2\x80\x8d", "U+200D zero width joiner"),
    (b"\xe2\x80\x8e", "U+200E left-to-right mark"),
    (b"\xe2\x80\x8f", "U+200F right-to-left mark"),
    (b"\xe2\x81\xa0", "U+2060 word joiner"),
    (b"\xe1\xa0\x8e", "U+180E Mongolian vowel separator"),
    (b"\xe2\x80\xaa", "U+202A left-to-right embedding"),
    (b"\xe2\x80\xab", "U+202B right-to-left embedding"),
    (b"\xe2\x80\xac", "U+202C pop directional formatting"),
    (b"\xe2\x80\xad", "U+202D left-to-right override"),
    (b"\xe2\x80\xae", "U+202E right-to-left override"),
    (b"\xe2\x81\xa6", "U+2066 left-to-right isolate"),
    (b"\xe2\x81\xa7", "U+2067 right-to-left isolate"),
    (b"\xe2\x81\xa8", "U+2068 first strong isolate"),
    (b"\xe2\x81\xa9", "U+2069 pop directional isolate"),
    (b"\xf3\xa0\x80", "tag character (U+E0000 block)"),
    (b"\xf3\xa0\x81", "tag character (U+E0000 block)"),
];

/// `path:line:column: name` for every hidden code point in `bytes` (column counts bytes, 1-based).
pub fn hidden_in(path: &str, bytes: &[u8]) -> Vec<String> {
    let mut hits = Vec::new();
    for (n, line) in bytes.split(|b| *b == b'\n').enumerate() {
        let mut i = 0;
        while i < line.len() {
            if let Some((pat, name)) = HIDDEN.iter().find(|(p, _)| line[i..].starts_with(p)) {
                let name = match (line.get(i + 3), pat[0]) {
                    (Some(b), 0xf3) if (0x80..=0xbf).contains(b) => {
                        let cp = 0xE0000 + (((pat[2] & 0x3f) as u32) << 6) + ((b & 0x3f) as u32);
                        format!("U+{cp:05X} tag character")
                    }
                    _ => name.to_string(),
                };
                hits.push(format!("{path}:{}:{}: {name}", n + 1, i + 1));
                i += pat.len();
            } else {
                i += 1;
            }
        }
    }
    hits
}

/// Tracked and untracked-but-not-ignored files under these pathspecs (gitignored files cannot be
/// committed by accident; `.claude/worktrees/` and build output stay out).
fn listed(root: &Path, specs: &[String]) -> Result<Vec<String>> {
    if specs.is_empty() {
        return Ok(Vec::new());
    }
    let mut args = vec!["--cached", "--others", "--exclude-standard", "--"];
    args.extend(specs.iter().map(String::as_str));
    let mut files = git::ls_files(root, &args)?;
    files.sort();
    files.dedup();
    Ok(files)
}

/// Walks the scan list. Symlinks are followed, since an agent loading the path reads the target:
/// a link to a file is scanned as that file; a link to a folder inside the repo has the folder's
/// files scanned wherever they live; a dangling link, or a link to a folder outside the repo,
/// cannot be vetted and is a hit. Each real file is read once.
struct Scan {
    root: std::path::PathBuf,
    canon_root: std::path::PathBuf,
    seen_files: BTreeSet<std::path::PathBuf>,
    seen_dirs: BTreeSet<String>,
    hits: Vec<String>,
}

impl Scan {
    /// One path from the scan list or from a listing, relative to the root.
    fn path(&mut self, rel: &str) -> Result<()> {
        let p = self.root.join(rel);
        let link = match fs::symlink_metadata(&p) {
            Ok(m) => m.file_type().is_symlink(),
            // Deleted in the working tree but still in the index: nothing on disk to load.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e).with_context(|| format!("cannot stat {rel}")),
        };
        let meta = match fs::metadata(&p) {
            Ok(m) => m,
            Err(_) if link => {
                self.hits.push(format!("{rel}: dangling symlink (cannot scan its target)"));
                return Ok(());
            }
            Err(e) => return Err(e).with_context(|| format!("cannot stat {rel}")),
        };
        if meta.is_dir() {
            if !link {
                // A real folder from the scan list: git lists its files.
                return self.tree(rel.trim_end_matches('/'));
            }
            let target = fs::canonicalize(&p).ok();
            match target.as_ref().and_then(|t| t.strip_prefix(&self.canon_root).ok()).and_then(|r| r.to_str()) {
                // Only Windows separators are rewritten: on Unix a `\` is part of a name.
                #[cfg(windows)]
                Some(r) => return self.tree(&r.replace('\\', "/")),
                #[cfg(not(windows))]
                Some(r) => return self.tree(r),
                None => {
                    self.hits.push(format!("{rel}: symlink to a folder outside the repo (cannot scan it)"));
                    return Ok(());
                }
            }
        }
        if !meta.is_file() {
            return Ok(());
        }
        let real = fs::canonicalize(&p).with_context(|| format!("cannot resolve {rel}"))?;
        if !self.seen_files.insert(real) {
            return Ok(());
        }
        let bytes = fs::read(&p).with_context(|| format!("cannot read {rel}"))?;
        self.hits.extend(hidden_in(rel, &bytes));
        Ok(())
    }

    /// The files git lists under a real (symlink-free) folder of the repo; "" is the root.
    fn tree(&mut self, rel: &str) -> Result<()> {
        if !self.seen_dirs.insert(rel.to_string()) {
            return Ok(());
        }
        // Literal: a folder named `a[b]` or `*` is that folder, not a pattern.
        let spec = if rel.is_empty() { ".".to_string() } else { format!(":(literal){rel}/") };
        for f in listed(&self.root, &[spec])? {
            self.path(&f)?;
        }
        Ok(())
    }
}

/// Check 2. Returns true when no hidden code point is found.
pub fn check_unicode(root: &Path) -> Result<bool> {
    let canon_root = fs::canonicalize(root).with_context(|| format!("cannot resolve {}", root.display()))?;
    let mut scan = Scan { root: root.to_path_buf(), canon_root, seen_files: BTreeSet::new(), seen_dirs: BTreeSet::new(), hits: Vec::new() };
    for t in UNICODE_SCAN {
        // The entry itself may be a symlink (`skills -> src/prompts`); git would list nothing
        // under `skills/`, so the entry is resolved first.
        scan.path(t.trim_end_matches('/'))?;
    }
    let hits = scan.hits;
    if hits.is_empty() {
        return Ok(true);
    }
    eprintln!();
    eprintln!("BLOCKED: trust-gate: hidden Unicode codepoint(s) found in instruction/doc files.");
    eprintln!("  (U+FEFF BOM, U+200B/C/D zero-width, U+200E/F + U+202A-E + U+2066-9 bidi, U+2060 word-joiner,");
    eprintln!("   U+180E Mongolian vowel sep, U+E0000 tags = prompt injection vector)");
    eprintln!();
    for h in &hits {
        eprintln!("  {h}");
    }
    eprintln!();
    eprintln!("  Fix: replace the raw codepoint with its name (e.g. U+FEFF) in the source.");
    eprintln!("  Never paste raw hidden chars — use codepoint escapes or literal names.");
    Ok(false)
}

/// `sos-dev trust-gate list`: the surfaces the baseline covers, one per line.
pub fn list(root: &Path) -> Result<()> {
    for f in tracked_surfaces(root)? {
        println!("{f}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn globs_follow_bash_expansion() {
        assert!(is_surface("scripts/bite"));
        assert!(is_surface("adapters/claude/hook.py"));
        assert!(!is_surface("adapters/README.md"));
        assert!(is_surface("crates/sos-dev/src/trust.rs"));
        assert!(is_surface("crates/sos-dev/Cargo.toml"));
        assert!(is_surface("scripts/sub/deep.sh"));
        assert!(!is_surface("scripts/.hidden"));
        assert!(!is_surface("scripts"));
        assert!(is_surface(".github/workflows/ci.yml"));
        assert!(!is_surface("crates/sos-cli/src/main.rs"));
        assert!(!is_surface("install.sh.bak"));
        assert!(segment_matches("a*c", "abbc"));
        assert!(!segment_matches("a*c", "abcd"));
        assert!(segment_matches("*", "x"));
    }

    #[test]
    fn hidden_code_points_named() {
        let hits = hidden_in("f.md", "ok\na\u{200b}b \u{202e}\n\u{e0041}".as_bytes());
        assert_eq!(hits, vec![
            "f.md:2:2: U+200B zero width space",
            "f.md:2:7: U+202E right-to-left override",
            "f.md:3:1: U+E0041 tag character",
        ]);
        assert_eq!(hidden_in("f.md", b"\xf3\xa0\x81"), vec!["f.md:1:1: tag character (U+E0000 block)"]);
        assert_eq!(hidden_in("f.md", "\u{feff}# t".as_bytes()), vec!["f.md:1:1: U+FEFF byte order mark"]);
        assert!(hidden_in("f.md", "Tiếng Việt — “ok” ✅".as_bytes()).is_empty());
    }
}
