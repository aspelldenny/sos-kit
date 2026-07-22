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
  &[&str]` lists commands proven bug-for-bug identical to Bash (currently `["map"]`);
  those get a dedicated hard-fail test. Commands NOT in the list stay
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

## `sos sync` — the HEAD-pin caveat (CHALLENGE Turn 1, anchor #7)

`sos_sync`'s classification (ADDED / UPDATED-take-newer / FLAGGED-customized) is not
just cosmetic-nondeterministic — it is **semantically** dependent on sos-kit's own
git history. `_blob_in_history()` (`bin/sos.sh:992-999`) walks
`git -C "$SOS_KIT_DIR" rev-list --all -- <path>` and checks whether the destination
file's blob hash matches ANY historical blob of the canonical path. This means:

- The `sync.golden` fixture in this repo was frozen against **sos-kit HEAD
  `d370c82f85a8458e6e23c7208a81054f20d4fba4`** (pre-P077a commit).
- Re-running `capture.sh` against a *later* sos-kit HEAD is only guaranteed to
  reproduce `sync.golden` byte-for-byte if the specific spine files exercised by the
  fixture (currently: whatever `bin/sos.sh` copies for a fresh `sos adopt --stack
  python` — see `capture.sh`'s `sync` section) have not themselves changed history in
  a way that flips a file between "unmodified stale" and "customized". For THIS
  fixture (empty adopt target, all-current sync), the risk is low (0 added/updated/
  flagged, 58 already-current) — but any future re-freeze of `sync.golden` MUST
  record the sos-kit HEAD sha alongside it, not just the fixture-repo state.
- **Do not** treat a future diff in `sync.golden` alone as a Rust-parity regression
  without first checking whether sos-kit's own history moved out from under the
  fixture. P077c (hard-fail cutover) should re-verify/re-pin this before flipping
  `HARD_FAIL`.

## Flipping to hard-fail (P077c)

`parity.rs` used to have a single `const HARD_FAIL: bool = false;`. **P077c1 refactored
this to a per-command set**, `const PARITY_ENFORCED: &[&str] = &["map"];` — add a
command's name to enforce it once its golden(s) match; no other harness rewrite is
needed (c2/c3/c4 reuse this mechanism for `sync`/`new`/`adopt`).
