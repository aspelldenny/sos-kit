# Golden oracle + parity harness (P077a)

`bin/sos.sh` is the canonical ORACLE for `new`/`adopt`/`map`/`sync` until P077e cutover.
This directory freezes its output so Rust's parity work (P077b–d) has something to
verify against — additive only, `bin/sos.sh` itself is never edited here.

## Layout

- `golden/capture.sh` — runs the 4 oracle-critical subcommands against throwaway
  fixture repos, normalizes non-deterministic bits, writes `golden/*.golden`.
- `golden/*.golden` — committed, frozen reference output (normalized).
- `parity.rs` — integration test: runs the Rust binary for the same 4 subcommands,
  diffs vs `golden/*.golden`. **Per-command hard-fail (P077c1):** `PARITY_ENFORCED:
  &[&str]` lists commands proven bug-for-bug identical to Bash (currently
  `["map", "sync", "new"]`); those get a dedicated hard-fail test. Commands NOT in the list stay
  **informational** — `parity_skeleton_informational` prints "not yet parity" for
  them but always passes (Rust doesn't implement them yet). Add a command's name to
  `PARITY_ENFORCED` once its golden(s) match — no other harness rewrite needed.

## Reproducing the fixtures

```bash
SOS_KIT_DIR=/path/to/sos-kit bash tests/golden/capture.sh /tmp/out
diff /tmp/out/new.golden tests/golden/new.golden     # (etc. for adopt/map/sync)
```

Two independent runs of `capture.sh` on 2026-07-22 (same sos-kit HEAD) produced
byte-identical normalized output for all 4 commands (`diff -r` clean) — this is the
determinism evidence referenced in `docs/discoveries/P077a.md`.

## Normalization rules (why they exist)

`bin/sos.sh`'s stdout for these 4 subcommands contains 3 categories of
non-deterministic content that would make a naive fixture flaky:

1. **Absolute paths** — the fixture target dir and `$SOS_KIT_DIR` both get printed
   verbatim (e.g. `sos new — bootstrap '/tmp/.../new-fixture'`). Normalized to
   `<TARGET>` / `<SOS_KIT_DIR>`.
2. **Dates** — `sos adopt`'s "Heads-up for your FIRST commit" hint embeds
   `$(date -u +%Y-%m-%d)` (see `bin/sos.sh` CHANGELOG-hint lines). Normalized to
   `<DATE>` (regex `[0-9]{4}-[0-9]{2}-[0-9]{2}`).
3. **Filesystem-enumeration order** — `sos new`'s "Category C placeholders to fill
   (# TODO)" list is built via `grep -rl "# TODO" ...` (`bin/sos.sh:583`), which is
   **not** sorted — file order depends on directory-entry order, which can differ
   across filesystems/runs even with identical content. `capture.sh`'s
   `sort_todo_block` normalizes just that block before freezing/diffing.

`sos map`'s surface lists are already sorted at the source (`scan_files()` pipes
through `sort`, `bin/sos.sh:296`) — no extra normalization needed there.

## `sos map` — two-fixture oracle (P077c1)

`sos_map` (`bin/sos.sh:279-352`) has TWO work-products, not one: it writes the real
scanned surfaces to a **file** (`<target>/docs/AGENT_MAP.yaml`) and only echoes a
1-line confirmation to **stdout**. `map.golden` (123b) freezes just that stdout line
— it contains no surface data at all. A parity harness that only diffs stdout is
therefore **blind to scan-correctness** (pattern-list, sort, OA-02 bug reproduction)
— a false-green class this harness exists to prevent (found during Worker CHALLENGE
Turn 1 on P077c1, see `docs/ticket/P077c1-rust-map-parity.md` Debate Log).

P077c1 fixes this additively:
- `capture.sh`'s `sos_map` branch now ALSO freezes the written file's content
  (normalized the same way) to `golden/map.agent_map.golden`.
- `parity.rs`'s `parity_map_enforced` test asserts BOTH `map.golden` (stdout) AND
  `map.agent_map.golden` (file content) — either mismatch hard-fails.
- The file content for this fixture has **zero non-deterministic bits** beyond the
  already-sorted relative paths (no embedded target path, no date) — normalize is a
  no-op pass-through, kept for consistency/future-proofing.

Future commands whose real work-product is a file rather than stdout (c3 `new`
generates files; c4 `adopt` writes onboarding files) should use this same two-fixture
pattern rather than relying on stdout-only parity (flagged as a feed-forward note in
`docs/plans/P077c-decomposition.md`).

## `sos sync` — synthetic fake-kit fixture (P077c2, two-fixture oracle)

`sos_sync`'s classification (ADDED / UPDATED-take-newer / FLAGGED-customized) is
**semantically** dependent on the kit's own git history — `_blob_in_history()`
(`bin/sos.sh:992-999`) walks `git -C "$SOS_KIT_DIR" rev-list --all -- <path>` and
checks whether the destination file's blob hash matches ANY historical blob of the
canonical path.

**P077a's original `sync.golden` pinned to real sos-kit HEAD** (a 0-change,
all-already-current scenario against a fresh `sos adopt` target) — this had two
problems, found during P077c2 CHALLENGE (Debate Log Turn 1): (1) it was **stdout-only
and 0-change**, exercising none of the ADDED/UPDATED/FLAGGED file-side-effect logic
(same false-green class OA-02 that P077c1 already fixed for `map`); (2) it drifted
every time sos-kit's own history advanced (real-HEAD-pin).

**P077c2 re-froze `sync.golden` against a SYNTHETIC self-contained fake-kit** instead
— `capture.sh`'s `build_fake_kit()` does a real `git init` + 2 commits (blob
`v1` → `v2` for one spine file) inside a throwaway dir, and `SOS_KIT_DIR` is pointed
at that fake-kit (NOT real sos-kit) when capturing/testing `sync`. This:

- **Eliminates the real-HEAD dependency entirely** — the fixture's own git history is
  frozen and self-contained; re-running `capture.sh` reproduces byte-identical output
  regardless of what commit real sos-kit is at. Verified: 2 independent captures on
  2026-07-22 produced byte-identical `sync.golden` + `sync.tree.golden`.
- **Exercises all 4 outcomes** in one deterministic scenario: `scripts/added.sh`
  (absent at target → ADDED), `scripts/updated.sh` (target has the fake-kit's
  historical `v1` blob, HEAD is `v2` → UPDATED take-newer), `scripts/flagged.sh`
  (target content matches no historical blob → FLAGGED into
  `.sos-sync-incoming/`), `scripts/current.sh` (target byte-identical to fake-kit
  HEAD → ALREADY-CURRENT). Also touches the `agents/*.md` → `.claude/agents/` and
  `skills/**/SKILL.md` → `.claude/skills/` walk branches (all ADDED), so the fixture
  isn't limited to the `scripts/` root.
- **Two-fixture oracle** (same pattern as `map`, applied to sync's real work-product
  = files on disk, not just the stdout summary): `sync.golden` freezes stdout;
  `sync.tree.golden` (new, P077c2) freezes a SORTED manifest of every mutated path
  (`<verb> <relpath> <sha256>`, verb ∈ `ADDED`/`UPDATED`/`INCOMING`) — this is what
  actually verifies file placement + content (e.g. that `.sos-sync-incoming/<rel>`
  really contains the kit's source bytes), which stdout counts alone cannot prove.
  `parity.rs`'s `parity_sync_enforced` hard-fails on either mismatch.

**Traversal-order finding (Debate Log Turn 1, load-bearing for future re-freezes):**
Bash's spine `find` (`bin/sos.sh:1030-1031,1043-1044`) is **UNSORTED** — it returns
raw filesystem directory-entry (creation) order, unlike `map`'s Bash `scan_files()`
which explicitly pipes through `sort`. This means `sync.golden`'s ADDED/UPDATED/
FLAGGED line order is filesystem-enumeration-order dependent, not alphabetical.
Rust's `sync.rs` therefore does **NOT** sort its walk (unlike `map.rs`'s
`hits.sort()`) — it preserves raw `WalkDir`/`read_dir` order to match. This held
bit-exact on the same machine (macOS/APFS) for the committed fixture; cross-platform
(e.g. a future Linux CI runner) order-match is unverified but not yet load-bearing —
no CI currently wires `cargo test` for `bootstrap/sos-rs`. `sync.tree.golden` is
SORTED at the freeze/assert layer specifically so it stays order-independent
regardless of this risk — only the stdout golden is exposed to it.

## `sos new` — synthetic fake-kit + doctor-absent (P077c3, three-fixture oracle)

`sos_new`'s real work-product is neither its 1-line stdout confirmation NOR a single
file — it's an entire freshly-bootstrapped repo tree, mixing COPIED kit assets
(verbatim, identical-by-construction for both Bash and Rust since both read the same
`$SOS_KIT_DIR`) with GENERATED-authored content (heredocs `sos_new` writes itself:
`.mcp.json`, `docs/security/INVARIANTS.md`'s appended block, `docs/AGENT_MAP.yaml`
stub, `CLAUDE.md`, `docs/ARCHITECTURE.md`, `CHANGELOG.md`, the stack manifest,
`.docs-gate.toml`, `.sos-stack.toml`).

**Why NOT hash the whole tree:** a full-content-hash fixture would (a) be large and
(b) **couple this fixture to every future kit-asset content change** — worse than
`sync`'s coupling risk, since `new` copies almost the entire kit. **Resolution:**
3-layer fixture, split by what each layer can prove without over-coupling:

- `new.golden` — stdout report only.
- `new.tree.golden` — **path-shape manifest**: every relpath under the bootstrapped
  target, sorted, **NO content**, excludes `.git/`. Proves Rust creates the exact
  same file/dir SET as Bash (catches a missing/extra `cp`), without caring what's
  inside a copied kit asset.
- `new.gen.golden` — **content-hash manifest**, `<relpath> <sha256>` sorted, but
  **ONLY for the GENERATED-authored files listed above** — copied kit assets are
  deliberately excluded (tree-shape already covers them; hashing them would recouple
  this fixture to unrelated kit content edits).

**Synthetic fake-kit, simpler than `sync`'s:** `new` only COPIES kit assets
verbatim — it never reads kit git history — so its fake-kit is a **plain directory
tree** (`build_fake_kit_new()` / `capture.sh`'s function of the same name), no
`git init` needed. It contains one minimal placeholder file per path `sos_new`
reads, EXCEPT `scripts/install-hooks.sh`, which is copied from the REAL sos-kit
checkout verbatim — that one script actually **runs** during `new`'s git-init step
(`git config core.hooksPath` etc.), so it must be functional; its content is never
gen-hashed or tree-differentiated by content, only by presence, so reusing the real
script doesn't recreate kit-content-coupling.

**Doctor-absent lever (CRITICAL finding, Worker CHALLENGE Turn 1):** the PREVIOUS
`new.golden` (P077a) had been captured with a real `doctor` binary present on the
capturing machine's `PATH` — its `[4/4]` block showed the full CONNECTED
`[WIRED] J1..J6` boundary-check output. This was a **host artifact, not a design
choice** — `sos_new`'s `[4/4]` step honors `DOCTOR_BIN` (default `doctor`) and only
takes the skip branch (`⏭ doctor not found — skip verify-setup...`) when the binary
can't be found. P077c3 **re-froze `new.golden`** by forcing
`DOCTOR_BIN=/nonexistent/doctor` during capture (`run_isolated_kit_doctor_absent`),
making the skip branch fire deterministically regardless of what's installed on
whatever machine runs `capture.sh` or the test suite. The CONNECTED path (real
`doctor` binary) is intentionally OUT of parity scope — it would need a pinned
`doctor` version, a separate concern from `new`'s own copy/generate/dispatch logic.

**Normalization gotchas found during P077c3 (load-bearing for future re-freezes):**

- **Full ISO-8601 timestamps** — `sos_init_security`'s `.sos-stack.toml` embeds
  `detected_at = "<ts>"` where `<ts>` is a FULL `date -u +%Y-%m-%dT%H:%M:%SZ`
  timestamp (time-of-day included), not just a bare date. Applying the pre-existing
  bare-date normalize rule FIRST left a half-stripped `<DATE>T14:23:01Z` — still
  non-deterministic. Fix: `strip_timestamp()` (full ISO-8601 → `<TIMESTAMP>`) MUST
  run **before** `normalize()`'s bare-date rule, both in `capture.sh` and in
  `parity.rs`'s Rust-side equivalent (`strip_timestamp`/`strip_bare_date`).
- **Locale-dependent `sort`** — a bare `sort` on macOS/Linux under a non-C locale can
  order `"claude"` before `"INVARIANTS"` (case-insensitive-ish collation), which then
  mismatches Rust's plain byte-order `Vec<String>::sort()`. Both `freeze_new_tree`
  and `freeze_new_gen` in `capture.sh` now pin `LC_ALL=C sort` so the fixture is
  locale-independent and matches Rust's byte-order sort exactly.
- **Filesystem-enumeration-order TODO list** — same underlying issue as the
  pre-existing `sort_todo_block` normalizer (see above), now also pinned to
  `LC_ALL=C sort` inside the awk pipeline for the same locale reason. `new.rs`
  itself sorts its TODO-file list directly (see module doc comment in `new.rs`) so
  its raw output already matches the canonical (sorted) golden with no extra
  harness-side reordering needed.

**Negative tests (P077c3):** (1) sabotaged `new.rs` to skip copying `phieu/` →
`parity_new_enforced` failed specifically on the **tree-manifest** assert (stdout
unaffected). (2) sabotaged one authored heredoc (`CLAUDE.md`'s Rules section) →
failed specifically on the **gen-content** assert (tree unaffected). Both reverted,
re-green — proof the 3-layer split isn't redundant, each layer catches a distinct
regression class.

## `sos adopt` — brownfield 4-collision fixture + preservation-assert (P077c4, four-layer oracle)

`sos_adopt`'s real work-product is the OPPOSITE discipline of `new`'s: it must
**RESPECT** whatever is already in the target repo (ADDITIVE + NON-CLOBBER), not
freely bootstrap a fresh tree. A stdout-only (or even 3-layer stdout+tree+gen)
fixture would be blind to the ONE property that actually matters for `adopt` —
whether a pre-existing file in the target ever gets overwritten. P077c4 adds a
4th layer specifically for this:

- `adopt.golden` — stdout report (**re-froze**, same host-artifact class as
  pre-c3 `new.golden`: the PREVIOUS golden had been captured with a real
  `doctor` on `PATH`, showing the CONNECTED `[WIRED] J1..J6` + `validate-map:
  paths resolve` block — not a deliberate capture choice. Re-froze under
  `DOCTOR_BIN=/nonexistent/doctor`, confirmed the skip branch fires
  deterministically).
- `adopt.tree.golden` — sorted path-shape manifest, excludes `.git*` (same
  `.git`-prefix filter as `new`'s/`sync`'s tree builders — this ALSO excludes
  `.gitignore`, an inherited quirk from the shared filter, not new to c4),
  **INCLUDES `.sos-adopt-incoming/**`** (the staged-collision directory is
  itself part of adopt's real work-product and must be tree-verified).
- `adopt.gen.golden` — content-hash manifest, GENERATED-authored files ONLY
  (`.mcp.json`, `docs/security/INVARIANTS.md`, `docs/AGENT_MAP.yaml`,
  `.docs-gate.toml`, `CHANGELOG.md`, `docs/ARCHITECTURE.md`, `.gitignore`,
  `.sos-stack.toml`) — copied/staged kit assets are tree-only, never hashed
  (kit-content-coupling guard, same rationale as `new`'s `NEW_GEN_FILES`).
- **preservation-assert** (in-test, `parity_adopt_enforced`, NOT a golden file)
  — the universal non-clobber property both Bash and Rust must hold,
  independent of what stdout/tree/gen happen to say: (i) every seeded
  pre-existing file's sha256 is IDENTICAL before vs. after `adopt` runs, (ii)
  every `.sos-adopt-incoming/<path>` byte-matches the kit source it was staged
  from. This is stronger than freezing "file X unchanged" into a golden — it's
  a property test against the fixture's OWN pre-adopt state, immune to
  content-coupling.

**Brownfield fixture, 4 collision cases** (`build_adopt_fixture()` /
`capture.sh`'s `build_adopt_brownfield()`) — `adopt` refuses an empty target
(bin/sos.sh:629-631, "use sos new instead"), so the fixture must seed a
non-empty repo exercising every non-clobber branch in one deterministic scenario:

| Case | Seed | adopt does | Layer that proves it |
|---|---|---|---|
| (a) spine ABSENT | (no seed) | copy → ADDED | tree (new path appears) |
| (b) spine COLLISION | `templates/INVARIANTS-template.md`, custom content | STAGE to `.sos-adopt-incoming/<path>`, target file UNTOUCHED | tree (staged path) + preservation-assert (i) target unchanged + (ii) staged byte-matches kit |
| (c) Cat-C doc EXISTING | `CHANGELOG.md` | generate SKIPS (never overwrite) | preservation-assert (i) unchanged |
| (d) source non-spine | `src/routes/api.py` + `src/models/user.py` + `pyproject.toml` | untouched, but SCANNED by map-within-adopt (OA-02 material) | preservation-assert (i) unchanged; gen (AGENT_MAP.yaml content) |

Fake-kit = `new`'s fake-kit shape (`build_new_fixture()`) PLUS
`templates/claude-settings.local.json` — **2nd fixture gotcha found during
Worker CHALLENGE Turn 1's live-probe** (adopt's `[1/4]`/`[3/4]` error without
it: `.claude/settings.local.json` copy-if-absent reads that template, and
`scripts/install-hooks.sh` must be the REAL, executable script since born-wire
actually runs it). Target committed clean (`git init` + `add -A` + `commit`)
so the dirty-warn branch does NOT fire (bin/sos.sh:642-646).

**Bug-for-bug divergences kept ON PURPOSE (P077c5 fixes both):**
- **OA-02** — `[2/4]` calls into `map`'s scan logic (via a subprocess
  re-invocation of the SAME compiled `sos` binary — `run_map_subcommand()` in
  `adopt.rs` — output fully discarded, matching Bash's `sos_map "$target"
  >/dev/null`; this reuses `map.rs`'s logic WITHOUT modifying `map.rs`) AFTER
  `[1/4]` has already copied kit assets into the target. `AGENT_MAP.yaml`
  therefore maps those kit assets too (e.g. `frontend` surface picking up the
  freshly-copied `templates/` dir) — confirmed reproducing live during
  CHALLENGE (byte-identical AGENT_MAP.yaml across 2 independent live runs, so
  this is a deterministic bug, not flaky nondeterminism).
- **Non-clobber list order** — `added`/`conflicts` in the stdout report follow
  raw filesystem/`find`-enumeration order (bin/sos.sh:661/679/708/717 have no
  `| sort`), NOT alphabetical — same divergence class as `sync`'s spine walk
  (see that section above). Confirmed same-platform deterministic via a live
  2-run probe during CHALLENGE (byte-identical stdout, tree, and AGENT_MAP.yaml
  across both runs); cross-platform order-match remains unverified (documented
  residual risk, not yet load-bearing — no CI wires `bootstrap/sos-rs` today).

**`.mcp.json` / `.claude/settings.local.json` exists-branch (jq-merge):** shells
out to the real `jq` binary (bug-for-bug — no new JSON-merge crate dependency
added). The parity fixture's brownfield target never seeds these 2 files, so
it always takes the create-if-absent branch — the jq-merge branch is
implemented for real-world correctness but is **out of hard-fail parity scope**
(phiếu Constraint 7; covering it would require a jq-dependent fixture branch,
a design expansion left for a future phiếu if needed).

**Negative tests (P077c4), one per layer, each fired independently:**
1. **Tree** — sabotaged `adopt_item`'s directory-copy branch to skip the actual
   `fs::copy` for `phieu/README.md` while still pushing the "+" line to
   `added` (i.e. stdout LIES that it copied) → failed specifically on the
   **tree-manifest** assert; stdout was unaffected (proves tree catches a
   report/reality mismatch that stdout alone can't).
2. **Gen** — sabotaged the `.docs-gate.toml` heredoc's `ticket_dir` value →
   failed specifically on the **gen-content** assert; tree/stdout unaffected.
3. **Preservation** — sabotaged the collision branch to copy the kit's version
   into BOTH `.sos-adopt-incoming/<path>` AND directly over the pre-existing
   target file (simulating the exact non-clobber violation this layer exists
   to catch) → failed specifically on the **preservation-assert**; stdout/
   tree/gen were all unaffected (same paths, same report line — proves this is
   the ONLY layer that can catch a real clobber-class regression). All 3
   reverted, `cargo test --workspace` returned to green after each.

## Flipping to hard-fail (P077c)

`parity.rs` used to have a single `const HARD_FAIL: bool = false;`. **P077c1 refactored
this to a per-command set**, `const PARITY_ENFORCED: &[&str] = &["map"];` — add a
command's name to enforce it once its golden(s) match; no other harness rewrite is
needed. **P077c2 added `"sync"`** (now `&["map", "sync"]`) with its own dedicated
`parity_sync_enforced` test (stdout + tree-manifest two-fixture assert, see above).
**P077c3 added `"new"`** (now `&["map", "sync", "new"]`) with its own
`parity_new_enforced` test (three-fixture assert: stdout + tree-shape + gen-content,
see "`sos new` — synthetic fake-kit + doctor-absent" above). **P077c4 added `"adopt"`**
(now `&["map", "sync", "new", "adopt"]`, 4/4 commands enforced — no command left
informational) with its own `parity_adopt_enforced` test (four-layer assert: stdout +
tree-shape + gen-content + preservation-assert, see "`sos adopt`" above).
