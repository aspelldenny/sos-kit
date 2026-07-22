# P078d decomposition — post-P079-dogfood fixes (Codex adapter render/enforcement)

> **Status:** P078d1 DRAFTED (2026-07-22) — startup-schema fixes. P078d2 DEFER (draft after d1 lands).
> **Parent:** P078 Codex native adapter (Active sprint) + `docs/adapters/P079-CODEX-DOGFOOD-FINDINGS-2026-07-22.md` (live-dogfood fix spec).
> **Depends:** P078b1/b2/b3 SHIPPED (Codex adapter foundation + render + enforcement); P079 dogfood RAN and produced 7 real bugs against `crates/sos-adapter-codex/src/templates.rs`.

## Why decompose (lean TÁCH)

P079 dogfood found **7 real bugs**, all cite `crates/sos-adapter-codex/src/templates.rs`, but they split into **two risk classes with two different review needs**:

- **Class A — STARTUP-BLOCKERS (3):** an untouched install literally cannot boot Codex 0.145.0. Pure render-format fixes; Codex gave the exact error + exact fix for each. Reversible, additive (templates.rs + tests only). Low blast if wrong (test catches). → **P078d1** (this decomposition's first phiếu).
- **Class B — ENFORCEMENT / SECURITY (4):** subagent marker lifecycle fail-open, approval bootstrap deadlock, multi-path `head -n1` guard bypass (a real security hole), spawn caveat. These touch **enforcement logic** (guards, approval gate, security boundary) → AUTO Tầng 1, security-critical, each needs its own CHALLENGE + care. → **P078d2** (defer, draft after d1 lands).

Bundling A+B = mixing "make it boot" (mechanical, oracle = format-parse) with "make it safe" (security logic, oracle = bypass/deadlock fixtures). Different oracles, different blast radius, different CHALLENGE depth. Splitting keeps each phiếu one-oracle + right-sized review.

## Sub-phiếu

| ID | Class | Deliverable | Oracle | Lane | Tầng | Dep |
|---|---|---|---|---|---|---|
| **P078d1** | A — startup | 3 render-format fixes in `templates.rs`: **#1** config.toml root settings (sandbox_mode/approval_policy) emit BEFORE `[agents]` table (TOML table-scope); **#2** rules `pattern` emit token LIST not string; **#3** hooks.json drop `_provenance`/`_partial_note`, fold into top-level `description`. + per-file schema-shape parse tests (revert→fail). templates.rs + tests only. | Rendered config.toml/rules/hooks.json parse via real `toml`/`serde_json` crate **+ Codex-0.145.0 schema-shape assert** + negative-test (revert fix → test FAIL) + cargo + ×20 + dep-direction | Guarded | 1 | P078b3 |
| **P078d2** | B — enforce/sec | **#4** SubagentStart/Stop set `.sos-state/{architect,worker}-active` (fix fail-open) · **#5** approval-gate self-init exemption (fix chicken-egg deadlock) · **#6** guards parse+check EVERY apply_patch path not just `head -n1` (fix security bypass) · **#7** AGENTS spawn caveat (full-history fork inherits parent agent_type) | Bypass + deadlock + fail-open fixtures; live-Codex spawn re-test for #7 | Guarded | 1 | P078d1 |

## P078d1 scope guard (do NOT pull in)

- CHỈ format-render: config.toml table order (#1) + rules list-pattern (#2) + hooks.json fields (#3), plus tests.
- **KHÔNG** touch enforcement logic — marker lifecycle (#4), approval bootstrap (#5), multi-path guard parsing (#6), spawn caveat (#7) = all d2. Even though #4/#6 also live in `templates.rs` (hook handler bodies), they are behavior, not format → d2.
- engine / install / core untouched. `sos-adapter-codex` other content-fns untouched.

## P078d2 items (defer detail — draft after d1)

Source lines from findings (`[needs Worker verify]` at d2 EXECUTE):
- **#4** `templates.rs:302` — SubagentStart/Stop hooks never create `.sos-state/architect-active` | `worker-active` → architect-guard + approval FAIL OPEN inside spawned agents. Main-thread guards worked; subagent marker lifecycle broken.
- **#5** `templates.rs:620` — approval-gate blocks ALL non-ticket patch when state-file missing, INCLUDING creating `.sos-state/ticket-state.env` itself → chicken-egg deadlock, no safe init.
- **#6** `templates.rs:379/481/538/606` — ALL guards extract only FIRST patch path (`head -n1`). Bypass: allowed-ticket path first + `.sos-state/ticket-state.env` (or `.env`/`src`) second → guard exits on first-path exemption, allows both. **SECURITY HOLE — must parse + check EVERY apply_patch path.**
- **#7** first custom-agent spawn failed: "full-history forked agents inherit parent agent type; omit agent_type or spawn without full-history fork" — AGENTS guidance missing this caveat.

## The structural-oracle gap (carry lesson into BOTH d1 and d2)

P078b2/b3 already shipped tests that assert **generic valid-TOML / valid-JSON** — and those tests PASSED while all 3 startup-blockers were live. Reason: a format-crate (`toml`, `serde_json`) parses the buggy output **without error** because the bug is **valid syntax, wrong Codex schema**:

- #1: root keys after `[agents]` are *legal TOML* (they just bind into the `[agents]` table) — `toml::from_str` succeeds; only Codex's `AgentRoleToml` deserialization rejects it.
- #2: `pattern = "…"` is *legal TOML* string; only Codex's rule schema wants a list.
- #3: `_provenance` is a *legal JSON* key; only Codex's hooks schema rejects unknown fields.

→ **Structural-valid ≠ Codex-accepts.** d1's new tests must assert **Codex-0.145.0-specific SHAPE** (root-level key presence, `pattern` is Array, hooks top-level keys ⊆ {description, hooks}) — an in-test approximation of Codex's schema. Even that approximation is hand-coded and can drift from Codex's real structs; **only live Codex 0.145.0 is ground truth.** Document this limitation in the phiếu's Discovery + `docs/adapters/` note. This is why P079 (live dogfood) caught what b2/b3's oracles could not.
