//! P077d2 — install engine (transaction plan / dry-run / non-clobber /
//! rollback / apply), driven THUẦN qua the `sos-core` `Adapter` trait's
//! `Plan` output. This module owns ZERO host-specific knowledge (`.claude`,
//! `CLAUDE_*`, Codex, ...) — it only executes `ManagedOperation`s it's
//! given (dep-direction: `sos-install` depends on `sos-core` only, same as
//! `sos-adapter-claude`; the two never depend on each other).
//!
//! Transaction 7-step (`docs/PORTABILITY_ARCHITECTURE.md:109-117`), this
//! engine implements step 1-4 + 6-7. Step 5 (tool-resolve) is a stub seam
//! for P077d3 (OA-07) — see `resolve_tools()` below.

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use sos_core::adapter::Plan;
use sos_core::manifest::ManagedManifest;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::Command;

/// On-disk manifest artifact name, written at project root (P077d2 CHỐT —
/// see `docs/PORTABILITY_ARCHITECTURE.md` "P077d2 status").
pub const MANIFEST_FILE_NAME: &str = ".sos-manifest.toml";

/// Non-clobber staging directory — mirrors `.sos-adopt-incoming` precedent
/// (`crates/sos-cli/src/commands/adopt.rs`), but install has its own name
/// since it's a distinct command/oracle.
pub const INCOMING_DIR_NAME: &str = ".sos-install-incoming";

/// sha256 hex digest of arbitrary bytes — same format used by `contract.rs`
/// (`sos-cli`), kept plain-hex (no `sha256:` prefix) to match
/// `ManagedManifest::content_hash`'s doc example (`manifest.rs` tests).
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// What the engine decided to do with one planned target, after resolving
/// it against the current filesystem + prior manifest record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Target absent — additive create.
    Create,
    /// Target present and its on-disk content ALREADY equals the desired
    /// content (byte-for-byte) — a true no-op. Distinct from `Update`:
    /// this is what makes re-running install idempotent (no fs write, no
    /// manifest mutation) instead of harmlessly rewriting identical bytes
    /// on every run.
    NoOp,
    /// Target present, on-disk content differs from desired, but its hash
    /// matches the manifest's recorded content_hash (unmodified by the
    /// user since last install) — overwrite permitted.
    Update,
    /// Target present, on-disk content differs from desired AND its hash
    /// differs from recorded (user-customized) OR no manifest record
    /// exists for it — NEVER overwrite; stage incoming copy instead.
    Conflict,
}

/// A resolved target decision — pure data, no mutation has happened yet.
/// `--dry-run` prints exactly this; `apply()` executes it.
#[derive(Debug, Clone)]
pub struct TargetDecision {
    pub target_path: String,
    pub decision: Decision,
    pub content: String,
    pub content_hash: String,
}

/// The on-disk `.sos-manifest.toml` shape — `[[managed]]` array-of-tables,
/// each entry a `ManagedManifest` (d1 schema, untouched).
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct OnDiskManifest {
    #[serde(default)]
    pub managed: Vec<ManagedManifest>,
}

pub fn manifest_path(project_root: &Path) -> PathBuf {
    project_root.join(MANIFEST_FILE_NAME)
}

/// Load `.sos-manifest.toml` if present; absent = fresh project (empty).
pub fn load_manifest(project_root: &Path) -> Result<OnDiskManifest> {
    let path = manifest_path(project_root);
    if !path.exists() {
        return Ok(OnDiskManifest::default());
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    let parsed: OnDiskManifest =
        toml::from_str(&content).with_context(|| format!("parse {}", path.display()))?;
    Ok(parsed)
}

fn save_manifest(project_root: &Path, manifest: &OnDiskManifest) -> Result<()> {
    let s = toml::to_string_pretty(manifest).context("serialize .sos-manifest.toml")?;
    fs::write(manifest_path(project_root), s)
        .with_context(|| format!("write {}", manifest_path(project_root).display()))
}

/// Resolve every `ManagedOperation` in `plan` against the current
/// filesystem + `manifest` — NO mutation. Used by both `--dry-run` (print
/// and stop) and `apply()` (resolve, then execute).
pub fn decide_targets(
    project_root: &Path,
    plan: &Plan,
    manifest: &OnDiskManifest,
) -> Vec<TargetDecision> {
    plan.operations
        .iter()
        .map(|op| {
            let target = project_root.join(&op.target_path);
            let content_hash = sha256_hex(op.content.as_bytes());
            let decision = if !target.exists() {
                Decision::Create
            } else {
                let existing_hash = fs::read(&target)
                    .map(|bytes| sha256_hex(&bytes))
                    .unwrap_or_default();
                if existing_hash == content_hash {
                    // Already the desired bytes — true no-op (idempotence).
                    Decision::NoOp
                } else {
                    let recorded = manifest
                        .managed
                        .iter()
                        .find(|m| m.target_path == op.target_path)
                        .map(|m| m.content_hash.clone());
                    if recorded.as_deref() == Some(existing_hash.as_str()) {
                        Decision::Update
                    } else {
                        Decision::Conflict
                    }
                }
            };
            TargetDecision {
                target_path: op.target_path.clone(),
                decision,
                content: op.content.clone(),
                content_hash,
            }
        })
        .collect()
}

/// `--dry-run`: compute + return the transaction plan. ZERO filesystem
/// mutation — caller (CLI) prints it. Does not touch `.sos-manifest.toml`
/// or `.sos-install-incoming`.
pub fn dry_run(project_root: &Path, plan: &Plan) -> Result<Vec<TargetDecision>> {
    let manifest = load_manifest(project_root)?;
    Ok(decide_targets(project_root, plan, &manifest))
}

/// Report of a completed (successful) `apply()` transaction.
#[derive(Debug, Default)]
pub struct ApplyReport {
    pub created: Vec<String>,
    pub updated: Vec<String>,
    pub conflicts: Vec<String>,
    /// Targets already at desired content — zero mutation performed
    /// (idempotence — re-running install must not rewrite unchanged bytes
    /// or bump `.sos-manifest.toml`).
    pub noop: Vec<String>,
}

/// Pre-mutation snapshot for one applied op — used to roll back to
/// byte-identical pre-transaction state if a LATER op in the same
/// transaction fails.
struct RollbackEntry {
    target_path: PathBuf,
    /// `None` = target didn't exist before this op (CREATE) -> rollback
    /// deletes it. `Some(bytes)` = target existed (UPDATE) -> rollback
    /// restores the exact prior bytes.
    prior_content: Option<Vec<u8>>,
}

/// Execute `plan` against `project_root`: resolve non-clobber decisions,
/// mutate the filesystem, record `.sos-manifest.toml`. On ANY op failure,
/// restores the filesystem to byte-identical pre-transaction state, does
/// NOT commit the manifest, and returns `Err` (LOUD, non-zero exit at the
/// CLI layer) — `core/POLICY.md` "Safe mutation" (backup+rollback,
/// missing-gate-fail-visible).
///
/// Step 5 (tool-resolve) is intentionally NOT called here — see
/// `resolve_tools()` below; the CLI wires it as a separate no-op step so
/// this function stays pure transaction-execution.
pub fn apply(
    project_root: &Path,
    plan: &Plan,
    owner: &str,
    source_version: &str,
) -> Result<ApplyReport> {
    let manifest = load_manifest(project_root)?;
    let decisions = decide_targets(project_root, plan, &manifest);

    let mut rollback_log: Vec<RollbackEntry> = Vec::new();
    let mut report = ApplyReport::default();
    let mut next_entries: Vec<ManagedManifest> = manifest.managed.clone();

    let outcome: Result<()> = (|| {
        for d in &decisions {
            let target = project_root.join(&d.target_path);
            match d.decision {
                Decision::NoOp => {
                    // Already correct — zero mutation, zero manifest
                    // touch. This is what makes re-running install a true
                    // no-op instead of harmlessly rewriting identical
                    // bytes (and bumping rollback_ref) on every run.
                    report.noop.push(d.target_path.clone());
                    continue;
                }
                Decision::Conflict => {
                    // Non-clobber: NEVER touch the existing target. Stage
                    // the incoming (kit-side) content for manual merge —
                    // mirrors `.sos-adopt-incoming` precedent.
                    let incoming = project_root.join(INCOMING_DIR_NAME).join(&d.target_path);
                    if let Some(parent) = incoming.parent() {
                        fs::create_dir_all(parent).with_context(|| {
                            format!("create incoming parent dir for {}", d.target_path)
                        })?;
                    }
                    fs::write(&incoming, d.content.as_bytes())
                        .with_context(|| format!("stage incoming copy for {}", d.target_path))?;
                    report.conflicts.push(d.target_path.clone());
                    continue; // no manifest entry mutation, nothing mutated to roll back
                }
                Decision::Create => {
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent)
                            .with_context(|| format!("create parent dir for {}", d.target_path))?;
                    }
                    fs::write(&target, d.content.as_bytes())
                        .with_context(|| format!("write {}", d.target_path))?;
                    rollback_log.push(RollbackEntry {
                        target_path: target.clone(),
                        prior_content: None,
                    });
                    report.created.push(d.target_path.clone());
                }
                Decision::Update => {
                    let prior = fs::read(&target)
                        .with_context(|| format!("snapshot prior content of {}", d.target_path))?;
                    fs::write(&target, d.content.as_bytes())
                        .with_context(|| format!("overwrite {}", d.target_path))?;
                    rollback_log.push(RollbackEntry {
                        target_path: target.clone(),
                        prior_content: Some(prior),
                    });
                    report.updated.push(d.target_path.clone());
                }
            }

            let rollback_ref = match d.decision {
                Decision::Update => Some(format!("pre-install:{}", d.content_hash)),
                _ => None,
            };
            next_entries.retain(|m| m.target_path != d.target_path);
            next_entries.push(ManagedManifest {
                owner: owner.to_string(),
                source_version: source_version.to_string(),
                source_identity: d.target_path.clone(),
                target_path: d.target_path.clone(),
                content_hash: d.content_hash.clone(),
                rollback_ref,
            });
        }
        Ok(())
    })();

    match outcome {
        Ok(()) => {
            save_manifest(
                project_root,
                &OnDiskManifest {
                    managed: next_entries,
                },
            )?;
            // P078g — render the embedded `hooks/pre-commit` +
            // `hooks/pre-push` scripts BEFORE arming, so `core.hooksPath`
            // never points at an empty dir (fixes P078f's arm-empty gap —
            // round-3 Test A2/A3/A4). Runs on the success path, same as
            // `arm_git_hooks` below (see its comment for the no-rollback
            // rationale — this is boundary-activation, not part of the
            // file-write transaction).
            render_embedded_hooks(project_root)?;
            // P078f — arm Git hooks (core.hooksPath + F09 hijack-guard),
            // ported from `scripts/install-hooks.sh`. Runs ONLY on the
            // success path, after the manifest is committed — hook-arming
            // failure (F09 abort) does NOT roll back the just-written
            // render/manifest (arming is a separate boundary-activation
            // step, not part of the file-write transaction; the render
            // itself already succeeded and stays on disk either way).
            arm_git_hooks(project_root)?;
            Ok(report)
        }
        Err(e) => {
            // Restore byte-identical pre-transaction state, reverse order.
            for entry in rollback_log.iter().rev() {
                match &entry.prior_content {
                    None => {
                        let _ = fs::remove_file(&entry.target_path);
                    }
                    Some(bytes) => {
                        let _ = fs::write(&entry.target_path, bytes);
                    }
                }
            }
            bail!("install transaction failed and was rolled back: {e}")
        }
    }
}

/// P078f — Git hook arming (`core.hooksPath` + F09 hijack-guard). Engine-
/// native port of `scripts/install-hooks.sh` (Worker CHALLENGE Turn 1
/// anchor #3 enumerated exactly 4 steps, no hidden 5th): chmod tracked
/// hooks, F09 guard, `git config --local core.hooksPath hooks`, rename
/// any stale `.git/hooks/*` to `*.pre-hookspath.bak`. Engine-native (not
/// shelling out to `install-hooks.sh`) per Constraint 1 / P059-P072
/// precedent — `sos install` must behave identically without a Bash
/// interpreter available (Windows Git Bash absence risk); `chmod` is done
/// via Rust std Unix perms (`#[cfg(unix)]`, no-op elsewhere) and `git
/// config`/`git rev-parse` are invoked via `git` itself (cross-platform by
/// construction — the tool being configured is assumed present).
///
/// Non-git-repo → warn-skip (install still succeeds, Constraint 4).
/// Non-clobber (Constraint 2/3): a pre-existing DIFFERENT `core.hooksPath`
/// triggers TTY confirm / non-TTY abort (fail-closed) BEFORE anything is
/// mutated; idempotent re-runs (hooksPath already == "hooks", Constraint
/// 6) never trip the guard.
///
/// P078g: also NEVER arms `core.hooksPath` when `hooks/pre-commit` /
/// `hooks/pre-push` are absent or non-executable in `project_root` (see
/// `hooks_present_and_executable`) — fail-loud (`Err`) beats a false-armed
/// empty backstop (Constraint 2). `pub` (not module-private) so the
/// install-smoke test can exercise this guard directly, isolated from the
/// render step (`render_embedded_hooks` normally runs first in `apply()`).
pub fn arm_git_hooks(project_root: &Path) -> Result<()> {
    if !is_git_repo(project_root) {
        eprintln!(
            "⚠️  {} is not a git repo — skipping hook arming (core.hooksPath, chmod, hooks/ backstop)",
            project_root.display()
        );
        return Ok(());
    }

    // Step 1 — chmod +x tracked hooks. Git silently ignores non-executable
    // hooks, so this is load-bearing on Unix; no-op on Windows.
    chmod_x_if_exists(&project_root.join("hooks/pre-commit"));
    chmod_x_if_exists(&project_root.join("hooks/pre-push"));

    // Step 2 — F09 hijack-guard: don't silently redirect an adopter's own
    // hook chain. Fires only when hooksPath is set AND != "hooks" (our own
    // prior value never re-triggers — Constraint 6 idempotence).
    if let Some(existing_hp) = git_config_get(project_root, "core.hooksPath") {
        if existing_hp != "hooks" {
            let proceed = if std::io::stdin().is_terminal() {
                eprintln!(
                    "⚠️  core.hooksPath is already set to '{existing_hp}' (this repo's own hook chain)."
                );
                eprintln!(
                    "    Pointing it at sos-kit's hooks/ would REDIRECT every hook lookup, silently"
                );
                eprintln!(
                    "    disabling your existing hooks. sos-kit's hooks/ may not contain them."
                );
                eprint!("    Override core.hooksPath -> hooks/ anyway? [y/N] ");
                use std::io::Write;
                let _ = std::io::stderr().flush();
                let mut ans = String::new();
                let _ = std::io::stdin().read_line(&mut ans);
                matches!(ans.trim(), "y" | "Y")
            } else {
                false
            };
            if !proceed {
                bail!(
                    "core.hooksPath already set to '{existing_hp}'; refusing to clobber \
                     (non-interactive install won't silently override) — run \
                     scripts/install-hooks.sh manually or `git config --local --unset core.hooksPath` first"
                );
            }
        }
    }

    // Step 2.5 (P078g) — never arm an empty hooksPath. Kit source is
    // embedded at compile time so this should never fire in practice —
    // this defends against future desync (e.g. a caller invoking
    // `arm_git_hooks` directly without having rendered first).
    if !hooks_present_and_executable(project_root) {
        eprintln!(
            "❌ hook scripts not rendered — refusing to arm empty hooksPath; boundary NOT active"
        );
        bail!(
            "hooks/pre-commit or hooks/pre-push missing/non-executable in {} — refusing to set \
             core.hooksPath (would arm an empty backstop, false-security)",
            project_root.display()
        );
    }

    // Step 3 — activate tracked hooks/.
    git_config_set(project_root, "core.hooksPath", "hooks")?;

    // Step 4 — retire stale `.git/hooks/*` copies (rename, never delete —
    // escape hatch for the adopter to recover their old hook chain).
    for h in ["pre-commit", "pre-push"] {
        let existing_hook = project_root.join(".git/hooks").join(h);
        if existing_hook.is_file() {
            let backup = project_root
                .join(".git/hooks")
                .join(format!("{h}.pre-hookspath.bak"));
            let _ = fs::rename(&existing_hook, &backup);
        }
    }

    Ok(())
}

#[cfg(unix)]
fn chmod_x_if_exists(p: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = fs::metadata(p) {
        let mut perm = meta.permissions();
        let mode = perm.mode() | 0o111;
        perm.set_mode(mode);
        let _ = fs::set_permissions(p, perm);
    }
}
#[cfg(not(unix))]
fn chmod_x_if_exists(_p: &Path) {}

fn is_git_repo(project_root: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(project_root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn git_config_get(project_root: &Path, key: &str) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .args(["config", "--local", key])
        .output()
        .ok()?;
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

fn git_config_set(project_root: &Path, key: &str, value: &str) -> Result<()> {
    let out = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .args(["config", "--local", key, value])
        .output()
        .with_context(|| format!("run git config --local {key} {value}"))?;
    if !out.status.success() {
        bail!(
            "git config --local {key} {value} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

/// P078g — kit-source `hooks/pre-push`, baked into the `sos-install` binary
/// at COMPILE time. `sos install` runs standalone in an arbitrary target
/// repo — it has no `SOS_KIT_DIR` (unlike `new`/`adopt`/`sync`, Worker
/// CHALLENGE Turn 1 Anchor #4 confirmed 0 hits), so the `new.rs:315-322`
/// runtime-copy-from-kit-root pattern doesn't apply here. `include_str!` is
/// the self-contained route, and it's trivially symmetric across both
/// runtimes because it renders in the engine's own `apply()` path (not
/// per-adapter `plan()` — `ClaudeAdapter::plan()` is currently a total
/// stub, see Discovery P078g).
///
/// `pre-push` is advisory-only (never hard-blocks non-interactively — see
/// `hooks/pre-push` header) and has ZERO delegated scripts (self-contained,
/// unlike the dev `[8/8]` `pre-commit`), so it carries no dependency-closure
/// risk and is rendered as-is for both `sos new` (dev, via `new.rs`
/// `copy_tree`) and `sos install` (this path). P078i (below) does NOT touch
/// it — see Discovery P078i.
const EMBEDDED_PRE_PUSH: &str = include_str!("../../../hooks/pre-push");

/// P078i — minimal, purpose-built backstop `pre-commit` + its 2 delegated
/// guard scripts, embedded verbatim for the INSTALL path only (`sos
/// install`, brownfield/adopted repos). Distinct from the dev `[8/8]`
/// `hooks/pre-commit` at the kit repo root (that one is rendered ONLY by
/// `sos new`, via `crates/sos-cli/src/commands/new.rs:312`'s runtime
/// `copy_tree` of the full `scripts/` tree — a different code path with
/// zero overlap, confirmed Worker CHALLENGE Turn 1 anchor #5/Final
/// consensus). The dev hook's `[6/8]`/`[7/8]` phases fail-OPEN when their
/// delegated scripts are absent (round-4 A3/A4: `.env` and code-on-default
/// commits silently allowed on a brownfield install that never got the
/// full `scripts/` tree) — this backstop enforces the SAME 2 invariants but
/// fail-CLOSED, and embeds its own 2 dependencies so the install path's
/// render output is a fully closed set (dependency-closure oracle, see
/// `docs/discoveries/P078i.md`).
const EMBEDDED_BACKSTOP_PRE_COMMIT: &str = include_str!("templates/backstop-pre-commit.sh");
const EMBEDDED_BLOCK_ENV: &str = include_str!("../../../scripts/block-env-commit.sh");
const EMBEDDED_NO_CODE_DEFAULT: &str = include_str!("../../../scripts/no-code-on-default.sh");

/// Render the embedded hook scripts into `project_root/hooks/` (+ their
/// delegated guards into `project_root/scripts/`) — MUST run before
/// `arm_git_hooks()` (Constraint 3: render-before-arm) so the backstop
/// scripts exist before `core.hooksPath` is ever pointed at them. Fixes
/// P078f's arm-empty gap (round-3 Test A2: `core.hooksPath` got set but
/// `hooks/pre-commit`/`hooks/pre-push` never existed on disk).
///
/// P078i: `hooks/pre-commit` now renders the minimal backstop template
/// (NOT the dev `[8/8]` hook — round-4 A3/A4 fixed by closing the
/// dependency gap those 2 phases fail-open on), and its 2 delegated guard
/// scripts (`scripts/block-env-commit.sh`, `scripts/no-code-on-default.sh`)
/// are rendered alongside it so the install path never produces a hook
/// that references a script it didn't also create.
///
/// Non-clobber: if a target already holds content DIFFERENT from the
/// embedded script (an adopter customized their own hook), it is left
/// untouched (warn only) — this extends F09's non-clobber philosophy
/// (round-3 A5) to the render step, not just the hooksPath guard.
fn render_embedded_hooks(project_root: &Path) -> Result<()> {
    for (rel, content) in [
        ("hooks/pre-commit", EMBEDDED_BACKSTOP_PRE_COMMIT),
        ("hooks/pre-push", EMBEDDED_PRE_PUSH),
        ("scripts/block-env-commit.sh", EMBEDDED_BLOCK_ENV),
        ("scripts/no-code-on-default.sh", EMBEDDED_NO_CODE_DEFAULT),
    ] {
        let target = project_root.join(rel);
        if target.exists() {
            let existing = fs::read(&target).unwrap_or_default();
            if existing == content.as_bytes() {
                continue; // already correct — no-op (idempotence)
            }
            eprintln!(
                "⚠️  {rel} already exists with different content — leaving it untouched \
                 (non-clobber; sos-kit's own hook script was NOT written here)"
            );
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create parent dir for {rel}"))?;
        }
        fs::write(&target, content.as_bytes())
            .with_context(|| format!("render embedded hook script {rel}"))?;
        // P078i: all 4 rendered artifacts (backstop pre-commit, pre-push,
        // + the 2 delegated guard scripts) get +x here so a fresh render
        // is immediately executable without depending on `arm_git_hooks`'s
        // narrower chmod (which only covers hooks/pre-commit + hooks/pre-push).
        chmod_x_if_exists(&target);
    }
    Ok(())
}

/// P078g — Constraint 2 (fail-loud > false-armed): `true` only if BOTH
/// `hooks/pre-commit` and `hooks/pre-push` exist as regular files AND (on
/// Unix) are executable. Called right before `arm_git_hooks()` sets
/// `core.hooksPath` — if this returns `false`, the caller MUST refuse to
/// arm rather than point `core.hooksPath` at an empty/non-executable dir.
fn hooks_present_and_executable(project_root: &Path) -> bool {
    for h in ["hooks/pre-commit", "hooks/pre-push"] {
        let p = project_root.join(h);
        if !p.is_file() {
            return false;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            match fs::metadata(&p) {
                Ok(meta) if meta.permissions().mode() & 0o111 != 0 => {}
                _ => return false,
            }
        }
    }
    true
}

/// P077d3 (OA-07): tool-manifest resolve — step 5 of the transaction
/// 7-step (`docs/PORTABILITY_ARCHITECTURE.md:109-117`), filled in.
///
/// Verify-ONLY (d3 GIỚI HẠN): reads the embedded `tool-manifest.toml` pin,
/// resolves each sister tool's installed `--version` against it
/// (`crate::tools::gate_required`), and fails LOUD if any REQUIRED tool is
/// missing/older-than-pinned/unparseable. Runs BEFORE `decide_targets()`/
/// `apply()` (same call position d2 wired at, `commands/install.rs`) — so
/// a step-5 failure happens before any filesystem mutation, which
/// trivially satisfies "fail rõ + rollback": there is nothing to roll
/// back yet because nothing was written. No atomic-upgrade/auto-fetch of
/// the external binaries happens here — that's P081 future.
pub fn resolve_tools() -> Result<Vec<crate::tools::ToolStatus>> {
    let manifest = crate::tools::embedded_manifest_parsed()?;
    crate::tools::gate_required(&manifest, None)
}
