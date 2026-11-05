//! The `.env` edit guard (port of scripts/env-guard.sh).

/// The block message when any path is a real .env file. Case-insensitive (macOS and Windows
/// filesystems are). The name is checked both as POSIX reads it (a backslash is part of a file name,
/// so `.env.local\notes.txt` is one name) and as Windows does (a backslash separates), so neither
/// form can hide it. A path with a
/// NUL byte is refused: no tool can write it, and the Python guard crashed on it, which blocked.
pub fn env_guard(paths: &[String]) -> Option<String> {
    let is_env = |base: &str| {
        let base = base.to_ascii_lowercase();
        base != ".env.example" && (base == ".env" || base.starts_with(".env."))
    };
    if let Some(p) = paths.iter().find(|p| p.contains('\0')) {
        return Some(format!("BLOCKED: a path to edit contains a NUL byte: {:?}\n\
                             How to fix: re-issue the edit with a plain file path.", p));
    }
    let blocked: Vec<&str> = paths.iter().map(String::as_str).filter(|p| {
        let p = p.trim_end_matches('/');
        is_env(p.rsplit('/').next().unwrap_or("")) || is_env(p.rsplit(['/', '\\']).next().unwrap_or(""))
    }).collect();
    if blocked.is_empty() {
        return None;
    }
    Some(format!(
        "BLOCKED: agents must not edit real .env files: {}\n\
         Why: .env holds live secrets; an agent edit puts them in its context and logs.\n\
         How to fix: edit .env.example (placeholders only) and ask Chủ nhà to put the real value in .env by hand.",
        blocked.join(" ")
    ))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_guard_blocks_real_env_files_only() {
        let v = |p: &[&str]| env_guard(&p.iter().map(|s| s.to_string()).collect::<Vec<_>>()).is_some();
        assert!(v(&["/r/.env"]));
        assert!(v(&["app/.Env.Local"]));
        assert!(v(&["C:\\work\\app\\.env"]));
        assert!(v(&["/REPO/.env.local\\notes.txt"]));  // POSIX: one file name
        assert!(v(&["notes\0.txt"]));
        assert!(!v(&["/r/.env.example", "notes.txt", ".envrc"]));
    }

}
