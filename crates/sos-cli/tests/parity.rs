// parity.rs — P077a parity-harness skeleton, flipped to a real gate by P077c1.
//
// Diffs the Rust `sos` binary's output for new/adopt/map/sync against the frozen
// Bash golden oracle (tests/golden/*.golden, see tests/README.md).
//
// P077a/b left this INFORMATIONAL (`HARD_FAIL: bool = false`, always PASS).
// P077c1 flips it to a per-command hard-fail SET: a command enters
// `PARITY_ENFORCED` once its Rust impl is proven correct against its oracle.
// c2 (sync) / c3 (new) / c4 (adopt) add their own name to the slice as they land
// — no other harness rewrite needed.
//
// `map` is also the first command with a TWO-fixture oracle: the Bash
// `sos_map` writes its real work-product to a FILE (`<target>/docs/AGENT_MAP.yaml`)
// and only echoes a 1-line confirmation to stdout. `map.golden` freezes that
// stdout line; `map.agent_map.golden` (added in P077c1) freezes the file
// content. Both are diffed; a mismatch in EITHER hard-fails when the command
// is in `PARITY_ENFORCED` — otherwise the stdout-only oracle is blind to
// scan-correctness (the false-green class this harness exists to prevent).
//
// P077c5 NOTE — `PARITY_ENFORCED` keeps its 4-name shape, but `map`/`adopt`'s
// SEMANTICS changed: from c1-c4 they were bug-for-bug ==Bash parity oracles;
// from c5 onward they are CORRECTNESS oracles (Rust intentionally != Bash —
// OA-02 fix; `bin/sos.sh` stays buggy by design, deferred to P077e cutover).
// `sync`/`new` remain pure Bash-parity, unchanged. Don't be misled by the name
// `PARITY_ENFORCED` for `map`/`adopt` — read `parity_map_enforced`/
// `parity_adopt_enforced`'s doc comments (below) for what they actually assert.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Commands whose Rust impl has reached bug-for-bug parity with the Bash
/// oracle. Add a name here once its golden(s) are proven to match — the rest
/// of this harness (fixture build, normalize, assert) is already generic.
const PARITY_ENFORCED: &[&str] = &["map", "sync", "new", "adopt"];

struct Case {
    name: &'static str,
    args: &'static [&'static str],
    golden: &'static str,
}

const CASES: &[Case] = &[
    Case { name: "new",   args: &["new", "/tmp/parity-fixture-new", "--stack", "python"], golden: "new.golden" },
    Case { name: "adopt", args: &["adopt", "/tmp/parity-fixture-adopt", "--stack", "python"], golden: "adopt.golden" },
    Case { name: "map",   args: &["map", "/tmp/parity-fixture-map"], golden: "map.golden" },
    Case { name: "sync",  args: &["sync", "/tmp/parity-fixture-sync"], golden: "sync.golden" },
];

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn read_golden(name: &str) -> String {
    fs::read_to_string(golden_dir().join(name))
        .unwrap_or_else(|e| panic!("missing golden fixture {name}: {e}"))
}

fn run_rust(args: &[&str]) -> String {
    let bin = env!("CARGO_BIN_EXE_sos");
    let out = Command::new(bin)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to exec {bin}: {e}"));
    let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&out.stderr));
    combined
}

/// Same as `run_rust`, but with `SOS_KIT_DIR` set — needed for `sync`, whose
/// provenance oracle is read from that env var (bug-for-bug w/ Bash's `$K`).
fn run_rust_with_kit(args: &[&str], kit: &Path) -> String {
    let bin = env!("CARGO_BIN_EXE_sos");
    let out = Command::new(bin)
        .args(args)
        .env("SOS_KIT_DIR", kit)
        .output()
        .unwrap_or_else(|e| panic!("failed to exec {bin}: {e}"));
    let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&out.stderr));
    combined
}

/// Same as `run_rust_with_kit`, but ALSO forces `DOCTOR_BIN=/nonexistent/doctor`
/// so `new`'s [4/4] verify-setup branch is deterministic regardless of whether
/// `doctor` happens to be installed on the machine running the test suite
/// (matches `capture.sh`'s `run_isolated_kit_doctor_absent`, P077c3).
fn run_rust_with_kit_doctor_absent(args: &[&str], kit: &Path) -> String {
    let bin = env!("CARGO_BIN_EXE_sos");
    let out = Command::new(bin)
        .args(args)
        .env("SOS_KIT_DIR", kit)
        .env("DOCTOR_BIN", "/nonexistent/doctor")
        .output()
        .unwrap_or_else(|e| panic!("failed to exec {bin}: {e}"));
    let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&out.stderr));
    combined
}

/// Same substitution rule as tests/golden/capture.sh normalize()'s target
/// abs-path -> <TARGET> line. (No <SOS_KIT_DIR>/<DATE> substitution needed
/// here — nothing this harness invokes echoes the kit dir or an embedded
/// date; kept generic anyway so future commands can extend it.)
fn normalize(raw: &str, target: &str) -> String {
    raw.replace(target, "<TARGET>")
}

/// A throwaway directory under the OS temp dir, removed on drop — no new
/// crate dependency needed (std::env::temp_dir + pid/nanos for uniqueness,
/// same idea as `mktemp -d` used by capture.sh).
///
/// P077c6 NOTE — pid+nanos alone is NOT collision-proof: all tests in one
/// `cargo test` binary run as THREADS in a single process (same `pid`), and
/// this machine's `SystemTime::now()` clock resolution is far coarser than
/// 1ns (measured: ~97.5% of back-to-back calls return an IDENTICAL nanos
/// value). Two parallel tests (e.g. `parity_new_enforced` and the nested
/// `build_new_fixture()` call inside `parity_adopt_enforced`) could sample
/// the same nanos and collide on the exact same dir path — whichever
/// `TempFixture` dropped first then `remove_dir_all`'d the OTHER test's
/// still-in-use fixture out from under it (root cause of the content-flaky
/// `parity_adopt_enforced` failures pinned in Debate Log Turn 2 / DIAGNOSE).
/// A process-wide monotonic counter makes the key collision-proof
/// independent of clock resolution — no two `TempFixture`s can ever resolve
/// to the same path, regardless of timing.
static TEMP_FIXTURE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

struct TempFixture(PathBuf);

impl TempFixture {
    fn new(name: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = TEMP_FIXTURE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "sos-parity-{name}-{}-{nanos}-{counter}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        TempFixture(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run_git(args: &[&str], cwd: &Path) {
    let status = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .status()
        .unwrap_or_else(|e| panic!("failed to exec git {:?}: {e}", args));
    assert!(status.success(), "git {:?} failed in {:?}", args, cwd);
}

/// Build the exact same synthetic fake-kit git repo + target that
/// `capture.sh`'s `build_fake_kit`/`build_sync_target` build (P077c2) — a
/// self-contained git history (v1 -> v2 commits) with zero dependency on
/// real sos-kit HEAD, exercising all 4 `sync` outcomes.
fn build_sync_fixture() -> (TempFixture, PathBuf, PathBuf) {
    let tmp = TempFixture::new("sync");
    let kit = tmp.path().join("sync-fake-kit");
    let target = tmp.path().join("sync-fixture");

    fs::create_dir_all(kit.join("scripts")).unwrap();
    fs::create_dir_all(kit.join("phieu")).unwrap();
    fs::create_dir_all(kit.join("templates")).unwrap();
    fs::create_dir_all(kit.join(".claude/agents")).unwrap();
    fs::create_dir_all(kit.join(".claude/commands")).unwrap();
    fs::create_dir_all(kit.join("skills/demo")).unwrap();
    fs::create_dir_all(kit.join("agents")).unwrap();

    run_git(&["init", "-q"], &kit);
    run_git(&["config", "user.email", "sos-kit-fixture@example.com"], &kit);
    run_git(&["config", "user.name", "sos-kit fixture"], &kit);

    fs::write(kit.join("scripts/updated.sh"), "kit content v1\n").unwrap();
    fs::write(kit.join("scripts/added.sh"), "kit content\n").unwrap();
    fs::write(kit.join("scripts/flagged.sh"), "kit content\n").unwrap();
    fs::write(kit.join("scripts/current.sh"), "kit content\n").unwrap();
    fs::write(kit.join("skills/demo/SKILL.md"), "# demo skill\n").unwrap();
    fs::write(kit.join("agents/demo.md"), "# demo agent\n").unwrap();
    run_git(&["add", "-A"], &kit);
    run_git(&["commit", "-q", "-m", "v1"], &kit);

    fs::write(kit.join("scripts/updated.sh"), "kit content v2\n").unwrap();
    run_git(&["add", "-A"], &kit);
    run_git(&["commit", "-q", "-m", "v2"], &kit);

    fs::create_dir_all(target.join("scripts")).unwrap();
    fs::create_dir_all(target.join(".claude/agents")).unwrap();
    fs::write(target.join("scripts/updated.sh"), "kit content v1\n").unwrap();
    fs::write(target.join("scripts/flagged.sh"), "custom content, never seen by kit\n").unwrap();
    fs::write(target.join("scripts/current.sh"), "kit content\n").unwrap();

    (tmp, kit, target)
}

/// Build the exact same synthetic fake-kit `capture.sh`'s `build_fake_kit_new`
/// builds (P077c3) — a plain directory tree (no git history needed, unlike
/// `sync`'s fake-kit, since `new` only COPIES kit assets verbatim). Covers
/// every "$K/..." path `sos_new` reads (bin/sos.sh:404-565).
fn build_new_fixture() -> (TempFixture, PathBuf, PathBuf) {
    let tmp = TempFixture::new("new");
    let kit = tmp.path().join("new-fake-kit");
    let target = tmp.path().join("new-fixture");

    fs::create_dir_all(kit.join(".claude/agents")).unwrap();
    fs::create_dir_all(kit.join(".claude/commands")).unwrap();
    fs::create_dir_all(kit.join("agents")).unwrap();
    fs::create_dir_all(kit.join("templates")).unwrap();
    fs::create_dir_all(kit.join("scripts")).unwrap();
    fs::create_dir_all(kit.join("phieu")).unwrap();
    fs::create_dir_all(kit.join("hooks")).unwrap();
    fs::create_dir_all(kit.join("skills/idea")).unwrap();
    fs::create_dir_all(kit.join("skills/attic/dummy")).unwrap();

    fs::write(kit.join(".claude/agents/worker.md"), "# fake worker agent\n").unwrap();
    fs::write(kit.join(".claude/settings.json"), "{\"fake\":\"settings\"}\n").unwrap();
    fs::write(kit.join(".claude/commands/idea.md"), "# fake idea command\n").unwrap();
    fs::write(kit.join("agents/orchestrator.md"), "# fake orchestrator\n").unwrap();
    fs::write(kit.join("templates/claude-settings.local.json"), "{\"fake\":\"local-settings\"}\n").unwrap();
    fs::write(kit.join("templates/INVARIANTS-template.md"), "# INVARIANTS template\n").unwrap();
    fs::write(kit.join("templates/BACKLOG_template.md"), "# BACKLOG template\n").unwrap();
    fs::write(kit.join("skills/idea/SKILL.md"), "# fake idea skill\n").unwrap();
    fs::write(kit.join("skills/attic/dummy/SKILL.md"), "# fake attic skill (must be SKIPPED by new)\n").unwrap();
    fs::write(kit.join("phieu/README.md"), "fake phieu readme\n").unwrap();
    fs::write(kit.join("hooks/pre-commit"), "fake-pre-commit\n").unwrap();
    fs::write(kit.join("hooks/pre-push"), "fake-pre-push\n").unwrap();
    fs::write(kit.join(".gitignore"), ".DS_Store\n").unwrap();

    // install-hooks.sh must actually FUNCTION (new's [+] git step shells out
    // to it) — reuse the REAL script from the real sos-kit checkout this test
    // binary was built from (content never gen-hashed/frozen, path-only).
    let real_install_hooks =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/install-hooks.sh");
    fs::copy(&real_install_hooks, kit.join("scripts/install-hooks.sh"))
        .unwrap_or_else(|e| panic!("copy install-hooks.sh from {:?}: {e}", real_install_hooks));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let p = kit.join("scripts/install-hooks.sh");
        let mut perm = fs::metadata(&p).unwrap().permissions();
        perm.set_mode(perm.mode() | 0o111);
        fs::set_permissions(&p, perm).unwrap();
    }

    (tmp, kit, target)
}

/// GENERATED-authored files `sos_new` writes — content-hashed. Mirrors
/// `capture.sh`'s `NEW_GEN_FILES`. Copied kit assets are NEVER in this list.
const NEW_GEN_FILES: &[&str] = &[
    ".mcp.json",
    "docs/security/INVARIANTS.md",
    "docs/AGENT_MAP.yaml",
    "CLAUDE.md",
    "docs/ARCHITECTURE.md",
    "CHANGELOG.md",
    "pyproject.toml",
    ".docs-gate.toml",
    ".sos-stack.toml",
];

fn strip_timestamp(s: &str) -> String {
    // Full ISO-8601 timestamp -> <TIMESTAMP> (mirrors capture.sh's strip_timestamp,
    // MUST run before any bare-date substitution to avoid a half-stripped
    // "<DATE>T10:24:22Z" leaking through non-deterministic, P077c3 anchor #10).
    // Operates on raw bytes (NOT &str slicing) — the ASCII pattern check never
    // needs a valid UTF-8 boundary, but content around it (e.g. "—") does, and
    // slicing a &str at an arbitrary byte offset panics on a non-boundary index
    // even when the slice's content is irrelevant to the match.
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if i + 20 <= bytes.len() && is_iso8601_at(&bytes[i..i + 20]) {
            out.extend_from_slice(b"<TIMESTAMP>");
            i += 20;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    // Safe: we only ever copy whole original bytes or substitute a pure-ASCII
    // marker for a byte run we verified is pure ASCII digits/separators.
    String::from_utf8(out).expect("strip_timestamp preserves UTF-8 validity")
}

fn strip_bare_date(s: &str) -> String {
    // Bare "YYYY-MM-DD" (not already consumed by strip_timestamp) -> <DATE>,
    // mirrors capture.sh normalize()'s date rule (e.g. CHANGELOG.md's
    // `sos new`-generated entry date).
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if i + 10 <= bytes.len() && is_bare_date_at(&bytes[i..i + 10]) {
            out.extend_from_slice(b"<DATE>");
            i += 10;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).expect("strip_bare_date preserves UTF-8 validity")
}

fn is_bare_date_at(b: &[u8]) -> bool {
    if b.len() != 10 {
        return false;
    }
    let digit = |i: usize| b[i].is_ascii_digit();
    (0..4).all(digit) && b[4] == b'-' && (5..7).all(digit) && b[7] == b'-' && (8..10).all(digit)
}

fn is_iso8601_at(b: &[u8]) -> bool {
    // "YYYY-MM-DDTHH:MM:SSZ" — 20 bytes, all-ASCII checked positions.
    if b.len() != 20 {
        return false;
    }
    let digit = |i: usize| b[i].is_ascii_digit();
    (0..4).all(digit)
        && b[4] == b'-'
        && (5..7).all(digit)
        && b[7] == b'-'
        && (8..10).all(digit)
        && b[10] == b'T'
        && (11..13).all(digit)
        && b[13] == b':'
        && (14..16).all(digit)
        && b[16] == b':'
        && (17..19).all(digit)
        && b[19] == b'Z'
}

/// Post-`new` path-shape manifest: every relpath under target, sorted,
/// EXCLUDING `.git/` — no content. Mirrors `capture.sh`'s `freeze_new_tree`.
fn build_new_tree_manifest(target: &Path) -> String {
    let mut lines: Vec<String> = Vec::new();
    for entry in WalkDir::new(target).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path == target {
            continue;
        }
        let rel = path.strip_prefix(target).unwrap().to_string_lossy().replace('\\', "/");
        if rel.starts_with(".git") {
            continue;
        }
        lines.push(rel);
    }
    lines.sort();
    lines.join("\n")
}

/// Post-`new` content-hash manifest, GENERATED-authored files only. Mirrors
/// `capture.sh`'s `freeze_new_gen`.
fn build_new_gen_manifest(target: &Path, target_str: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for f in NEW_GEN_FILES {
        let p = target.join(f);
        if !p.is_file() {
            continue;
        }
        let raw = fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()));
        let normalized = strip_bare_date(&strip_timestamp(&raw)).replace(target_str, "<TARGET>");
        let hash = sha256_hex_of_bytes(normalized.as_bytes());
        lines.push(format!("{f} {hash}"));
    }
    lines.sort();
    lines.join("\n")
}

fn sha256_hex_of_bytes(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[test]
fn parity_new_enforced() {
    assert!(
        PARITY_ENFORCED.contains(&"new"),
        "this test only makes sense while new is in PARITY_ENFORCED"
    );

    let (_tmp, kit, target) = build_new_fixture();
    let target_str = target.to_string_lossy().into_owned();
    let kit_str = kit.to_string_lossy().into_owned();

    let raw = run_rust_with_kit_doctor_absent(&["new", &target_str, "--stack", "python"], &kit);
    let stdout_actual = normalize(&raw, &target_str).replace(&kit_str, "<SOS_KIT_DIR>");
    let stdout_golden = read_golden("new.golden");
    assert_eq!(
        stdout_actual.trim(),
        stdout_golden.trim(),
        "new stdout mismatch vs new.golden (hard-fail — new is PARITY_ENFORCED). \
         NOTE: captured with DOCTOR_BIN=/nonexistent/doctor — a mismatch may mean \
         the doctor-absent skip branch didn't fire, not just a content regression."
    );

    let tree_actual = build_new_tree_manifest(&target);
    let tree_golden = read_golden("new.tree.golden");
    assert_eq!(
        tree_actual.trim(),
        tree_golden.trim(),
        "new file-tree manifest mismatch vs new.tree.golden (hard-fail — new is PARITY_ENFORCED)"
    );

    let gen_actual = build_new_gen_manifest(&target, &target_str);
    let gen_golden = read_golden("new.gen.golden");
    assert_eq!(
        gen_actual.trim(),
        gen_golden.trim(),
        "new gen-content manifest mismatch vs new.gen.golden (hard-fail — new is PARITY_ENFORCED)"
    );
}

/// Build the same synthetic fake-kit + 4-collision BROWNFIELD target
/// `capture.sh`'s `build_fake_kit_adopt()`/`build_adopt_brownfield()` build
/// (P077c4). Fake-kit = `build_new_fixture()`'s shape (`adopt`'s [1/4] copies
/// the exact same spine `new` does) PLUS `templates/claude-settings.local.json`
/// (2nd fixture gotcha found during Worker CHALLENGE Turn 1's live-probe:
/// [1/4]/[3/4] error without it — `.claude/settings.local.json` copy-if-absent
/// + `scripts/install-hooks.sh` must actually be the REAL, executable script).
///
/// Brownfield target seeds all 4 collision cases (a) spine ABSENT (no seed —
/// implicit), (b) spine COLLISION (`templates/INVARIANTS-template.md`, content
/// differs from kit's — must be STAGED, target file UNCHANGED), (c) Cat-C doc
/// EXISTING (`CHANGELOG.md` — generate must SKIP), (d) source non-spine
/// (`src/routes/api.py` + `src/models/user.py` + `pyproject.toml` — untouched,
/// but feeds the map-within-adopt OA-02 scan). Committed clean (git init +
/// commit) so the dirty-warn branch does NOT fire (deterministic stdout).
fn build_adopt_fixture() -> (TempFixture, PathBuf, PathBuf) {
    let (tmp, kit, _unused_new_target) = build_new_fixture();
    fs::write(
        kit.join("templates/claude-settings.local.json"),
        "{\"permissions\":{\"allow\":[]}}\n",
    )
    .unwrap();

    let target = tmp.path().join("adopt-fixture");
    fs::create_dir_all(target.join("templates")).unwrap();
    fs::create_dir_all(target.join("docs")).unwrap();
    fs::create_dir_all(target.join("src/routes")).unwrap();
    fs::create_dir_all(target.join("src/models")).unwrap();
    fs::write(
        target.join("templates/INVARIANTS-template.md"),
        "custom INVARIANTS content — brownfield repo's own, never seen by kit\n",
    )
    .unwrap();
    fs::write(
        target.join("CHANGELOG.md"),
        "# my own changelog — pre-existing, never touched by adopt\n",
    )
    .unwrap();
    fs::write(target.join("src/routes/api.py"), "def h(): pass\n").unwrap();
    fs::write(target.join("src/models/user.py"), "class M: pass\n").unwrap();
    fs::write(target.join("pyproject.toml"), "[tool.poetry]\n").unwrap();

    run_git(&["init", "-q"], &target);
    run_git(&["config", "user.email", "sos-kit-fixture@example.com"], &target);
    run_git(&["config", "user.name", "sos-kit fixture"], &target);
    run_git(&["add", "-A"], &target);
    run_git(&["commit", "-q", "-m", "seed"], &target);

    (tmp, kit, target)
}

/// GENERATED-authored files `sos_adopt` writes when absent — content-hashed
/// into `adopt.gen.golden`. Copied/staged kit assets are NEVER in this list
/// (tree-shape + preservation-assert cover them instead — kit-content-coupling
/// guard, same rationale as `NEW_GEN_FILES`).
const ADOPT_GEN_FILES: &[&str] = &[
    ".mcp.json",
    "docs/security/INVARIANTS.md",
    "docs/AGENT_MAP.yaml",
    ".docs-gate.toml",
    "CHANGELOG.md",
    "docs/ARCHITECTURE.md",
    ".gitignore",
    ".sos-stack.toml",
];

/// Post-`adopt` path-shape manifest: every relpath under target, sorted,
/// EXCLUDING `.git*` (bug-for-bug with the pre-existing `new`/`sync` tree
/// builders' `.git` prefix filter, which also excludes `.gitignore` —
/// inherited, not introduced by c4). INCLUDES `.sos-adopt-incoming/**`.
fn build_adopt_tree_manifest(target: &Path) -> String {
    let mut lines: Vec<String> = Vec::new();
    for entry in WalkDir::new(target).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path == target {
            continue;
        }
        let rel = path.strip_prefix(target).unwrap().to_string_lossy().replace('\\', "/");
        if rel.starts_with(".git") {
            continue;
        }
        lines.push(rel);
    }
    lines.sort();
    lines.join("\n")
}

/// Post-`adopt` content-hash manifest, GENERATED-authored files only. Mirrors
/// `capture.sh`'s `freeze_adopt_gen`.
fn build_adopt_gen_manifest(target: &Path, target_str: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for f in ADOPT_GEN_FILES {
        let p = target.join(f);
        if !p.is_file() {
            continue;
        }
        let raw = fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()));
        let normalized = strip_bare_date(&strip_timestamp(&raw)).replace(target_str, "<TARGET>");
        let hash = sha256_hex_of_bytes(normalized.as_bytes());
        lines.push(format!("{f} {hash}"));
    }
    lines.sort();
    lines.join("\n")
}

/// Seeded pre-existing files whose sha256 the preservation-assert (in-test
/// INVARIANT, NOT a golden) checks are UNCHANGED before/after `adopt` runs —
/// this is the non-clobber guarantee's proof, independent of tree/gen/stdout.
const ADOPT_PRESERVED_SEEDS: &[&str] = &[
    "templates/INVARIANTS-template.md",
    "CHANGELOG.md",
    "src/routes/api.py",
    "src/models/user.py",
    "pyproject.toml",
];

/// Staged-collision pairs: (target-side `.sos-adopt-incoming/<path>`, kit-side
/// source `<path>`) — the preservation-assert's 2nd half: staged copies must
/// byte-match the kit source they were staged from.
const ADOPT_STAGED_COLLISIONS: &[&str] = &["templates/INVARIANTS-template.md"];

/// Stdout + tree-manifest + preservation asserts stay Bash-PARITY (unchanged
/// by c5). The gen-content assert's `docs/AGENT_MAP.yaml` line is the ONE
/// CORRECTNESS-oracle exception (P077c5, map-within-adopt inherits the OA-02
/// fix) — its golden line is hand-verified against the corrected Rust binary,
/// never auto-captured from Bash (see capture.sh's `freeze_adopt_gen` guard).
#[test]
fn parity_adopt_enforced() {
    assert!(
        PARITY_ENFORCED.contains(&"adopt"),
        "this test only makes sense while adopt is in PARITY_ENFORCED"
    );

    let (_tmp, kit, target) = build_adopt_fixture();
    let target_str = target.to_string_lossy().into_owned();
    let kit_str = kit.to_string_lossy().into_owned();

    // Preservation baseline — hash every seeded pre-existing file BEFORE adopt runs.
    let before: Vec<(&str, String)> =
        ADOPT_PRESERVED_SEEDS.iter().map(|f| (*f, sha256_hex(&target.join(f)))).collect();

    let raw = run_rust_with_kit_doctor_absent(&["adopt", &target_str, "--stack", "python"], &kit);

    // ---- Assert 1: stdout == adopt.golden ----
    let stdout_actual =
        strip_bare_date(&strip_timestamp(&normalize(&raw, &target_str))).replace(&kit_str, "<SOS_KIT_DIR>");
    let stdout_golden = read_golden("adopt.golden");
    assert_eq!(
        stdout_actual.trim(),
        stdout_golden.trim(),
        "adopt stdout mismatch vs adopt.golden (hard-fail — adopt is PARITY_ENFORCED). \
         NOTE: captured with DOCTOR_BIN=/nonexistent/doctor — a mismatch may mean \
         the doctor-absent skip branch didn't fire, not just a content regression."
    );

    // ---- Assert 2: tree-manifest == adopt.tree.golden ----
    let tree_actual = build_adopt_tree_manifest(&target);
    let tree_golden = read_golden("adopt.tree.golden");
    assert_eq!(
        tree_actual.trim(),
        tree_golden.trim(),
        "adopt file-tree manifest mismatch vs adopt.tree.golden (hard-fail — adopt is PARITY_ENFORCED)"
    );

    // ---- Assert 3: gen-content == adopt.gen.golden ----
    let gen_actual = build_adopt_gen_manifest(&target, &target_str);
    let gen_golden = read_golden("adopt.gen.golden");
    assert_eq!(
        gen_actual.trim(),
        gen_golden.trim(),
        "adopt gen-content manifest mismatch vs adopt.gen.golden (hard-fail — adopt is PARITY_ENFORCED)"
    );

    // ---- Assert 4: preservation invariant (NOT a golden — universal property) ----
    for (f, before_hash) in &before {
        let after_hash = sha256_hex(&target.join(f));
        assert_eq!(
            before_hash, &after_hash,
            "preservation-assert FAILED: pre-existing seed '{f}' changed during adopt \
             (non-clobber violated — adopt must NEVER overwrite an existing target file)"
        );
    }
    for rel in ADOPT_STAGED_COLLISIONS {
        let staged = target.join(".sos-adopt-incoming").join(rel);
        let kit_src = kit.join(rel);
        let staged_bytes =
            fs::read(&staged).unwrap_or_else(|e| panic!("preservation-assert: staged copy missing at {}: {e}", staged.display()));
        let kit_bytes = fs::read(&kit_src).unwrap_or_else(|e| panic!("read kit source {}: {e}", kit_src.display()));
        assert_eq!(
            staged_bytes, kit_bytes,
            "preservation-assert FAILED: staged '.sos-adopt-incoming/{rel}' does not byte-match kit source \
             (non-clobber staging must copy the kit's version verbatim)"
        );
    }
}

fn sha256_hex(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    format!("{:x}", hasher.finalize())
}

/// Post-sync tree-manifest: `<verb> <relpath> <sha256>`, sorted — mirrors
/// `capture.sh`'s `freeze_sync_tree`. Order-independent by construction.
fn build_tree_manifest(target: &Path) -> String {
    let mut lines: Vec<String> = Vec::new();
    for (rel, verb) in [
        ("scripts/added.sh", "ADDED"),
        (".claude/agents/demo.md", "ADDED"),
        (".claude/skills/demo/SKILL.md", "ADDED"),
    ] {
        let p = target.join(rel);
        if p.is_file() {
            lines.push(format!("{verb} {rel} {}", sha256_hex(&p)));
        }
    }
    let updated = target.join("scripts/updated.sh");
    if updated.is_file() {
        lines.push(format!("UPDATED scripts/updated.sh {}", sha256_hex(&updated)));
    }
    let incoming = target.join(".sos-sync-incoming/scripts/flagged.sh");
    if incoming.is_file() {
        lines.push(format!(
            "INCOMING .sos-sync-incoming/scripts/flagged.sh {}",
            sha256_hex(&incoming)
        ));
    }
    lines.sort();
    lines.join("\n")
}

#[test]
fn parity_sync_enforced() {
    assert!(
        PARITY_ENFORCED.contains(&"sync"),
        "this test only makes sense while sync is in PARITY_ENFORCED"
    );

    let (_tmp, kit, target) = build_sync_fixture();
    let target_str = target.to_string_lossy().into_owned();
    let kit_str = kit.to_string_lossy().into_owned();

    let raw = run_rust_with_kit(&["sync", &target_str], &kit);
    let stdout_actual = normalize(&raw, &target_str).replace(&kit_str, "<SOS_KIT_DIR>");
    let stdout_golden = read_golden("sync.golden");
    assert_eq!(
        stdout_actual.trim(),
        stdout_golden.trim(),
        "sync stdout mismatch vs sync.golden (hard-fail — sync is PARITY_ENFORCED). \
         NOTE: Bash's spine `find` is UNSORTED (Debate Log Turn 1 finding) — a mismatch \
         here may be a traversal-order divergence, not just a logic bug; check ordering \
         before assuming a content regression."
    );

    let tree_actual = build_tree_manifest(&target);
    let tree_golden = read_golden("sync.tree.golden");
    assert_eq!(
        tree_actual.trim(),
        tree_golden.trim(),
        "sync file-tree manifest mismatch vs sync.tree.golden (hard-fail — sync is PARITY_ENFORCED)"
    );
}

/// Build the same fixture `capture.sh` uses for `map` (routes/ + models/
/// files) so the harness diffs against the exact same input Bash was frozen
/// against.
fn build_map_fixture() -> (TempFixture, PathBuf) {
    let tmp = TempFixture::new("map");
    let target = tmp.path().join("map-fixture");
    fs::create_dir_all(target.join("src/routes")).unwrap();
    fs::create_dir_all(target.join("src/models")).unwrap();
    fs::write(target.join("src/routes/api.py"), "def h(): pass\n").unwrap();
    fs::write(target.join("src/models/user.py"), "class M: pass\n").unwrap();
    (tmp, target)
}

/// CORRECTNESS oracle (P077c5) — asserts Rust `map` output against a
/// hand-authored/frozen-from-corrected-Rust golden, NOT against Bash (Bash
/// `sos_map` still carries OA-02 by design). See capture.sh's map-section
/// guard comment for why this golden is never auto-regenerated from Bash.
#[test]
fn parity_map_enforced() {
    assert!(
        PARITY_ENFORCED.contains(&"map"),
        "this test only makes sense while map is in PARITY_ENFORCED"
    );

    let (_tmp, target) = build_map_fixture();
    let target_str = target.to_string_lossy().into_owned();

    let stdout_actual = normalize(&run_rust(&["map", &target_str]), &target_str);
    let stdout_golden = read_golden("map.golden");
    assert_eq!(
        stdout_actual.trim(),
        stdout_golden.trim(),
        "map stdout mismatch vs map.golden (hard-fail — map is PARITY_ENFORCED)"
    );

    let file_path = target.join("docs/AGENT_MAP.yaml");
    let file_raw = fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Rust map did not write {}: {e}", file_path.display()));
    let file_actual = normalize(&file_raw, &target_str);
    let file_golden = read_golden("map.agent_map.golden");
    assert_eq!(
        file_actual.trim(),
        file_golden.trim(),
        "map AGENT_MAP.yaml content mismatch vs map.agent_map.golden (hard-fail — map is PARITY_ENFORCED)"
    );
}

#[test]
fn parity_skeleton_informational() {
    let mut any_mismatch = false;

    for case in CASES {
        if PARITY_ENFORCED.contains(&case.name) {
            // Enforced commands have their own dedicated hard-fail test
            // (see parity_map_enforced) — this loop stays informational-only
            // for the commands that haven't reached parity yet.
            continue;
        }

        let golden = read_golden(case.golden);
        let actual = run_rust(case.args);

        if actual.trim() == golden.trim() {
            println!("P077a: {} — PARITY (Rust output matches Bash golden)", case.name);
        } else {
            any_mismatch = true;
            eprintln!(
                "P077a: {} not yet parity — Rust command unimplemented/differs from Bash golden ({})",
                case.name, case.golden
            );
        }
    }

    // Informational mode (P077a/b) for commands outside PARITY_ENFORCED:
    // always pass regardless of mismatch. Commands in PARITY_ENFORCED are
    // hard-failed by their own dedicated test above instead.
    let _ = any_mismatch;
}

// ---------------------------------------------------------------------------
// P077c5 — OA-02 acceptance fixtures (Task 6).
//
// These are the CORRECTNESS oracle's hard-fail proof, independent of the
// Bash-golden diffs above: they assert the specific properties OA-02's audit
// (`docs/retro/OUTSIDER_AUDIT_SYSTEM_GAPS_2026-07-22.md:127-130`) demanded,
// on synthetic self-contained fixtures (pattern reused from c1-c4).
// ---------------------------------------------------------------------------

/// Parse `docs/AGENT_MAP.yaml`'s `edit:` list entries under a given
/// `<surface_name>:` block into a flat Vec<String> (test-only mini-parser —
/// good enough for the fixed shape `render_surface_block()` emits).
fn agent_map_surface_entries(yaml: &str, surface_name: &str) -> Option<Vec<String>> {
    let marker = format!("  {surface_name}:");
    let start = yaml.find(&marker)?;
    let rest = &yaml[start + marker.len()..];
    let mut entries = Vec::new();
    for line in rest.lines() {
        if line.starts_with("      - ") {
            entries.push(line.trim_start_matches("      - ").to_string());
        } else if line.starts_with("  ") && !line.starts_with("    ") {
            // Reached the next top-level surface block — stop.
            break;
        }
    }
    Some(entries)
}

/// Acceptance #1 (audit `:128`) — a standard Rust crate must map `src/main.rs`
/// (or another file under `src/**`) into a generic source surface. Regression
/// this guards: OA-02's original finding (Rust crate adopted, no source
/// surface emitted at all).
#[test]
fn oa02_rust_crate_maps_src_main() {
    let tmp = TempFixture::new("oa02-rust");
    let target = tmp.path().join("rust-crate-fixture");
    fs::create_dir_all(target.join("src")).unwrap();
    fs::write(target.join("Cargo.toml"), "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n").unwrap();
    fs::write(target.join("src/main.rs"), "fn main() {}\n").unwrap();

    let target_str = target.to_string_lossy().into_owned();
    let _stdout = run_rust(&["map", &target_str]);

    let map_content = fs::read_to_string(target.join("docs/AGENT_MAP.yaml"))
        .expect("Rust map did not write docs/AGENT_MAP.yaml");

    let entries = agent_map_surface_entries(&map_content, "rust_src")
        .unwrap_or_else(|| panic!("no `rust_src` surface emitted for a Cargo.toml+src/main.rs fixture:\n{map_content}"));
    assert!(
        entries.iter().any(|e| e == "src/main.rs"),
        "rust_src surface did not include src/main.rs (entries: {entries:?})"
    );
    assert_eq!(map_content.trim_start().find("status:").is_some(), true);
    assert!(
        map_content.contains("status: coverage_unknown"),
        "fresh scan must emit status: coverage_unknown (3-verdict, not draft_needs_review)"
    );
}

/// Acceptance #2 (audit `:129`) — a kit-managed `templates/` dir present in the
/// target must NOT be scanned into a `frontend` (or any) product surface.
/// Regression this guards: the exact OA-02 finding from the brownfield smoke
/// test (`frontend` surface pointing at freshly-adopted `templates/`).
#[test]
fn oa02_templates_excluded_from_frontend() {
    let tmp = TempFixture::new("oa02-templates");
    let target = tmp.path().join("templates-fixture");
    fs::create_dir_all(target.join("templates")).unwrap();
    fs::create_dir_all(target.join("src/routes")).unwrap();
    fs::write(target.join("templates/BACKLOG_template.md"), "# backlog\n").unwrap();
    fs::write(target.join("src/routes/api.py"), "def h(): pass\n").unwrap();

    let target_str = target.to_string_lossy().into_owned();
    let _stdout = run_rust(&["map", &target_str]);

    let map_content = fs::read_to_string(target.join("docs/AGENT_MAP.yaml"))
        .expect("Rust map did not write docs/AGENT_MAP.yaml");

    assert!(
        !map_content.contains("frontend:"),
        "kit-managed templates/ leaked into a frontend surface (OA-02 regression):\n{map_content}"
    );
    assert!(
        !map_content.contains("BACKLOG_template.md"),
        "kit-managed templates/BACKLOG_template.md appeared in ANY surface (OA-02 regression):\n{map_content}"
    );
    // Sanity: the fixture's real product code (routes_handlers) still maps —
    // proves the exclude-list only removes kit-managed roots, not everything.
    assert!(
        map_content.contains("routes_handlers:") && map_content.contains("src/routes/api.py"),
        "exclude-list over-blocked real product surfaces:\n{map_content}"
    );
}

// ---------------------------------------------------------------------------
// P086 — trust-baseline seed acceptance tests. Unlike the fixtures above
// (synthetic minimal fake-kit, no `scripts/trust-gate.sh`), these need the
// REAL sos-kit spine (full `hooks/pre-commit` [8/8] chain + `scripts/*.sh`
// guards) to prove the actual first-commit deadlock (P086 context) is fixed —
// same pattern `crates/sos-install/tests/install.rs` uses for its own
// end-to-end hook tests (`repo_root_hook_bytes` et al.).
// ---------------------------------------------------------------------------

fn real_kit_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn configure_git_identity(root: &Path) {
    run_git(&["config", "user.email", "sos-kit-fixture@example.com"], root);
    run_git(&["config", "user.name", "sos-kit fixture"], root);
}

/// Acceptance (a) — a fixture `sos new` repo's FIRST real `git commit` passes
/// all 8 hook phases with zero hand intervention (pristine, zero-seed — same
/// oracle pattern as `install_end_to_end_blocks_a_real_env_commit_after_fresh_install`).
/// Regression this guards: the exact P086 deadlock (`.sos-trust-baseline not
/// found` → BLOCKED) reproduced pre-fix on Linux dogfood 2026-07-24.
#[test]
fn new_first_commit_passes_all_hooks_zero_seed() {
    let tmp = TempFixture::new("p086-new-first-commit");
    let target = tmp.path().join("fresh");
    let target_str = target.to_string_lossy().into_owned();
    let kit = real_kit_root();

    let raw = run_rust_with_kit(&["new", &target_str, "--stack", "python"], &kit);
    assert!(
        raw.contains("trust baseline seeded"),
        "sos new must report trust baseline seeded (got: {raw})"
    );
    assert!(
        target.join(".sos-trust-baseline").is_file(),
        ".sos-trust-baseline must exist on disk after sos new"
    );

    configure_git_identity(&target);
    let commit = git_commit_test(&target, &["commit", "-m", "bootstrap"]);
    assert!(
        commit.status.success(),
        "first commit after `sos new` must pass all 8 hook phases zero-hand-fix (P086): stdout={}\nstderr={}",
        String::from_utf8_lossy(&commit.stdout),
        String::from_utf8_lossy(&commit.stderr)
    );
}

/// Acceptance (b) — adopting into a brownfield repo with the user's OWN
/// untracked file present must NOT stage that file (Luật chơi #3: adopt's
/// seed step is scoped to `added_paths`, never `git add -A`, on a brownfield
/// target that may carry the user's own work-in-flight).
#[test]
fn adopt_does_not_stage_users_untracked_file() {
    let tmp = TempFixture::new("p086-adopt-no-stage");
    let target = tmp.path().join("brown");
    fs::create_dir_all(&target).unwrap();
    run_git(&["init", "-q"], &target);
    configure_git_identity(&target);
    fs::write(target.join("README.md"), "hello\n").unwrap();
    run_git(&["add", "README.md"], &target);
    run_git(&["commit", "-q", "-m", "init brownfield"], &target);

    // The user's own work-in-flight — untracked, never staged by them.
    fs::write(target.join("my-scratch-notes.txt"), "wip, not ready\n").unwrap();

    let target_str = target.to_string_lossy().into_owned();
    let kit = real_kit_root();
    let raw = run_rust_with_kit(&["adopt", &target_str], &kit);
    let _ = raw;

    let status = Command::new("git")
        .args(["status", "--porcelain", "my-scratch-notes.txt"])
        .current_dir(&target)
        .output()
        .expect("git status must run");
    let out = String::from_utf8_lossy(&status.stdout);
    assert_eq!(
        out.trim(),
        "?? my-scratch-notes.txt",
        "adopt must leave the user's untracked file untouched/unstaged (Luật chơi #3), got: {out:?}"
    );
}

fn git_commit_test(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap_or_else(|e| panic!("failed to exec git {:?}: {e}", args))
}

/// Acceptance #3 (audit `:130`) — nested/monorepo packages (`sim/`, `tools/`,
/// `migrations/`, and a nested Rust sub-package) still produce their expected
/// surfaces. Regression this guards: stack-detection or surface-scan being
/// root-only and missing sub-package source.
#[test]
fn oa02_monorepo_nested_packages_mapped() {
    let tmp = TempFixture::new("oa02-monorepo");
    let target = tmp.path().join("monorepo-fixture");
    fs::create_dir_all(target.join("migrations")).unwrap();
    fs::create_dir_all(target.join("packages/core/src")).unwrap();
    fs::write(target.join("migrations/0001_init.sql"), "-- init\n").unwrap();
    fs::write(target.join("packages/core/Cargo.toml"), "[package]\nname = \"core\"\nversion = \"0.1.0\"\n").unwrap();
    fs::write(target.join("packages/core/src/lib.rs"), "pub fn hi() {}\n").unwrap();

    let target_str = target.to_string_lossy().into_owned();
    let _stdout = run_rust(&["map", &target_str]);

    let map_content = fs::read_to_string(target.join("docs/AGENT_MAP.yaml"))
        .expect("Rust map did not write docs/AGENT_MAP.yaml");

    assert!(
        map_content.contains("migrations:"),
        "nested migrations/ dir not detected as a migrations surface:\n{map_content}"
    );
    let rust_entries = agent_map_surface_entries(&map_content, "rust_src")
        .unwrap_or_else(|| panic!("nested Cargo.toml sub-package did not trigger rust_src surface:\n{map_content}"));
    assert!(
        rust_entries.iter().any(|e| e.ends_with("packages/core/src/lib.rs")),
        "rust_src surface missing the nested sub-package's src/lib.rs (entries: {rust_entries:?})"
    );
}
