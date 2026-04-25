use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

use crate::state::{self, HistoryEntry, Phase};

const GENESIS_PATH: &str = "docs/ticket/P000-genesis.md";

pub fn run(skip: Vec<String>, reason: Option<String>) -> Result<()> {
    if !Path::new(GENESIS_PATH).exists() {
        bail!("No P000-genesis.md. This isn't a Genesis-managed project.");
    }
    println!("─────────────────────────────────────────");
    println!("Phase N+1 — Launch Gate");
    println!("─────────────────────────────────────────\n");

    let raw = fs::read_to_string(GENESIS_PATH)?;
    let (total, ticked, untick_lines) = parse_checklist(&raw);
    println!("  Ticked: {ticked} / {total}");

    if total == 0 {
        bail!("Could not parse Launch Checklist. Verify P000 structure.");
    }

    let untick_count = total - ticked;
    let skip_count = skip.len();
    if ticked < total && skip_count < untick_count {
        eprintln!("\n✗ HARD BLOCK — checklist incomplete ({ticked}/{total}).\n");
        eprintln!("Untick items:");
        for line in untick_lines.iter().take(20) {
            eprintln!("  {line}");
        }
        eprintln!("\nBypass with --skip <items> --reason \"...\" (audited). Not recommended.");
        std::process::exit(1);
    }

    if skip_count > 0 {
        if reason.is_none() {
            bail!("--skip requires --reason \"...\" for audit trail.");
        }
        println!("⚠ Skipping {skip_count} checklist items: {:?}", skip);
        println!("  Reason: {}", reason.as_ref().unwrap());
    }

    println!("\n✓ Checklist gate passed");
    println!("  Now run: guard check_all && ship canary");

    let now = chrono::Utc::now().to_rfc3339();
    state::append_history(HistoryEntry {
        event: "launch".into(),
        timestamp: now,
        spec_hash: None,
        by: Some("Chủ nhà".into()),
        reason: if skip_count > 0 { reason } else { None },
    })?;
    state::set_phase(Phase::Launched)?;

    println!("\n🎉 Launched. Don't forget docs/DISCOVERIES.md retro entry.");
    Ok(())
}

fn parse_checklist(content: &str) -> (usize, usize, Vec<String>) {
    let start = content.find("## 5. Launch Checklist");
    let end = content.find("## 6.");
    let section = match (start, end) {
        (Some(s), Some(e)) if s < e => &content[s..e],
        _ => return (0, 0, vec![]),
    };
    let mut total = 0;
    let mut ticked = 0;
    let mut untick: Vec<String> = vec![];
    for line in section.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            total += 1;
            ticked += 1;
        } else if trimmed.starts_with("- [ ]") {
            total += 1;
            untick.push(trimmed.to_string());
        }
    }
    (total, ticked, untick)
}
