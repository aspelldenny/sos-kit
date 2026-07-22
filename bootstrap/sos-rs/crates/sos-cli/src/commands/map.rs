// commands/map.rs — Rust port of `bin/sos.sh` sos_map (bin/sos.sh:279-352).
//
// P077c1: bug-for-bug parity with the Bash oracle. Do NOT fix OA-02 here
// (no generic src/*.rs surface, no kit-asset exclusion, no 3-verdict) —
// that divergence is P077c5's job, on purpose, against a different oracle.
//
// Two work-products, both parity-checked by tests/parity.rs:
//   (i)  file write  <target>/docs/AGENT_MAP.yaml (surfaces, sorted, relative paths)
//   (ii) stdout      single confirmation line

use anyhow::Result;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// One scan rule: directory-name filters get grouped via `pattern_dirs`,
/// so a single scan_files pass can express both dir-substring predicates
/// and file-extension/basename predicates like the Bash `find` expression.
struct Surface {
    name: &'static str,
    // Path must contain one of these path-segment substrings (dir match).
    path_substrs: &'static [&'static str],
    // If non-empty, only files whose name matches one of these dir basenames
    // (used for `-type d -name X` style rules like migrations/templates).
    dir_name_match: &'static [&'static str],
    // File extensions/basenames to accept (empty = accept any, used for dir rules).
    name_match: &'static [&'static str],
    // true = collect directories, false = collect files.
    collect_dirs: bool,
    // depth cap (Bash `-maxdepth 2` for config_runtime); 0 = unbounded.
    max_depth: usize,
}

const SURFACES: &[Surface] = &[
    Surface {
        name: "routes_handlers",
        path_substrs: &["/routes/", "/handlers/", "/views/", "/controllers/", "/api/"],
        dir_name_match: &[],
        name_match: &[".py", ".ts", ".js", ".rs"],
        collect_dirs: false,
        max_depth: 0,
    },
    Surface {
        name: "models_schema",
        path_substrs: &["/models/", "/entities/"],
        dir_name_match: &[],
        name_match: &[".py", ".ts", ".rs"],
        collect_dirs: false,
        max_depth: 0,
    },
    Surface {
        name: "services_logic",
        path_substrs: &["/services/", "/lib/"],
        dir_name_match: &[],
        name_match: &[".py", ".ts", ".rs"],
        collect_dirs: false,
        max_depth: 0,
    },
    Surface {
        name: "migrations",
        path_substrs: &[],
        dir_name_match: &["migrations"],
        name_match: &[],
        collect_dirs: true,
        max_depth: 0,
    },
    Surface {
        name: "frontend",
        path_substrs: &[],
        dir_name_match: &["templates", "components", "static"],
        name_match: &[],
        collect_dirs: true,
        max_depth: 0,
    },
    Surface {
        name: "config_runtime",
        path_substrs: &[],
        dir_name_match: &[],
        name_match: &[], // handled specially below (exact basenames)
        collect_dirs: false,
        max_depth: 2,
    },
];

const CONFIG_BASENAMES: &[&str] = &["config.py", "settings.py", "Dockerfile", ".flaskenv"];
const CONFIG_PREFIX: &str = "docker-compose"; // docker-compose*.yml

const NOISE_EXCLUDE: &[&str] = &[
    "/.git/",
    "/node_modules/",
    "/__pycache__/",
    "/.sos-adopt-incoming/",
    "/migrations/versions/",
    "/.venv/",
    "/venv/",
    "/dist/",
    "/build/",
];

fn is_noise(path_str: &str) -> bool {
    NOISE_EXCLUDE.iter().any(|n| path_str.contains(n))
}

fn matches_ext(name: &str, exts: &[&str]) -> bool {
    exts.iter().any(|e| name.ends_with(e))
}

fn is_models_schema_name(name: &str, exts: &[&str]) -> bool {
    // Bash: -name 'schema.*' -o (models/entities path with .py/.ts/.rs ext)
    name.starts_with("schema.") || matches_ext(name, exts)
}

/// Mirrors `bin/sos.sh` scan_files(): find matches under target (noise
/// excluded), relpath-stripped, sorted, capped at 25.
fn scan_surface(target: &Path, surface: &Surface) -> Vec<String> {
    let mut hits: Vec<String> = Vec::new();

    for entry in WalkDir::new(target).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let rel = match path.strip_prefix(target) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if rel.as_os_str().is_empty() {
            continue; // skip target dir itself
        }
        let path_str = path.to_string_lossy();
        if is_noise(&path_str) {
            continue;
        }
        if surface.max_depth > 0 {
            let depth = rel.components().count();
            if depth > surface.max_depth {
                continue;
            }
        }

        let is_dir = entry.file_type().is_dir();
        if surface.collect_dirs {
            if !is_dir {
                continue;
            }
            let dir_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };
            if surface.dir_name_match.iter().any(|d| d == &dir_name) {
                hits.push(rel.to_string_lossy().replace('\\', "/"));
            }
            continue;
        }

        if is_dir {
            continue;
        }
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        // config_runtime: special exact-basename / prefix rule, no path_substrs.
        if surface.name == "config_runtime" {
            if CONFIG_BASENAMES.contains(&file_name)
                || (file_name.starts_with(CONFIG_PREFIX) && file_name.ends_with(".yml"))
            {
                hits.push(rel.to_string_lossy().replace('\\', "/"));
            }
            continue;
        }

        let path_hit = surface.path_substrs.is_empty()
            || surface.path_substrs.iter().any(|s| path_str.contains(s));
        if !path_hit {
            continue;
        }

        let name_hit = if surface.name == "models_schema" {
            is_models_schema_name(file_name, surface.name_match)
        } else {
            surface.name_match.is_empty() || matches_ext(file_name, surface.name_match)
        };
        if !name_hit {
            continue;
        }

        hits.push(rel.to_string_lossy().replace('\\', "/"));
    }

    hits.sort();
    hits.truncate(25);
    hits
}

fn render_surface_block(name: &str, hits: &[String]) -> String {
    let mut out = format!(
        "  {name}:\n    load_bearing: true                # NEEDS_JUDGMENT — confirm: true=architect reads deep, false=leaf. Default true = safe (over-read beats miss-blast).\n    edit:\n"
    );
    for h in hits {
        out.push_str("      - ");
        out.push_str(h);
        out.push('\n');
    }
    out.push_str("    blast: \"TODO: what breaks if this changes — describe blast radius (you).\"\n");
    out
}

const HEAD: &str = "# AGENT_MAP — scan-generated DRAFT (sos map). status: draft_needs_review.\n#\n# SOUND half (auto): the surfaces + edit-paths below are REAL dirs/files in THIS repo, coarse-\n#   grouped by directory pattern. This is NOT the tarot example (no \"map nói dối\").\n# PARTIAL half (YOU): set load_bearing (true|false) + write blast for each — the scan can't\n#   judge importance (product knowledge). Split coarse regions into semantic surfaces as you\n#   refine (e.g. routes_handlers → auth / ratings / privacy).\n#\n# Validator: doctor validate-map --map docs/AGENT_MAP.yaml (every path must exist).\nversion: 1\nstatus: draft_needs_review\ngenerated_by: \"sos map (scan)\"\n\nsurfaces:\n";

const TAIL: &str = "\nnever_default_read:\n  - docs/CHANGELOG.md\n  - docs/DISCOVERIES.md\n  - docs/BACKLOG.md\n  - docs/ticket/archive/**\n";

const UNMAPPED_STUB: &str = "# AGENT_MAP — UNMAPPED DRAFT. status: draft_unmapped.\n";

pub fn run(target: &str) -> Result<()> {
    let target_dir = Path::new(target);
    if !target_dir.is_dir() {
        println!("✗ {target} is not a directory");
        return Ok(());
    }
    let docs_dir = target_dir.join("docs");
    fs::create_dir_all(&docs_dir)?;
    let out = docs_dir.join("AGENT_MAP.yaml");

    let mut surfaces = String::new();
    for s in SURFACES {
        let hits = scan_surface(target_dir, s);
        if hits.is_empty() {
            continue;
        }
        surfaces.push_str(&render_surface_block(s.name, &hits));
    }

    if surfaces.is_empty() {
        fs::write(&out, UNMAPPED_STUB)?;
        println!(
            "  ⚠ sos map: no code surfaces detected → wrote draft_unmapped stub ({}). Fill by hand.",
            out.display()
        );
        return Ok(());
    }

    let mut content = String::new();
    content.push_str(HEAD);
    content.push_str(&surfaces);
    content.push_str(TAIL);
    fs::write(&out, content)?;

    println!(
        "  ✓ sos map: scanned {} → {} (draft_needs_review — set load_bearing + blast by hand)",
        target,
        out.display()
    );
    Ok(())
}
