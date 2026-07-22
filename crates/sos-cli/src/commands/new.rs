// commands/new.rs — Rust port of `bin/sos.sh` sos_new (bin/sos.sh:355-604).
//
// P077c3: bug-for-bug parity with the Bash oracle. Bootstraps a fresh repo from
// sos-kit golden:
//   [1/4] Category A — freeze: copy proven spine verbatim (deref agent/skill symlinks).
//   [2/4] Category C — skeleton: generate per-repo files with `# TODO` markers.
//   [3/4] Category B — defaults: `.docs-gate.toml` + stack-detection `.sos-stack.toml`.
//   [4/4] VALIDATOR — `doctor verify-setup` (wiring) + grep `# TODO` (content to fill).
//   [+]   git init + arm hooks (no auto-commit — the bootstrap commit is the user's).
//
// Three-layer parity fixture (tests/parity.rs `parity_new_enforced`):
//   new.golden      — stdout report (captured with DOCTOR_BIN=/nonexistent/doctor,
//                      so [4/4] takes the deterministic doctor-absent skip branch —
//                      see tests/README.md "sos new — synthetic fake-kit + doctor-absent").
//   new.tree.golden — path-shape manifest (sorted, no content, excludes `.git/`).
//   new.gen.golden  — content-hash manifest, GENERATED-authored files ONLY. Copied
//                      kit assets are tree-only (never hashed) — hashing them would
//                      couple this fixture to every future kit-asset content change
//                      (kit-content-coupling, the exact thing this split guards against).
//
// Deliberate divergence from Bash's raw runtime behavior (documented, not a parity
// bug): Bash's "Category C placeholders to fill" list is built via `grep -rl`
// (bin/sos.sh:583), which is filesystem-enumeration-order dependent, NOT sorted.
// `capture.sh`'s `sort_todo_block` normalizes that block before freezing the golden,
// so the FROZEN ORACLE is always sorted even though live Bash's raw stdout isn't.
// This Rust port sorts that list directly (like `map.rs`'s `hits.sort()`, whose Bash
// counterpart also explicitly sorts) so the binary's raw output already matches the
// canonical (sorted) golden — no extra order-normalization needed in the test harness.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

const MCP_JSON: &str = r#"{
  "mcpServers": {
    "doctor": {
      "_comment": "Mechanical gate — lane-check, validate-map, rotate-check, runtime-scan. Universal (zero-config). Install: cargo install --path ~/doctor.",
      "command": "doctor",
      "args": ["serve"]
    },
    "docs-gate": {
      "_comment": "Docs compliance — CHANGELOG/ARCHITECTURE/Discovery checks (same engine the pre-commit hook runs as CLI). Install: cargo install --path ~/docs-gate.",
      "command": "docs-gate",
      "args": ["serve"]
    },
    "ship": {
      "_comment": "Release workflow — test, commit, push, PR, canary, deploy. The commit/push/PR/check layer is universal (any repo); canary/deploy need a target. Install: cargo install --path ~/ship.",
      "command": "ship",
      "args": ["serve"]
    },
    "guard": {
      "_comment": "Pre-deploy infra gate — schema drift + env sync over SSH. Inert until the repo has a deploy/SSH target + config. Install: cargo install --path ~/guard.",
      "command": "guard",
      "args": ["serve"]
    },
    "vps": {
      "_comment": "VPS ops — docker status/logs/restart/stats. Inert until ~/.vps.toml names a host. Install: cargo install --path ~/vps.",
      "command": "vps",
      "args": ["serve"]
    }
  }
}
"#;

const AGENT_MAP_STUB: &str = "# AGENT_MAP — UNMAPPED DRAFT. status: draft_unmapped.\n# This repo has no mapped surfaces yet. Do NOT route phiếu by this file until filled.\n#   Brownfield (existing code): run `sos map .` to scan-generate real surfaces.\n#   Greenfield: add surfaces as you build them. configs/AGENT_MAP.example.yaml is a SHAPE\n#     reference (schema), NOT content to copy — copying its tarot surfaces = \"map nói dối\".\nversion: 1\nstatus: draft_unmapped\nsurfaces: {}\n";

const INVARIANTS_APPEND: &str = "\n## INV-LOCAL (project-specific — FILL THESE)\n\n# TODO: add at least one `## INV-LOCAL-NNN — <title>` for this product, or write `# No local INV` if none.\n";

const DOCS_GATE_TOML: &str = r#"# docs-gate config — bootstrap default (sos new). Mirrors sos-kit's working config.
# Paths are relative to docs_dir; CHANGELOG.md lives at repo root via "../".
docs_dir = "docs"
changelog = "../CHANGELOG.md"
changelog_max_age_days = 1
changelog_staged = false   # docs-gate v0.1.0 "../" normalization bug — age check still enforces freshness

rules = []
staleness = []
doc_structure = []
count_check = []
cross_doc = []

[architecture]
enabled = true
file = "ARCHITECTURE.md"   # resolves to docs/ARCHITECTURE.md (relative to docs_dir)
required_sections = 0      # flexible — skeleton ships; tighten when filled
required_non_empty = []

[ticket]
ticket_dir = "docs/ticket"
type_pattern = ""
valid_types = []
exclude_files = []
"#;

/// `cp -R[L]`: recursively copy a directory tree. `deref = true` follows
/// symlinks while walking (matches Bash's `cp -RL` for `.claude/agents`,
/// `.claude/commands`, and each living skill dir — those contain per-file
/// symlinks, e.g. `.claude/agents/*.md -> ../../agents/*.md`).
fn copy_tree(src: &Path, dst: &Path, deref: bool) -> Result<()> {
    if !src.is_dir() {
        return Ok(());
    }
    for entry in WalkDir::new(src).follow_links(deref).into_iter().filter_map(|e| e.ok()) {
        let rel = entry.path().strip_prefix(src)?;
        let target_path = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target_path)?;
        } else {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target_path)?;
        }
    }
    Ok(())
}

fn strip_copy_cruft(target: &Path) {
    let mut to_remove: Vec<PathBuf> = Vec::new();
    let mut dirs_to_remove: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(target).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if entry.file_type().is_dir() && name == "__pycache__" {
            dirs_to_remove.push(path.to_path_buf());
        } else if entry.file_type().is_file()
            && (name == ".DS_Store" || path.extension().and_then(|e| e.to_str()) == Some("pyc"))
        {
            to_remove.push(path.to_path_buf());
        }
    }
    for p in to_remove {
        let _ = fs::remove_file(p);
    }
    for d in dirs_to_remove {
        let _ = fs::remove_dir_all(d);
    }
}

#[cfg(unix)]
fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(p)
        .map(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0))
        .unwrap_or(false)
}
#[cfg(not(unix))]
fn is_executable(p: &Path) -> bool {
    p.is_file()
}

#[cfg(unix)]
fn chmod_x(p: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = fs::metadata(p) {
        let mut perm = meta.permissions();
        let mode = perm.mode() | 0o111;
        perm.set_mode(mode);
        let _ = fs::set_permissions(p, perm);
    }
}
#[cfg(not(unix))]
fn chmod_x(_p: &Path) {}

/// Bash: `command -v "$doctor_bin" >/dev/null 2>&1 || [[ -x "$doctor_bin" ]]`.
fn doctor_available(bin: &str) -> bool {
    if bin.contains('/') {
        return is_executable(Path::new(bin));
    }
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            if is_executable(&dir.join(bin)) {
                return true;
            }
        }
    }
    false
}

/// Replicates `sos_init_security` (`bin/sos.sh:121-`) detection-only (the
/// function's own echo/warn output is discarded by Bash's
/// `sos_init_security >/dev/null 2>&1` at the `new` call site — bin/sos.sh:565
/// — so only the FILE it writes is observable parity surface, not its stdout).
fn write_sos_stack_toml(target: &Path) {
    let stack_file = target.join(".sos-stack.toml");
    if stack_file.exists() {
        return;
    }
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let mut out = format!(
        "# .sos-stack.toml — written by 'sos init security' on {ts}\n# Generic stack metadata. Consumed by advisory-scan (P041) + security-review (P042).\n# Schema version 1. Bump if breaking.\n\nschema_version = 1\ndetected_at = \"{ts}\"\nsos_kit_version = \"P040\"\n\n"
    );
    let mut stacks_found = 0;

    if target.join("package.json").is_file() {
        let (lock, format, parser) = if target.join("pnpm-lock.yaml").is_file() {
            ("pnpm-lock.yaml", "pnpm-v9", "scripts/parsers/pnpm_lock_v9.py")
        } else if target.join("package-lock.json").is_file() {
            ("package-lock.json", "npm-v3", "scripts/parsers/package_lock_v3.py")
        } else {
            ("", "", "")
        };
        out.push_str(&format!(
            "[[stack]]\ntype = \"node\"\nmanifest = \"package.json\"\nlock_file = \"{lock}\"\nlock_format = \"{format}\"\nparser = \"{parser}\"\n\n"
        ));
        stacks_found += 1;
    }
    if target.join("pyproject.toml").is_file() {
        out.push_str("[[stack]]\ntype = \"python\"\nmanifest = \"pyproject.toml\"\nlock_file = \"\"\nlock_format = \"pyproject-toml\"\nparser = \"scripts/parsers/pyproject_toml.py\"\n\n");
        stacks_found += 1;
    }
    if target.join("requirements.txt").is_file() {
        out.push_str("[[stack]]\ntype = \"python\"\nmanifest = \"requirements.txt\"\nlock_file = \"requirements.txt\"\nlock_format = \"requirements-txt\"\nparser = \"scripts/parsers/requirements_txt.py\"\n\n");
        stacks_found += 1;
    }
    if target.join("Cargo.toml").is_file() {
        let lock = if target.join("Cargo.lock").is_file() { "Cargo.lock" } else { "" };
        out.push_str(&format!(
            "[[stack]]\ntype = \"rust\"\nmanifest = \"Cargo.toml\"\nlock_file = \"{lock}\"\nlock_format = \"cargo-lock\"\nparser = \"scripts/parsers/cargo_lock.py\"\n\n"
        ));
        stacks_found += 1;
    }
    if target.join("go.mod").is_file() {
        let lock = if target.join("go.sum").is_file() { "go.sum" } else { "" };
        out.push_str(&format!(
            "[[stack]]\ntype = \"go\"\nmanifest = \"go.mod\"\nlock_file = \"{lock}\"\nlock_format = \"go-sum\"\nparser = \"scripts/parsers/go_sum.py\"\n\n"
        ));
        stacks_found += 1;
    }
    if stacks_found == 0 {
        out.push_str("# No stack manifest detected at project root.\n# Expected one of: package.json / pyproject.toml / requirements.txt / Cargo.toml / go.mod\n# Add [[stack]] entries manually if your project layout is non-standard, or\n# re-run 'sos init security' from a directory containing one of the above.\n");
    }
    let _ = fs::write(&stack_file, out);
}

pub fn run(target: &str, stack: Option<&str>, _pilot: bool, force: bool) -> Result<()> {
    let stack = match stack {
        Some(s) if s == "python" || s == "rust" || s == "ts" => s,
        Some(s) => {
            println!("✗ Unknown stack: {s} (expected python|rust|ts)");
            return Ok(());
        }
        None => {
            println!("✗ --stack required (python|rust|ts)");
            return Ok(());
        }
    };

    let kit_str = std::env::var("SOS_KIT_DIR").unwrap_or_default();
    let kit = PathBuf::from(&kit_str);
    if !kit.join(".claude/agents").is_dir() {
        println!("✗ SOS_KIT_DIR ({kit_str}) is not sos-kit (no .claude/agents). Set SOS_KIT_DIR.");
        return Ok(());
    }

    let target_dir = PathBuf::from(target);
    let non_empty = target_dir.exists()
        && fs::read_dir(&target_dir).map(|mut d| d.next().is_some()).unwrap_or(false);
    if non_empty && !force {
        println!("✗ {target} exists and is non-empty — refusing to bootstrap into it.");
        println!("  Pick an empty/new dir, or pass --force to overwrite into it.");
        println!("  (Adopting an existing repo = future 'sos adopt', not 'sos new'.)");
        return Ok(());
    }

    println!("─────────────────────────────────────────");
    println!("sos new — bootstrap '{target}' (stack: {stack})");
    println!("─────────────────────────────────────────");
    let name = target_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(target)
        .to_string();

    // ---- 1. Category A — FREEZE (copy proven spine verbatim) ----
    println!("[1/4] Category A — freeze (copy from golden)");
    for d in [".claude", "docs/ticket/done", "docs/security", "src", "tests", "hooks"] {
        fs::create_dir_all(target_dir.join(d))?;
    }
    copy_tree(&kit.join(".claude/agents"), &target_dir.join(".claude/agents"), true)?;
    copy_tree(&kit.join(".claude/commands"), &target_dir.join(".claude/commands"), true)?;
    if kit.join(".claude/settings.json").is_file() {
        fs::copy(kit.join(".claude/settings.json"), target_dir.join(".claude/settings.json"))?;
    }
    if kit.join("agents/orchestrator.md").is_file() {
        fs::copy(
            kit.join("agents/orchestrator.md"),
            target_dir.join(".claude/agents/orchestrator.md"),
        )?;
    }
    if kit.join("templates/claude-settings.local.json").is_file() {
        fs::copy(
            kit.join("templates/claude-settings.local.json"),
            target_dir.join(".claude/settings.local.json"),
        )?;
    }
    fs::create_dir_all(target_dir.join(".claude/skills"))?;
    if let Ok(entries) = fs::read_dir(kit.join("skills")) {
        for e in entries.filter_map(|e| e.ok()) {
            if !e.path().is_dir() {
                continue;
            }
            let sk_name = e.file_name();
            if sk_name.to_str() == Some("attic") {
                continue;
            }
            copy_tree(&e.path(), &target_dir.join(".claude/skills").join(&sk_name), true)?;
        }
    }
    copy_tree(&kit.join("scripts"), &target_dir.join("scripts"), false)?;
    copy_tree(&kit.join("phieu"), &target_dir.join("phieu"), false)?;
    copy_tree(&kit.join("templates"), &target_dir.join("templates"), false)?;
    if kit.join("hooks/pre-commit").is_file() {
        fs::copy(kit.join("hooks/pre-commit"), target_dir.join("hooks/pre-commit"))?;
    }
    if kit.join("hooks/pre-push").is_file() {
        fs::copy(kit.join("hooks/pre-push"), target_dir.join("hooks/pre-push"))?;
    }
    chmod_x(&target_dir.join("hooks/pre-commit"));
    chmod_x(&target_dir.join("hooks/pre-push"));
    if kit.join(".gitignore").is_file() {
        fs::copy(kit.join(".gitignore"), target_dir.join(".gitignore"))?;
    }
    fs::write(target_dir.join(".mcp.json"), MCP_JSON)?;
    strip_copy_cruft(&target_dir);
    println!("  ✓ agents(deref) + commands + settings.json + skills(5 living) + scripts + phieu + templates + hooks/pre-commit + .gitignore + .mcp.json(doctor)");

    // ---- 2. Category C — SKELETON (+ # TODO markers) ----
    println!("[2/4] Category C — generate skeletons (+ # TODO)");
    if kit.join("templates/INVARIANTS-template.md").is_file() {
        fs::copy(
            kit.join("templates/INVARIANTS-template.md"),
            target_dir.join("docs/security/INVARIANTS.md"),
        )?;
    }
    {
        let inv_path = target_dir.join("docs/security/INVARIANTS.md");
        let mut existing = fs::read_to_string(&inv_path).unwrap_or_default();
        existing.push_str(INVARIANTS_APPEND);
        fs::write(&inv_path, existing)?;
    }
    fs::write(target_dir.join("docs/AGENT_MAP.yaml"), AGENT_MAP_STUB)?;
    if kit.join("templates/BACKLOG_template.md").is_file() {
        fs::copy(kit.join("templates/BACKLOG_template.md"), target_dir.join("docs/BACKLOG.md"))?;
    }
    let claude_md = format!(
        "# CLAUDE.md — {name}\n\n> Project context for Claude Code. Workflow doctrine: sos-kit (3 roles + Quản đốc orchestrator).\n> **PRD / single source of truth: docs/PROJECT.md** — read it before writing phiếu or code.\n>   (Create it via the /init skill, or drop your own PRD there. CLAUDE.md is a summary; PROJECT.md is canonical.)\n\n## Project context\n\n# TODO: fill stack, role (product/tool/util), and core constraints.\n#       Bootstrap --stack was: {stack} (a placeholder if your real stack isn't python/rust/ts — fix this line).\n\n## Rules\n\nInherit sos-kit role separation: Chủ nhà / Kiến trúc sư / Thợ + Quản đốc.\n"
    );
    fs::write(target_dir.join("CLAUDE.md"), claude_md)?;
    match stack {
        "python" => {
            let p = target_dir.join("pyproject.toml");
            if !p.is_file() {
                fs::write(&p, format!("[project]\nname = \"{name}\"\nversion = \"0.1.0\"\n# TODO: fill dependencies\n"))?;
            }
        }
        "rust" => {
            let p = target_dir.join("Cargo.toml");
            if !p.is_file() {
                fs::write(&p, format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n# TODO: fill dependencies\n"))?;
            }
            let main_rs = target_dir.join("src/main.rs");
            if !main_rs.is_file() {
                fs::write(&main_rs, "fn main() {}\n")?;
            }
        }
        "ts" => {
            let p = target_dir.join("package.json");
            if !p.is_file() {
                fs::write(&p, format!("{{\n  \"name\": \"{name}\",\n  \"version\": \"0.0.0\"\n}}\n"))?;
            }
        }
        _ => unreachable!("stack already validated"),
    }
    let architecture_md = format!(
        "# Architecture — {name}\n\n> # TODO: fill. docs-gate [architecture] points here (.docs-gate.toml).\n\n## Overview\n\n# TODO: what this repo is, one paragraph.\n\n## Components\n\n# TODO: main modules / surfaces.\n\n## Data flow\n\n# TODO: how data moves through the system.\n"
    );
    fs::write(target_dir.join("docs/ARCHITECTURE.md"), architecture_md)?;
    let changelog_path = target_dir.join("CHANGELOG.md");
    if !changelog_path.is_file() {
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        fs::write(
            &changelog_path,
            format!("# Changelog\n\nFormat loosely follows Keep a Changelog.\n\n## v0.0.0 — bootstrap via `sos new` — {date}\n\n- Repo bootstrapped from sos-kit golden.\n"),
        )?;
    }
    println!("  ✓ INVARIANTS.md + AGENT_MAP.yaml + BACKLOG.md + CLAUDE.md + ARCHITECTURE.md + CHANGELOG.md + stack manifest");

    // ---- 3. Category B — DEFAULTS (tunable) ----
    println!("[3/4] Category B — defaults");
    let docs_gate_path = target_dir.join(".docs-gate.toml");
    if !docs_gate_path.is_file() {
        fs::write(&docs_gate_path, DOCS_GATE_TOML)?;
    }
    write_sos_stack_toml(&target_dir);
    println!("  ✓ .docs-gate.toml + .sos-stack.toml");

    // ---- 4. VALIDATOR — composite (wiring + content-to-fill) ----
    println!("[4/4] Validator");
    let doctor_bin = std::env::var("DOCTOR_BIN").unwrap_or_else(|_| "doctor".to_string());
    if doctor_available(&doctor_bin) {
        let out = Command::new(&doctor_bin)
            .arg("verify-setup")
            .arg("--repo")
            .arg(&target_dir)
            .output();
        match out {
            Ok(o) => {
                let mut combined = String::from_utf8_lossy(&o.stdout).into_owned();
                combined.push_str(&String::from_utf8_lossy(&o.stderr));
                for line in combined.lines() {
                    println!("    {line}");
                }
                if o.status.success() {
                    println!("  ✓ verify-setup: CONNECTED");
                } else {
                    println!("  ⚠ verify-setup: rc={} — wiring gap above", o.status.code().unwrap_or(-1));
                }
            }
            Err(_) => {
                println!("  ⚠ verify-setup: failed to execute {doctor_bin}");
            }
        }
    } else {
        println!("  ⏭ doctor not found — skip verify-setup (install: cargo install --path ~/doctor, or set DOCTOR_BIN=/path/to/doctor)");
    }
    println!("  Category C placeholders to fill (# TODO):");
    {
        let mut todo_files: Vec<String> = Vec::new();
        let mut scan_roots: Vec<PathBuf> = vec![target_dir.join("docs")];
        let claude_md_path = target_dir.join("CLAUDE.md");
        for root in &scan_roots.clone() {
            if !root.is_dir() {
                continue;
            }
            for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
                if !entry.file_type().is_file() {
                    continue;
                }
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if content.contains("# TODO") {
                        if let Ok(rel) = entry.path().strip_prefix(&target_dir) {
                            todo_files.push(rel.to_string_lossy().replace('\\', "/"));
                        }
                    }
                }
            }
        }
        scan_roots.clear();
        if claude_md_path.is_file() {
            if let Ok(content) = fs::read_to_string(&claude_md_path) {
                if content.contains("# TODO") {
                    todo_files.push("CLAUDE.md".to_string());
                }
            }
        }
        // Deliberate divergence from Bash's raw (filesystem-enumeration-order,
        // unsorted) `grep -rl` — sort so the canonical output matches the
        // frozen (normalized-sorted) golden directly. See module doc comment.
        todo_files.sort();
        for f in &todo_files {
            println!("    - {f}");
        }
    }

    // ---- 5. Git + arm hooks ----
    println!("[+] Git init + arm hooks");
    let git_dir = target_dir.join(".git");
    if !git_dir.is_dir() {
        let init = Command::new("git").arg("init").arg("-q").current_dir(&target_dir).status();
        let _ = init;
        let _ = Command::new("git")
            .arg("symbolic-ref")
            .arg("HEAD")
            .arg("refs/heads/main")
            .current_dir(&target_dir)
            .status();
        run_install_hooks(&target_dir);
        println!("  ✓ git repo (branch main) + pre-commit/pre-push hooks armed");
    } else {
        run_install_hooks(&target_dir);
        println!("  ✓ pre-existing git repo — pre-commit/pre-push hooks armed");
    }

    println!();
    println!("✓ sos new done: {target}");
    println!("  Next: fill # TODO (AGENT_MAP surfaces, INV-LOCAL, CLAUDE.md context) → git add -A && git commit.");

    Ok(())
}

fn run_install_hooks(target_dir: &Path) {
    if let Ok(out) = Command::new("bash")
        .arg("scripts/install-hooks.sh")
        .current_dir(target_dir)
        .output()
    {
        let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
        combined.push_str(&String::from_utf8_lossy(&out.stderr));
        for line in combined.lines() {
            println!("    {line}");
        }
    }
}
