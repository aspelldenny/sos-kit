//! Fail-closed corners that a cases.jsonl line cannot express (raw bytes, payload shapes,
//! check-config lint), each from the wave 1d review.

use serde_json::json;
use sos_rules::check::{Ran, Runner};
use sos_rules::config::{load, Places};
use std::path::{Path, PathBuf};
use std::time::Duration;

struct NoRun;
impl Runner for NoRun {
    fn run(&self, _: &Path, _: &[String], _: &Path, _: Duration) -> Result<Ran, String> {
        Err("no checks in this test".into())
    }
}

fn set(dir: &Path, toml: &str) -> (sos_rules::config::RuleSet, Places) {
    let f = dir.join("r.toml");
    std::fs::write(&f, toml).unwrap();
    let places = Places { home: Some(dir.to_path_buf()), cwd: dir.to_path_buf() };
    (load(&[f], &places).expect("rules load"), places)
}

const ROLE_RULE: &str = r#"
version = 1
[[rule]]
id = "no-read-for-tho"
tools = ["Read"]
when.role = { sources = [{ file = "role", trim = true }], one_of = ["tho"] }
paths.glob = ["**/SOUL.md"]
decision = "block"
message = "no"
"#;

#[test]
fn a_damaged_role_marker_blocks() {
    let d = tempfile::tempdir().unwrap();
    let (s, places) = set(d.path(), ROLE_RULE);
    std::fs::write(d.path().join("role"), b"tho\xff").unwrap();
    let p = json!({"hook_event_name": "PreToolUse", "tool_name": "Read", "tool_input": {"file_path": "SOUL.md"}});
    assert!(!sos_rules::decide(&s, &p, &places, &NoRun).blocks.is_empty(), "non-UTF-8 marker must block, not read as a near-miss");
}

#[test]
fn a_non_string_event_blocks() {
    let d = tempfile::tempdir().unwrap();
    let (s, places) = set(d.path(), ROLE_RULE);
    for ev in [json!(1), json!(["PreToolUse"]), json!({"x": 1}), json!(null)] {
        let p = json!({"hook_event_name": ev, "tool_name": "Read", "tool_input": {"file_path": "SOUL.md"}});
        assert!(!sos_rules::decide(&s, &p, &places, &NoRun).blocks.is_empty(), "{ev}");
    }
    let p = json!({"hook_event_name": "Notification", "tool_name": "Read", "tool_input": {"file_path": "SOUL.md"}});
    assert!(sos_rules::decide(&s, &p, &places, &NoRun).blocks.is_empty(), "an unknown string event is not ours");
}

#[test]
fn lint_finds_a_shell_c_behind_wrappers() {
    let d = tempfile::tempdir().unwrap();
    for (program, args) in [("env", r#"["sh", "-c", "cat {path}"]"#), ("timeout", r#"["5", "bash.exe", "-c", "cat {path}"]"#),
                            (r"C:\\Git\\bin\\bash.exe", r#"["-lc", "cat {path}"]"#)] {
        let toml = format!(r#"
version = 1
[[rule]]
id = "x"
tools = ["Read"]
paths.glob = ["**/a.md"]
check = {{ program = "{program}", args = {args}, timeout_secs = 5, fires_on = "success" }}
decision = "block"
message = "no"
"#);
        let f = d.path().join("r.toml");
        std::fs::write(&f, toml).unwrap();
        let places = Places { home: None, cwd: PathBuf::from(d.path()) };
        let e = load(&[f], &places).err().unwrap_or_else(|| panic!("{program} {args} must be refused"));
        assert!(e.contains("-c"), "{e}");
    }
}

#[test]
fn a_null_event_blocks_a_post_only_set() {
    let d = tempfile::tempdir().unwrap();
    let (s, places) = set(d.path(), r#"
version = 1
[[rule]]
id = "post-md"
event = "post"
tools = ["Write"]
paths.glob = ["**/*.md"]
decision = "block"
message = "no"
"#);
    let p = json!({"hook_event_name": null, "tool_name": "Write", "tool_input": {"file_path": "a.md"}});
    assert!(!sos_rules::decide(&s, &p, &places, &NoRun).blocks.is_empty(), "null is not a missing event");
}

#[test]
fn lint_refuses_a_check_line_it_cannot_follow() {
    let d = tempfile::tempdir().unwrap();
    for args in [r#"["-S", "sh -c 'printf {arg.0}'"]"#, r#"["--split-string=sh -c 'printf {arg.0}'"]"#, r#"["-u", "X", "env", "-S", "sh -c x"]"#,
                 r#"["-vS", "sh -c 'printf {arg.0}'"]"#, r#"["--spl", "sh -c 'printf {arg.0}'"]"#] {
        let toml = format!(r#"
version = 1
[[rule]]
id = "x"
command.program = ["echo"]
check = {{ program = "env", args = {args}, timeout_secs = 5, fires_on = "failure" }}
decision = "block"
message = "no"
"#);
        let f = d.path().join("r.toml");
        std::fs::write(&f, toml).unwrap();
        let places = Places { home: None, cwd: PathBuf::from(d.path()) };
        assert!(load(&[f], &places).is_err(), "env {args} must be refused");
    }
}

struct MustNotRun;
impl Runner for MustNotRun {
    fn run(&self, p: &Path, a: &[String], _: &Path, _: Duration) -> Result<Ran, String> {
        panic!("check ran: {} {a:?}", p.display());
    }
}

fn check_rule(program: &str, args: &str, on_error: &str) -> String {
    format!(r#"
version = 1
[[rule]]
id = "x"
command.program = ["echo"]
check = {{ program = "{program}", args = {args}, timeout_secs = 5, fires_on = "failure", on_error = "{on_error}" }}
decision = "block"
message = "no"
"#)
}

#[test]
fn lint_refuses_a_placeholder_given_to_a_shell() {
    // Review 1d round 4: `sh {arg.0} {arg.1}` with `echo -c 'code'` would run `sh -c code`.
    let d = tempfile::tempdir().unwrap();
    for (program, args) in [("sh", r#"["{arg.0}", "{arg.1}"]"#), ("env", r#"["bash", "x.sh", "{arg.0}"]"#)] {
        let f = d.path().join("r.toml");
        std::fs::write(&f, check_rule(program, args, "block")).unwrap();
        let places = Places { home: None, cwd: PathBuf::from(d.path()) };
        assert!(load(&[f], &places).is_err(), "{program} {args} must be refused");
    }
}

#[test]
fn a_value_that_becomes_a_program_never_runs() {
    // The lint cannot see what `{arg.0}` will be: the argv is re-read after substitution and a
    // changed program list blocks without running, even with on_error = allow.
    let d = tempfile::tempdir().unwrap();
    let (s, places) = set(d.path(), &check_rule("env", r#"["{arg.0}", "{arg.1}", "{arg.2}"]"#, "allow"));
    let p = json!({"hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": "echo sh -c 'printf CHECK_EXECUTED'"}});
    assert!(!sos_rules::decide(&s, &p, &places, &MustNotRun).blocks.is_empty());
}
