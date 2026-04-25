use anyhow::Result;
use std::path::Path;

use crate::state;

pub fn run() -> Result<()> {
    if Path::new("docs/PROJECT.md").exists() {
        println!("⚠ docs/PROJECT.md already exists. This project is past phase 0.");
        println!("  Use '/insight' skill to refine, or 'sos status' to see current phase.");
        return Ok(());
    }
    state::init_if_missing()?;
    println!("─────────────────────────────────────────");
    println!("Phase 0 — Vision Capture");
    println!("─────────────────────────────────────────\n");
    println!("Open Claude Code in this directory and run skill /init.\n");
    println!("The /init skill will:");
    println!("  1. Ask 3 questions (project type, persona, pitch)");
    println!("  2. Generate docs/PROJECT.md (+ SOUL.md, CHARACTER.md if persona)");
    println!("  3. Initialize phiếu workflow");
    println!("  4. Copy GENESIS_TEMPLATE.md → docs/ticket/P000-genesis.md (draft)");
    println!("  5. Update .sos/state.toml → phase = VISION_CAPTURED\n");
    println!("After /init done: 'sos blueprint' to continue Phase 1.");
    Ok(())
}
