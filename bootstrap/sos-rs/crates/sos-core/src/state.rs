// .sos/state.toml management — read/write/transition

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Phase {
    Init,
    VisionCaptured,
    BlueprintDrafted,
    Locked,
    Scaffolded,
    Iterating,
    Launched,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub state: StateMeta,
    #[serde(default)]
    pub vision: Option<VisionMeta>,
    #[serde(default)]
    pub applied_recipes: Vec<AppliedRecipe>,
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateMeta {
    pub phase: Phase,
    pub created_at: String,
    pub last_updated: String,
    #[serde(default)]
    pub spec_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisionMeta {
    pub project_name: String,
    pub has_persona: bool,
    pub docs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppliedRecipe {
    pub name: String,
    pub phieu: String,
    pub applied_at: String,
    pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub event: String,
    pub timestamp: String,
    #[serde(default)]
    pub spec_hash: Option<String>,
    #[serde(default)]
    pub by: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

const STATE_PATH: &str = ".sos/state.toml";

pub fn load() -> Result<State> {
    let raw = fs::read_to_string(STATE_PATH)
        .with_context(|| format!("missing {STATE_PATH} — run `sos init` first"))?;
    Ok(toml::from_str(&raw)?)
}

pub fn save(s: &State) -> Result<()> {
    fs::create_dir_all(".sos")?;
    let raw = toml::to_string_pretty(s)?;
    fs::write(STATE_PATH, raw)?;
    Ok(())
}

pub fn init_if_missing() -> Result<()> {
    if Path::new(STATE_PATH).exists() {
        return Ok(());
    }
    let now = chrono::Utc::now().to_rfc3339();
    let state = State {
        state: StateMeta {
            phase: Phase::Init,
            created_at: now.clone(),
            last_updated: now,
            spec_hash: None,
        },
        vision: None,
        applied_recipes: vec![],
        history: vec![],
    };
    save(&state)?;
    println!("✓ Created {STATE_PATH}");
    Ok(())
}

pub fn set_phase(new_phase: Phase) -> Result<()> {
    let mut s = load()?;
    s.state.phase = new_phase.clone();
    s.state.last_updated = chrono::Utc::now().to_rfc3339();
    save(&s)?;
    println!("✓ State → {new_phase:?}");
    Ok(())
}

pub fn append_history(entry: HistoryEntry) -> Result<()> {
    let mut s = load()?;
    s.history.push(entry);
    save(&s)
}
