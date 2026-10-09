# Security review — main..p088-windows-checkout-layer (P087 88381b1 + P088 f9647bf)

Date: 2026-07-24 · Mode: branch/range (advisory, no PR) · Reviewer: Giám sát (boundary-check)

<!-- security-review-start -->
Security Review (ADVISORY — không block merge)

INV-1 (env var → env template update): PASS — no new env-var reads in `+` lines (`SOS_KIT_DIR` references in adopt.rs/new.rs are pre-existing; only comments/labels touch it). N/A.
INV-2 (external service → timeout + error handling): PASS — no HTTP/network calls added; new `Command::new("git")` invocations are local shell-outs with existing `.status()` error tolerance. N/A.
INV-3 (cross-user resource → ownership binding): PASS — no API routes/handlers or user-scoped data; single-user CLI + repo scripts. N/A.
INV-4 (webhook → signature verify + replay protection): PASS — no webhook handlers or signature-header access anywhere in the diff. N/A.
INV-5 (dependency major bump → changelog/migration audit): PASS — no manifest version-bump pairs in the diff (no Cargo.toml/package manifest touched). N/A.
INV-LOCAL-1 (install.sh release-asset integrity): PASS — `install.sh` untouched in this range; its baseline hash (63813f0d…) unchanged, `.sha256` verify block intact.

Special-attention 1 (trust-gate `hash_file()` normalization): PASS — symmetric single-point fix (wrapper feeds both generate `:202` and compare `:239`; baseline written via it never contains `*`, covering the raw `awk '{print $NF}'` read at `:254`); hash field untouched so same-content→same-line and tamper→detected both hold; `*`-strip on path field only, no aliasing bypass against the fixed `git ls-files` surface set; whitespace-path truncation is a pre-existing `$NF` assumption, not worsened.
Special-attention 2 (`.sos-trust-baseline` rebaseline): FLAG — set integrity OK (same 23 paths, zero added/dropped, zero `*`-prefix), BUT the rewritten hashes for `.claude/settings.json`/`.mcp.json` encode CRLF working-tree bytes: committed blobs are LF and unchanged since the Linux-seeded P086 baseline (`git diff 068fdd5 main -- <both>` empty), so correct LF hashes = the OLD values; P087 rebaselined on Windows pre-LF-forcing, and P088 (which forces LF at checkout everywhere) never re-rebaselined → every fresh clone on any platform false-BLOCKs at hook [8/8] on first commit (fail-closed, no bypass — but reintroduces the P086 deadlock class for 2 surfaces and trains rebaseline rubber-stamping). Fix: rebaseline on an LF-materialized checkout; the two lines should revert to `cd0cb9c4…`/`5e75a6a7…`.
Special-attention 3 (`.gitattributes` EOL policy): FLAG (residual gap) — LF-forcing strengthens baseline stability (all 23 hashed surfaces now glob-covered) and targeted-per-glob is sound, BUT `.sos-trust-baseline` itself (extensionless) matches no rule: fresh Windows `autocrlf=true` clones check it out CRLF and the byte-exact `diff` at trust-gate.sh:249 then reports every line changed → Windows fresh-clone false-BLOCK class not fully closed. Add `.sos-trust-baseline text eol=lf`.

Verdict: NEEDS_REVIEW (>=1 FLAG)
<!-- security-review-end -->
