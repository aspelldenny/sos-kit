//! `sos install | update | check` end to end: the real binary on throwaway git repos.

use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::{Command, Output};

fn sos(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sos")).current_dir(dir).args(args).output().unwrap()
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git").current_dir(dir).args(args)
        .env("GIT_AUTHOR_NAME", "t").env("GIT_AUTHOR_EMAIL", "t@t").env("GIT_COMMITTER_NAME", "t").env("GIT_COMMITTER_EMAIL", "t@t")
        .status().unwrap().success();
    assert!(ok, "git {args:?}");
}

fn repo() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    git(d.path(), &["init", "-q"]);
    d
}

fn out(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn sha(b: &[u8]) -> String {
    Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect()
}

fn upstream(p: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(p.join("harness-lite/UPSTREAM.json")).unwrap()).unwrap()
}

#[test]
fn dry_run_writes_nothing() {
    let d = repo();
    let o = sos(d.path(), &["install", "--dry-run"]);
    assert!(o.status.success(), "{}", out(&o));
    let entries: Vec<_> = std::fs::read_dir(d.path()).unwrap().map(|e| e.unwrap().file_name()).collect();
    assert_eq!(entries, vec![".git"], "{entries:?}");
}

#[test]
fn install_then_check_then_reinstall_refused() {
    let d = repo();
    let p = d.path();
    let o = sos(p, &["install"]);
    assert!(o.status.success(), "{}", out(&o));
    for f in ["AGENTS.md", "CLAUDE.md", ".sos.toml", ".claude/settings.json", ".codex/hooks.json",
              "harness-lite/CONTRACT.md", "harness-lite/hooks/pre-commit", "harness-lite/UPSTREAM.json"] {
        assert!(p.join(f).exists(), "{f} missing");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let m = std::fs::metadata(p.join("harness-lite/hooks/pre-commit")).unwrap().permissions().mode();
        assert!(m & 0o111 != 0, "hook must be executable");
    }
    let hp = Command::new("git").current_dir(p).args(["config", "core.hooksPath"]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&hp.stdout).trim(), "harness-lite/hooks");
    // Every recorded hash matches the file written.
    for (path, h) in upstream(p)["files"].as_object().unwrap() {
        assert_eq!(&sha(&std::fs::read(p.join(path)).unwrap()), h.as_str().unwrap(), "{path}");
    }
    let o = sos(p, &["install"]);
    assert_eq!(o.status.code(), Some(2), "{}", out(&o));
    assert!(out(&o).contains("sos update"));
}

#[test]
fn project_files_are_never_overwritten_and_foreign_hooks_kept() {
    let d = repo();
    let p = d.path();
    std::fs::write(p.join("AGENTS.md"), "# mine\n").unwrap();
    std::fs::create_dir_all(p.join(".claude")).unwrap();
    std::fs::write(p.join(".claude/settings.json"), "{}\n").unwrap();
    git(p, &["config", "core.hooksPath", "hooks"]);
    let o = sos(p, &["install"]);
    assert!(o.status.success(), "{}", out(&o));
    assert_eq!(std::fs::read_to_string(p.join("AGENTS.md")).unwrap(), "# mine\n");
    assert_eq!(std::fs::read_to_string(p.join(".claude/settings.json")).unwrap(), "{}\n");
    let text = out(&o);
    assert!(text.contains("AGENTS.md exists not pointing"), "{text}");
    assert!(text.contains("stays hooks"), "{text}");
    let hp = Command::new("git").current_dir(p).args(["config", "core.hooksPath"]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&hp.stdout).trim(), "hooks");
    // check: the foreign hook dir has no pre-commit running sos gate -> error
    let c = sos(p, &["check"]);
    assert_eq!(c.status.code(), Some(1), "{}", out(&c));
    assert!(out(&c).contains("does not run `sos gate all`"));
}

#[test]
fn legacy_copy_conflicts_instead_of_overwriting() {
    let d = repo();
    let p = d.path();
    std::fs::create_dir_all(p.join("harness-lite")).unwrap();
    std::fs::write(p.join("harness-lite/CONTRACT.md"), "old local contract\n").unwrap();
    let o = sos(p, &["install"]);
    assert_eq!(o.status.code(), Some(1), "conflicts exit 1: {}", out(&o));
    assert_eq!(std::fs::read_to_string(p.join("harness-lite/CONTRACT.md")).unwrap(), "old local contract\n");
    assert!(p.join("harness-lite/CONTRACT.md.sos-new").exists());
}

#[test]
fn update_replaces_unmodified_keeps_edited_and_removes_dropped() {
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    // Simulate an older install: two files at an "old kit" version recorded in UPSTREAM.json.
    let mut u = upstream(p);
    let files = u["files"].as_object_mut().unwrap();
    std::fs::write(p.join("harness-lite/roles/worker.md"), "old worker\n").unwrap();
    files.insert("harness-lite/roles/worker.md".into(), sha(b"old worker\n").into());
    std::fs::write(p.join("harness-lite/roles/reviewer.md"), "my edited reviewer\n").unwrap();
    files.insert("harness-lite/roles/reviewer.md".into(), sha(b"old reviewer\n").into());
    std::fs::write(p.join("harness-lite/scripts/gone.sh"), "x\n").unwrap();
    files.insert("harness-lite/scripts/gone.sh".into(), sha(b"x\n").into());
    std::fs::write(p.join("harness-lite/scripts/gone-edited.sh"), "edited\n").unwrap();
    files.insert("harness-lite/scripts/gone-edited.sh".into(), sha(b"y\n").into());
    std::fs::write(p.join("harness-lite/UPSTREAM.json"), serde_json::to_string(&u).unwrap()).unwrap();

    let dry = sos(p, &["update", "--dry-run"]);
    assert!(std::fs::read_to_string(p.join("harness-lite/roles/worker.md")).unwrap() == "old worker\n", "{}", out(&dry));

    let o = sos(p, &["update"]);
    let text = out(&o);
    assert_eq!(o.status.code(), Some(1), "one conflict: {text}");
    assert_ne!(std::fs::read_to_string(p.join("harness-lite/roles/worker.md")).unwrap(), "old worker\n", "unmodified file updated");
    assert_eq!(std::fs::read_to_string(p.join("harness-lite/roles/reviewer.md")).unwrap(), "my edited reviewer\n", "edited file kept");
    assert!(p.join("harness-lite/roles/reviewer.md.sos-new").exists());
    assert!(!p.join("harness-lite/scripts/gone.sh").exists(), "dropped + unmodified removed");
    assert!(p.join("harness-lite/scripts/gone-edited.sh").exists(), "dropped + edited kept");
    let u = upstream(p);
    assert!(u["files"].get("harness-lite/scripts/gone.sh").is_none());
}

#[test]
fn not_a_repo_or_not_top_level_is_an_error() {
    let d = tempfile::tempdir().unwrap();
    assert_eq!(sos(d.path(), &["install"]).status.code(), Some(2));
    let r = repo();
    std::fs::create_dir_all(r.path().join("sub")).unwrap();
    let o = sos(&r.path().join("sub"), &["install"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(out(&o).contains("top level"));
}

#[cfg(unix)]
#[test]
fn symlinked_targets_are_refused_before_any_write() {
    let d = repo();
    let p = d.path();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("victim.md"), "keep me\n").unwrap();
    std::fs::create_dir_all(p.join("harness-lite")).unwrap();
    std::os::unix::fs::symlink(outside.path().join("victim.md"), p.join("harness-lite/CONTRACT.md")).unwrap();
    let o = sos(p, &["install"]);
    assert_eq!(o.status.code(), Some(2), "{}", out(&o));
    assert!(out(&o).contains("symlink"));
    assert_eq!(std::fs::read_to_string(outside.path().join("victim.md")).unwrap(), "keep me\n");
    assert!(!p.join("harness-lite/UPSTREAM.json").exists(), "nothing written");
    assert!(!p.join("AGENTS.md").exists(), "nothing written");
}

#[test]
fn manifest_paths_outside_harness_lite_are_rejected() {
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    std::fs::write(p.join("notes.txt"), "mine\n").unwrap();
    let mut u = upstream(p);
    u["files"].as_object_mut().unwrap().insert("harness-lite/../notes.txt".into(), sha(b"mine\n").into());
    std::fs::write(p.join("harness-lite/UPSTREAM.json"), serde_json::to_string(&u).unwrap()).unwrap();
    let o = sos(p, &["update"]);
    assert_eq!(o.status.code(), Some(2), "{}", out(&o));
    assert!(p.join("notes.txt").exists());
}

#[test]
fn pending_merge_is_not_overwritten_and_update_stays_nonzero() {
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    let mut u = upstream(p);
    std::fs::write(p.join("harness-lite/CONTRACT.md"), "my contract\n").unwrap();
    u["files"].as_object_mut().unwrap().insert("harness-lite/CONTRACT.md".into(), sha(b"old\n").into());
    std::fs::write(p.join("harness-lite/UPSTREAM.json"), serde_json::to_string(&u).unwrap()).unwrap();
    std::fs::write(p.join("harness-lite/CONTRACT.md.sos-new"), "half merged\n").unwrap();
    let o = sos(p, &["update"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    assert_eq!(std::fs::read_to_string(p.join("harness-lite/CONTRACT.md.sos-new")).unwrap(), "half merged\n");
    // Second run: still pending -> still non-zero.
    assert_eq!(sos(p, &["update"]).status.code(), Some(1));
}

#[test]
fn global_hooks_path_counts_as_in_use() {
    let d = repo();
    let p = d.path();
    let home = tempfile::tempdir().unwrap();
    std::fs::write(home.path().join(".gitconfig"), "[core]\n\thooksPath = /custom/hooks\n").unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_sos")).current_dir(p).arg("install")
        .env("HOME", home.path()).env("XDG_CONFIG_HOME", home.path()).env("GIT_CONFIG_NOSYSTEM", "1").output().unwrap();
    assert!(o.status.success(), "{}", out(&o));
    assert!(out(&o).contains("stays /custom/hooks"), "{}", out(&o));
    let local = Command::new("git").current_dir(p).args(["config", "--local", "--get", "core.hooksPath"]).output().unwrap();
    assert!(!local.status.success(), "no local override written");
}

#[cfg(unix)]
#[test]
fn unreadable_gitignore_is_not_replaced() {
    let d = repo();
    let p = d.path();
    std::fs::write(p.join(".gitignore"), b"secret/\n\xff\n").unwrap();
    let o = sos(p, &["install"]);
    assert_eq!(o.status.code(), Some(2), "{}", out(&o));
    assert_eq!(std::fs::read(p.join(".gitignore")).unwrap(), b"secret/\n\xff\n");
}
