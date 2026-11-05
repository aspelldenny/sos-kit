//! Path handling shared by rules and actions: `~` expansion, joining, normalising (symlinks
//! resolved through the longest existing ancestor) and glob compilation.

use regex::Regex;
use std::path::{Component, Path, PathBuf};

/// Last path component, by either separator (a check program may be written either way).
pub fn basename_of(p: &str) -> &str {
    p.rsplit(['/', '\\']).next().unwrap_or(p)
}

/// `~` / `~/x` → home; relative → `base.join`; absolute stays. Empty or NUL is an error.
pub fn expand(p: &str, home: Option<&Path>, base: &Path) -> Result<PathBuf, String> {
    if p.is_empty() {
        return Err("đường dẫn rỗng".into());
    }
    if p.contains('\0') {
        return Err("đường dẫn chứa NUL".into());
    }
    let rest = if p == "~" {
        Some("")
    } else {
        p.strip_prefix("~/").or_else(|| if cfg!(windows) { p.strip_prefix("~\\") } else { None })
    };
    if let Some(rest) = rest {
        let h = home.ok_or("không biết thư mục nhà (HOME) để mở `~`")?;
        return Ok(if rest.is_empty() { h.to_path_buf() } else { h.join(rest) });
    }
    let path = Path::new(p);
    Ok(if path.is_absolute() { path.to_path_buf() } else { base.join(path) })
}

pub(crate) fn strip_verbatim(p: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let s = p.to_string_lossy();
        if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{rest}"));
        }
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
    }
    p
}

/// Lexically apply `.` and `..` to `rest` on top of `base`.
fn push_lexical(mut base: PathBuf, rest: &[Component]) -> PathBuf {
    for c in rest {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                base.pop();
            }
            other => base.push(other.as_os_str()),
        }
    }
    base
}

/// Absolute path with symlinks of the longest existing ancestor resolved, the rest applied
/// lexically. A dangling symlink is an error (where it points cannot be known safely).
pub fn normalize(p: &Path) -> Result<PathBuf, String> {
    if !p.is_absolute() {
        return Err(format!("đường dẫn không tuyệt đối: {}", p.display()));
    }
    let comps: Vec<Component> = p.components().collect();
    // Resolve `..` lexically first only where it cannot cross a symlink we have not seen yet:
    // walk from the full path down to the root and canonicalize the first ancestor that exists.
    for i in (1..=comps.len()).rev() {
        let prefix: PathBuf = comps[..i].iter().map(|c| c.as_os_str()).collect();
        match std::fs::symlink_metadata(&prefix) {
            Ok(_) => {
                let canon = std::fs::canonicalize(&prefix).map_err(|e| {
                    format!("không lần được đường dẫn {} ({e}; symlink treo?)", prefix.display())
                })?;
                return Ok(push_lexical(strip_verbatim(canon), &comps[i..]));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            // Windows: a name with `*` or `?` (a Glob pattern) cannot exist (ERROR_INVALID_NAME).
            Err(e) if cfg!(windows) && e.raw_os_error() == Some(123) => continue,
            Err(e) => return Err(format!("không đọc được {} ({e})", prefix.display())),
        }
    }
    Ok(push_lexical(PathBuf::new(), &comps))
}

/// `/`-joined form used for glob matching.
pub fn slash(p: &Path) -> String {
    let s = p.to_string_lossy();
    if cfg!(windows) { s.replace('\\', "/") } else { s.into_owned() }
}

/// Compile a rule glob. `**/x` stays relative (matches at any depth); `~/x` and absolute globs
/// get their fixed prefix normalised so they compare against normalised action paths. Any other
/// relative glob is refused: `*.md` would silently match nothing against an absolute path.
pub fn glob(g: &str, home: Option<&Path>, _root: &Path) -> Result<Regex, String> {
    if g.is_empty() {
        return Err("glob rỗng".into());
    }
    if g.contains('{') || g.contains('[') {
        return Err(format!("glob `{g}`: chưa hỗ trợ `{{…}}` hay `[…]`; ghi thành nhiều glob"));
    }
    if g.starts_with("**/") {
        return Ok(sos_gates::glob::to_regex(g));
    }
    let is_abs = g == "~" || g.starts_with("~/") || Path::new(g).is_absolute() || g.starts_with('/');
    if !is_abs {
        return Err(format!("glob `{g}`: glob tương đối phải bắt đầu bằng `**/` (vd `**/{g}`)"));
    }
    // Split at the first component holding a wildcard; normalise the fixed part.
    let g = g.replace('\\', "/");
    let parts: Vec<&str> = g.split('/').collect();
    let k = parts.iter().position(|c| c.contains('*') || c.contains('?')).unwrap_or(parts.len());
    let fixed = parts[..k].join("/");
    let fixed = if fixed.is_empty() { "/".to_string() } else { fixed };
    let fixed = normalize(&expand(&fixed, home, Path::new("/"))?)?;
    let mut out = slash(&fixed);
    if k < parts.len() {
        if !out.ends_with('/') {
            out.push('/');
        }
        out.push_str(&parts[k..].join("/"));
    }
    Ok(sos_gates::glob::to_regex(&out))
}

/// `p` matches `set`: outside every `outside` root (when any) and some glob (when any).
pub fn in_set(p: &Path, set: &crate::config::PathSet) -> bool {
    let outside_ok = set.outside.is_empty() || set.outside.iter().all(|o| !p.starts_with(o));
    let glob_ok = set.globs.is_empty() || {
        let s = slash(p);
        set.globs.iter().any(|g| g.is_match(&s))
    };
    outside_ok && glob_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_home_and_relative() {
        let h = Path::new("/h");
        assert_eq!(expand("~", Some(h), Path::new("/b")).unwrap(), PathBuf::from("/h"));
        assert_eq!(expand("~/x", Some(h), Path::new("/b")).unwrap(), PathBuf::from("/h/x"));
        assert_eq!(expand("x/y", Some(h), Path::new("/b")).unwrap(), PathBuf::from("/b/x/y"));
        assert!(expand("~/x", None, Path::new("/b")).is_err());
        assert!(expand("", Some(h), Path::new("/b")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn normalize_resolves_symlinked_ancestor() {
        let t = tempfile::tempdir().unwrap();
        let real = t.path().join("real");
        std::fs::create_dir(&real).unwrap();
        std::os::unix::fs::symlink(&real, t.path().join("link")).unwrap();
        let n = normalize(&t.path().join("link/new/../f.md")).unwrap();
        assert_eq!(n, std::fs::canonicalize(&real).unwrap().join("f.md"));
        std::os::unix::fs::symlink(t.path().join("nowhere"), t.path().join("dangling")).unwrap();
        assert!(normalize(&t.path().join("dangling/x")).is_err());
    }

    #[test]
    fn relative_glob_needs_double_star() {
        assert!(glob("*.md", None, Path::new("/")).is_err());
        assert!(glob("**/SOUL.md", None, Path::new("/")).unwrap().is_match("/a/b/SOUL.md"));
        assert!(glob("{a,b}", None, Path::new("/")).is_err());
    }
}
