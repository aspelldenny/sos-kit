// commands/sync.rs — Rust port of `bin/sos.sh` sos_sync (bin/sos.sh:973-1053).
//
// P077c2: bug-for-bug parity with the Bash oracle. KIT-LAG cure (P067):
// re-sync sos-kit spine into a repo adopted from an OLDER kit version —
// take-newer the spine files the repo HASN'T customized, flag genuinely-
// customized ones. Provenance oracle = sos-kit's OWN git history (walked via
// shelling out to `git`, no git2 dep — see anchor #10 / Debate Log Turn 1).
//
// IMPORTANT — traversal order (Debate Log Turn 1 finding): Bash's `find` for
// the spine-walk roots (`scripts`/`phieu`/`templates`, `skills/**/SKILL.md`)
// is UNSORTED — it returns raw filesystem directory-entry order, NOT
// alphabetical (unlike `map.rs`'s `hits.sort()`, whose Bash counterpart
// pipes through `sort`). To match the stdout report (`sync.golden`)
// byte-for-byte, this module must NOT sort the walk — it preserves raw
// `read_dir`/`WalkDir` enumeration order, same as `find` on the same
// filesystem. `sync.tree.golden` (file-manifest fixture) is sorted
// separately at the test layer, so it stays order-independent regardless.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// One spine-sync outcome for a single destination path.
enum Outcome {
    Added(String),
    Updated(String),
    Flagged(String),
}

/// `git hash-object <path>` — matches Bash's `git hash-object "$dest"`
/// (`bin/sos.sh:1007`). No git2 dep (anchor #10) — shell out, bug-for-bug.
fn git_hash_object(path: &Path) -> Option<String> {
    let out = Command::new("git").arg("hash-object").arg(path).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Replicates `_blob_in_history` (`bin/sos.sh:992-999`): walk
/// `git -C "$kit" rev-list --all -- <rel>`, and for each commit check
/// `git -C "$kit" rev-parse "<commit>:<rel>"` against `want`. Return true if
/// ANY historical blob of the canonical path matches.
fn blob_in_history(kit: &Path, rel: &str, want: &str) -> bool {
    let rev_list = Command::new("git")
        .arg("-C")
        .arg(kit)
        .arg("rev-list")
        .arg("--all")
        .arg("--")
        .arg(rel)
        .output();
    let rev_list = match rev_list {
        Ok(o) if o.status.success() => o,
        _ => return false,
    };
    let commits = String::from_utf8_lossy(&rev_list.stdout);
    for c in commits.lines() {
        let c = c.trim();
        if c.is_empty() {
            continue;
        }
        let spec = format!("{c}:{rel}");
        let out = Command::new("git").arg("-C").arg(kit).arg("rev-parse").arg(&spec).output();
        if let Ok(o) = out {
            if o.status.success() {
                let h = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if h == want {
                    return true;
                }
            }
        }
    }
    false
}

/// Replicates `_sync_one` (`bin/sos.sh:1000-1014`): copy `canon` (relative to
/// `kit`) to `destrel` (relative to `target`), classifying ADDED / UPDATED /
/// FLAGGED / (silent) ALREADY-CURRENT. Returns `None` for ALREADY-CURRENT or
/// if the source file doesn't exist in the kit (mirrors Bash's early return).
fn sync_one(kit: &Path, target: &Path, canon: &str, destrel: &str) -> Option<Outcome> {
    let src = kit.join(canon);
    let dest = target.join(destrel);
    if !src.is_file() {
        return None;
    }
    if !dest.exists() {
        if let Some(parent) = dest.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::copy(&src, &dest);
        return Some(Outcome::Added(destrel.to_string()));
    }
    // cmp -s equivalent: byte-identical -> ALREADY-CURRENT, no-op.
    if let (Ok(a), Ok(b)) = (fs::read(&src), fs::read(&dest)) {
        if a == b {
            return None;
        }
    }
    let blob = match git_hash_object(&dest) {
        Some(b) => b,
        None => return None,
    };
    if blob_in_history(kit, canon, &blob) {
        let _ = fs::copy(&src, &dest);
        Some(Outcome::Updated(destrel.to_string()))
    } else {
        let incoming_dest = target.join(".sos-sync-incoming").join(destrel);
        if let Some(parent) = incoming_dest.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::copy(&src, &incoming_dest);
        Some(Outcome::Flagged(destrel.to_string()))
    }
}

/// Spine-walk roots (`bin/sos.sh:1027-1032`): `scripts`/`phieu`/`templates`,
/// recursive `find -type f`, excluding `__pycache__`/`.pyc`/`.DS_Store`.
/// NOT sorted — raw WalkDir enumeration order, matching Bash's unsorted `find`.
fn walk_root_files(kit: &Path, root: &str) -> Vec<String> {
    let root_dir = kit.join(root);
    if !root_dir.is_dir() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for entry in WalkDir::new(&root_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        let path_str = path.to_string_lossy();
        if path_str.contains("/__pycache__/")
            || path.extension().and_then(|e| e.to_str()) == Some("pyc")
            || path.file_name().and_then(|n| n.to_str()) == Some(".DS_Store")
        {
            continue;
        }
        if let Ok(rel) = path.strip_prefix(kit) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    out
}

/// `skills/**/SKILL.md` (`bin/sos.sh:1043-1044`), not under `attic/`. NOT sorted.
fn walk_skill_files(kit: &Path) -> Vec<String> {
    let skills_dir = kit.join("skills");
    if !skills_dir.is_dir() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for entry in WalkDir::new(&skills_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        if path.file_name().and_then(|n| n.to_str()) != Some("SKILL.md") {
            continue;
        }
        let path_str = path.to_string_lossy();
        if path_str.contains("/attic/") {
            continue;
        }
        if let Ok(rel) = path.strip_prefix(kit) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    out
}

/// `agents/*.md` (`bin/sos.sh:1036-1039`), non-recursive, skip README.md. NOT sorted.
fn walk_agents_files(kit: &Path) -> Vec<String> {
    let agents_dir = kit.join("agents");
    let mut out = Vec::new();
    let entries = match fs::read_dir(&agents_dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let base = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if base == "README.md" {
            continue;
        }
        out.push(format!("agents/{base}"));
    }
    out
}

/// `.claude/commands/*` (`bin/sos.sh:1040-1041`), non-recursive. NOT sorted.
fn walk_commands_files(kit: &Path) -> Vec<String> {
    let dir = kit.join(".claude/commands");
    let mut out = Vec::new();
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let base = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        out.push(format!(".claude/commands/{base}"));
    }
    out
}

pub fn run(target: &str) -> Result<()> {
    let target_dir = PathBuf::from(target);
    if !target_dir.is_dir() {
        println!("✗ {target} not found");
        return Ok(());
    }
    let kit_str = std::env::var("SOS_KIT_DIR").unwrap_or_default();
    let kit = PathBuf::from(&kit_str);
    if !kit.join(".git").is_dir() {
        println!(
            "✗ SOS_KIT_DIR ({kit_str}) has no .git — sync needs kit history as provenance oracle"
        );
        return Ok(());
    }
    if !kit.join(".claude/agents").is_dir() {
        println!("✗ SOS_KIT_DIR ({kit_str}) is not sos-kit");
        return Ok(());
    }

    let mut added: Vec<String> = Vec::new();
    let mut updated: Vec<String> = Vec::new();
    let mut flagged: Vec<String> = Vec::new();
    let mut current = 0usize;

    let mut record = |outcome: Option<Outcome>| match outcome {
        Some(Outcome::Added(p)) => added.push(p),
        Some(Outcome::Updated(p)) => updated.push(p),
        Some(Outcome::Flagged(p)) => flagged.push(p),
        None => current += 1,
    };

    println!(
        "sos sync — re-sync spine into '{target}'  (take-newer unmodified, flag customized)"
    );
    println!("  provenance oracle: {kit_str} git history");

    // Dirty-warn (bin/sos.sh:1021-1025).
    let target_git_status = if target_dir.join(".git").is_dir() {
        Command::new("git")
            .arg("-C")
            .arg(&target_dir)
            .arg("status")
            .arg("--porcelain")
            .output()
            .ok()
    } else {
        None
    };
    if let Some(out) = target_git_status {
        if out.status.success() && !out.stdout.is_empty() {
            println!("  ⚠ target has UNCOMMITTED changes (possibly a LIVE session mid-phiếu).");
            println!(
                "    sync only ADDS/UPDATES spine files, but: do NOT 'git add -A' afterwards —"
            );
            println!("    you would stage the repo's in-flight work. Let that repo's own session commit.");
        }
    }

    // Spine-walk roots: scripts/phieu/templates (bin/sos.sh:1027-1032).
    for root in ["scripts", "phieu", "templates"] {
        for rel in walk_root_files(&kit, root) {
            let outcome = sync_one(&kit, &target_dir, &rel, &rel);
            record(outcome);
        }
    }
    // Fixed single files (bin/sos.sh:1033-1035).
    for rel in [
        "hooks/pre-commit",
        "hooks/pre-push",
        "docs/ORCHESTRATION.md",
        ".claude/settings.json",
    ] {
        if kit.join(rel).is_file() {
            let outcome = sync_one(&kit, &target_dir, rel, rel);
            record(outcome);
        }
    }
    // agents/*.md -> .claude/agents/*.md (bin/sos.sh:1036-1039).
    for canon in walk_agents_files(&kit) {
        let base = canon.trim_start_matches("agents/");
        let destrel = format!(".claude/agents/{base}");
        let outcome = sync_one(&kit, &target_dir, &canon, &destrel);
        record(outcome);
    }
    // .claude/commands/* (bin/sos.sh:1040-1041).
    for canon in walk_commands_files(&kit) {
        let outcome = sync_one(&kit, &target_dir, &canon, &canon);
        record(outcome);
    }
    // skills/**/SKILL.md -> .claude/skills/**/SKILL.md (bin/sos.sh:1043-1044).
    for canon in walk_skill_files(&kit) {
        let rel = canon.trim_start_matches("skills/");
        let destrel = format!(".claude/skills/{rel}");
        let outcome = sync_one(&kit, &target_dir, &canon, &destrel);
        record(outcome);
    }

    println!();
    println!("=== sos sync report ===");
    println!("ADDED (absent -> copied): {}", added.len());
    if added.is_empty() {
        println!("    (none)");
    } else {
        for p in &added {
            println!("    + {p}");
        }
    }
    println!("UPDATED (take-newer, unmodified stale): {}", updated.len());
    if updated.is_empty() {
        println!("    (none)");
    } else {
        for p in &updated {
            println!("    ^ {p}");
        }
    }
    println!(
        "FLAGGED (repo-customized -> .sos-sync-incoming, merge by hand): {}",
        flagged.len()
    );
    if flagged.is_empty() {
        println!("    (none)");
    } else {
        for p in &flagged {
            println!("    ~ {p}");
        }
    }
    println!("ALREADY-CURRENT: {current}");
    println!("Identity files (CLAUDE.md/BACKLOG/.docs-gate.toml/INVARIANTS/AGENT_MAP) untouched by design.");

    Ok(())
}
