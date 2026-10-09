//! Gates against a real throwaway git repo: they must read the STAGED blob, not the working tree.

use sos_gates::config::{self, DocsConfig, FeaturesConfig, TextConfig};
use sos_gates::{docs, features, local_secrets, text};
use std::path::Path;
use std::process::Command;

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

const F1: &str = r#"{"features":[{"id":"F01","title":"A","passes":true,"verify":["x"]},{"id":"F02","title":"B","passes":false,"verify":["y"]}]}"#;

#[test]
fn features_reads_staged_blob_and_compares_with_head() {
    let d = repo();
    let p = d.path();
    std::fs::create_dir_all(p.join("docs")).unwrap();
    std::fs::write(p.join("docs/FEATURES.json"), F1).unwrap();
    git(p, &["add", "."]);
    git(p, &["commit", "-qm", "init"]);
    let cfg = FeaturesConfig::default();

    // Not staged -> skipped, even if the working tree is broken.
    std::fs::write(p.join("docs/FEATURES.json"), "{broken").unwrap();
    let r = features::gate(p, &cfg).unwrap();
    assert!(r.skipped.is_some() && r.ok());

    // Staged deletion of F02 -> blocked.
    std::fs::write(p.join("docs/FEATURES.json"), r#"{"features":[{"id":"F01","title":"A","passes":true,"verify":["x"]}]}"#).unwrap();
    git(p, &["add", "docs/FEATURES.json"]);
    let r = features::gate(p, &cfg).unwrap();
    assert!(!r.ok() && r.errors.iter().any(|e| e.contains("F02")), "{:?}", r.errors);

    // Working tree fixed but not re-staged -> still blocked (gate reads the index).
    std::fs::write(p.join("docs/FEATURES.json"), F1).unwrap();
    assert!(!features::gate(p, &cfg).unwrap().ok());
    git(p, &["add", "docs/FEATURES.json"]);
    assert!(features::gate(p, &cfg).unwrap().ok());
}

#[test]
fn features_first_commit_has_no_head() {
    let d = repo();
    let p = d.path();
    std::fs::create_dir_all(p.join("docs")).unwrap();
    std::fs::write(p.join("docs/FEATURES.json"), F1).unwrap();
    git(p, &["add", "."]);
    assert!(features::gate(p, &FeaturesConfig::default()).unwrap().ok());
}

#[test]
fn text_staged_uses_globs_and_index_content() {
    let d = repo();
    let p = d.path();
    std::fs::create_dir_all(p.join("App/Views")).unwrap();
    std::fs::write(p.join("App/Views/Home.swift"), "Text(\"Act now!\")").unwrap();
    std::fs::write(p.join("notes.md"), "act now").unwrap();
    git(p, &["add", "."]);
    // Working tree cleaned after staging: the staged text is what gets committed.
    std::fs::write(p.join("App/Views/Home.swift"), "Text(\"Hello\")").unwrap();
    let cfg = TextConfig { files: vec!["App/**/*.swift".into()], banned: vec!["act now".into()], ..TextConfig::default() };
    let r = text::gate(p, &cfg, &[], true).unwrap();
    assert_eq!(r.errors.len(), 1, "{:?}", r.errors);
    assert!(r.errors[0].starts_with("App/Views/Home.swift:1"));

    let unconfigured = text::gate(p, &TextConfig::default(), &[], true).unwrap();
    assert!(unconfigured.skipped.is_some());
}

#[test]
fn features_staged_deletion_is_blocked() {
    let d = repo();
    let p = d.path();
    std::fs::create_dir_all(p.join("docs")).unwrap();
    std::fs::write(p.join("docs/FEATURES.json"), F1).unwrap();
    git(p, &["add", "."]);
    git(p, &["commit", "-qm", "init"]);
    git(p, &["rm", "-q", "docs/FEATURES.json"]);
    let r = features::gate(p, &FeaturesConfig::default()).unwrap();
    assert!(!r.ok() && r.errors[0].contains("deleted"), "{:?}", r);
}

#[test]
fn docs_cap_reads_the_staged_version() {
    let d = repo();
    let p = d.path();
    std::fs::write(p.join("STATE.md"), "x\n".repeat(20)).unwrap();
    git(p, &["add", "."]);
    std::fs::write(p.join("STATE.md"), "short\n").unwrap(); // unstaged shrink must not hide it
    let cfg = DocsConfig { files: vec!["STATE.md".into()], soft: 5, hard: 10 };
    assert!(!docs::gate(p, &cfg).ok());
}

#[test]
fn config_comes_from_the_index() {
    let d = repo();
    let p = d.path();
    std::fs::write(p.join(".sos.toml"), "[text]\nfiles = [\"*.txt\"]\nbanned = [\"act now\"]\n").unwrap();
    std::fs::write(p.join("a.txt"), "act now").unwrap();
    git(p, &["add", "."]);
    std::fs::write(p.join(".sos.toml"), "[text]\nfiles = []\n").unwrap(); // unstaged edit
    let cfg = config::load(p).unwrap();
    assert!(!text::gate(p, &cfg.text, &[], true).unwrap().ok());
}

#[test]
fn nested_tracked_env_is_caught() {
    let d = repo();
    let p = d.path();
    std::fs::create_dir_all(p.join("services")).unwrap();
    std::fs::write(p.join("services/.env"), "A=1").unwrap();
    std::fs::write(p.join(".env.example"), "A=").unwrap();
    git(p, &["add", "."]);
    std::fs::remove_file(p.join("services/.env")).unwrap(); // deleted copy, still in the index
    let r = local_secrets::gate(p).unwrap();
    assert!(r.errors.iter().any(|e| e.contains("services/.env is tracked")), "{:?}", r.errors);
    assert!(!r.errors.iter().any(|e| e.contains(".env.example")), "{:?}", r.errors);
}

#[test]
fn leak_patterns_that_match_empty_are_rejected() {
    // Patterns that can match "" hide real matches, so they are rejected at load time.
    for p in ["^|OpenAI", "x*", "(OpenAI)?"] {
        let r = text::Rules::from_config(&TextConfig { leaks: vec![p.into()], base: false, ..TextConfig::default() });
        assert!(r.is_err(), "{p} should be rejected");
    }
}

#[test]
fn staged_removal_of_config_and_doc_is_respected() {
    let d = repo();
    let p = d.path();
    std::fs::write(p.join(".sos.toml"), "[text]\nfiles = [\"*.txt\"]\nbanned = [\"act now\"]\n").unwrap();
    std::fs::write(p.join("STATE.md"), "x\n".repeat(20)).unwrap();
    git(p, &["add", "."]);
    git(p, &["commit", "-qm", "init"]);
    git(p, &["rm", "-q", "--cached", ".sos.toml", "STATE.md"]); // working copies stay
    let cfg = config::load(p).unwrap();
    assert!(cfg.text.banned.is_empty(), "removed config must not apply");
    let docs_cfg = DocsConfig { files: vec!["STATE.md".into()], soft: 5, hard: 10 };
    assert!(docs::gate(p, &docs_cfg).ok());
}

#[cfg(unix)]
#[test]
fn unreadable_doc_dir_is_an_error() {
    use std::os::unix::fs::PermissionsExt;
    if std::env::var("USER").as_deref() == Ok("root") {
        return; // root ignores permissions
    }
    let d = repo();
    let p = d.path();
    std::fs::create_dir_all(p.join("private")).unwrap();
    std::fs::write(p.join("private/STATE.md"), "x").unwrap();
    std::fs::set_permissions(p.join("private"), std::fs::Permissions::from_mode(0o000)).unwrap();
    let r = docs::gate(p, &DocsConfig { files: vec!["private/STATE.md".into()], soft: 5, hard: 10 });
    std::fs::set_permissions(p.join("private"), std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(!r.ok(), "{:?}", r);
}

#[test]
fn repo_gates() {
    use sos_gates::config::GitConfig;
    use sos_gates::repo;
    let d = repo();
    let p = d.path();
    git(p, &["checkout", "-q", "-b", "main"]);
    std::fs::write(p.join("README.md"), "x").unwrap();
    git(p, &["add", "."]);
    git(p, &["commit", "-qm", "init"]);

    // .env staged anywhere -> blocked; .env.example fine
    std::fs::create_dir_all(p.join("svc")).unwrap();
    std::fs::write(p.join("svc/.ENV.local"), "A=1").unwrap();
    std::fs::write(p.join(".env.example"), "A=").unwrap();
    std::fs::write(p.join(".environment.ts"), "export {}").unwrap(); // not an env file
    git(p, &["add", "-f", "."]);
    let r = repo::env_commit(p).unwrap();
    assert_eq!(r.errors.len(), 1, "{:?}", r.errors);
    git(p, &["rm", "-q", "--cached", "svc/.ENV.local"]);
    assert!(repo::env_commit(p).unwrap().ok());

    // case collision, including a directory segment
    std::fs::create_dir_all(p.join("Docs")).unwrap();
    std::fs::write(p.join("Docs/a.md"), "x").unwrap();
    std::fs::create_dir_all(p.join("docs")).unwrap();
    git(p, &["add", "Docs/a.md"]);
    git(p, &["update-index", "--add", "--cacheinfo", &format!("100644,{},docs/b.md", blob_id(p))]);
    let r = repo::case_collision(p).unwrap();
    assert!(r.errors.iter().any(|e| e.contains("Docs") && e.contains("docs")), "{:?}", r.errors);
    git(p, &["rm", "-q", "--cached", "docs/b.md"]);
    assert!(repo::case_collision(p).unwrap().ok());

    // branch rule: off by default; on -> code on main blocked, markdown allowed
    let on = GitConfig { protect_default_branch: true, default_branch: String::new() };
    std::fs::write(p.join("main.rs"), "fn main(){}").unwrap();
    git(p, &["add", "main.rs"]);
    assert!(repo::branch(p, &GitConfig::default()).unwrap().skipped.is_some());
    let r = repo::branch(p, &on).unwrap();
    assert!(r.errors.iter().any(|e| e.contains("main.rs")), "{:?}", r);
    git(p, &["checkout", "-q", "-b", "feat/x"]);
    assert!(repo::branch(p, &on).unwrap().ok());
}

fn blob_id(p: &Path) -> String {
    let o = Command::new("git").current_dir(p).args(["hash-object", "-w", "--stdin"]).output().unwrap();
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

#[test]
fn branch_rule_applies_to_the_first_commit() {
    use sos_gates::config::GitConfig;
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    git(p, &["init", "-q", "-b", "main"]);
    std::fs::write(p.join("main.rs"), "x").unwrap();
    git(p, &["add", "."]);
    let on = GitConfig { protect_default_branch: true, default_branch: String::new() };
    let r = sos_gates::repo::branch(p, &on).unwrap();
    assert!(!r.ok(), "{:?}", r);
}
