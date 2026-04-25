use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::state::{self, HistoryEntry, Phase};

const GENESIS_PATH: &str = "docs/ticket/P000-genesis.md";

pub fn run() -> Result<()> {
    if !Path::new("docs/BLUEPRINT.md").exists() {
        bail!("docs/BLUEPRINT.md missing. Run 'sos blueprint' first.");
    }
    if !Path::new(GENESIS_PATH).exists() {
        bail!(
            "{} missing. /init skill should have copied GENESIS_TEMPLATE.md there.",
            GENESIS_PATH
        );
    }

    println!("Open Claude Code and have Kiến trúc sư fill {GENESIS_PATH}");
    println!("  - Vision Anchor (from PROJECT.md + SOUL.md)");
    println!("  - MVP Scope (Core features + Can ship without)");
    println!("  - Tech Commitments + Recipes to apply (from BLUEPRINT.md)");
    println!("  - Verification Anchors (project-specific invariants)");
    println!("  - Launch Checklist (copy from phieu/LAUNCH_CHECKLIST.md)\n");
    print!("When P000-genesis.md is ready, type 'lock' to compute spec_hash and lock: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim() != "lock" {
        println!("Aborted. Re-run 'sos contract' when ready.");
        return Ok(());
    }

    // Compute spec_hash on frozen sections (## 1. Vision Anchor through start of ## 4.)
    let content = fs::read_to_string(GENESIS_PATH)
        .with_context(|| format!("read {GENESIS_PATH}"))?;
    let frozen = extract_frozen_sections(&content);
    if frozen.is_empty() {
        bail!("Could not parse frozen sections (## 1, ## 2, ## 3) from {GENESIS_PATH}");
    }

    let mut hasher = Sha256::new();
    hasher.update(frozen.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    let now = chrono::Utc::now().to_rfc3339();

    // Update P000 header (Spec Hash + Locked at lines)
    let updated = update_genesis_header(&content, &hash, &now);
    fs::write(GENESIS_PATH, updated)?;

    // Update state
    let mut s = state::load()?;
    s.state.spec_hash = Some(format!("sha256:{hash}"));
    s.state.last_updated = now.clone();
    s.history.push(HistoryEntry {
        event: "contract.lock".into(),
        timestamp: now,
        spec_hash: Some(format!("sha256:{hash}")),
        by: Some("Chủ nhà".into()),
        reason: Some("Genesis".into()),
    });
    state::save(&s)?;
    state::set_phase(Phase::Locked)?;

    println!("✓ {GENESIS_PATH} locked");
    println!("  spec_hash: sha256:{hash}\n");
    println!("Next: 'sos apply --all' to scaffold via recipes.");
    Ok(())
}

fn extract_frozen_sections(content: &str) -> String {
    let start = content.find("## 1. Vision Anchor");
    let end = content.find("## 4. Verification Anchors");
    match (start, end) {
        (Some(s), Some(e)) if s < e => content[s..e].to_string(),
        _ => String::new(),
    }
}

fn update_genesis_header(content: &str, hash: &str, ts: &str) -> String {
    let mut out = String::with_capacity(content.len());
    for line in content.lines() {
        if line.starts_with("> **Spec Hash:**") {
            out.push_str(&format!("> **Spec Hash:** `sha256:{hash}`\n"));
        } else if line.starts_with("> **Locked at:**") {
            out.push_str(&format!("> **Locked at:** `{ts}`\n"));
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}
