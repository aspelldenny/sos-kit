//! `sos install | update | check` end to end: the real binary on throwaway git repos.

use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::{Command, Output};

/// PATH with the sos under test first: the hooks call `sos` by name, and `sos check` looks for it.
fn path_with_sos() -> std::ffi::OsString {
    let bin = Path::new(env!("CARGO_BIN_EXE_sos")).parent().unwrap().to_path_buf();
    let rest = std::env::var_os("PATH").unwrap_or_default();
    std::env::join_paths(std::iter::once(bin).chain(std::env::split_paths(&rest))).unwrap()
}

fn sos(dir: &Path, args: &[&str]) -> Output {
    // A hung run must fail its test by name, not stall the whole suite.
    // Same-family reviewer unless a test asks: the default depends on whether codex is on PATH.
    let mut child = Command::new(env!("CARGO_BIN_EXE_sos")).current_dir(dir).args(args).env("SOS_REVIEWER", "same").env_remove("SOS_PYTHON")
        .env("PATH", path_with_sos())
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
fn crlf_checkout_is_not_a_local_edit() {
    // Windows, core.autocrlf=true: git checks the kit's LF files out as CRLF (POD-agent, 23/10/2026).
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    let advise = p.join("harness-lite/scripts/advise");
    std::fs::write(&advise, std::fs::read_to_string(&advise).unwrap().replace('\n', "\r\n")).unwrap();
    let worker = p.join(".claude/agents/worker.md");
    std::fs::write(&worker, std::fs::read_to_string(&worker).unwrap().replace('\n', "\r\n")).unwrap();
    let c = out(&sos(p, &["check"]));
    assert!(!c.contains("edited locally") && !c.contains("worker.md sets no"), "{c}");
    // But a CRLF script does not run under bash: check says so and update writes it back as LF.
    assert!(c.contains("harness-lite/scripts/advise has CRLF line endings"), "{c}");
    assert!(std::fs::read_to_string(p.join("harness-lite/.gitattributes")).unwrap().contains("eol=lf"));
    let mut u = upstream(p);
    u["files"].as_object_mut().unwrap().insert("harness-lite/roles/worker.md".into(), sha(b"old worker\n").into());
    std::fs::write(p.join("harness-lite/UPSTREAM.json"), serde_json::to_string(&u).unwrap()).unwrap();
    std::fs::write(p.join("harness-lite/roles/worker.md"), "old worker\r\n").unwrap();
    let o = sos(p, &["update"]);
    assert!(o.status.success() && !p.join("harness-lite/roles/worker.md.sos-new").exists(), "{}", out(&o));
    assert_ne!(std::fs::read_to_string(p.join("harness-lite/roles/worker.md")).unwrap(), "old worker\r\n");
    assert!(!std::fs::read(&advise).unwrap().contains(&b'\r'), "update rewrote the CRLF script as LF");
    assert!(!out(&sos(p, &["check"])).contains("CRLF"));
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
    // Git for Windows reads `/custom/hooks` under its install dir: the message names git's own answer.
    assert!(out(&o).contains("/custom/hooks (set by you)") && !out(&o).contains(r"\\?\"), "{}", out(&o));
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
    c.current_dir(dir).args(args).env("PATH", path_with_sos());
    for (k, v) in env {
        c.env(k, v);
    }
    c.output().unwrap()
}

#[cfg(unix)]
#[test]
fn hooks_call_sos_and_need_no_python() {
    // Windows: `python3` is the Store alias that exists on PATH but only fails (POD-agent, 23/10/2026).
    use std::os::unix::fs::PermissionsExt;
    let d = repo();
    let p = d.path();
    std::fs::create_dir(p.join("Store Python")).unwrap();
    let fake = p.join("Store Python/python");
    std::fs::write(&fake, "#!/bin/sh\necho 'Python was not found' >&2; exit 49\n").unwrap();
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
    let py = fake.to_str().unwrap();
    let o = sos_env(p, &["install"], &[("SOS_PYTHON", py)]);
    assert!(o.status.success(), "{}", out(&o));
    for (f, agent) in [(".claude/settings.json", "claude"), (".codex/hooks.json", "codex")] {
        let text = std::fs::read_to_string(p.join(f)).unwrap();
        assert!(text.contains(&format!("sos hook {agent} pre-edit --protocol 1")) && !text.contains("python") && !text.contains("hook.py"), "{f}: {text}");
    }
    // The advisor runs inside `sos` now: current wiring needs no Python at all.
    let c = out(&sos_env(p, &["check"], &[("SOS_PYTHON", py)]));
    assert!(!c.contains(&format!("{py} not found on PATH")), "{c}");
    // No sos on PATH while the hooks call it: every edit would be blocked.
    // A PATH holding only git: /usr/bin is not enough, Ubuntu runners ship an unrelated
    // /usr/bin/sos (sosreport).
    let bare = d.path().join("bare-path");
    std::fs::create_dir(&bare).unwrap();
    let real_git = std::env::split_paths(&std::env::var_os("PATH").unwrap()).map(|d| d.join("git")).find(|g| g.is_file()).unwrap();
    std::os::unix::fs::symlink(real_git, bare.join("git")).unwrap();
    let c = Command::new(env!("CARGO_BIN_EXE_sos")).current_dir(p).arg("check").env("PATH", &bare).output().unwrap();
    assert!(out(&c).contains("sos is not on PATH"), "{}", out(&c));
}

/// Runs one hook command the way the agent does (`sh -c`), payload on stdin.
#[cfg(unix)]
fn run_hook(dir: &Path, cmd: &str, payload: &str, path: &std::ffi::OsStr) -> Output {
    use std::io::Write;
    let mut child = Command::new("sh").arg("-c").arg(cmd).current_dir(dir).env("PATH", path).env("CLAUDE_PROJECT_DIR", dir)
        .stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().unwrap();
    let _ = child.stdin.take().unwrap().write_all(payload.as_bytes());
    child.wait_with_output().unwrap()
}

#[cfg(unix)]
#[test]
fn launcher_blocks_whenever_sos_cannot_run() {
    // Fail-closed (review 05/11/2026 points 1-2): missing, failing, too old, npm shim with no binary.
    use std::os::unix::fs::PermissionsExt;
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(p.join(".claude/settings.json")).unwrap()).unwrap();
    let cmd = v["hooks"]["PreToolUse"][0]["hooks"][0]["command"].as_str().unwrap().to_string();
    let edit = |f: &str| format!(r#"{{"tool_name":"Edit","tool_input":{{"file_path":"{f}"}}}}"#);
    // The real one: .env blocked with its own message, a normal file allowed.
    let o = run_hook(p, &cmd, &edit(".env"), &path_with_sos());
    assert!(o.status.code() == Some(2) && out(&o).contains("must not edit real .env") && !out(&o).contains("could not run"), "{}", out(&o));
    assert_eq!(run_hook(p, &cmd, &edit("notes.md"), &path_with_sos()).status.code(), Some(0));
    let fakes = tempfile::tempdir().unwrap();
    let make = |name: &str, body: &str| {
        let dir = fakes.path().join(name);
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("sos"), body).unwrap();
        std::fs::set_permissions(dir.join("sos"), std::fs::Permissions::from_mode(0o755)).unwrap();
        std::env::join_paths([dir, "/usr/bin".into(), "/bin".into()]).unwrap()
    };
    let missing = std::env::join_paths(["/usr/bin", "/bin"]).unwrap();
    let failing = make("failing", "#!/bin/sh\nexit 1\n");
    // 0.3.13: clap does not know `hook` (or `--protocol`) and exits 2 with a usage error.
    let old = make("old", "#!/bin/sh\necho \"error: unrecognized subcommand 'hook'\" >&2\nexit 2\n");
    let sh = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bin/sos-npm");
    let npm = fakes.path().join("npm");
    std::fs::create_dir_all(npm.join("pkg/bin")).unwrap();
    std::fs::copy(&sh, npm.join("pkg/bin/sos-npm")).unwrap();
    std::fs::write(npm.join("pkg/package.json"), "{\n  \"version\": \"9.9.9\",\n}\n").unwrap();
    std::fs::create_dir_all(npm.join("pkg/scripts")).unwrap();
    std::fs::write(npm.join("pkg/scripts/npm-postinstall.sh"), "echo should-not-download >&2; exit 1\n").unwrap();
    std::fs::create_dir_all(npm.join("bin")).unwrap();
    std::os::unix::fs::symlink(npm.join("pkg/bin/sos-npm"), npm.join("bin/sos")).unwrap();
    let shim = std::env::join_paths([npm.join("bin"), "/usr/bin".into(), "/bin".into()]).unwrap();
    for (name, path) in [("missing", &missing), ("failing", &failing), ("old", &old), ("npm shim, no binary", &shim)] {
        let mut c = Command::new("sh");
        c.arg("-c").arg(&cmd).current_dir(p).env("PATH", path).env("SOS_BIN_DIR", npm.join("empty")).env("HOME", npm.join("home"));
        use std::io::Write;
        let mut child = c.stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().unwrap();
        // sh may exit before reading (sos missing): a broken pipe is not the failure under test.
        let _ = child.stdin.take().unwrap().write_all(edit("notes.md").as_bytes());
        let o = child.wait_with_output().unwrap();
        assert_eq!(o.status.code(), Some(2), "{name}: {}", out(&o));
        assert!(!out(&o).contains("should-not-download"), "{name}: a hook must never download: {}", out(&o));
    }
    // A protocol this sos does not know is refused, not guessed at.
    let o = run_hook(p, "sos hook claude pre-edit --protocol 99", &edit("notes.md"), &path_with_sos());
    assert!(o.status.code() == Some(2) && out(&o).contains("protocol 99"), "{}", out(&o));
}

/// Wiring an app got from sos 0.3.13: Python adapters, no pre-bash, plus entries of its own.
const OLD_SETTINGS: &str = r#"{
  "advisorModel": "opus",
  "hooks": {
    "SessionStart": [
      { "hooks": [ { "type": "command", "command": "python3 \"$CLAUDE_PROJECT_DIR/harness-lite/adapters/claude/hook.py\" session-start" } ] }
    ],
    "PreToolUse": [
      { "matcher": "Edit|Write|MultiEdit|NotebookEdit",
        "hooks": [ { "type": "command", "command": "python3 \"$CLAUDE_PROJECT_DIR/harness-lite/adapters/claude/hook.py\" pre-edit" } ] },
      { "matcher": "Bash", "hooks": [ { "type": "command", "command": "python3 scripts/money-guard.py", "timeout": 5 } ] }
    ],
    "PostToolUse": [
      { "matcher": "Bash",
        "hooks": [ { "type": "command", "command": "python3 \"$CLAUDE_PROJECT_DIR/harness-lite/adapters/claude/hook.py\" post-bash", "timeout": 320 } ] }
    ]
  },
  "permissions": { "allow": ["Bash(git status)"] }
}
"#;

#[test]
fn update_wiring_rewrites_only_kit_commands_and_is_idempotent() {
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    std::fs::write(p.join(".claude/settings.json"), OLD_SETTINGS).unwrap();
    let c = out(&sos(p, &["check"]));
    assert!(c.contains("still calls harness-lite/adapters/claude/hook.py") && c.contains("no PreToolUse Bash hook"), "{c}");
    // Dry run: shows, writes nothing.
    let o = sos(p, &["update", "--wiring", "--dry-run"]);
    assert!(o.status.success(), "{}", out(&o));
    assert!(out(&o).contains("PreToolUse: python3") && out(&o).contains("+ pre-bash"), "{}", out(&o));
    assert_eq!(std::fs::read_to_string(p.join(".claude/settings.json")).unwrap(), OLD_SETTINGS);
    let o = sos(p, &["update", "--wiring"]);
    assert!(o.status.success(), "{}", out(&o));
    let text = std::fs::read_to_string(p.join(".claude/settings.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(!text.contains("hook.py"), "{text}");
    assert_eq!(v["advisorModel"], "opus");
    assert_eq!(v["permissions"]["allow"][0], "Bash(git status)");
    // Key order kept: the file reads as before, not re-sorted.
    assert!(text.find("advisorModel").unwrap() < text.find("\"hooks\"").unwrap() && text.find("\"hooks\"").unwrap() < text.find("permissions").unwrap(), "{text}");
    let pre = v["hooks"]["PreToolUse"].as_array().unwrap();
    assert_eq!(pre[1]["hooks"][0]["command"], "python3 scripts/money-guard.py", "foreign entry untouched");
    assert_eq!(pre[1]["hooks"][0]["timeout"], 5);
    assert!(pre[0]["hooks"][0]["command"].as_str().unwrap().starts_with("sos hook claude pre-edit --protocol 1; rc=$?"));
    assert!(pre[2]["hooks"][0]["command"].as_str().unwrap().starts_with("sos hook claude pre-bash --protocol 1"), "{text}");
    assert_eq!(v["hooks"]["PostToolUse"][0]["hooks"][0]["timeout"], 320);
    assert_eq!(v["hooks"]["PostToolUseFailure"][0]["hooks"][0]["command"], "sos hook claude post-bash --protocol 1");
    // Again: nothing to do, nothing duplicated.
    let o = sos(p, &["update", "--wiring"]);
    assert!(out(&o).contains(".claude/settings.json: hook wiring up to date"), "{}", out(&o));
    assert_eq!(std::fs::read_to_string(p.join(".claude/settings.json")).unwrap(), text);
    let c = out(&sos(p, &["check"]));
    assert!(!c.contains("hook.py") && !c.contains("PreToolUse Bash"), "{c}");
    // Broken JSON: reported, left as is, nonzero.
    std::fs::write(p.join(".codex/hooks.json"), "{ not json").unwrap();
    let o = sos(p, &["update", "--wiring"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    assert_eq!(std::fs::read_to_string(p.join(".codex/hooks.json")).unwrap(), "{ not json");
}

#[cfg(windows)]
#[test]
fn update_wiring_on_windows_writes_the_launcher_every_shell_can_run() {
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    let file = p.join(".codex/hooks.json");
    let installed = std::fs::read_to_string(&file).unwrap();
    assert!(installed.contains(" -c "), "{installed}");
    let hook = "\\\"$(git rev-parse --show-toplevel)/harness-lite/adapters/codex/hook.py\\\"";
    let old = format!(r#"{{"hooks":{{"PreToolUse":[{{"matcher":"apply_patch","hooks":[{{"type":"command","command":"python {hook} pre-edit"}}]}}]}}}}"#);
    std::fs::write(&file, old).unwrap();
    let o = sos(p, &["update", "--wiring"]);
    assert!(o.status.success(), "{}", out(&o));
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(!text.contains("rc=$?") && !text.contains("hook.py"), "{text}");
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(v["hooks"]["PreToolUse"][0]["hooks"][0]["command"].as_str().unwrap().starts_with("python -c "), "{text}");
}

#[test]
fn update_keeps_a_dropped_file_the_wiring_still_calls() {
    // Someone jumps straight to the release that drops the adapters (review 05/11/2026, point 9).
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    let gone = "harness-lite/adapters/claude/old-hook.py";
    std::fs::write(p.join(gone), "print('old')\n").unwrap();
    let mut u = upstream(p);
    u["files"][gone] = serde_json::Value::String(sha(b"print('old')\n"));
    std::fs::write(p.join("harness-lite/UPSTREAM.json"), serde_json::to_string_pretty(&u).unwrap()).unwrap();
    let mut s: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(p.join(".claude/settings.json")).unwrap()).unwrap();
    s["hooks"]["Stop"] = serde_json::json!([{ "hooks": [{ "type": "command", "command": format!("python3 my-wrapper.py {gone}") }] }]);
    std::fs::write(p.join(".claude/settings.json"), s.to_string()).unwrap();
    let o = sos(p, &["update"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    assert!(out(&o).contains("KEPT") && p.join(gone).exists(), "{}", out(&o));
    assert!(upstream(p)["files"].get(gone).is_some(), "still recorded, so the next update retries");
    // Nothing calls it any more: removed.
    s["hooks"].as_object_mut().unwrap().remove("Stop");
    std::fs::write(p.join(".claude/settings.json"), s.to_string()).unwrap();
    let o = sos(p, &["update"]);
    assert!(o.status.success() && !p.join(gone).exists(), "{}", out(&o));
}

#[cfg(unix)]
#[test]
fn gate_pre_push_runs_the_local_hook_after_the_gates() {
    use std::os::unix::fs::PermissionsExt;
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    // gitleaks may be missing here; a stand-in that finds nothing.
    let bin = tempfile::tempdir().unwrap();
    std::fs::write(bin.path().join("gitleaks"), "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::set_permissions(bin.path().join("gitleaks"), std::fs::Permissions::from_mode(0o755)).unwrap();
    let path = std::env::join_paths(std::iter::once(bin.path().to_path_buf()).chain(std::env::split_paths(&path_with_sos()))).unwrap();
    let hook = |args: &[&str], stdin: &str| {
        use std::io::Write;
        let mut child = Command::new("harness-lite/hooks/pre-push").args(args).current_dir(p).env("PATH", &path)
            .stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().unwrap();
        child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
        child.wait_with_output().unwrap()
    };
    let o = hook(&["origin", "url"], "");
    assert!(o.status.success(), "no local hook: {}", out(&o));
    let local = p.join("harness-lite/hooks/pre-push.local");
    std::fs::write(&local, "#!/bin/sh\necho \"args=$*\" > got; cat >> got; exit 3\n").unwrap();
    let o = hook(&["origin", "url"], "refs/heads/x 1 refs/heads/x 2\n");
    assert_eq!(o.status.code(), Some(1), "present but not executable fails: {}", out(&o));
    assert!(out(&o).contains("exists but is not executable"), "{}", out(&o));
    std::fs::set_permissions(&local, std::fs::Permissions::from_mode(0o755)).unwrap();
    let o = hook(&["origin", "url"], "refs/heads/x 1 refs/heads/x 2\n");
    assert_eq!(o.status.code(), Some(3), "its exit code decides: {}", out(&o));
    assert_eq!(std::fs::read_to_string(p.join("got")).unwrap(), "args=origin url\nrefs/heads/x 1 refs/heads/x 2\n");
}

#[cfg(windows)]
#[test]
fn codex_hooks_on_windows_run_from_cmd_and_powershell() {
    // Which shell Codex starts hooks with on Windows is not documented; the command must work in each.
    // Under a non-ASCII folder with Python's UTF-8 mode off: git's UTF-8 output must not crash it (review 23/10/2026).
    let d = tempfile::tempdir().unwrap();
    let p = &d.path().join("Ápp");
    std::fs::create_dir(p).unwrap();
    git(p, &["init", "-q"]);
    let o = sos(p, &["install"]);
    assert!(o.status.success(), "{}", out(&o));
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(p.join(".codex/hooks.json")).unwrap()).unwrap();
    let cmd = v["hooks"]["PreToolUse"][0]["hooks"][0]["command"].as_str().unwrap().to_string();
    assert!(cmd.starts_with("python -c ") && !cmd.contains('$'), "{cmd}");
    std::fs::create_dir_all(p.join("sub")).unwrap();
    let patch = |file: &str| format!(r#"{{"tool_name":"apply_patch","tool_input":{{"command":"*** Begin Patch\n*** Add File: {file}\n+x\n*** End Patch"}}}}"#);
    let run = |shell: &[&str], dir: &Path, input: String| {
        use std::io::Write;
        use std::os::windows::process::CommandExt;
        let mut c = Command::new(shell[0]);
        c.args(&shell[1..]);
        // cmd reads its command line as is; PowerShell parses argv, which undoes Rust's quoting.
        if shell[0] == "cmd" { c.raw_arg(&cmd) } else { c.arg(&cmd) };
        c.current_dir(dir).env_remove("PYTHONUTF8").env_remove("PYTHONIOENCODING").env("PATH", path_with_sos())
            .stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
        let mut child = c.spawn().unwrap();
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        child.wait_with_output().unwrap()
    };
    let outside = tempfile::tempdir().unwrap();
    // A block is deny JSON with exit 0: `powershell -Command` turns exit 2 into 1, which does not block.
    // The whole stdout must be exactly one JSON object (spec §3), not deny text among other output.
    let denied = |o: &Output| o.status.code() == Some(0)
        && serde_json::from_slice::<serde_json::Value>(&o.stdout).is_ok_and(|v| {
            v["hookSpecificOutput"]["permissionDecision"] == "deny" && v.as_object().is_some_and(|m| m.len() == 1)
        });
    for shell in [&["cmd", "/d", "/c"][..], &["powershell", "-NoProfile", "-Command"][..]] {
        let o = run(shell, &p.join("sub"), patch(".env"));
        assert!(denied(&o) && out(&o).contains("must not edit real .env"), "{shell:?} .env: {:?} {}", o.status, out(&o));
        let o = run(shell, &p.join("sub"), patch("notes.md"));
        assert!(o.status.code() == Some(0) && !out(&o).contains("deny"), "{shell:?} notes.md: {}", out(&o));
        let o = run(shell, outside.path(), patch(".env"));
        assert!(denied(&o) && out(&o).contains("must not edit real .env"), "{shell:?} outside a repo: {:?} {}", o.status, out(&o));
    }
    // No sos on PATH, or one too old for --protocol: still deny JSON.
    let fakes = tempfile::tempdir().unwrap();
    std::fs::write(fakes.path().join("sos.cmd"), "@echo error: unrecognized subcommand 'hook' 1>&2\r\n@exit /b 2\r\n").unwrap();
    let sysroot = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    let base = format!("{sysroot}\\System32;{sysroot};{sysroot}\\System32\\WindowsPowerShell\\v1.0");
    let py_dir = Command::new("python").args(["-c", "import sys,os;print(os.path.dirname(sys.executable))"]).output().unwrap();
    let py_dir = String::from_utf8_lossy(&py_dir.stdout).trim().to_string();
    let git_dir = Command::new("where").arg("git").output().unwrap();
    let git_dir = Path::new(String::from_utf8_lossy(&git_dir.stdout).lines().next().unwrap().trim()).parent().unwrap().display().to_string();
    let crash = tempfile::tempdir().unwrap();
    std::fs::write(crash.path().join("sos.cmd"), "@echo garbage on stdout\r\n@exit /b 101\r\n").unwrap();
    for (name, path, want) in [("missing", format!("{py_dir};{git_dir};{base}"), "not on PATH"),
                               ("crashing", format!("{};{py_dir};{git_dir};{base}", crash.path().display()), "exited 101"),
                               ("too old", format!("{};{py_dir};{git_dir};{base}", fakes.path().display()), "exited 2")] {
        for shell in [&["cmd", "/d", "/c"][..], &["powershell", "-NoProfile", "-Command"][..]] {
            let o = run_with_path(shell, &p.join("sub"), patch("notes.md"), &cmd, &path);
            assert!(denied(&o) && out(&o).contains(want), "{name} {shell:?}: {:?} {}", o.status, out(&o));
        }
    }
}

#[cfg(windows)]
fn run_with_path(shell: &[&str], dir: &Path, input: String, cmd: &str, path: &str) -> Output {
    use std::io::Write;
    use std::os::windows::process::CommandExt;
    let mut c = Command::new(shell[0]);
    c.args(&shell[1..]);
    if shell[0] == "cmd" { c.raw_arg(cmd) } else { c.arg(cmd) };
    c.current_dir(dir).env("PATH", path)
        .stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
    let mut child = c.spawn().unwrap();
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
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

#[test]
fn check_flags_hook_wiring_without_the_simulator_guard() {
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    assert!(p.join("harness-lite/scripts/sim-guard").exists());
    assert!(!out(&sos(p, &["check"])).contains("pre-bash"), "template wiring has the guard");
    let old = std::fs::read_to_string(p.join(".claude/settings.json")).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&old).unwrap();
    v["hooks"]["PreToolUse"].as_array_mut().unwrap().retain(|m| m["matcher"] != "Bash");
    std::fs::write(p.join(".claude/settings.json"), v.to_string()).unwrap();
    let c = out(&sos(p, &["check"]));
    assert!(c.contains(".claude/settings.json has no PreToolUse Bash hook") && !c.contains(".codex/hooks.json has no PreToolUse"), "{c}");
}

#[test]
fn removal_check_reads_backslash_paths_and_keeps_the_dropped_runtime_together() {
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    let mut u = upstream(p);
    for (f, body) in [("harness-lite/adapters/codex/old.py", "a\n"), ("harness-lite/scripts/old-guard.sh", "b\n")] {
        std::fs::write(p.join(f), body).unwrap();
        u["files"][f] = serde_json::Value::String(sha(body.as_bytes()));
    }
    std::fs::write(p.join("harness-lite/UPSTREAM.json"), serde_json::to_string_pretty(&u).unwrap()).unwrap();
    let mut s: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(p.join(".codex/hooks.json")).unwrap()).unwrap();
    s["hooks"]["Stop"] = serde_json::json!([{ "hooks": [{ "type": "command", "command": "python \"Harness-Lite\\adapters\\codex\\old.py\" stop" }] }]);
    std::fs::write(p.join(".codex/hooks.json"), s.to_string()).unwrap();
    let o = sos(p, &["update"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    assert!(p.join("harness-lite/adapters/codex/old.py").exists() && p.join("harness-lite/scripts/old-guard.sh").exists(), "{}", out(&o));
}

#[cfg(unix)]
#[test]
fn unreadable_wiring_keeps_dropped_files_and_wiring_never_writes_through_symlinks() {
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    let gone = "harness-lite/adapters/claude/old.py";
    std::fs::write(p.join(gone), "x\n").unwrap();
    let mut u = upstream(p);
    u["files"][gone] = serde_json::Value::String(sha(b"x\n"));
    std::fs::write(p.join("harness-lite/UPSTREAM.json"), serde_json::to_string_pretty(&u).unwrap()).unwrap();
    std::fs::write(p.join(".claude/settings.json"), b"{\"hooks\": \"\xff\"}").unwrap();
    let o = sos(p, &["update"]);
    assert!(o.status.code() == Some(1) && p.join(gone).exists(), "invalid UTF-8 counts as unknown: {}", out(&o));
    // A .codex folder linked out of the repo is not written (the temp-file case: wiring.rs unit test).
    std::fs::write(p.join(".claude/settings.json"), OLD_SETTINGS).unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("hooks.json"), OLD_SETTINGS.replace("claude", "codex")).unwrap();
    std::fs::remove_dir_all(p.join(".codex")).unwrap();
    std::os::unix::fs::symlink(outside.path(), p.join(".codex")).unwrap();
    let o = sos(p, &["update", "--wiring"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    assert_eq!(std::fs::read_to_string(outside.path().join("hooks.json")).unwrap(), OLD_SETTINGS.replace("claude", "codex"));
    assert!(!std::fs::read_to_string(p.join(".claude/settings.json")).unwrap().contains("hook.py"), "the in-repo file is still rewritten");
}

// The number says what every session starts from: the first section of the state doc, not all of it.
#[test]
fn check_counts_the_bytes_a_session_loads() {
    let d = repo();
    let p = d.path();
    assert!(sos(p, &["install"]).status.success());
    std::fs::write(p.join("AGENTS.md"), "a\n").unwrap();
    std::fs::write(p.join("CLAUDE.md"), "b\n").unwrap();
    std::fs::write(p.join("harness-lite/CONTRACT.md"), "c\n").unwrap();
    std::fs::write(p.join("harness-lite/roles/orchestrator.md"), "d\n").unwrap();
    std::fs::create_dir_all(p.join("docs")).unwrap();
    std::fs::write(p.join("docs/BACKLOG.md"), format!("# B\n\n## Now\n- [ ] x\n## Later\n{}\n", "y".repeat(5000))).unwrap();
    let c = out(&sos(p, &["check"]));
    assert!(c.contains("loaded at session start: 23 bytes"), "{c}");
    assert!(c.contains("first section of docs/BACKLOG.md"), "{c}");
}
