//! `sos install | update | check` end to end: the real binary on throwaway git repos.

use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::{Command, Output};

fn sos(dir: &Path, args: &[&str]) -> Output {
    // A hung run must fail its test by name, not stall the whole suite.
    // Same-family reviewer unless a test asks: the default depends on whether codex is on PATH.
    let mut child = Command::new(env!("CARGO_BIN_EXE_sos")).current_dir(dir).args(args).env("SOS_REVIEWER", "same")
        .stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().unwrap();
    let start = std::time::Instant::now();
    while child.try_wait().unwrap().is_none() {
        if start.elapsed() > std::time::Duration::from_secs(60) {
            let _ = child.kill();
            panic!("sos {args:?} still running after 60 s");
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    child.wait_with_output().unwrap()
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
    assert!(text.contains("core.hooksPath stays") && text.contains("/hooks (set by you)"), "{text}");
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

#[cfg(unix)]
fn exec_hook(p: &Path, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(p, body).unwrap();
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(unix)]
#[test]
fn existing_dot_git_hooks_are_not_silently_disabled() {
    let d = repo();
    let p = d.path();
    exec_hook(&p.join(".git/hooks/pre-commit"), "#!/bin/sh\ntouch \"$(git rev-parse --show-toplevel)/old-hook-ran\"\n");
    let o = sos(p, &["install"]);
    assert!(o.status.success(), "{}", out(&o));
    assert!(out(&o).contains("existing hooks in") && out(&o).contains("pre-commit"), "{}", out(&o));
    let hp = Command::new("git").current_dir(p).args(["config", "--get", "core.hooksPath"]).output().unwrap();
    assert!(!hp.status.success(), "hooksPath must not be set while an old hook is active");
    assert_eq!(sos(p, &["check"]).status.code(), Some(1), "check reports the unwired gates");
}

#[cfg(unix)]
#[test]
fn force_hooks_keeps_old_hooks_running_after_the_gates() {
    let d = repo();
    let p = d.path();
    exec_hook(&p.join(".git/hooks/pre-commit"), "#!/bin/sh\ntouch \"$(git rev-parse --show-toplevel)/old-hook-ran\"\n");
    exec_hook(&p.join(".git/hooks/commit-msg"), "#!/bin/sh\nexit 0\n");
    let o = sos(p, &["install", "--force-hooks"]);
    assert!(o.status.success(), "{}", out(&o));
    assert!(p.join("harness-lite/hooks/pre-commit.local").exists());
    assert!(p.join("harness-lite/hooks/commit-msg").exists());
    let body = std::fs::read_to_string(p.join("harness-lite/hooks/pre-commit")).unwrap();
    assert!(body.contains("pre-commit.local"), "kit hook chains the local one");
}

#[cfg(unix)]
#[test]
fn hook_migration_conflict_stops_before_any_write() {
    let d = repo();
    let p = d.path();
    exec_hook(&p.join(".git/hooks/pre-commit"), "#!/bin/sh\necho old\n");
    std::fs::create_dir_all(p.join("harness-lite/hooks")).unwrap();
    std::fs::write(p.join("harness-lite/hooks/pre-commit.local"), "different\n").unwrap();
    let o = sos(p, &["install", "--force-hooks"]);
    assert_eq!(o.status.code(), Some(2), "{}", out(&o));
    assert!(!p.join("AGENTS.md").exists() && !p.join("harness-lite/CONTRACT.md").exists(), "nothing written");
}

#[test]
fn role_wrappers_carry_model_and_effort_and_check_flags_missing_ones() {
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    let w = std::fs::read_to_string(p.join(".claude/agents/worker.md")).unwrap();
    assert!(w.contains("model: sonnet") && w.contains("effort: medium"), "{w}");
    let r = std::fs::read_to_string(p.join(".claude/agents/reviewer.md")).unwrap();
    assert!(r.contains("effort: high"), "{r}");
    let s: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(p.join(".claude/settings.json")).unwrap()).unwrap();
    assert_eq!(s["advisorModel"], "opus");
    assert!(!out(&sos(p, &["check"])).contains("sets no"));
    std::fs::write(p.join(".claude/agents/worker.md"), "---\nname: worker\n---\nbody\n").unwrap();
    let c = out(&sos(p, &["check"]));
    assert!(c.contains("worker.md sets no model or effort"), "{c}");
}

#[test]
fn check_flags_role_drift_retired_tools_missing_paths_and_merged_worktrees() {
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    assert!(!out(&sos(p, &["check"])).contains("make them agree"), "template wrappers match template [roles]");
    let toml = std::fs::read_to_string(p.join(".sos.toml")).unwrap()
        .replace("[roles.worker]\nbackend = \"claude\"\nmodel = \"sonnet\"", "[roles.worker]\nbackend = \"claude\"\nmodel = \"haiku\"");
    std::fs::write(p.join(".sos.toml"), toml).unwrap();
    let mut agents = std::fs::read_to_string(p.join("AGENTS.md")).unwrap();
    agents.push_str("\nWording: `quality-gate` in pre-commit; rules in `.quality-gate.toml`. Roles: `agents/roles/worker.md`, `harness-lite/roles/worker.md`.\n");
    std::fs::write(p.join("AGENTS.md"), agents).unwrap();
    git(p, &["add", "-A"]);
    git(p, &["commit", "-qm", "init", "--no-verify"]);
    git(p, &["branch", "-M", "main"]);
    git(p, &["worktree", "add", "-q", "-b", "fresh-slice", p.join("fresh").to_str().unwrap()]);
    git(p, &["worktree", "add", "-q", "-b", "done-slice", p.join("wt").to_str().unwrap()]);
    std::fs::write(p.join("wt/slice.txt"), "x\n").unwrap();
    git(&p.join("wt"), &["add", "slice.txt"]);
    git(&p.join("wt"), &["commit", "-qm", "slice", "--no-verify"]);
    git(p, &["merge", "-q", "--ff-only", "done-slice"]);
    let c = out(&sos(p, &["check"]));
    assert!(c.contains("worker.md says sonnet/medium but .sos.toml [roles.worker] says haiku/medium"), "{c}");
    assert!(c.contains("retired tools: `quality-gate`") && c.contains("`agents/roles/`"), "{c}");
    assert!(!c.contains(".quality-gate.toml` (now"), "{c}");
    assert!(c.contains("paths that do not exist: agents/roles/worker.md") && !c.contains("harness-lite/roles/worker.md,"), "{c}");
    assert!(c.contains("1 clean worktree(s) on branches already merged") && c.contains("/wt") && !c.contains("/fresh"), "{c}");
}

#[test]
fn check_warns_on_oversized_entry_and_state() {
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    let c = out(&sos(p, &["check"]));
    assert!(!c.contains("keep under"), "fresh install is within budget: {c}");
    let mut agents = std::fs::read_to_string(p.join("AGENTS.md")).unwrap();
    agents.push_str("\nRotate with `doc-rotate apply`; caps were `doc-rotate cap-check`.\n");
    agents.push_str(&"filler\n".repeat(200));
    std::fs::write(p.join("AGENTS.md"), agents).unwrap();
    let backlog = format!("# Backlog\n\n## Now\n{}## Later\n- [ ] x\n", "- [ ] item\n".repeat(90));
    std::fs::write(p.join("docs/BACKLOG.md"), backlog).unwrap();
    let c = out(&sos(p, &["check"]));
    assert!(c.contains("AGENTS.md + CLAUDE.md are") && c.contains("keep under 200"), "{c}");
    assert!(c.contains("docs/BACKLOG.md first section is 91 lines"), "{c}");
    assert!(c.contains("`doc-rotate cap-check` (now `sos gate docs`)") && !c.contains("`doc-rotate` (now"), "{c}");
}

#[test]
fn check_warns_when_the_advisor_cannot_run() {
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    std::fs::create_dir_all(p.join(".advise-state")).unwrap();
    let log = |lines: &[&str]| std::fs::write(p.join(".advise-state/calls.jsonl"), lines.join("\n") + "\n").unwrap();
    let down = r#"{"ts": "2026-10-17T22:59:22", "backend": "claude", "ok": false, "error": "Failed to authenticate", "fix": "run `claude` in a terminal and /login"}"#;
    let role = r#"{"ts": "2026-10-18T09:00:00", "kind": "role", "role": "reviewer", "ok": true}"#;
    let fine = r#"{"ts": "2026-10-18T10:00:00", "backend": "claude", "ok": true}"#;
    let plain = r#"{"ts": "2026-10-18T11:00:00", "backend": "claude", "ok": false, "error": "exit 1", "fix": ""}"#;
    log(&[down, role]);
    let c = out(&sos(p, &["check"]));
    assert!(c.contains("advisor's last call (2026-10-17T22:59:22) could not run") && c.contains("/login"), "{c}");
    log(&[down, fine]);
    assert!(!out(&sos(p, &["check"])).contains("advisor"), "a later good call clears it");
    log(&[plain]);
    assert!(!out(&sos(p, &["check"])).contains("advisor"), "an ordinary failure is not a broken advisor");
}

fn sos_env(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_sos"));
    c.current_dir(dir).args(args);
    for (k, v) in env {
        c.env(k, v);
    }
    c.output().unwrap()
}

#[test]
fn install_sets_a_reviewer_from_the_other_family_when_both_clis_exist() {
    let d = repo();
    let p = d.path();
    let o = sos_env(p, &["install"], &[("SOS_REVIEWER", "other")]);
    assert!(o.status.success(), "{}", out(&o));
    let toml = std::fs::read_to_string(p.join(".sos.toml")).unwrap();
    assert!(toml.contains("[roles.reviewer.claude]   # Claude built it: Codex reviews\nbackend = \"codex\""), "{toml}");
    assert!(toml.contains("[roles.reviewer.codex]") && !toml.contains("[roles.reviewer]\n"), "{toml}");
    let c = out(&sos(p, &["check"]));
    assert!(c.contains("sos check: ok") && !c.contains("reviewer"), "{c}");
    std::fs::write(p.join("b.md"), "review it").unwrap();
    let r = out(&sos(p, &["role", "reviewer", "--from", "claude", "--brief", "b.md", "--dry-run"]));
    assert!(r.contains("\"codex\",\"exec\",\"-m\",\"gpt-6-astra\""), "{r}");
    assert!(r.contains("\"-s\",\"read-only\""), "{r}");

    let d2 = repo();
    assert!(sos_env(d2.path(), &["install"], &[("SOS_REVIEWER", "same")]).status.success());
    let same = std::fs::read_to_string(d2.path().join(".sos.toml")).unwrap();
    assert!(same.contains("[roles.reviewer]\nbackend = \"claude\"") && !same.contains("Claude built it: Codex reviews"), "{same}");
}
