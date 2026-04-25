use anyhow::{bail, Result};
use std::path::PathBuf;

pub fn new(name: &str) -> Result<()> {
    let kit_dir = sos_kit_dir();
    let path = kit_dir.join("recipes").join(format!("{name}.md"));
    if path.exists() {
        bail!(
            "Recipe exists: {}\n  To revise → invoke /forge skill with 'update' option.",
            path.display()
        );
    }
    println!("Open Claude Code and invoke skill /forge with arg: {name}");
    println!(
        "Skill /forge will: research official docs → write recipe → save to {} → commit.",
        path.display()
    );
    Ok(())
}

fn sos_kit_dir() -> PathBuf {
    std::env::var("SOS_KIT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}
