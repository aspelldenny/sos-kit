//! `sos role`: the real binary with stub `codex` / `claude` CLIs on PATH.

use std::path::Path;
use std::process::{Command, Output};

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git").current_dir(dir).args(args)
        .env("GIT_AUTHOR_NAME", "t").env("GIT_AUTHOR_EMAIL", "t@t").env("GIT_COMMITTER_NAME", "t").env("GIT_COMMITTER_EMAIL", "t@t")
        .status().unwrap().success();
    assert!(ok, "git {args:?}");
}

/// A repo with a role table, a state doc, a brief, and stub CLIs in `bin/`.
fn setup() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("repo");
    std::fs::create_dir_all(p.join("docs")).unwrap();
    git(&p, &["init", "-q"]);
    std::fs::write(p.join(".sos.toml"), "[state]\nfile = \"docs/PROGRESS.md\"\n\
        [roles.reviewer.claude]\nbackend = \"codex\"\nmodel = \"gpt-x\"\neffort = \"high\"\nsandbox = \"read-only\"\n\
        [roles.reviewer.codex]\nbackend = \"claude\"\nmodel = \"opus\"\neffort = \"high\"\n\
        [roles.worker]\nbackend = \"codex\"\nmodel = \"gpt-y\"\neffort = \"medium\"\n").unwrap();
    std::fs::write(p.join("docs/PROGRESS.md"), "# Progress\n\n## Current\n\nP5 refund fence\n\n## Done\n\nP1\n").unwrap();
    std::fs::write(d.path().join("brief.md"), "Check the refund journey.\n").unwrap();
    git(&p, &["add", "-A"]);
    git(&p, &["commit", "-qm", "init"]);
    let bin = d.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    // codex: save stdin, write the -o file, optionally hang.
    let codex = "#!/usr/bin/env bash\nout=\nwhile [ $# -gt 0 ]; do [ \"$1\" = -o ] && out=$2; shift; done\n\
                 cat > \"$STUB_DIR/prompt.txt\"\n[ -n \"$STUB_SLEEP\" ] && sleep \"$STUB_SLEEP\"\necho 'REVIEW: 1 blocking' > \"$out\"\n";
    let claude = "#!/usr/bin/env bash\ncat > \"$STUB_DIR/prompt.txt\"\necho \"$@\" > \"$STUB_DIR/args.txt\"\n\
                  echo '{\"result\":\"CLAUDE REVIEW ok\",\"is_error\":false}'\n";
    for (name, body) in [("codex", codex), ("claude", claude)] {
        std::fs::write(bin.join(name), body).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(bin.join(name), std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
    d
}

fn sos(d: &Path, args: &[&str], extra: &[(&str, &str)]) -> Output {
    let path = format!("{}:{}", d.join("bin").display(), std::env::var("PATH").unwrap());
    let mut c = Command::new(env!("CARGO_BIN_EXE_sos"));
    c.current_dir(d.join("repo")).args(args).env("PATH", path).env("STUB_DIR", d);
    for (k, v) in extra {
        c.env(k, v);
    }
    c.output().unwrap()
}

fn out(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

#[test]
fn reviewer_for_claude_work_runs_codex_with_stdin_closed() {
    let d = setup();
    let brief = d.path().join("brief.md");
    let o = sos(d.path(), &["role", "reviewer", "--from", "claude", "--brief", brief.to_str().unwrap()], &[]);
    assert!(o.status.success(), "{}", out(&o));
    assert!(String::from_utf8_lossy(&o.stdout).contains("REVIEW: 1 blocking"));
    let prompt = std::fs::read_to_string(d.path().join("prompt.txt")).unwrap();
    assert!(prompt.contains("Check the refund journey") && prompt.contains("P5 refund fence"), "{prompt}");
    assert!(!prompt.contains("P1"), "only the first state section: {prompt}");
    let log = std::fs::read_to_string(d.path().join("repo/.advise-state/calls.jsonl")).unwrap();
    assert!(log.contains("\"role\":\"reviewer\"") && log.contains("gpt-x"), "{log}");
}

#[test]
fn reviewer_for_codex_work_runs_claude_read_only() {
    let d = setup();
    let brief = d.path().join("brief.md");
    let o = sos(d.path(), &["role", "reviewer", "--from", "codex", "--brief", brief.to_str().unwrap()], &[]);
    assert!(o.status.success(), "{}", out(&o));
    assert!(String::from_utf8_lossy(&o.stdout).contains("CLAUDE REVIEW ok"));
    let args = std::fs::read_to_string(d.path().join("args.txt")).unwrap();
    assert!(args.contains("--model opus") && args.contains("--effort high") && args.contains("--tools Read,Glob,Grep"), "{args}");
}

#[test]
fn hang_is_stopped_at_the_timeout() {
    let d = setup();
    let brief = d.path().join("brief.md");
    let o = sos(d.path(), &["role", "reviewer", "--from", "claude", "--brief", brief.to_str().unwrap(), "--timeout", "1"],
                &[("STUB_SLEEP", "20")]);
    assert_eq!(o.status.code(), Some(124), "{}", out(&o));
    assert!(out(&o).contains("stopped after 1s"));
}

#[test]
fn missing_role_and_main_checkout_worker_are_refused() {
    let d = setup();
    let brief = d.path().join("brief.md");
    let o = sos(d.path(), &["role", "architect", "--from", "claude", "--brief", brief.to_str().unwrap()], &[]);
    assert_eq!(o.status.code(), Some(2), "{}", out(&o));
    assert!(out(&o).contains("add [roles.architect]"), "{}", out(&o));
    let o = sos(d.path(), &["role", "worker", "--from", "claude", "--brief", brief.to_str().unwrap()], &[]);
    assert_eq!(o.status.code(), Some(2), "{}", out(&o));
    assert!(out(&o).contains("own git worktree"), "{}", out(&o));
    let o = sos(d.path(), &["role", "worker", "--from", "claude", "--brief", brief.to_str().unwrap(), "--dry-run"], &[]);
    assert!(o.status.success() && out(&o).contains("gpt-y") && out(&o).contains("model_reasoning_effort=\\\"medium\\\""), "{}", out(&o));
}

#[test]
fn long_vietnamese_state_and_claude_error_are_handled() {
    let d = setup();
    let p = d.path().join("repo");
    std::fs::write(p.join("docs/PROGRESS.md"), format!("# P\n\n## Hiện tại\n\n{}\n", "Kiểm tra hoàn tiền ế. ".repeat(300))).unwrap();
    let brief = d.path().join("brief.md");
    let o = sos(d.path(), &["role", "reviewer", "--from", "claude", "--brief", brief.to_str().unwrap()], &[]);
    assert!(o.status.success(), "{}", out(&o));
    std::fs::write(d.path().join("bin/claude"), "#!/usr/bin/env bash\ncat >/dev/null\necho '{\"result\":\"Reached max turns ế\",\"is_error\":true}'\n").unwrap();
    let o = sos(d.path(), &["role", "reviewer", "--from", "codex", "--brief", brief.to_str().unwrap()], &[]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    assert!(out(&o).contains("Reached max turns"), "{}", out(&o));
}
