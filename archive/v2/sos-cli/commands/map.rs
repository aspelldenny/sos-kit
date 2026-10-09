// commands/map.rs — Rust port of `bin/sos.sh` sos_map (bin/sos.sh:279-352).
//
// P077c1 shipped bug-for-bug parity with the Bash oracle (the 6 dir-pattern
// surfaces, no generic source surface, no kit-asset exclusion, single
// `draft_needs_review` status). P077c5 INTENTIONALLY DIVERGES from Bash to
// fix OA-02 (`docs/retro/OUTSIDER_AUDIT_SYSTEM_GAPS_2026-07-22.md:93-131`):
//   Part 1 — stack-aware generic source surface (Rust/Python/Node/Go/Swift).
//   Part 2 — exclude kit-managed asset roots from every surface (KIT_MANAGED_ROOTS).
//   Part 3 — 3-verdict status field (PATH_VALID / COVERAGE_UNKNOWN / COVERAGE_REVIEWED)
//            replacing the single `draft_needs_review` string. Fresh scan always
//            emits `coverage_unknown` — map never self-asserts COVERAGE_REVIEWED.
// `bin/sos.sh` is UNCHANGED (Rust-only fix, invariant across P077c; Bash fix
// deferred to the P077e cutover per this phiếu's Constraint 8).
//
// Two work-products, both parity-checked by tests/parity.rs (as a CORRECTNESS
// oracle for map/adopt from c5 onward — see tests/README.md):
//   (i)  file write  <target>/docs/AGENT_MAP.yaml (surfaces, sorted, relative paths)
//   (ii) stdout      single confirmation line

use anyhow::Result;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path};
use walkdir::WalkDir;

/// Kit-managed root directories (adopt's copy targets — verified against
/// `adopt.rs:506-524` at CHALLENGE). Any path whose FIRST relative path
/// component matches one of these is excluded from every surface, in both
/// standalone `sos map` and map-within-adopt (adopt copies these same roots
/// in step `[1/4]`, before map runs). This is the OA-02 Part 2 fix: kit
/// assets are never mis-scanned as product surfaces.
const KIT_MANAGED_ROOTS: &[&str] = &["templates", "phieu", "scripts", "hooks", ".claude"];

fn is_kit_managed(rel: &Path) -> bool {
    matches!(
        rel.components().next(),
        Some(Component::Normal(seg)) if KIT_MANAGED_ROOTS.iter().any(|r| seg.to_str() == Some(*r))
    )
}

/// Stack-detection marker: manifest basename(s) that signal a stack is
/// present anywhere in the target tree, and the generic source surface to
/// emit if so. Matches by extension only (no dir-name requirement) — this
/// deliberately catches `src/**`, nested modules, and monorepo sub-packages
/// alike (OA-02 Part 1 acceptance: "repo có sim/, tools/, migrations/, nested
/// packages phải có expected surfaces").
struct StackMarker {
    manifest_basenames: &'static [&'static str],
    surface_name: &'static str,
    ext_match: &'static [&'static str],
}

const STACK_MARKERS: &[StackMarker] = &[
    StackMarker { manifest_basenames: &["Cargo.toml"], surface_name: "rust_src", ext_match: &[".rs"] },
    StackMarker {
        manifest_basenames: &["pyproject.toml", "setup.py", "requirements.txt"],
        surface_name: "python_src",
        ext_match: &[".py"],
    },
    StackMarker { manifest_basenames: &["package.json"], surface_name: "node_src", ext_match: &[".ts", ".js"] },
    StackMarker { manifest_basenames: &["go.mod"], surface_name: "go_src", ext_match: &[".go"] },
    StackMarker { manifest_basenames: &["Package.swift"], surface_name: "swift_src", ext_match: &[".swift"] },
];

/// Scan the whole tree once for manifest basenames (excluding noise + kit-managed
/// roots) to decide which stack-specific source surface(s) to emit. Unbounded
/// depth on purpose — a manifest nested in a monorepo sub-package still counts.
fn detect_present_stacks(target: &Path) -> HashSet<&'static str> {
    let mut found = HashSet::new();
    for entry in WalkDir::new(target).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let rel = match path.strip_prefix(target) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if rel.as_os_str().is_empty() || entry.file_type().is_dir() {
            continue;
        }
        if is_kit_managed(rel) {
            continue;
        }
        // Forward-slash normalize before any POSIX-style substring match
        // (native `\` on Windows never matches "/xyz/" substrings — BUG 1).
        let path_str = path.to_string_lossy().replace('\\', "/");
        if is_noise(&path_str) {
            continue;
        }
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        for marker in STACK_MARKERS {
            if marker.manifest_basenames.contains(&file_name) {
                found.insert(marker.surface_name);
            }
        }
    }
    found
}

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

/// `path_str` must be forward-slash-normalized (Windows native `\` breaks the
/// POSIX-style substring match) — normalize at each call site before calling.
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
        if is_kit_managed(rel) {
            continue; // OA-02 Part 2: never scan kit-managed asset roots
        }
        // Forward-slash normalize before any POSIX-style substring match
        // (native `\` on Windows never matches "/xyz/" substrings — BUG 1).
        let path_str = path.to_string_lossy().replace('\\', "/");
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

// P077c5 Part 3 — 3-verdict status (replaces the single `draft_needs_review`):
//   PATH_VALID       — every entry path in the map exists (property `doctor validate-map` asserts).
//   COVERAGE_UNKNOWN — fresh machine scan, no human/architect coverage review yet. This is what
//                       `sos map` itself ALWAYS emits (both the normal-surfaces case below and the
//                       zero-surfaces stub case — a fresh empty scan is still "unknown coverage",
//                       not a distinct 4th state).
//   COVERAGE_REVIEWED — human/architect has confirmed load-bearing surfaces. `sos map` never sets
//                       this itself; it is a manual upgrade after review.
// `COVERAGE_UNKNOWN` (not `draft_needs_review`) is NOT routing authority — it signals "not yet
// human-reviewed for completeness", nothing more.
const HEAD: &str = "# AGENT_MAP — scan-generated DRAFT (sos map). status: coverage_unknown.\n#\n# SOUND half (auto): the surfaces + edit-paths below are REAL dirs/files in THIS repo, coarse-\n#   grouped by directory pattern. This is NOT the tarot example (no \"map nói dối\").\n# PARTIAL half (YOU): set load_bearing (true|false) + write blast for each — the scan can't\n#   judge importance (product knowledge). Split coarse regions into semantic surfaces as you\n#   refine (e.g. routes_handlers → auth / ratings / privacy).\n#\n# 3-verdict status: PATH_VALID (validate-map property) / COVERAGE_UNKNOWN (fresh scan, default) /\n#   COVERAGE_REVIEWED (human-set only — map never self-asserts this).\n# Validator: doctor validate-map --map docs/AGENT_MAP.yaml (every path must exist).\nversion: 1\nstatus: coverage_unknown\ngenerated_by: \"sos map (scan)\"\n\nsurfaces:\n";

const TAIL: &str = "\nnever_default_read:\n  - docs/CHANGELOG.md\n  - docs/DISCOVERIES.md\n  - docs/BACKLOG.md\n  - docs/ticket/archive/**\n";

// O1.1 (Worker CHALLENGE, self-decided Tầng 2): zero-surface stub folded into
// COVERAGE_UNKNOWN too — it is still a fresh unreviewed scan, just empty. Does
// NOT become a 4th verdict.
const UNMAPPED_STUB: &str = "# AGENT_MAP — UNMAPPED DRAFT. status: coverage_unknown.\n";

/// Render `out`'s display path for stdout with the `target` CLI arg
/// preserved byte-for-byte (the parity test harness substring-replaces the
/// literal `target` arg with `<TARGET>` — see tests/parity.rs `normalize()`;
/// if we forward-slash the WHOLE display path first, that substring stops
/// matching a native-backslash `target` on Windows) but the suffix AFTER
/// `target` forced to forward-slash (parity golden expects `<TARGET>/docs/...`,
/// not `<TARGET>\docs\...`).
fn display_out_path(target: &str, out: &Path) -> String {
    let out_str = out.to_string_lossy().replace('\\', "/");
    let target_norm = target.replace('\\', "/");
    match out_str.strip_prefix(&target_norm) {
        Some(suffix) => format!("{target}{suffix}"),
        None => out_str,
    }
}

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

    // OA-02 Part 1 — stack-aware generic source surface(s), additive to the 6
    // pre-existing dir-pattern surfaces above.
    let present_stacks = detect_present_stacks(target_dir);
    for marker in STACK_MARKERS {
        if !present_stacks.contains(marker.surface_name) {
            continue;
        }
        let stack_surface = Surface {
            name: marker.surface_name,
            path_substrs: &[],
            dir_name_match: &[],
            name_match: marker.ext_match,
            collect_dirs: false,
            max_depth: 0,
        };
        let hits = scan_surface(target_dir, &stack_surface);
        if hits.is_empty() {
            continue;
        }
        surfaces.push_str(&render_surface_block(stack_surface.name, &hits));
    }

    if surfaces.is_empty() {
        fs::write(&out, UNMAPPED_STUB)?;
        println!(
            "  ⚠ sos map: no code surfaces detected → wrote coverage_unknown stub ({}). Fill by hand.",
            display_out_path(target, &out)
        );
        return Ok(());
    }

    let mut content = String::new();
    content.push_str(HEAD);
    content.push_str(&surfaces);
    content.push_str(TAIL);
    fs::write(&out, content)?;

    println!(
        "  ✓ sos map: scanned {} → {} (coverage_unknown — set load_bearing + blast by hand)",
        target,
        display_out_path(target, &out)
    );
    Ok(())
}
