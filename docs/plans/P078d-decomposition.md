# P078d decomposition — post-P079-dogfood fixes (Codex adapter render/enforcement)

> **Status:** P078d1 DRAFTED (2026-07-22) — startup-schema fixes. **P078d2 SPLIT → d2a DRAFTED (2026-07-22) + d2b DEFER (needs SubagentStart probe).** **Round-2 usability → P078e (approval deadlock + actor-check) + P078f (install arm-hooks) — SPLIT 2026-07-23 per P078e Worker CHALLENGE O2.1.**
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

## Why split round-2 usability → P078e / P078f (2026-07-23, security-isolation)

P079 **round-2** live-dogfood (after d1/d2a/d2b PASS) found **2 new usability gaps** — initially bundled as P078e V1 (Task 1 approval deadlock + Task 2 `sos install` arm-hooks). Worker CHALLENGE on P078e V1 raised **2 Tầng-1 security objections** (both verified true):

- **[O1.1] gap#1 exemption thiếu actor-check:** V1's approval-exemption gated by **path-set only** (any-write when ticket-state.env alone), not by **actor**. Same `apply_patch` tool the Worker uses → Worker could write `approved_version=<version>` → **repeated worker self-approve** (d2a accepted a ONE-shot bootstrap chicken-egg; extending to every update = permanent self-approve). Multi-path #6 blocks bundled-code but does NOT block "worker writes ticket-state.env alone in a separate patch right after its code patch merged". → FIX must add **actor-check** (exempt only when `.sos-state/{worker,architect}-active` absent = main-thread), mirroring `orchestrator-guard.sh:596`. Stays in **P078e**.
- **[O2.1] gap#2 arm-hooks is heavier than "bare git config":** `scripts/install-hooks.sh` = F09 hijack-guard (TTY confirm / non-TTY abort when hooksPath is custom) + `chmod +x` + `.git/hooks/{pre-commit,pre-push}` rename-`.bak`. **Non-trivial security-arming** — hits the split-threshold P078e V1 itself declared (§Decomposition "if install-hooks.sh carries security-arming forcing a large port → split P078f"). → **split to P078f** (P085 heuristic #4 security-isolation).

**Result:** **P078e V2 = ONLY Task 1** (approval-exemption + actor-check + tests), 1 crate, security-isolated. **P078f = gap#2 arm-hooks** (install-flow, separate security surface). Splitting keeps the small actor-check deadlock fix landable immediately without dragging in the cross-platform install-arming port.

## Sub-phiếu

| ID | Class | Deliverable | Oracle | Lane | Tầng | Dep |
|---|---|---|---|---|---|---|
| **P078d1** | A — startup | 3 render-format fixes: **#1** config.toml root settings emit BEFORE first table; **#2** rules `pattern` token LIST; **#3** hooks.json drop `_provenance`/`_partial_note` → `description`. + schema-shape tests. | config.toml/hooks.json real-parse (`toml`/`serde_json`) + rules structural-string + negative-test | Guarded | 1 | P078b3 | 
| **P078d2a** | B — enforce/sec (specifiable now) | **#6** guards parse+check EVERY apply_patch path (block-if-any) — fix `head -n1` security bypass · **#5** approval-gate self-bootstrap exemption + install skeleton-state (fix chicken-egg deadlock) · **#7** AGENTS spawn caveat (full-history fork inherits parent agent_type) | Real-payload guard fixtures (b3 shape) — multi-path bypass BLOCK + single-path ALLOW; bootstrap-init ALLOW/BLOCK; negative-test. `[oracle: multi-path guard block-all-violating + bootstrap-safe + real-payload]` | Guarded | 1 | P078d1 |
| **P078d2b** | B — enforce (needs probe) | **#4** SubagentStart/Stop actually create `.sos-state/{architect,worker}-active` (fix marker fail-open) — after SubagentStart matcher/payload probe resolves what field the matcher matches on | Live-Codex spawn re-test (P079 round-2): spawn architect → forbidden Rust apply_patch must BLOCK; spawn worker → pre-approval STATE patch must BLOCK | Guarded | 1 | P078d2a + **Codex SubagentStart probe (Chủ nhà)** |
| **P078e** | Round-2 usability/sec | gap#1 approval **update-transition** exemption (create→any-write ticket-state.env alone) **+ actor-check** (exempt only main-thread: `.sos-state/{worker,architect}-active` absent, mirror `orchestrator-guard.sh:596`) — fixes deadlock + blocks worker self-approve. Codex caveat: in-subagent enforcement MISSING (#4/d2b) → actor-check best-effort on Codex (human-review-at-commit backstop), full on Claude. | `run_guard` mock-payload + marker fixture — update ALLOW (main-thread) / worker-marker-SET self-approve BLOCK / bundle BLOCK / pre-approval BLOCK + negative-test. `[oracle: actor-gated exemption + real-payload + marker-fixture]` | Normal (re-declared V2; Worker re-check at EXECUTE) | 1 | P078d2a (all-path #6 + bootstrap exemption) |
| **P078f** | Round-2 install-arm (security) | gap#2 `sos install --runtime {claude,codex}` arm Git hooks — port `scripts/install-hooks.sh` security-arming: **`core.hooksPath=hooks`** + **F09 hijack-guard** (TTY confirm / non-TTY abort when hooksPath already custom) + **`chmod +x` hooks** + **`.git/hooks/{pre-commit,pre-push}` rename-`.bak`**. Non-clobber, symmetric claude+codex, non-git-safe. Engine-native (Windows-portable) vs invoke script = P078f decides. Closes d2b honest-MISSING hole (Git backstop armed-by-default). | install-smoke temp-git-repo — assert `git config core.hooksPath`==`hooks` + hooks chmod'd + existing hooks `.bak`'d + non-clobber user-hooksPath + non-git warn-skip + negative-test. `[oracle: install-smoke temp-git assert git-config + hijack-guard behavior]` | Guarded (security-arming + install-flow LAN to every adopter) | 1 | P078e (or parallel — independent surface: install-engine vs adapter guard) |

## P078d2a scope guard (do NOT pull in)

- CHỈ: #6 multi-path guard parsing (all-path, block-if-any) + #5 approval self-bootstrap exemption + install skeleton-state (non-clobber) + #7 AGENTS caveat + tests.
- **KHÔNG** touch #4 marker lifecycle (`templates.rs:302/315-317`) = **d2b**, blocked on probe. Even though #4 also lives in `templates.rs`, its root cause (matcher semantics) is unknown → cannot spec.
- KHÔNG regress d1 startup render (config.toml/rules/hooks.json). engine/install-engine core/core/adapter-claude untouched (if skeleton-state emit turns out to belong to install-engine, confirm at EXECUTE + note scope-boundary in Discovery).

## P078e scope guard (do NOT pull in — set 2026-07-23)

- CHỈ: gap#1 approval update-transition exemption + actor-check + tests, in `crates/sos-adapter-codex/src/{templates.rs,lib.rs}` + docs (CAPABILITY/SECURITY/CHANGELOG/Discovery).
- **KHÔNG** touch install flow / arm-hooks (`crates/sos-install/**`, `crates/sos-cli/**`, `scripts/install-hooks.sh`) = **P078f**. Even though both are "round-2 usability", arm-hooks is a separate security surface (install-time hook-arming, cross-platform, F09 hijack-guard) with a different oracle (install-smoke temp-git vs guard mock-payload).
- **Marker-path coupling:** actor-check MUST read the same `.sos-state/{worker,architect}-active` path that d2b's SubagentStart hook touches — verify at EXECUTE (anchor #4). Codex in-subagent hook non-firing (#4/d2b) means actor-check is best-effort on Codex; do NOT claim full protection.

## P078f items (round-2 install-arm — draft/execute 2026-07-23+)

- gap#2 `sos install` does NOT arm Git hooks → `core.hooksPath` unset after install → Git boundary (pre-commit/pre-push) OFF-by-default, undermining d2b honest-MISSING story (which declares "in-subagent enforcement MISSING → rely on universal Git backstop" — but the backstop is off).
- **Scope = port `scripts/install-hooks.sh` security-arming** into `sos install` (engine `crates/sos-install/src/engine.rs` + CLI `crates/sos-cli/src/commands/install.rs`): `core.hooksPath=hooks` + F09 hijack-guard (TTY confirm / non-TTY abort when hooksPath already set to a custom value) + `chmod +x hooks/*` + rename any existing `.git/hooks/{pre-commit,pre-push}` to `.bak`. Non-clobber (never overwrite user's custom hooksPath), symmetric claude+codex, non-git-safe (warn-skip).
- **P078f decides:** engine-native (Windows-portable, reimplement the arming steps in Rust) vs invoke `install-hooks.sh` (portability risk — bash on Windows; precedent P059/P072 favor native). Worker reads `install-hooks.sh` in full first to enumerate every arming step, then chooses.
- **Independent from P078e** (different surface: install-engine vs adapter guard) — can run parallel or after.

## The structural-oracle gap (carry lesson into d1/d2a/d2b/e/f)

P078b2/b3 shipped tests that assert **generic valid-TOML / valid-JSON** — those PASSED while all 3 startup-blockers were live, because a format-crate parses the buggy output without error (bug = valid syntax, wrong Codex schema). d1's tests assert **Codex-0.145.0 SHAPE**; even so, only live Codex is ground truth. **Same lesson for d2a/d2b/e/f enforcement:** a guard that "runs" is not a guard that "blocks the real bypass" — d2a/e's oracle must feed a **REAL apply_patch multi-file payload** (+ marker fixture for e's actor-check) and assert the guard BLOCKS (not just parses); #4's marker fix (d2b) can only be validated by a **live spawn** (P079 round-2), because the failure mode was hook-firing/matcher semantics invisible to any render-time test; P078f's arm-hooks must assert **`git config` actually set + hooks executable + existing hooks backed up** in a temp git repo, not just "install ran". This is why P079 (live dogfood) caught what b2/b3's structural oracles could not.
