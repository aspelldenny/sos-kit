//! `sos check-links <dir>` — relative links and heading anchors across a folder of Markdown
//! (wave 3.2; port of the product-description skill's `references/check-links.py`).
//!
//! Every `[text](target)` (not an image `![…](…)`) outside fenced code is checked: the file must
//! exist, and a `#anchor` into a `.md` file must match one of its heading slugs. Targets with a
//! scheme (`http:`, `mailto:` …) are skipped. Slugs follow the script: drop inline-code ticks, link
//! targets and `*`/`_`, NFKD + lowercase, keep letters, digits, spaces and hyphens, spaces to
//! hyphens, repeats get `-1`, `-2` …. Exits 1 when anything is broken.
//!
//! Text rules follow Python's `str` rules so both versions flag the same links (`\s` includes
//! U+001C..U+001F, lines split like `str.splitlines`). Intended differences: a folder that does not
//! exist (or is not a folder) is an error, exit 2 (the script reported "0 files … 0 broken" and
//! passed); a `.md` file that cannot be read or is not UTF-8 is reported as broken instead of
//! crashing the script; anchors into such a file count as missing; a FIFO or socket named `.md` is
//! reported as not a regular file instead of blocking; output lines end in LF on Windows too; a
//! file name that is not UTF-8 is printed with U+FFFD (the script crashed printing it); exit 1
//! when stdout cannot be written.

use anyhow::Result;
use std::io::Write;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf, MAIN_SEPARATOR};
use unicode_normalization::UnicodeNormalization;

use crate::predict::{abspath, emit, splitlines};

#[derive(clap::Args)]
pub struct Args {
    /// The folder to check (default: the current folder).
    #[arg(default_value = ".")]
    dir: PathBuf,
}

struct Pats {
    link: Regex,
    heading: Regex,
    fence: Regex,
    scheme: Regex,
    code: Regex,
    inner_link: Regex,
    emphasis: Regex,
    not_slug: Regex,
}

impl Pats {
    fn new() -> Self {
        // `\s` here is Python's: Unicode whitespace plus U+001C..U+001F.
        let s = r"[\s\x1c-\x1f]";
        Pats {
            // The script's `(?<!\!)` look-behind is applied by hand in `links`.
            link: Regex::new(&format!(r#"\[[^\]]*\]\(([^)\s\x1c-\x1f]+)(?:{s}+"[^"]*")?\)"#)).unwrap(),
            heading: Regex::new(&format!(r"^(#{{1,6}}){s}+(.*?){s}*#*{s}*$")).unwrap(),
            fence: Regex::new(&format!(r"^{s}*(```|~~~)")).unwrap(),
            scheme: Regex::new(r"^[a-z][a-z0-9+.-]*:").unwrap(),
            code: Regex::new(r"`([^`]*)`").unwrap(),
            inner_link: Regex::new(r"\[([^\]]*)\]\([^)]*\)").unwrap(),
            emphasis: Regex::new(r"[*_]").unwrap(),
            // Python's `isalnum`: letters and numbers.
            not_slug: Regex::new(r"[^\p{L}\p{N} \-]").unwrap(),
        }
    }

    /// Link targets on one line, like `re.findall` with the look-behind: a match right after `!`
    /// is dropped and the search resumes one character later.
    fn links<'a>(&self, line: &'a str) -> Vec<&'a str> {
        let mut out = Vec::new();
        let mut pos = 0;
        while let Some(c) = self.link.captures_at(line, pos) {
            let m = c.get(0).unwrap();
            if line[..m.start()].ends_with('!') {
                pos = m.start() + 1; // `[` is one byte
                continue;
            }
            out.push(c.get(1).unwrap().as_str());
            pos = m.end();
        }
        out
    }

    fn slugify(&self, text: &str) -> String {
        let t = self.code.replace_all(text, "$1");
        let t = self.inner_link.replace_all(&t, "$1");
        let t = self.emphasis.replace_all(&t, "");
        let t: String = t.nfkd().collect::<String>().to_lowercase();
        self.not_slug.replace_all(&t, "").replace(' ', "-")
    }
}

/// Lines with fenced code blanked, numbered as in the file.
fn strip_fences<'a>(p: &Pats, lines: Vec<&'a str>) -> Vec<&'a str> {
    let mut in_fence = false;
    lines.into_iter().map(|l| {
        if p.fence.is_match(l) {
            in_fence = !in_fence;
            ""
        } else if in_fence { "" } else { l }
    }).collect()
}

fn read_text(path: &Path) -> Result<String, String> {
    // Read what Python's open() reads (character devices such as /dev/null included), but not a
    // pipe or socket, which would block.
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        if let Ok(m) = std::fs::metadata(path) {
            let t = m.file_type();
            if t.is_fifo() || t.is_socket() {
                return Err("not a regular file".into());
            }
        }
    }
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read: {e}"))?;
    String::from_utf8(bytes).map_err(|_| "not UTF-8".into())
}

fn headings_of(p: &Pats, path: &Path) -> HashSet<String> {
    let Ok(text) = read_text(path) else { return HashSet::new() };
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut slugs = HashSet::new();
    for line in strip_fences(p, splitlines(&text)) {
        let Some(c) = p.heading.captures(line) else { continue };
        let base = p.slugify(c.get(2).unwrap().as_str());
        let n = seen.entry(base.clone()).or_insert(0);
        slugs.insert(if *n == 0 { base.clone() } else { format!("{base}-{n}") });
        *n += 1;
    }
    slugs
}

/// `.md` files under `root` like `os.walk`: a folder's files in directory order, then each
/// subfolder; hidden folders and `node_modules` are not entered, nor is a link to a folder. Each
/// comes with its relative name, joined with the platform separator, for printing and sorting.
fn md_files(dir: &Path, rel: &str, out: &mut Vec<(String, PathBuf)>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut dirs = Vec::new();
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let r = if rel.is_empty() { name.clone() } else { format!("{rel}{MAIN_SEPARATOR}{name}") };
        let path = e.path();
        if path.is_dir() {
            let is_link = e.file_type().is_ok_and(|t| t.is_symlink());
            if !is_link && !name.starts_with('.') && name != "node_modules" {
                dirs.push((r, path));
            }
        } else if name.ends_with(".md") {
            out.push((r, path));
        }
    }
    for (r, path) in dirs {
        md_files(&path, &r, out);
    }
}

/// `os.path.join(folder, target)`. On Windows a drive-relative target (`C:b.md`) on the folder's
/// own drive is joined under the folder, as ntpath does; Rust's join would drop the folder.
fn py_join(folder: &Path, target: &str) -> PathBuf {
    #[cfg(windows)]
    {
        let t = Path::new(target);
        let mut comps = t.components();
        if let (Some(Component::Prefix(tp)), Some(Component::Prefix(fp))) = (comps.next(), folder.components().next()) {
            if !t.has_root() && tp.as_os_str().eq_ignore_ascii_case(fp.as_os_str()) {
                return folder.join(comps.as_path());
            }
        }
    }
    folder.join(target)
}

/// `os.path.normpath`: `.` and `..` folded without following links. A `..` above the root is
/// dropped; one with nothing to fold in a path without a root (`D:../b.md`) is kept.
fn normpath(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => match out.components().next_back() {
                Some(Component::Normal(_)) => {
                    out.pop();
                }
                Some(Component::RootDir) => {}
                _ => out.push(".."),
            },
            other => out.push(other),
        }
    }
    out
}

pub fn run(a: Args) -> Result<i32> {
    let root = abspath(&a.dir);
    if !root.is_dir() {
        let _ = writeln!(std::io::stderr(), "sos check-links: {} is not a folder", root.display());
        return Ok(2);
    }
    let p = Pats::new();
    let mut files = Vec::new();
    md_files(&root, "", &mut files);
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut anchors: HashMap<PathBuf, HashSet<String>> = HashMap::new();
    let mut problems = Vec::new();
    let mut checked = 0usize;
    for (rel, path) in &files {
        let text = match read_text(path) {
            Ok(t) => t,
            Err(e) => {
                problems.push(format!("{rel}: {e}"));
                continue;
            }
        };
        for (i, line) in strip_fences(&p, splitlines(&text)).into_iter().enumerate() {
            for target in p.links(line) {
                if p.scheme.is_match(target) {
                    continue;
                }
                checked += 1;
                let (file_part, anchor) = target.split_once('#').unwrap_or((target, ""));
                let dest = if file_part.is_empty() {
                    path.clone()
                } else {
                    normpath(&py_join(path.parent().unwrap_or(&root), file_part))
                };
                if !dest.exists() {
                    problems.push(format!("{rel}:{}: missing file: {target}", i + 1));
                    continue;
                }
                if !anchor.is_empty() && dest.to_string_lossy().ends_with(".md") {
                    let set = anchors.entry(dest.clone()).or_insert_with(|| headings_of(&p, &dest));
                    if !set.contains(&anchor.to_lowercase()) {
                        problems.push(format!("{rel}:{}: missing anchor: {target}", i + 1));
                    }
                }
            }
        }
    }
    let mut out = String::new();
    for pr in &problems {
        out.push_str(pr);
        out.push('\n');
    }
    out.push_str(&format!("{} files, {checked} relative links, {} broken\n", files.len(), problems.len()));
    Ok(emit(&out, if problems.is_empty() { 0 } else { 1 }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_like_the_script() {
        let p = Pats::new();
        assert_eq!(p.slugify("Kết quả `sos gate` [xem](a.md) **đậm**"), "ket-qua-sos-gate-xem-đam");
        assert_eq!(p.slugify("Step 2: Đổi chỗ gọi"), "step-2-đoi-cho-goi");
        assert_eq!(p.slugify("A  B"), "a--b");
    }

    #[cfg(windows)]
    #[test]
    fn drive_relative_targets_join_like_ntpath() {
        let f = Path::new(r"C:\repo\docs");
        assert_eq!(py_join(f, "C:b.md"), Path::new(r"C:\repo\docs\b.md"));
        assert_eq!(py_join(f, "c:../b.md"), Path::new(r"C:\repo\docs\../b.md"));
        assert_eq!(py_join(f, "D:b.md"), Path::new("D:b.md"));
        assert_eq!(py_join(f, r"C:\x\b.md"), Path::new(r"C:\x\b.md"));
        assert_eq!(py_join(f, "b.md"), Path::new(r"C:\repo\docs\b.md"));
        // Another drive stays drive-relative, and normpath keeps the `..` it cannot fold.
        assert_eq!(normpath(&py_join(f, "D:../b.md")), Path::new(r"D:..\b.md"));
        assert_eq!(normpath(&py_join(f, "D:sub/../../b.md")), Path::new(r"D:..\b.md"));
        assert_eq!(normpath(&py_join(f, "c:../b.md")), Path::new(r"C:\repo\b.md"));
    }

    #[test]
    fn normpath_folds_like_posixpath() {
        assert_eq!(normpath(Path::new("/a/./b/../c")), Path::new("/a/c"));
        assert_eq!(normpath(Path::new("/../x")), Path::new("/x"));
        assert_eq!(normpath(Path::new("../a/../../b")), Path::new("../../b"));
    }

    #[test]
    fn image_links_are_skipped_but_a_later_link_counts() {
        let p = Pats::new();
        assert_eq!(p.links("![x](a.png) [y](b.md) ![z [w](c.md)"), ["b.md", "c.md"]);
        assert_eq!(p.links(r#"[t](d.md "title") [u](e.md#x)"#), ["d.md", "e.md#x"]);
    }
}
