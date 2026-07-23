//! P077d2 — install correctness oracle. NOT a Bash-parity fixture (`sos
//! install` is a NEW command, no Bash counterpart — see
//! `docs/plans/P077d-decomposition.md:9`). Oracle = 5 hard-fail
//! correctness fixtures against a deterministic `MockAdapter`, proving the
//! ENGINE (`sos-install::engine`) is correct INDEPENDENT of which adapter
//! drives it (the whole point of the d1 `Adapter` trait abstraction).

use sos_core::adapter::{Adapter, Artifact, Asset, Capabilities, Findings, ManagedOperation, Plan, RemovalPlan};
use sos_core::manifest::ManagedManifest;
use sos_install::engine::{self, Decision};
use std::fs;
use std::path::{Path, PathBuf};

// --- Collision-safe TempFixture (mirrors `crates/sos-cli/tests/parity.rs`
// pattern, anchor #8 — AtomicU64 counter, replicated per-crate since test
// binaries don't share a crate-level test-util today). ---
static TEMP_FIXTURE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

struct TempFixture(PathBuf);

impl TempFixture {
    fn new(name: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = TEMP_FIXTURE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "sos-install-{name}-{}-{nanos}-{counter}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        TempFixture(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Deterministic test adapter — NOT `sos-adapter-claude`. Proves the
/// engine is drivable purely through the `Adapter` trait, isolated from
/// any real runtime rendering (Worker CHALLENGE Turn 1 seam-risk finding).
struct MockAdapter {
    ops: Vec<ManagedOperation>,
}

impl Adapter for MockAdapter {
    fn detect(&self) -> Capabilities {
        Capabilities::default()
    }

    fn plan(&self, _capabilities: &Capabilities) -> Plan {
        Plan {
            operations: self.ops.clone(),
        }
    }

    fn render(&self, asset: &Asset, _capabilities: &Capabilities) -> Artifact {
        Artifact {
            target_path: asset.identity.clone(),
            content: asset.content.clone(),
        }
    }

    fn verify(&self) -> Findings {
        Findings::default()
    }

    fn uninstall(&self, _manifest: &ManagedManifest) -> RemovalPlan {
        RemovalPlan::default()
    }
}

fn op(target_path: &str, content: &str) -> ManagedOperation {
    ManagedOperation {
        description: format!("write {target_path}"),
        target_path: target_path.to_string(),
        content: content.to_string(),
    }
}

const OWNER: &str = "mock-adapter-test";
const SRC_VERSION: &str = "0.0.0-test";

#[test]
fn additive_greenfield_creates_all_targets_and_manifest() {
    let fixture = TempFixture::new("additive");
    let root = fixture.path();

    let adapter = MockAdapter {
        ops: vec![
            op("a.txt", "alpha"),
            op("nested/b.txt", "beta"),
            op("nested/deep/c.txt", "gamma"),
        ],
    };
    let plan = adapter.plan(&adapter.detect());

    let report = engine::apply(root, &plan, OWNER, SRC_VERSION).expect("apply must succeed");
    assert_eq!(report.created.len(), 3, "all 3 targets should be CREATE");
    assert!(report.updated.is_empty());
    assert!(report.conflicts.is_empty());

    for (path, content) in [("a.txt", "alpha"), ("nested/b.txt", "beta"), ("nested/deep/c.txt", "gamma")] {
        let bytes = fs::read(root.join(path)).unwrap_or_else(|_| panic!("{path} must exist"));
        assert_eq!(bytes, content.as_bytes(), "{path} content must match plan exactly");
    }

    let manifest = engine::load_manifest(root).expect("manifest must load");
    assert_eq!(manifest.managed.len(), 3, ".sos-manifest.toml must record 1 entry per file");
    for entry in &manifest.managed {
        assert!(entry.rollback_ref.is_none(), "fresh CREATE must have rollback_ref=None");
        let expected_hash = engine::sha256_hex(
            match entry.target_path.as_str() {
                "a.txt" => "alpha",
                "nested/b.txt" => "beta",
                "nested/deep/c.txt" => "gamma",
                other => panic!("unexpected manifest entry {other}"),
            }
            .as_bytes(),
        );
        assert_eq!(entry.content_hash, expected_hash, "content_hash must match target path's content");
    }
}

#[test]
fn non_clobber_customized_target_stages_incoming_and_preserves_original() {
    let fixture = TempFixture::new("non-clobber-conflict");
    let root = fixture.path();

    // Seed a pre-existing, user-customized file with NO manifest record —
    // the "no record at all" branch of Task 1 rule 3.
    fs::write(root.join("customized.txt"), "USER WROTE THIS").unwrap();

    let adapter = MockAdapter {
        ops: vec![op("customized.txt", "kit incoming content")],
    };
    let plan = adapter.plan(&adapter.detect());

    let report = engine::apply(root, &plan, OWNER, SRC_VERSION).expect("apply must succeed (conflict is not a fatal error)");
    assert_eq!(report.conflicts, vec!["customized.txt".to_string()]);
    assert!(report.created.is_empty());
    assert!(report.updated.is_empty());

    // Original untouched.
    let original = fs::read(root.join("customized.txt")).unwrap();
    assert_eq!(original, b"USER WROTE THIS", "non-clobber: original must be byte-unchanged");

    // Incoming copy byte-matches kit content.
    let incoming = fs::read(root.join(".sos-install-incoming").join("customized.txt")).unwrap();
    assert_eq!(incoming, b"kit incoming content", "incoming staged copy must byte-match kit source");

    // No manifest entry recorded for a conflicted target.
    let manifest = engine::load_manifest(root).expect("manifest must load (may be absent/empty)");
    assert!(
        manifest.managed.iter().all(|m| m.target_path != "customized.txt"),
        "conflicted target must NOT get a manifest entry"
    );
}

#[test]
fn non_clobber_unmodified_target_allows_update() {
    let fixture = TempFixture::new("non-clobber-update");
    let root = fixture.path();

    // First install: create.
    let adapter1 = MockAdapter {
        ops: vec![op("tracked.txt", "v1 content")],
    };
    let plan1 = adapter1.plan(&adapter1.detect());
    engine::apply(root, &plan1, OWNER, SRC_VERSION).expect("first apply must succeed");

    // Second install: kit ships NEW content, target hash still == recorded
    // (untouched by user since last install) -> UPDATE must be allowed.
    let adapter2 = MockAdapter {
        ops: vec![op("tracked.txt", "v2 content — kit updated")],
    };
    let plan2 = adapter2.plan(&adapter2.detect());
    let report = engine::apply(root, &plan2, OWNER, SRC_VERSION).expect("second apply must succeed");

    assert_eq!(report.updated, vec!["tracked.txt".to_string()], "unmodified target must be UPDATE, not CONFLICT");
    assert!(report.conflicts.is_empty());

    let content = fs::read(root.join("tracked.txt")).unwrap();
    assert_eq!(content, b"v2 content \xe2\x80\x94 kit updated", "target must now hold new kit content");

    let manifest = engine::load_manifest(root).unwrap();
    let entry = manifest.managed.iter().find(|m| m.target_path == "tracked.txt").expect("entry must exist");
    assert!(entry.rollback_ref.is_some(), "UPDATE must set rollback_ref (mutation occurred)");
}

#[test]
fn rollback_restores_byte_identical_pre_transaction_state_on_mid_op_failure() {
    let fixture = TempFixture::new("rollback");
    let root = fixture.path();

    // Pre-existing UPDATE target (unmodified since a prior install) so op 1
    // succeeds (mutates it), THEN op 2 fails for real (its target's parent
    // path is occupied by a plain FILE, so `create_dir_all` errors) —
    // proving rollback via genuine io::Error, no artificial fail-injection
    // hook in the engine.
    let seed_adapter = MockAdapter {
        ops: vec![op("will-update.txt", "pre-existing tracked content")],
    };
    let seed_plan = seed_adapter.plan(&seed_adapter.detect());
    engine::apply(root, &seed_plan, OWNER, SRC_VERSION).expect("seed apply must succeed");

    let pre_txn_bytes = fs::read(root.join("will-update.txt")).unwrap();
    assert_eq!(pre_txn_bytes, b"pre-existing tracked content");

    // Block the 2nd op's directory path with a plain file.
    fs::write(root.join("blocked-parent"), b"i am a file, not a dir").unwrap();

    let failing_adapter = MockAdapter {
        ops: vec![
            op("will-update.txt", "NEW content that must be rolled back"),
            op("blocked-parent/child.txt", "unreachable"),
        ],
    };
    let failing_plan = failing_adapter.plan(&failing_adapter.detect());

    let manifest_before = engine::load_manifest(root).unwrap();
    let result = engine::apply(root, &failing_plan, OWNER, SRC_VERSION);
    assert!(result.is_err(), "transaction must fail (op 2 hits a real io error)");

    // Op 1 (successful mutate) must be rolled back to byte-identical
    // pre-transaction content.
    let restored = fs::read(root.join("will-update.txt")).unwrap();
    assert_eq!(restored, pre_txn_bytes, "rollback must restore byte-identical pre-transaction state");

    // Manifest must NOT have committed the failed transaction's changes.
    let manifest_after = engine::load_manifest(root).unwrap();
    assert_eq!(
        manifest_before.managed, manifest_after.managed,
        ".sos-manifest.toml must be unchanged (uncommitted) after a failed+rolled-back transaction"
    );

    // The blocked file itself must be untouched (engine never overwrote it).
    let blocked = fs::read(root.join("blocked-parent")).unwrap();
    assert_eq!(blocked, b"i am a file, not a dir");
}

#[test]
fn idempotence_second_identical_run_is_a_no_op() {
    let fixture = TempFixture::new("idempotence");
    let root = fixture.path();

    let adapter = MockAdapter {
        ops: vec![op("stable.txt", "stable content"), op("nested/stable2.txt", "also stable")],
    };
    let plan = adapter.plan(&adapter.detect());

    engine::apply(root, &plan, OWNER, SRC_VERSION).expect("first run must succeed");
    let manifest_path = engine::manifest_path(root);
    let manifest_bytes_run1 = fs::read(&manifest_path).unwrap();

    let report2 = engine::apply(root, &plan, OWNER, SRC_VERSION).expect("second identical run must succeed");
    // Both targets already hold the exact desired bytes -> engine
    // classifies them NoOp (zero mutation), NOT create/update/conflict.
    assert_eq!(
        report2.noop,
        vec!["stable.txt".to_string(), "nested/stable2.txt".to_string()],
        "re-run with identical content must be a true no-op"
    );
    assert!(report2.created.is_empty(), "re-run must not re-CREATE anything");
    assert!(report2.updated.is_empty(), "re-run must not rewrite unchanged bytes");
    assert!(report2.conflicts.is_empty(), "re-run must not conflict against its own prior install");

    let manifest_bytes_run2 = fs::read(&manifest_path).unwrap();
    assert_eq!(
        manifest_bytes_run1, manifest_bytes_run2,
        ".sos-manifest.toml must be byte-unchanged across an idempotent re-run"
    );
}

#[test]
fn dry_run_computes_plan_with_zero_filesystem_mutation() {
    let fixture = TempFixture::new("dry-run");
    let root = fixture.path();

    let adapter = MockAdapter {
        ops: vec![op("would-create.txt", "hello"), op("nested/would-create2.txt", "world")],
    };
    let plan = adapter.plan(&adapter.detect());

    // Snapshot dir tree before.
    let before = dir_entries_sorted(root);

    let decisions = engine::dry_run(root, &plan).expect("dry-run must compute successfully");
    assert_eq!(decisions.len(), 2);
    assert!(decisions.iter().all(|d| d.decision == Decision::Create), "greenfield dry-run must report all CREATE");

    let after = dir_entries_sorted(root);
    assert_eq!(before, after, "dry-run must cause ZERO filesystem mutation");
    assert!(!engine::manifest_path(root).exists(), "dry-run must not write .sos-manifest.toml");
    assert!(!root.join(".sos-install-incoming").exists(), "dry-run must not write .sos-install-incoming");
}

// ---------------------------------------------------------------------
// P078f — Git hook arming (`core.hooksPath` + F09 hijack-guard) install-
// smoke oracle. Runs `engine::apply()` against a REAL temp git repo (not
// just filesystem writes) — asserts `core.hooksPath`, chmod, `.bak`
// rename, non-clobber abort, non-git warn-skip, and idempotence (full
// Nghiệm thu checklist, P078f phiếu).
// ---------------------------------------------------------------------

fn git_in(root: &Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("git command must spawn")
}

fn git_init(root: &Path) {
    let out = git_in(root, &["init", "-q"]);
    assert!(
        out.status.success(),
        "git init must succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn git_config_get_test(root: &Path, key: &str) -> Option<String> {
    let out = git_in(root, &["config", "--local", key]);
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn hook_arming_plan() -> Plan {
    let adapter = MockAdapter {
        ops: vec![
            op("hooks/pre-commit", "#!/usr/bin/env bash\necho pre-commit\n"),
            op("hooks/pre-push", "#!/usr/bin/env bash\necho pre-push\n"),
        ],
    };
    adapter.plan(&adapter.detect())
}

#[test]
fn hook_arming_sets_hookspath_and_chmods_tracked_hooks_in_git_repo() {
    let fixture = TempFixture::new("hook-arming-basic");
    let root = fixture.path();
    git_init(root);

    let plan = hook_arming_plan();
    let report =
        engine::apply(root, &plan, OWNER, SRC_VERSION).expect("apply must succeed in a git repo");
    assert_eq!(report.created.len(), 2);

    assert_eq!(
        git_config_get_test(root, "core.hooksPath").as_deref(),
        Some("hooks"),
        "core.hooksPath must be armed to 'hooks' after a successful install"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for h in ["hooks/pre-commit", "hooks/pre-push"] {
            let meta = fs::metadata(root.join(h)).unwrap_or_else(|_| panic!("{h} must exist"));
            assert!(
                meta.permissions().mode() & 0o111 != 0,
                "{h} must be chmod +x after arming"
            );
        }
    }
}

#[test]
fn hook_arming_renames_stale_dot_git_hooks_to_bak_without_deleting() {
    let fixture = TempFixture::new("hook-arming-bak");
    let root = fixture.path();
    git_init(root);

    let stale_hook_path = root.join(".git/hooks/pre-commit");
    fs::write(&stale_hook_path, "#!/bin/sh\necho stale\n").expect("seed stale .git/hooks/pre-commit");

    let plan = hook_arming_plan();
    engine::apply(root, &plan, OWNER, SRC_VERSION).expect("apply must succeed");

    assert!(
        !stale_hook_path.exists(),
        "stale hook must be moved away, not left in place"
    );
    let backup = root.join(".git/hooks/pre-commit.pre-hookspath.bak");
    let backed_up = fs::read_to_string(&backup).expect("backup file must exist with original content");
    assert_eq!(
        backed_up, "#!/bin/sh\necho stale\n",
        "rename must preserve original content byte-for-byte"
    );
}

#[test]
fn hook_arming_non_clobber_aborts_on_foreign_hookspath_non_interactively() {
    let fixture = TempFixture::new("hook-arming-non-clobber");
    let root = fixture.path();
    git_init(root);
    let out = git_in(root, &["config", "--local", "core.hooksPath", "custom-dir"]);
    assert!(out.status.success());

    let plan = hook_arming_plan();
    let result = engine::apply(root, &plan, OWNER, SRC_VERSION);

    assert!(
        result.is_err(),
        "non-TTY install must ABORT rather than silently clobber a foreign core.hooksPath"
    );
    assert_eq!(
        git_config_get_test(root, "core.hooksPath").as_deref(),
        Some("custom-dir"),
        "F09 guard must leave the foreign hooksPath untouched on abort"
    );
}

#[test]
fn hook_arming_non_git_dir_warns_and_still_succeeds() {
    let fixture = TempFixture::new("hook-arming-non-git");
    let root = fixture.path();
    // Deliberately NOT git-init'd.

    let plan = hook_arming_plan();
    let report = engine::apply(root, &plan, OWNER, SRC_VERSION)
        .expect("install in a non-git dir must still succeed (warn-skip, not fail)");
    assert_eq!(
        report.created.len(),
        2,
        "files must still be rendered even though hook arming is skipped"
    );
    assert!(
        !root.join(".git").exists(),
        "must not have created a .git dir as a side effect"
    );
}

#[test]
fn hook_arming_is_idempotent_on_second_install() {
    let fixture = TempFixture::new("hook-arming-idempotent");
    let root = fixture.path();
    git_init(root);

    let plan = hook_arming_plan();
    engine::apply(root, &plan, OWNER, SRC_VERSION).expect("first install must succeed");
    assert_eq!(
        git_config_get_test(root, "core.hooksPath").as_deref(),
        Some("hooks")
    );

    // Second run: our own prior value ("hooks") must NOT trip the F09
    // guard (Constraint 6).
    let second = engine::apply(root, &plan, OWNER, SRC_VERSION);
    assert!(
        second.is_ok(),
        "re-running install must not abort — our own hooksPath value is idempotent, not foreign"
    );
    assert_eq!(
        git_config_get_test(root, "core.hooksPath").as_deref(),
        Some("hooks")
    );
}

fn dir_entries_sorted(root: &Path) -> Vec<String> {
    if !root.exists() {
        return Vec::new();
    }
    let mut out = Vec::new();
    fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
        for entry in walkdir_min(dir) {
            let rel = entry.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
            out.push(rel);
        }
    }
    fn walkdir_min(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        if let Ok(rd) = fs::read_dir(dir) {
            for entry in rd.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    out.extend(walkdir_min(&path));
                } else {
                    out.push(path);
                }
            }
        }
        out
    }
    walk(root, root, &mut out);
    out.sort();
    out
}
