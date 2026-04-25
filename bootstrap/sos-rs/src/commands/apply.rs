use anyhow::{bail, Result};
use std::fs;
use std::path::PathBuf;

use crate::state::{self, Phase};

pub fn run(recipe: &str) -> Result<()> {
    let s = state::load()?;
    if !matches!(s.state.phase, Phase::Locked | Phase::Scaffolded | Phase::Iterating) {
        bail!(
            "State is {:?} — must run 'sos contract' first to lock P000.",
            s.state.phase
        );
    }

    if recipe == "--all" {
        return apply_all();
    }

    let kit_dir = sos_kit_dir();
    let recipe_path = kit_dir.join("recipes").join(format!("{recipe}.md"));
    if !recipe_path.exists() {
        bail!(
            "Recipe not found: {}\n  Forge it first: sos recipe new {}",
            recipe_path.display(),
            recipe
        );
    }

    println!("─────────────────────────────────────────");
    println!("Phase 3 — Apply: {recipe}");
    println!("─────────────────────────────────────────\n");
    println!("Recipe: {}\n", recipe_path.display());
    println!("Open Claude Code and invoke skill /apply with arg: {recipe}");
    println!("Skill will:");
    println!("  1. Read recipe + verify Inputs satisfied");
    println!("  2. Generate sub-phiếu P000.N");
    println!("  3. Execute Steps (with plan mode if > 5 steps)");
    println!("  4. Run Verification anchors");
    println!("  5. Update state.toml + DISCOVERIES.md + commit");

    state::set_phase(Phase::Scaffolded)?;
    Ok(())
}

fn apply_all() -> Result<()> {
    println!("Reading recipe list from docs/ticket/P000-genesis.md...");
    let raw = fs::read_to_string("docs/ticket/P000-genesis.md")?;
    let recipes = parse_recipe_list(&raw);
    if recipes.is_empty() {
        bail!("No recipes parsed from P000-genesis.md. Did Kiến trúc sư fill 'Recipes to apply'?");
    }
    println!("Recipes to apply (in order):");
    for r in &recipes {
        println!("  - {r}");
    }
    println!("\nOpen Claude Code and invoke /apply per recipe in order.");
    Ok(())
}

fn parse_recipe_list(content: &str) -> Vec<String> {
    let start = content.find("### Recipes to apply");
    let end = content.find("### Recipes thiếu");
    let section = match (start, end) {
        (Some(s), Some(e)) if s < e => &content[s..e],
        _ => return vec![],
    };
    section
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix(|c: char| c.is_ascii_digit()) {
                let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
                let rest = rest.trim_start_matches('.');
                let rest = rest.trim();
                let rest = rest.trim_matches('`');
                if rest.is_empty() || rest.starts_with('[') {
                    None
                } else {
                    Some(rest.to_string())
                }
            } else {
                None
            }
        })
        .collect()
}

fn sos_kit_dir() -> PathBuf {
    std::env::var("SOS_KIT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}
