# P078d decomposition — post-P079-dogfood fixes (Codex adapter render/enforcement)

> **Status:** P078d1 DRAFTED (2026-07-22) — startup-schema fixes. **P078d2 SPLIT → d2a DRAFTED (2026-07-22) + d2b DEFER (needs SubagentStart probe).**
> **Parent:** P078 Codex native adapter (Active sprint) + `docs/adapters/P079-CODEX-DOGFOOD-FINDINGS-2026-07-22.md` (live-dogfood fix spec).
> **Depends:** P078b1/b2/b3 SHIPPED (Codex adapter foundation + render + enforcement); P079 dogfood RAN and produced 7 real bugs against `crates/sos-adapter-codex/src/templates.rs`.

## Why decompose (lean TÁCH)

P079 dogfood found **7 real bugs**, all cite `crates/sos-adapter-codex/src/templates.rs`, but they split into **risk classes with different review needs**:

- **Class A — STARTUP-BLOCKERS (3):** an untouched install literally cannot boot Codex 0.145.0. Pure render-format fixes; Codex gave the exact error + exact fix for each. Reversible, additive (templates.rs + tests only). Low blast if wrong (test catches). → **P078d1**.
- **Class B — ENFORCEMENT / SECURITY (4):** subagent marker lifecycle fail-open (#4), approval bootstrap deadlock (#5), multi-path `head -n1` guard bypass (#6, a real security hole), spawn caveat (#7). These touch **enforcement logic** → AUTO Tầng 1, security-critical, each needs its own CHALLENGE + care. → **P078d2**, further split into **d2a** (specifiable now) + **d2b** (needs Codex probe).

Bundling A+B = mixing "make it boot" (mechanical, oracle = format-parse) with "make it safe" (security logic, oracle = bypass/deadlock fixtures). Different oracles, different blast radius, different CHALLENGE depth.

## Why split P078d2 → d2a / d2b (probe dependency)

Of the 4 Class-B bugs, **3 are specifiable NGAY** (no Codex probe needed) and **1 needs a live SubagentStart probe**:

- **#5 / #6 / #7 specifiable now:** apply_patch multi-file format is known (Sếp b3 sample: multiple `*** (Add|Update|Delete|Move) File: <path>` lines); guards + approval-gate are shell scripts we control; #7 is a doc line. → **d2a**.
- **#4 (marker fail-open) needs probe:** the wiring is present (SubagentStart hooks `templates.rs:315-317` `"matcher":"architect"/"worker"` → `touch .sos-state/{architect,worker}-active`) but dogfood shows the marker is **NEVER created** → architect-guard + approval FAIL-OPEN inside spawned agents. Root cause hypothesis: **SubagentStart `matcher` semantics do NOT match agents by "architect"/"worker"** as assumed — unknown what field the matcher matches on. Cannot spec the fix without knowing the real SubagentStart payload/matcher contract of Codex 0.145.0. → **d2b, blocked on Codex probe** (same class as the apply_patch b3 probe). Orchestrator escalates Chủ nhà to run a SubagentStart probe.

**Lean TÁCH rationale:** #6 is a LIVE security hole → must patch SOON. If bundled with #4, the fix ships only after the probe returns — delaying the security patch on an unknown-latency dependency. Splitting lets d2a (security + bootstrap + caveat) land immediately; d2b (#4) follows once the probe lands. **d2a coupling bonus:** #5's self-bootstrap exemption is only safe because #6's all-path check lands in the same phiếu (all-path check blocks the "ticket-state.env + malicious path" combo) — so #5 and #6 belong together in d2a.

## Sub-phiếu

| ID | Class | Deliverable | Oracle | Lane | Tầng | Dep |
|---|---|---|---|---|---|---|
| **P078d1** | A — startup | 3 render-format fixes: **#1** config.toml root settings emit BEFORE first table; **#2** rules `pattern` token LIST; **#3** hooks.json drop `_provenance`/`_partial_note` → `description`. + schema-shape tests. | config.toml/hooks.json real-parse (`toml`/`serde_json`) + rules structural-string + negative-test | Guarded | 1 | P078b3 | 
| **P078d2a** | B — enforce/sec (specifiable now) | **#6** guards parse+check EVERY apply_patch path (block-if-any) — fix `head -n1` security bypass · **#5** approval-gate self-bootstrap exemption + install skeleton-state (fix chicken-egg deadlock) · **#7** AGENTS spawn caveat (full-history fork inherits parent agent_type) | Real-payload guard fixtures (b3 shape) — multi-path bypass BLOCK + single-path ALLOW; bootstrap-init ALLOW/BLOCK; negative-test. `[oracle: multi-path guard block-all-violating + bootstrap-safe + real-payload]` | Guarded | 1 | P078d1 |
| **P078d2b** | B — enforce (needs probe) | **#4** SubagentStart/Stop actually create `.sos-state/{architect,worker}-active` (fix marker fail-open) — after SubagentStart matcher/payload probe resolves what field the matcher matches on | Live-Codex spawn re-test (P079 round-2): spawn architect → forbidden Rust apply_patch must BLOCK; spawn worker → pre-approval STATE patch must BLOCK | Guarded | 1 | P078d2a + **Codex SubagentStart probe (Chủ nhà)** |

## P078d2a scope guard (do NOT pull in)

- CHỈ: #6 multi-path guard parsing (all-path, block-if-any) + #5 approval self-bootstrap exemption + install skeleton-state (non-clobber) + #7 AGENTS caveat + tests.
- **KHÔNG** touch #4 marker lifecycle (`templates.rs:302/315-317`) = **d2b**, blocked on probe. Even though #4 also lives in `templates.rs`, its root cause (matcher semantics) is unknown → cannot spec.
- KHÔNG regress d1 startup render (config.toml/rules/hooks.json). engine/install-engine core/core/adapter-claude untouched (if skeleton-state emit turns out to belong to install-engine, confirm at EXECUTE + note scope-boundary in Discovery).

## P078d2b items (defer — draft after probe + d2a lands)

- **#4** `templates.rs:302` (SubagentStart/Stop hooks) + `:315-317` (`matcher:"architect"/"worker"` → touch marker) — markers never created → architect-guard + approval FAIL OPEN inside spawned agents. **Blocked:** need Codex 0.145.0 SubagentStart payload/matcher contract (what field does matcher match on? agent name? agent_type? spawn arg?). Orchestrator escalates Chủ nhà to run a SubagentStart probe (like the apply_patch b3 probe). Once probe returns → draft d2b with the correct matcher + a live-spawn behavioral oracle (P079 round-2).

## The structural-oracle gap (carry lesson into d1/d2a/d2b)

P078b2/b3 shipped tests that assert **generic valid-TOML / valid-JSON** — those PASSED while all 3 startup-blockers were live, because a format-crate parses the buggy output without error (bug = valid syntax, wrong Codex schema). d1's tests assert **Codex-0.145.0 SHAPE**; even so, only live Codex is ground truth. **Same lesson for d2a/d2b enforcement:** a guard that "runs" is not a guard that "blocks the real bypass" — d2a's oracle must feed a **REAL apply_patch multi-file payload** and assert the guard BLOCKS (not just parses); #4's marker fix (d2b) can only be validated by a **live spawn** (P079 round-2), because the failure mode was hook-firing/matcher semantics invisible to any render-time test. This is why P079 (live dogfood) caught what b2/b3's structural oracles could not.
