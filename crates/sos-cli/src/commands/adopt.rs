// commands/adopt.rs — Rust port of `bin/sos.sh` sos_adopt (bin/sos.sh:606-971).
//
// P077c4: bug-for-bug parity with the Bash oracle. Brownfield retrofit — the
// OPPOSITE discipline of `new` (bin/sos.sh:355-604, ported in P077c3): `adopt`
// RESPECTS what's already in the target repo.
//   ADDITIVE     — copy spine items only where the destination is ABSENT.
//   NON-CLOBBER  — destination exists -> stage sos-kit's version to
//                  .sos-adopt-incoming/<path> + report for manual merge.
//                  NEVER overwrite the repo's own files.
//   REPORT       — verify-setup + validate-map verdict for triage.
//
// Bug-for-bug divergences kept ON PURPOSE (P077c5 is the fix phiếu, not this one):
//   - OA-02: [2/4] calls into `map`'s logic (via subprocess of this same binary,
//     see `run_map_subcommand` below) AFTER [1/4] has already copied kit assets
//     into the target -> AGENT_MAP.yaml maps those kit assets too. NOT excluded.
//   - Non-clobber `added`/`conflicts` list order = raw filesystem/WalkDir
//     traversal order (NOT sorted) -- same class of divergence as `sync.rs`
//     (see that module's doc comment); Bash's `find` for these loops has no
//     `| sort` either (bin/sos.sh:661/679/708/717).
//
// Four-layer parity fixture (tests/parity.rs `parity_adopt_enforced`):
//   adopt.golden       — stdout report (captured with DOCTOR_BIN=/nonexistent/doctor,
//                         RE-FROZE from a stale CONNECTED-path capture — see
//                         tests/README.md "sos adopt" section).
//   adopt.tree.golden  — path-shape manifest (sorted, no content, excludes `.git/`,
//                         INCLUDES `.sos-adopt-incoming/**`).
//   adopt.gen.golden   — content-hash manifest, GENERATED-authored files ONLY
//                         (copied/staged kit assets are tree-only, never hashed).
//   preservation-assert (in-test, NOT a golden file) — universal property both
//     Bash and Rust must hold: every seeded pre-existing file's sha256 is
//     IDENTICAL before/after adopt runs, and every `.sos-adopt-incoming/<path>`
//     byte-matches the kit source it was staged from.
//
// `.mcp.json` / `.claude/settings.local.json` EXISTS-branch (jq-merge): shells
// out to the real `jq` binary (bug-for-bug with Bash — no new JSON-merge crate
// dependency added). NOT exercised by the parity fixture (Constraint 7 — the
// fixture always hits the create-if-absent branch); implemented for real-world
// correctness but out of hard-fail parity scope.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

const MCP_JSON_HEREDOC: &str = r#"{
  "mcpServers": {
    "doctor": {
      "_comment": "sos-kit mechanical gate (lane/map/rotate/runtime + verify-setup). PATH-rel.",
      "command": "doctor",
      "args": ["serve"]
    },
    "docs-gate": {
      "_comment": "Docs compliance — same engine the pre-commit hook runs as CLI. PATH-rel.",
      "command": "docs-gate",
      "args": ["serve"]
    },
    "ship": {
      "_comment": "Release workflow — test/commit/PR/canary. PATH-rel.",
      "command": "ship",
      "args": ["serve"]
    },
    "guard": {
      "_comment": "Pre-deploy infra gate — inert until repo has a deploy/SSH target.",
      "command": "guard",
      "args": ["serve"]
    },
    "vps": {
      "_comment": "VPS ops — inert until ~/.vps.toml names a host.",
      "command": "vps",
      "args": ["serve"]
    }
  }
}
"#;

const MCP_JQ_FILTER: &str = r#"
        .mcpServers.doctor      //= {"_comment":"sos-kit mechanical gate (lane/map/rotate/runtime + verify-setup). PATH-rel.","command":"doctor","args":["serve"]} |
        .mcpServers."docs-gate" //= {"_comment":"Docs compliance — same engine the pre-commit hook runs as CLI. PATH-rel.","command":"docs-gate","args":["serve"]} |
        .mcpServers.ship        //= {"_comment":"Release workflow — test/commit/PR/canary. PATH-rel.","command":"ship","args":["serve"]} |
        .mcpServers.guard       //= {"_comment":"Pre-deploy infra gate — inert until repo has a deploy/SSH target.","command":"guard","args":["serve"]} |
        .mcpServers.vps         //= {"_comment":"VPS ops — inert until ~/.vps.toml names a host.","command":"vps","args":["serve"]}"#;

const MARKER_PERMS_JSON: &str = r#"["Bash(mkdir -p .sos-state)","Bash(touch .sos-state/architect-active)","Bash(rm -f .sos-state/architect-active)","Bash(touch .sos-state/worker-active)","Bash(rm -f .sos-state/worker-active)"]"#;

const DOCS_GATE_TOML: &str = r#"docs_dir = "docs"
changelog = "../CHANGELOG.md"
changelog_max_age_days = 1
changelog_staged = false
rules = []
staleness = []
doc_structure = []
count_check = []
cross_doc = []

[architecture]
enabled = true
file = "ARCHITECTURE.md"
required_sections = 0
required_non_empty = []

[ticket]
ticket_dir = "docs/ticket"
type_pattern = ""
valid_types = []
exclude_files = []
"#;

const GITIGNORE_PATTERNS: &[&str] = &[
    ".sos-state/",
    ".backup/",
    ".sos-adopt-incoming/",
    ".claude/settings.local.json",
    "target/",
    "__pycache__/",
    "*.pyc",
    "*.egg-info/",
    "node_modules/",
    "dist/",
];

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

fn jq_available() -> bool {
    Command::new("jq").arg("--version").output().is_ok()
}

/// Path-substring noise filter, same rules `adopt_item`'s dir-walk and the
/// skills-remap loop both apply (bin/sos.sh: `-not -path '*/__pycache__/*'
/// -not -name '*.pyc' -not -name '.DS_Store'`).
fn is_noise(path: &Path) -> bool {
    // P088 Task 0 anchor #3 correction (Discovery Report): component-based
    // check, NOT string-substring on `/` — `to_string_lossy()` uses the
    // platform's native separator (`\` on Windows), so a `"/__pycache__/"`
    // substring match silently never fires on Windows.
    if path.components().any(|c| c.as_os_str() == "__pycache__") {
        return true;
    }
    if path.extension().and_then(|e| e.to_str()) == Some("pyc") {
        return true;
    }
    if path.file_name().and_then(|n| n.to_str()) == Some(".DS_Store") {
        return true;
    }
    false
}

/// Core non-clobber primitive, mirrors `adopt_item()` (bin/sos.sh:655-690).
///
/// `src_rel` is relative to the kit root. `dest_rel_override` (used only for
/// the plain-FILE case, matching Bash's `rel="${2:-$1}"` — for directories
/// Bash's `frel` is always computed relative to `$K`, ignoring the 2nd arg)
/// lets the caller remap a single file's destination (e.g.
/// `agents/orchestrator.md` -> `.claude/agents/orchestrator.md`).
///
/// Traversal order for the directory case is RAW WalkDir enumeration order
/// (NOT sorted) — bug-for-bug with Bash's unsorted `find -L` (bin/sos.sh:679).
fn adopt_item(
    kit: &Path,
    target: &Path,
    incoming: &Path,
    src_rel: &str,
    dest_rel_override: Option<&str>,
    added: &mut Vec<String>,
    conflicts: &mut Vec<String>,
    added_paths: &mut Vec<PathBuf>,
) {
    let src = kit.join(src_rel);
    if !src.exists() {
        return;
    }
    if src.is_dir() {
        let kreal = fs::canonicalize(kit).unwrap_or_else(|_| kit.to_path_buf());
        // L1 (order determinism, P086): sort entries before processing so
        // `added`/`conflicts` order is filesystem-independent — Linux
        // dogfood readdir order differed from the macOS-captured golden.
        let mut entries: Vec<_> = WalkDir::new(&src)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| !e.file_type().is_dir())
            .collect();
        entries.sort_by(|a, b| a.path().cmp(b.path()));
        for entry in entries {
            let fpath = entry.path();
            if is_noise(fpath) {
                continue;
            }
            let frel = match fpath.strip_prefix(kit) {
                Ok(r) => r.to_string_lossy().replace('\\', "/"),
                Err(_) => continue,
            };
            // Symlink-escape guard (Giám sát advisory on the `find -L` change,
            // bin/sos.sh:662-667): the deref'd target must resolve INSIDE the
            // kit tree.
            let real = fs::canonicalize(fpath).unwrap_or_else(|_| fpath.to_path_buf());
            if !real.starts_with(&kreal) {
                println!("  ⚠ skip (symlink escapes kit tree): {frel}");
                continue;
            }
            let fdest = target.join(&frel);
            if fdest.exists() {
                let idest = incoming.join(&frel);
                if let Some(parent) = idest.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::copy(fpath, &idest);
                conflicts.push(format!("    ~ {frel}"));
            } else {
                if let Some(parent) = fdest.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::copy(fpath, &fdest);
                added.push(format!("    + {frel}"));
                added_paths.push(fdest);
            }
        }
    } else {
        let rel = dest_rel_override.unwrap_or(src_rel);
        let dest = target.join(rel);
        if dest.exists() {
            let idest = incoming.join(rel);
            if let Some(parent) = idest.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::copy(&src, &idest);
            conflicts.push(format!("    ~ {rel}"));
        } else {
            if let Some(parent) = dest.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::copy(&src, &dest);
            added.push(format!("    + {rel}"));
            added_paths.push(dest);
        }
    }
}

/// Skills remap loop (bin/sos.sh:706-718): `skills/<rel>` -> `.claude/skills/<rel>`,
/// skipping `attic/`. Plain `find` (no `-L` in Bash — skills/ has no symlinks
/// today), so `follow_links` stays default (false). NOT sorted (raw order).
fn adopt_skills(
    kit: &Path,
    target: &Path,
    incoming: &Path,
    added: &mut Vec<String>,
    conflicts: &mut Vec<String>,
    added_paths: &mut Vec<PathBuf>,
) {
    let skills_dir = kit.join("skills");
    if !skills_dir.is_dir() {
        return;
    }
    // L1 (order determinism, P086): sort before processing (see adopt_item).
    let mut entries: Vec<_> = WalkDir::new(&skills_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
        .collect();
    entries.sort_by(|a, b| a.path().cmp(b.path()));
    for entry in entries {
        let path = entry.path();
        // P088 Task 0 anchor #3 correction: component-based check (see
        // `is_noise` comment above) — `"/attic/"` substring on
        // `to_string_lossy()` never matched on Windows (native `\` separator),
        // so `skills/attic/**` leaked into `.claude/skills/attic/**` on adopt.
        let in_attic = path.components().any(|c| c.as_os_str() == "attic");
        if in_attic || is_noise(path) {
            continue;
        }
        let rel_in_skills = match path.strip_prefix(&skills_dir) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        let srel = format!(".claude/skills/{rel_in_skills}");
        let sdest = target.join(&srel);
        if sdest.exists() {
            let idest = incoming.join(&srel);
            if let Some(parent) = idest.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::copy(path, &idest);
            conflicts.push(format!("    ~ {srel}"));
        } else {
            if let Some(parent) = sdest.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::copy(path, &sdest);
            added.push(format!("    + {srel}"));
            added_paths.push(sdest);
        }
    }
}

/// `.mcp.json` create-if-absent / jq-merge-if-exists (bin/sos.sh:720-751).
/// The jq-merge branch is NOT exercised by the parity fixture (Constraint 7 —
/// fixture always takes create-if-absent) but is implemented for real usage.
fn wire_mcp_json(target: &Path, added: &mut Vec<String>, conflicts: &mut Vec<String>, added_paths: &mut Vec<PathBuf>) {
    let dest = target.join(".mcp.json");
    if !dest.is_file() {
        let _ = fs::write(&dest, MCP_JSON_HEREDOC);
        added.push("    + .mcp.json (kit tools wired: doctor/docs-gate/ship/guard/vps — PATH-rel, same set as sos new)".to_string());
        added_paths.push(dest);
        return;
    }
    if jq_available() {
        let out = Command::new("jq").arg(MCP_JQ_FILTER).arg(&dest).output();
        match out {
            Ok(o) if o.status.success() => {
                let existing = fs::read(&dest).unwrap_or_default();
                if o.stdout == existing {
                    conflicts.push("    ~ .mcp.json (kit tool entries already present — kept)".to_string());
                } else {
                    let _ = fs::write(&dest, &o.stdout);
                    added.push("    + .mcp.json (kit tool entries jq-merged: doctor/docs-gate/ship/guard/vps — repo's own MCP servers kept)".to_string());
                    added_paths.push(dest.clone());
                }
            }
            _ => {
                conflicts.push("    ~ .mcp.json (exists, jq-merge failed — add the kit server entries by hand; see sos new .mcp.json)".to_string());
            }
        }
    } else {
        conflicts.push("    ~ .mcp.json (exists — add the kit server entries by hand; repo's own MCP kept. Tip: install jq for auto-merge)".to_string());
    }
}

/// `.claude/settings.local.json` copy-if-absent / jq-merge-if-exists
/// (bin/sos.sh:782-803). Same out-of-fixture-scope note as `wire_mcp_json`.
fn wire_settings_local(
    kit: &Path,
    target: &Path,
    added: &mut Vec<String>,
    conflicts: &mut Vec<String>,
    _added_paths: &mut Vec<PathBuf>,
) {
    let slj = target.join(".claude/settings.local.json");
    if !slj.is_file() {
        let _ = fs::create_dir_all(target.join(".claude"));
        let src = kit.join("templates/claude-settings.local.json");
        if src.is_file() {
            let _ = fs::copy(&src, &slj);
        }
        added.push("    + .claude/settings.local.json (5 marker perms — per-user, don't commit)".to_string());
        // NOT pushed to added_paths — gitignored (per-user, "don't commit"),
        // `git add` on it would warn "ignored" for no benefit.
        return;
    }
    if !jq_available() {
        conflicts.push("    ~ .claude/settings.local.json (exists — add 5 marker perms by hand, INSTALL.md §2.5. Tip: install jq for auto-merge)".to_string());
        return;
    }
    let contains = Command::new("jq")
        .arg("-e")
        .arg("--argjson")
        .arg("p")
        .arg(MARKER_PERMS_JSON)
        .arg("(.permissions.allow // []) | contains($p)")
        .arg(&slj)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if contains {
        conflicts.push("    ~ .claude/settings.local.json (5 marker perms already present — kept)".to_string());
        return;
    }
    let merge = Command::new("jq")
        .arg("--argjson")
        .arg("p")
        .arg(MARKER_PERMS_JSON)
        .arg(".permissions.allow = ((.permissions.allow // []) + $p | unique)")
        .arg(&slj)
        .output();
    match merge {
        Ok(o) if o.status.success() => {
            let _ = fs::write(&slj, &o.stdout);
            added.push("    + .claude/settings.local.json (5 marker perms jq-merged — existing perms kept)".to_string());
        }
        _ => {
            conflicts.push("    ~ .claude/settings.local.json (exists, jq-merge failed — add 5 marker perms by hand, INSTALL.md §2.5)".to_string());
        }
    }
}

/// Calls into `map`'s scan logic by re-invoking this SAME compiled binary as a
/// subprocess (`sos map <target>`), output fully discarded — matches Bash's
/// `sos_map "$target" >/dev/null` (bin/sos.sh:814). This reuses `map.rs`'s
/// logic WITHOUT modifying map.rs (constraint: map.rs is untouched, shared
/// with c1/c5), while still suppressing map's own stdout the way Bash's
/// redirect does. `[1/4]` has already copied kit assets into `target` by the
/// time this runs, but from P077c5 onward `map.rs`'s KIT_MANAGED_ROOTS
/// exclude-list (Part 2) skips those same roots regardless of caller — this
/// function needed ZERO code change to inherit the OA-02 fix, since it just
/// re-invokes the same (now-fixed) `sos map` binary.
fn run_map_subcommand(target: &Path) {
    if let Ok(exe) = std::env::current_exe() {
        let _ = Command::new(exe).arg("map").arg(target).output();
    }
}

/// `sos_init_security` (bin/sos.sh:121-...), ported with its return-code
/// semantics preserved (adopt's [3/4] branches on this): 0 = stack(s)
/// detected, 1 = `.sos-stack.toml` already exists (no-op), 2 = no stack
/// manifest found (file still written, with a hint comment).
fn init_security(target: &Path) -> u8 {
    let stack_file = target.join(".sos-stack.toml");
    if stack_file.is_file() {
        return 1;
    }
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let mut out = format!(
        "# .sos-stack.toml — written by 'sos init security' on {ts}\n# Generic stack metadata. Consumed by advisory-scan (P041) + security-review (P042).\n# Schema version 1. Bump if breaking.\n\nschema_version = 1\ndetected_at = \"{ts}\"\nsos_kit_version = \"{}\"\n\n",
        env!("CARGO_PKG_VERSION")
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
    if stacks_found > 0 {
        0
    } else {
        2
    }
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

pub fn run(target: &str, _stack: Option<&str>) -> Result<()> {
    // _stack: parsed, bug-for-bug unused (bin/sos.sh sets $stack but never
    // reads it again in sos_adopt — same class of dead flag as `new --pilot`).
    let target_dir = PathBuf::from(target);
    if !target_dir.is_dir() {
        println!("✗ {target} does not exist — for a NEW repo use: sos new {target} --stack <X>");
        return Ok(());
    }
    let is_empty = fs::read_dir(&target_dir).map(|mut d| d.next().is_none()).unwrap_or(true);
    if is_empty {
        println!("✗ {target} is empty (greenfield) — use: sos new {target} --stack <X>");
        return Ok(());
    }
    let kit_str = std::env::var("SOS_KIT_DIR").unwrap_or_default();
    let kit = PathBuf::from(&kit_str);
    if !kit.join(".claude/agents").is_dir() {
        println!("✗ SOS_KIT_DIR ({kit_str}) is not sos-kit (no .claude/agents). Set SOS_KIT_DIR.");
        return Ok(());
    }
    // P088 ITEM 2 (c) — same stub-check as `sos new` (crate::commands::new).
    // adopt_item(".claude/agents"/".claude/commands") follows symlinks
    // (adopt.rs:199-200) from the SOS_KIT_DIR source, same propagation risk.
    // adopt_skills() itself sources from canonical `skills/` (no symlinks
    // there — O1.1), so `.claude/skills/*` stubs in the kit checkout are NOT
    // a risk via this path, but the shared helper still scans that subdir for
    // completeness/reuse.
    crate::commands::new::warn_symlink_stubs(&kit, "SOS_KIT_DIR (kit source)");

    println!("─────────────────────────────────────────");
    println!("sos adopt — retrofit spine into existing '{target}' (ADDITIVE + NON-CLOBBER)");
    println!("─────────────────────────────────────────");

    // Dirty-warn (bin/sos.sh:642-646).
    if target_dir.join(".git").is_dir() {
        let status = Command::new("git")
            .arg("-C")
            .arg(&target_dir)
            .arg("status")
            .arg("--porcelain")
            .output();
        if let Ok(out) = status {
            if out.status.success() && !out.stdout.is_empty() {
                println!("⚠ target has uncommitted changes — adopt only ADDS files (never edits yours),");
                println!("  but git status will mix adopt output with your WIP. Commit/stash first to keep");
                println!("  the adopt reviewable as its own commit. Proceeding.");
            }
        }
    }

    let incoming = target_dir.join(".sos-adopt-incoming");
    let mut added: Vec<String> = Vec::new();
    let mut conflicts: Vec<String> = Vec::new();
    // P086 Task 3 (O1.1 phương án A): parallel path accumulator — `added` is a
    // pre-formatted stdout display list (mixes clean paths with trailing
    // description text, some lines aren't paths at all), so it can't be used
    // verbatim as `git add` args. Every call site that writes a REAL file into
    // `target_dir` also pushes that path here.
    let mut added_paths: Vec<PathBuf> = Vec::new();

    println!("[1/4] Spine (copy-if-absent; existing → staged to .sos-adopt-incoming/)");
    adopt_item(&kit, &target_dir, &incoming, ".claude/agents", None, &mut added, &mut conflicts, &mut added_paths);
    adopt_item(&kit, &target_dir, &incoming, ".claude/commands", None, &mut added, &mut conflicts, &mut added_paths);
    adopt_item(&kit, &target_dir, &incoming, ".claude/settings.json", None, &mut added, &mut conflicts, &mut added_paths);
    if kit.join("agents/orchestrator.md").is_file() {
        adopt_item(
            &kit,
            &target_dir,
            &incoming,
            "agents/orchestrator.md",
            Some(".claude/agents/orchestrator.md"),
            &mut added,
            &mut conflicts,
            &mut added_paths,
        );
    }
    adopt_item(&kit, &target_dir, &incoming, "scripts", None, &mut added, &mut conflicts, &mut added_paths);
    adopt_item(&kit, &target_dir, &incoming, "phieu", None, &mut added, &mut conflicts, &mut added_paths);
    adopt_item(&kit, &target_dir, &incoming, "templates", None, &mut added, &mut conflicts, &mut added_paths);
    adopt_item(&kit, &target_dir, &incoming, "hooks/pre-commit", None, &mut added, &mut conflicts, &mut added_paths);
    adopt_item(&kit, &target_dir, &incoming, "hooks/pre-push", None, &mut added, &mut conflicts, &mut added_paths);
    adopt_item(&kit, &target_dir, &incoming, "docs/ORCHESTRATION.md", None, &mut added, &mut conflicts, &mut added_paths);
    chmod_x(&target_dir.join("hooks/pre-commit"));
    chmod_x(&target_dir.join("hooks/pre-push"));

    adopt_skills(&kit, &target_dir, &incoming, &mut added, &mut conflicts, &mut added_paths);
    wire_mcp_json(&target_dir, &mut added, &mut conflicts, &mut added_paths);
    wire_settings_local(&kit, &target_dir, &mut added, &mut conflicts, &mut added_paths);
    println!("  done");

    println!("[2/4] Category C — generate ONLY if missing (never overwrite existing docs)");
    let _ = fs::create_dir_all(target_dir.join("docs/security"));
    let _ = fs::create_dir_all(target_dir.join("docs/ticket/done"));

    let inv_path = target_dir.join("docs/security/INVARIANTS.md");
    if !inv_path.is_file() {
        if kit.join("templates/INVARIANTS-template.md").is_file() {
            let _ = fs::copy(kit.join("templates/INVARIANTS-template.md"), &inv_path);
        }
        let mut existing = fs::read_to_string(&inv_path).unwrap_or_default();
        existing.push_str("\n## INV-LOCAL (project-specific — FILL THESE)\n\n# TODO: add `## INV-LOCAL-NNN — <title>`, or `# No local INV`.\n");
        let _ = fs::write(&inv_path, existing);
        added.push("    + docs/security/INVARIANTS.md".to_string());
        added_paths.push(inv_path.clone());
    } else {
        conflicts.push("    ~ docs/security/INVARIANTS.md (exists — kept)".to_string());
    }

    let map_path = target_dir.join("docs/AGENT_MAP.yaml");
    if !map_path.is_file() {
        run_map_subcommand(&target_dir);
        added.push("    + docs/AGENT_MAP.yaml (scanned real surfaces — set load_bearing + blast by hand)".to_string());
        added_paths.push(map_path.clone());
    } else {
        conflicts.push("    ~ docs/AGENT_MAP.yaml (exists — kept)".to_string());
    }

    let docs_gate_path = target_dir.join(".docs-gate.toml");
    if !docs_gate_path.is_file() {
        let _ = fs::write(&docs_gate_path, DOCS_GATE_TOML);
        added.push("    + .docs-gate.toml".to_string());
        added_paths.push(docs_gate_path.clone());
    } else {
        conflicts.push("    ~ .docs-gate.toml (exists — kept; verify ticket_dir + architecture)".to_string());
    }

    let name = fs::canonicalize(&target_dir)
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| target.to_string());

    let changelog_root = target_dir.join("CHANGELOG.md");
    let changelog_docs = target_dir.join("docs/CHANGELOG.md");
    if changelog_root.is_file() {
        conflicts.push("    ~ CHANGELOG.md (exists — kept)".to_string());
    } else if changelog_docs.is_file() {
        conflicts.push("    ~ docs/CHANGELOG.md (exists — repo's own location, kept; no root skeleton)".to_string());
    } else {
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let _ = fs::write(
            &changelog_root,
            format!("# Changelog\n\nFormat loosely follows Keep a Changelog.\n\n## v0.0.0 — sos adopt — {date}\n\n- Spine retrofitted via `sos adopt`.\n"),
        );
        added.push("    + CHANGELOG.md (skeleton)".to_string());
        added_paths.push(changelog_root.clone());
    }

    let arch_path = target_dir.join("docs/ARCHITECTURE.md");
    if arch_path.is_file() {
        conflicts.push("    ~ docs/ARCHITECTURE.md (exists — kept)".to_string());
    } else {
        let architecture_md = format!(
            "# Architecture — {name}\n\n> # TODO: fill. docs-gate [architecture] points here.\n\n## Overview\n\n# TODO: what this repo is, one paragraph.\n\n## Components\n\n# TODO: main modules / surfaces.\n\n## Data flow\n\n# TODO: how data moves through the system.\n"
        );
        let _ = fs::write(&arch_path, architecture_md);
        added.push("    + docs/ARCHITECTURE.md (skeleton)".to_string());
        added_paths.push(arch_path.clone());
    }

    let backlog_path = target_dir.join("docs/BACKLOG.md");
    if backlog_path.is_file() {
        conflicts.push("    ~ docs/BACKLOG.md (exists — kept)".to_string());
    } else if kit.join("templates/BACKLOG_template.md").is_file() {
        let _ = fs::copy(kit.join("templates/BACKLOG_template.md"), &backlog_path);
        added.push("    + docs/BACKLOG.md (template — fill Active sprint)".to_string());
        added_paths.push(backlog_path.clone());
    }

    if target_dir.join("CLAUDE.md").is_file() {
        conflicts.push("    ~ CLAUDE.md (exists — kept)".to_string());
    } else {
        conflicts.push("    ! CLAUDE.md (MISSING — add a project CLAUDE.md by hand; adopt won't guess product identity)".to_string());
    }

    // .gitignore — append-if-missing per line (bin/sos.sh:888-892).
    let gi_path = target_dir.join(".gitignore");
    let mut gi_content = fs::read_to_string(&gi_path).unwrap_or_default();
    let mut gi_added = 0;
    for pat in GITIGNORE_PATTERNS {
        let already_present = gi_content.lines().any(|l| l == *pat);
        if !already_present {
            if !gi_content.is_empty() && !gi_content.ends_with('\n') {
                gi_content.push('\n');
            }
            gi_content.push_str(pat);
            gi_content.push('\n');
            gi_added += 1;
        }
    }
    let _ = fs::write(&gi_path, gi_content);
    if gi_added > 0 {
        added.push(format!("    + .gitignore ({gi_added} build/runtime ignore lines)"));
        added_paths.push(gi_path.clone());
    }

    // .phieu-counter (bin/sos.sh:897-900).
    let counter_path = target_dir.join(".phieu-counter");
    if !counter_path.is_file() {
        let _ = fs::write(&counter_path, "0\n");
        added.push("    + .phieu-counter (seed 0 — source phieu/phieu.sh to use `phieu <slug>`)".to_string());
        added_paths.push(counter_path.clone());
    }

    // ---- [3/4] Born-wire ----
    println!("[3/4] Born-wire (arm hooks + stack detect)");
    let mut hooks_armed = false;
    let install_hooks = target_dir.join("scripts/install-hooks.sh");
    if target_dir.join(".git").is_dir() && install_hooks.is_file() {
        let out = Command::new("bash")
            .arg("scripts/install-hooks.sh")
            .current_dir(&target_dir)
            .output();
        match out {
            Ok(o) => {
                let mut combined = String::from_utf8_lossy(&o.stdout).into_owned();
                combined.push_str(&String::from_utf8_lossy(&o.stderr));
                for line in combined.lines() {
                    println!("    {line}");
                }
                if o.status.success() {
                    hooks_armed = true;
                    added.push("    + hooks ARMED (core.hooksPath → hooks/; pre-existing .git/hooks moved to .bak if any)".to_string());
                } else {
                    conflicts.push("    ~ hooks NOT armed (F09 guard declined — existing hook setup detected; review then run: bash scripts/install-hooks.sh)".to_string());
                }
            }
            Err(_) => {
                conflicts.push("    ~ hooks NOT armed (F09 guard declined — existing hook setup detected; review then run: bash scripts/install-hooks.sh)".to_string());
            }
        }
    } else {
        conflicts.push("    ~ hooks NOT armed (no .git or scripts/install-hooks.sh missing)".to_string());
    }

    match init_security(&target_dir) {
        0 => {
            added.push("    + .sos-stack.toml (stack detected — advisory-scan/security-review ready)".to_string());
            added_paths.push(target_dir.join(".sos-stack.toml"));
        }
        1 => conflicts.push("    ~ .sos-stack.toml (already exists — kept)".to_string()),
        // rc=2: no manifest detected, but init_security still WRITES the file
        // (hint comment) — a real file, so still stage it for the seed step.
        _ => {
            conflicts.push("    ~ .sos-stack.toml (no stack manifest detected — see file comments, add [[stack]] by hand)".to_string());
            added_paths.push(target_dir.join(".sos-stack.toml"));
        }
    }

    // P086 Task 3 — seed trust baseline, ONLY when arm actually succeeded
    // (repo not armed = don't touch staging at all). Runs AFTER `.sos-stack.toml`
    // detection above so that file (written by init_security, which runs after
    // the arm-hooks leg) is included in `added_paths` before we stage. Order
    // load-bearing (same as `new`): stage → rebaseline → stage baseline.
    // Scoped to `added_paths` (Luật chơi #3 — NOT `git add -A`, this is a
    // brownfield repo that may have the user's own work-in-flight).
    if hooks_armed {
        for p in &added_paths {
            let _ = Command::new("git")
                .arg("-c")
                .arg("core.autocrlf=false")
                .arg("add")
                .arg(p)
                .current_dir(&target_dir)
                .status();
        }
        let rebaseline = Command::new("bash")
            .arg("scripts/trust-gate.sh")
            .arg("rebaseline")
            .current_dir(&target_dir)
            .output();
        match rebaseline {
            Ok(ro) if ro.status.success() => {
                let _ = Command::new("git")
                    .arg("-c")
                    .arg("core.autocrlf=false")
                    .arg("add")
                    .arg(".sos-trust-baseline")
                    .current_dir(&target_dir)
                    .status();
                added.push("    + .sos-trust-baseline (seeded — first commit sẽ qua [8/8])".to_string());
            }
            _ => {
                conflicts.push("    ~ .sos-trust-baseline NOT seeded (trust-gate.sh missing/failed) — run: bash scripts/trust-gate.sh rebaseline && git add .sos-trust-baseline".to_string());
            }
        }
    }

    // ---- [4/4] Validator ----
    println!("[4/4] Validator");
    let doctor_bin = std::env::var("DOCTOR_BIN").unwrap_or_else(|_| "doctor".to_string());
    if doctor_available(&doctor_bin) {
        let out = Command::new(&doctor_bin).arg("verify-setup").arg("--repo").arg(&target_dir).output();
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
                    println!("  ⚠ verify-setup DORMANT (rc={}) — triage EACH joint:", o.status.code().unwrap_or(-1));
                    println!("      (A) wire genuinely not connected → fix the repo");
                    println!("      (B) verify-setup doesn't know THIS repo's pattern → teach it / label brittle (NOT a repo bug)");
                    println!("      (C) product gap → you decide the content");
                }
            }
            Err(_) => {
                println!("  ⚠ verify-setup: failed to execute {doctor_bin}");
            }
        }
        if map_target_has_agent_map(&target_dir) {
            let vm = Command::new(&doctor_bin)
                .arg("validate-map")
                .arg("--map")
                .arg("docs/AGENT_MAP.yaml")
                .current_dir(&target_dir)
                .output();
            let ok = vm.map(|o| o.status.success()).unwrap_or(false);
            if ok {
                println!("  ✓ validate-map: AGENT_MAP paths resolve");
            } else {
                println!("  ⚠ validate-map: map drift — run: doctor validate-map --map docs/AGENT_MAP.yaml");
            }
        }
    } else {
        println!("  ⏭ doctor not found (set DOCTOR_BIN=/path or cargo install --path ~/doctor)");
    }

    println!();
    println!("═══ sos adopt report ═══");
    println!("ADDED (copied — were absent):");
    if added.is_empty() {
        println!("    (none)");
    } else {
        for l in &added {
            println!("{l}");
        }
    }
    println!("REVIEW / merge by hand (existing-or-missing — adopt did NOT touch):");
    if conflicts.is_empty() {
        println!("    (none)");
    } else {
        for l in &conflicts {
            println!("{l}");
        }
    }
    println!();
    // P088 Task 0 anchor #3 correction: build with `target` (the raw `&str`
    // arg, forward-slash) rather than `incoming.display()` — `PathBuf::join`
    // renders with the platform's native separator (`\` on Windows), which
    // would print `<TARGET>\.sos-adopt-incoming/<path>` and mismatch the
    // forward-slash golden.
    println!(
        "Next: diff staged wiring  (git diff --no-index <existing> {target}/.sos-adopt-incoming/<path>)  → merge by hand;",
    );
    println!("      add MISSING docs; fill # TODO; then: doctor verify-setup --repo {target}");
    println!();
    println!("Heads-up for your FIRST commit (hooks are now armed):");
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    println!("  - docs-gate freshness: add a new CHANGELOG entry (e.g. `## <ver> — sos-kit adopted — {date}`)");
    println!("  - fill docs/BACKLOG.md Active sprint (1-2 real items) — the session banner reads it");
    println!("  - restart Claude Code in the repo so agents/skills load");
    if !hooks_armed {
        // F09-decline (P086 Task 5): hooks weren't armed here, so trust
        // baseline wasn't seeded either — nudge the manual-arm path.
        println!("  - hooks NOT armed here — after arming by hand (bash scripts/install-hooks.sh), also run: bash scripts/trust-gate.sh rebaseline && git add .sos-trust-baseline");
    }

    Ok(())
}

fn map_target_has_agent_map(target: &Path) -> bool {
    target.join("docs/AGENT_MAP.yaml").is_file()
}
