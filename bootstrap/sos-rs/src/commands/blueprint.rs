use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::state::{self, Phase};

pub fn run() -> Result<()> {
    if !Path::new("docs/PROJECT.md").exists() {
        bail!("docs/PROJECT.md missing. Run 'sos init' first.");
    }
    state::init_if_missing()?;
    let s = state::load()?;
    if !matches!(s.state.phase, Phase::VisionCaptured | Phase::Init) {
        eprintln!("⚠ State is {:?} — blueprint expected after vision capture.", s.state.phase);
    }

    println!("─────────────────────────────────────────");
    println!("Phase 1 — Blueprint (Stack + Recipes)");
    println!("─────────────────────────────────────────\n");
    println!("Open Claude Code and have Kiến trúc sư:");
    println!("  1. Read docs/PROJECT.md (+ SOUL.md if exists)");
    println!("  2. Pick tech stack appropriate to vision");
    println!("  3. List recipes from sos-kit/recipes/ in order of apply");
    println!("  4. Flag missing recipes → forge before contract phase");
    println!("  5. Write everything to docs/BLUEPRINT.md\n");

    println!("Recipes available:");
    let kit_dir = sos_kit_dir();
    let recipes_dir = kit_dir.join("recipes");
    if recipes_dir.is_dir() {
        let mut found: Vec<String> = WalkDir::new(&recipes_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
            .filter(|e| {
                let name = e.path().file_name().and_then(|s| s.to_str()).unwrap_or("");
                !["_TEMPLATE.md", "README.md"].contains(&name)
            })
            .filter_map(|e| {
                e.path()
                    .strip_prefix(&recipes_dir)
                    .ok()
                    .map(|p| p.with_extension("").display().to_string())
            })
            .collect();
        found.sort();
        for r in &found {
            println!("  - {r}");
        }
    } else {
        println!("  (recipes/ not found at {})", recipes_dir.display());
    }
    println!("\nAfter BLUEPRINT.md ready → 'sos contract' to lock as P000-genesis.md.");

    state::set_phase(Phase::BlueprintDrafted)?;
    Ok(())
}

fn sos_kit_dir() -> PathBuf {
    if let Ok(p) = std::env::var("SOS_KIT_DIR") {
        return PathBuf::from(p);
    }
    // Best-effort: assume binary is installed and SOS_KIT_DIR not set.
    // Fall back to current working dir.
    PathBuf::from(".")
}
