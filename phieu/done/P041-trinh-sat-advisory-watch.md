# PHIẾU P041: Trinh sát (advisory-watch) generic agent

> **Loại:** Feature (new specialist subagent + new slash command + new inbox template + first parser implementation)
> **Ưu tiên:** P1 (wave 1 — security pipeline skeleton; P042 reads same `.sos-stack.toml` schema; user-visible `/advisory-scan` command)
> **Tầng:** 1 (móng nhà — new public subagent contract, new slash command surface, new file format `templates/advisory-inbox.md` sentinel schema; downstream phiếu / user projects depend on these interfaces)
> **Ảnh hưởng:** `agents/advisory-watch.md` (NEW), `templates/advisory-inbox.md` (NEW), `.claude/commands/advisory-scan.md` (NEW — first file in this dir for sos-kit), `scripts/parsers/pnpm_lock_v9.py` (implement, was stub), `scripts/parsers/package_lock_v3.py` (implement, was stub — if effort allows), `docs/LAYERS.md` (access matrix row), `docs/HANDOFF.md` (new handoff pattern note), `README.md`, `docs/SETUP.md` (Security subsection), `CLAUDE.md` (repo structure)
> **Dependency:** P040 SHIPPED — `.sos-stack.toml` schema + 6 parser stubs underscores. PR #11 / `8047525`.

---

## Context

### Vấn đề hiện tại

P040 đã ship foundation: `sos init security` detect stack + write `.sos-stack.toml` + drop 6 parser skeleton stubs returning `[]`. P041 next step trong wave 1: codify Trinh sát (advisory-watch) — specialist subagent đọc `.sos-stack.toml`, invoke parser, query GHSA + vendor advisory pages, return sentinel-wrapped rows cho caller append vào inbox.

Gap thực tế:
- Subagent file `agents/advisory-watch.md` chưa tồn tại trong sos-kit (Architect Glob 2026-05-25: `agents/` chỉ có `orchestrator.md` + `architect.md` + `worker.md` + `README.md`).
- `.claude/commands/` directory chưa tồn tại trong sos-kit (Architect Glob 2026-05-25: 0 files). P041 là file đầu tiên — must create directory.
- 6 parser stubs từ P040 vẫn return `[]` + TODO. Trinh sát cần ≥1 parser thực tế để demo workflow end-to-end. Priority = `pnpm_lock_v9.py` (Node ecosystem dominant; sos-kit `package.json` itself uses pnpm; tarot dogfood proved YAML 2-level layout fragility — phiếu codifies correct logic upfront).
- `templates/advisory-inbox.md` chưa tồn tại — user project cần queue template để `/advisory-scan` có chỗ append rows.
- Tarot source `~/tarot/.claude/agents/advisory-watch.md` chứa tarot-specific paths (`scripts/extract-pnpm-versions.py`, INV-107 10-deps critical list, `docs/security/advisory-inbox.md` hardcoded). Port về sos-kit phải strip generic-able.

### Giải pháp

5 deliverable cores + 1 doc consolidation:

1. **`agents/advisory-watch.md` (NEW)** — generic specialist subagent file. Frontmatter `tools: Read, Grep, Glob, WebFetch, WebSearch, Bash` (Bash is **scoped** to running parser scripts only — see Constraint #3 + agent body "Bash usage" section). Body: invocation contract (input: `.sos-stack.toml` path or implicit project root; output: sentinel-wrapped advisory rows block in final report). Persona name: **Trinh sát** (Vietnamese, matches `docs/LAYERS.md` persona convention + tarot description alias). Filename: English `advisory-watch.md` to match existing `architect.md` / `worker.md` / `orchestrator.md` pattern (English filenames, Vietnamese persona names in body).
2. **`templates/advisory-inbox.md` (NEW)** — empty queue template. Sentinel wrappers `<!-- advisory-start -->` / `<!-- advisory-end -->` mark the append region. 1 example row inside as schema reference (commented out or example status `dismissed` so it doesn't pollute real scans). User copies to `docs/security/advisory-inbox.md` in their project (or wherever they want — slash command will use a configurable path with sensible default).
3. **`.claude/commands/advisory-scan.md` (NEW)** — slash command file. **Orchestrator-side flow = spawn-only** (per Sếp REDESIGN 2026-05-25): (a) verify `.sos-stack.toml` exists, (b) spawn Trinh sát subagent via Task tool (no parser invocation in main session), (c) parse subagent's returned sentinel block, (d) append rows between `<!-- advisory-start -->` / `<!-- advisory-end -->` markers in inbox. **Parser invocation happens INSIDE Trinh sát subagent** (subagent has scoped Bash for parser scripts only). All inbox `Write` happens in main session (orchestrator), not subagent — subagent stays Write/Edit-free.
4. **`scripts/parsers/pnpm_lock_v9.py` (implement, replacing P040 stub)** — fill `parse(path: Path) -> list[dict]` per the contract documented in stub docstring (name / version / ecosystem / source keys). Logic: parse YAML 2-level layout (importers → . → dependencies/devDependencies → name → specifier+version object), strip peer-suffix `0.92.0(zod@4.3.6)` → `0.92.0`, return direct deps only (NOT transitive — leave transitive to Dependabot / future P0xx). Tarot dogfood P273 + P282 lessons encoded as comments in code.
5. **`scripts/parsers/package_lock_v3.py` (implement, replacing P040 stub) — IF effort allows.** If Worker dry-run hits time cap during EXECUTE, package-lock implementation can defer to a follow-on phiếu (note in Discovery Report). The OTHER 4 parsers (`requirements_txt.py`, `pyproject_toml.py`, `cargo_lock.py`, `go_sum.py`) remain stubs in P041 — phiếu MUST NOT touch them; they ship empty per P040 contract.
6. **Doc consolidation** — `docs/LAYERS.md` access matrix gains a Trinh sát row (specialist subagent, scoped Bash for parsers, no code edit). `docs/HANDOFF.md` mentions the new specialist-subagent handoff pattern (orchestrator → Trinh sát → orchestrator-appends-to-inbox; user gates per-row). `README.md` Components / Subagents table gains Trinh sát row. `docs/SETUP.md` gains "Security" subsection: `sos init security` → `/advisory-scan` → inbox flow. `CLAUDE.md` repo structure tree gains `.claude/commands/` mention + `templates/advisory-inbox.md` mention.

**Generic-able invariants** (the "strip tarot-specific" requirement):
- NO hardcoded package allowlist (no INV-107 10-deps list).
- NO `scripts/extract-pnpm-versions.py` reference — sos-kit parsers ARE the canonical extraction.
- NO `docs/security/advisory-inbox.md` hardcoded path — agent receives inbox path as input (or defaults).
- NO project-specific INV references (INV-102 nginx, INV-105 credits, etc. — all tarot artifacts).
- Sentinel marker rename: tarot uses `<!-- INBOX_APPEND_START/END -->` (caps + verb-form). sos-kit uses `<!-- advisory-start --> ... <!-- advisory-end -->` (lowercase + noun-form) per Sếp spec in user message. Markers are LOAD-BEARING — slash command grep matches them exactly.

**GHSA query strategy** (codified in agent body, not phiếu Constraints — agent owns the how):
- Primary source: GitHub Advisory Database (GHSA) per-package pages + ecosystem search filter (`https://github.com/advisories?query=ecosystem%3Anpm+<package>`).
- Vendor pages secondary (only nếu agent body lists them — generic kit ships GHSA primary, vendor optional).
- WebSearch tertiary (bound to `<dep> <version> CVE <year>` — NO generic queries).
- Severity sourcing rule: upstream official ONLY (GHSA reviewed badge, vendor official advisory page). NO third-party rescore inflation. Tarot P281 lesson encoded verbatim in agent body.
- OSV.dev DEFERRED — tarot P284 proved WebFetch is GET-only, OSV needs POST. Agent body explains deferral + future trigger (Bash tool addition).
- Rate-limit handling: GHSA serves HTML; agent fetches per-package not bulk. No explicit rate-limit code in P041 — defer to discovery if hit during dry-run.
- Cache TTL: NONE in P041 (no state file). State persistence (`.advisory-scan-state` JSON schema from tarot P282) DEFERRED to follow-on phiếu — adds complexity, not blocking first-version usefulness. Agent body notes deferral.

### Scope

- CHỈ sửa / tạo:
  - `agents/advisory-watch.md` — NEW specialist subagent file (≤180 lines target).
  - `templates/advisory-inbox.md` — NEW empty queue template with sentinel wrappers.
  - `.claude/commands/advisory-scan.md` — NEW slash command file (FIRST file in this dir for sos-kit — Worker creates directory).
  - `scripts/parsers/pnpm_lock_v9.py` — implement `parse()` (was stub returning `[]`).
  - `scripts/parsers/package_lock_v3.py` — implement `parse()` IF effort allows (Worker may defer with Discovery Report note).
  - `docs/LAYERS.md` — add Trinh sát row to access matrix; brief mention in 3-role intro (specialist subagents are not new roles, just read-only specialists).
  - `docs/HANDOFF.md` — add Handoff 5 OR appendix note for "specialist subagent ↔ orchestrator-mediated inbox" pattern.
  - `README.md` — Components/Subagents table row for Trinh sát; brief security-pipeline mention in Pipeline diagram or after Components section.
  - `docs/SETUP.md` — add "Security pipeline" subsection (after stack-detection mention from P040): `sos init security` → `/advisory-scan` → inbox flow. **Also add PyYAML install requirement** (Worker decides exact placement: SETUP.md "Security" subsection OR slash command pre-flight `python3 -c 'import yaml' || pip3 install pyyaml`).
  - `CLAUDE.md` — repo structure tree adds `.claude/commands/` + `templates/advisory-inbox.md` + `agents/advisory-watch.md` lines.
- KHÔNG sửa:
  - `agents/orchestrator.md`, `agents/architect.md`, `agents/worker.md` — main 3-role contracts unchanged. (Quản đốc's invocation of `/advisory-scan` is documented in the slash command file, not in orchestrator handbook.)
  - `phieu/TICKET_TEMPLATE.md` — phiếu format unchanged.
  - `bin/sos.sh` — `sos init security` from P040 already writes `.sos-stack.toml`; no new subcommand in P041.
  - `templates/.sos-stack.toml.example` — schema from P040 unchanged.
  - Other 4 parser stubs: `requirements_txt.py`, `pyproject_toml.py`, `cargo_lock.py`, `go_sum.py` — STAY empty stubs (P040 contract preserved; future phiếu fills them).
  - `hooks/pre-commit` — no commit gating on advisory inbox in P041.
  - `scripts/architect-guard.sh` — Architect-block on `.py` reads remains (Trinh sát is a separate subagent, not Architect; this hook does not affect Trinh sát's tools).
  - `bootstrap/sos-rs/` — Rust port out of scope (P033, Next sprint).
  - `docs/PHILOSOPHY.md`, `docs/ORCHESTRATION.md` — no edits; specialist subagent doesn't change 6 principles or state machine.
  - `phieu/DISCOVERY_PROTOCOL.md`, `phieu/RELAY_PROTOCOL.md`, `phieu/AUDIT_PROTOCOL.md` — protocols unchanged.
  - `docs/BACKLOG.md` — orchestrator handles move from Active → Recently shipped post-merge, not Worker mid-EXECUTE.

### Skills consulted (optional)

<!-- None for P041 — no `/frontend-design` / `/security-review` skill invoked by orchestrator before DRAFT. Trinh sát IS the security-review-adjacent skill conceptually but it's a subagent, not a Skill, and orchestrator did not freeze any skill output into this phiếu. Leave blank. -->

---

## Task 0 — Verification Anchors

> **Architect humility note:** Architect Read `phieu/TICKET_TEMPLATE.md` + `docs/BACKLOG.md` + `~/tarot/.claude/agents/advisory-watch.md` (tarot source — explicit Read succeeded) + `phieu/done/P040-bootstrap-stack-detection.md` (giọng + Task 0 pattern) + `phieu/done/P043-doc-drift-consolidate.md` (recent giọng) + `agents/architect.md` + `agents/worker.md` + `docs/LAYERS.md` + `docs/HANDOFF.md` + `README.md` (first 130 lines) + `docs/SETUP.md` (first 60 lines) + `docs/DISCOVERIES.md` (index) + `templates/.sos-stack.toml.example` end-to-end trong DRAFT session 2026-05-25. Architect attempted Read `scripts/parsers/pnpm_lock_v9.py` — BLOCKED by `architect-guard.sh` envelope (correct behavior; envelope is the feature). Architect Read tarot source confirms parser interface pattern from outside; Worker re-verifies actual P040 stub at EXECUTE.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `agents/` directory has 3 subagent files + README (no `advisory-watch.md` exists). P041 file = NEW. | `ls agents/` | ✅ [verified] — Architect Glob 2026-05-25: `agents/architect.md`, `agents/worker.md`, `agents/orchestrator.md`, `agents/README.md` only. |
| 2 | `.claude/commands/` directory does NOT exist at HEAD. P041 file = FIRST file there. Worker must create dir. | `ls .claue/commands/ 2>/dev/null` + `Glob(".claude/commands/*")` | ✅ [verified] — Architect Glob 2026-05-25 returned 0 files. |
| 3 | `templates/` directory currently has 4 files: `BACKLOG_template.md`, `claude-settings.local.json`, `.docs-gate.toml`, `.sos-stack.toml.example`. `advisory-inbox.md` does NOT exist. | `ls templates/` | ✅ [verified] — Architect Glob 2026-05-25. |
| 4 | `scripts/parsers/` directory has 6 files from P040 (all stubs returning `[]`): `cargo_lock.py`, `go_sum.py`, `package_lock_v3.py`, `pnpm_lock_v9.py`, `pyproject_toml.py`, `requirements_txt.py`. Underscore filenames per P040 V2 Sếp decision. | `ls scripts/parsers/` | ✅ [verified] — Architect Glob 2026-05-25 confirms all 6 underscore filenames present. |
| 5 | `scripts/parsers/pnpm_lock_v9.py` content = stub returning `[]` + TODO(P041) comment + `parse(path: Path) -> list[dict]` signature. Optional keys `license`, `integrity` documented in docstring. | `cat scripts/parsers/pnpm_lock_v9.py` | ⚠️ [Architect cannot Read — envelope blocks `.py`] — `architect-guard.sh:58` blocks Architect Read of `.py` files. Worker EXECUTE verifies stub matches P040 spec (from `phieu/done/P040-bootstrap-stack-detection.md:506-545` Task 3 template) before implementing. If stub drifted from P040 spec → Worker logs Discovery + escalates. |
| 6 | Tarot source file `~/tarot/.claude/agents/advisory-watch.md` exists + readable by Architect (outside-repo Read). Frontmatter has `tools: Read, Grep, Glob, WebFetch, WebSearch`, `model: sonnet`, persona alias "Trinh sát". | `cat ~/tarot/.claude/agents/advisory-watch.md \| head -10` | ✅ [verified] — Architect Read full file 2026-05-25, 221 lines. Frontmatter at lines 1-6: `tools: Read, Grep, Glob, WebFetch, WebSearch`, `model: sonnet`. Description (line 3) explicit: "Vietnamese alias: Trinh sát". |
| 7 | Tarot uses sentinel markers `<!-- INBOX_APPEND_START -->` / `<!-- INBOX_APPEND_END -->` at line 168 + 171. sos-kit (per Sếp user message) uses different markers: `<!-- advisory-start -->` / `<!-- advisory-end -->`. Markers are LOAD-BEARING (slash command greps them). | `grep "INBOX_APPEND" ~/tarot/.claude/agents/advisory-watch.md` | ✅ [verified] — tarot markers confirmed at lines 168, 171, 199, 212. sos-kit P041 MUST USE `<!-- advisory-start -->` / `<!-- advisory-end -->` (per Sếp user message) — this is rename, not preserve. Agent body + slash command + template all must align. |
| 8 | Tarot's parser invocation = `python3 scripts/extract-pnpm-versions.py` (line 36) — tarot-specific. P041 generic version invokes parser from `.sos-stack.toml` `parser =` field (`scripts/parsers/pnpm_lock_v9.py` per P040 stub path). | `grep "extract-pnpm-versions" ~/tarot/.claude/agents/advisory-watch.md` | ✅ [verified] — tarot line 36. P041 strips this reference; agent reads `.sos-stack.toml` and invokes `python3 <parser>` where `<parser>` = the value of `parser` field per stack entry. |
| 9 | Tarot's INV-107 10-deps critical list (line 40-43) is tarot-specific scope-bounding. P041 generic version: NO hardcoded allowlist; agent invokes parser → parser returns direct deps from manifest+lockfile → query GHSA for ALL returned deps. (Bound by parser's "direct deps only" semantic, not by hardcoded list.) | `grep "INV-107\|10 critical" ~/tarot/.claude/agents/advisory-watch.md` | ✅ [verified] — tarot line 40-43 lists 10 deps explicitly. P041 strips list. Generic boundedness comes from `source = "direct"` filter in parser (P040 stub contract). |
| 10 | Tarot mentions Python deps via `requirements.txt` Read at line 47-50. P041 same pattern (use `requirements_txt.py` parser — stub in P040), but P041 stays Node-focused for first implementation (`pnpm_lock_v9.py`); Python parser stays stub. Agent body mentions multi-ecosystem support but only Node has working parser in P041. | (design decision, not code anchor) | ✅ [design DECIDED] — Architect spec: Trinh sát body documents multi-ecosystem support generically (reads `.sos-stack.toml` `[[stack]]` entries), but P041 ships only `pnpm_lock_v9.py` (+ optionally `package_lock_v3.py`) as working parsers. If `.sos-stack.toml` has non-Node stacks → Trinh sát skips them with "parser stub returns empty — implementation deferred" message in report. |
| 11 | Tarot's Bước 1.5 state file (`docs/security/.advisory-scan-state`) + 3-way fallback parse logic (Case A JSON / Case B legacy / Case C missing) at lines 53-99. **DEFERRED in P041.** Agent body notes "state persistence deferred to follow-on phiếu; first version scans last-7-days default with no dedup". | (design decision) | ✅ [design DECIDED] — Architect explicit defer rationale: state file adds 60+ lines complexity for incremental-scan feature that is OPTIMIZATION, not blocking first-version usefulness. Trade-off documented in agent body + this phiếu Constraints. |
| 12 | Tarot frontmatter line 3 description is a single LONG paragraph. sos-kit `agents/architect.md` (Architect's own file) line 3 has similar pattern. P041 follows same convention: 1-paragraph description, then frontmatter ends, then body H1 + persona intro. | `head -10 agents/architect.md` | ✅ [verified] — Architect Read `agents/architect.md` 2026-05-25, line 3 = single-paragraph description ending with "Invoke when need to write phiếu/ticket/plan for a feature." Same pattern in `agents/worker.md:3`. P041 follows. |
| 13 | Tarot line 168-171 example output row format: `\| 2026-05-24 \| GHSA-xxxx-yyyy \| https://github.com/advisories/GHSA-xxxx-yyyy \| next@<=15.5.17 \| src/middleware.ts:42 \| High \| open \| - \|` (8 pipe-separated columns: date, advisory ID, URL, dep@version-range, file:line OR "indirect", severity, status, notes). P041 preserves exact 8-column format for slash-command-compatible append. | `sed -n '167,172p' ~/tarot/.claude/agents/advisory-watch.md` | ✅ [verified] — tarot 8-column format confirmed. P041 inherits verbatim for portability. |
| 14 | Tarot line 119 "Severity sourcing rule" — upstream official ONLY (nginx.org / PyPA / GHSA reviewed badge / vendor official page). NO third-party rescore inflation. P281 CVE-2026-9256 nginx precedent fail case. P041 generic version: PRESERVE rule verbatim because it's NOT tarot-specific — it's a general security-data-quality rule. | `sed -n '119p;173,193p' ~/tarot/.claude/agents/advisory-watch.md` | ✅ [verified] — tarot rule generic-portable. P041 includes verbatim in agent body. |
| 15 | `docs/LAYERS.md` access matrix table (lines 21-28 after P043) has 4 columns: Chủ nhà / Quản đốc / Kiến trúc sư / Thợ. Adding Trinh sát = adding a 5th column OR a new row (Architect recommends row to avoid making the matrix unreadable — 5 columns wide is hard to read in markdown). | `sed -n '21,28p' docs/LAYERS.md` | ✅ [verified] — Architect Read 2026-05-25, table has 4 columns. P041 ROW addition (Trinh sát) under access matrix or as a separate "Specialist subagents" mini-table. Worker self-decides exact placement (Tầng 2 layout), but the FACT of Trinh sát documented in LAYERS = Tầng 1 contract. |
| 16 | `docs/HANDOFF.md` has 5 numbered handoffs (0, 1, 2, 2.5, 3, 4). Trinh sát's "orchestrator-mediated inbox" handoff = NEW pattern, doesn't fit existing 5 cleanly. Architect proposes appendix subsection "Specialist subagents (P041+)" rather than renumbering. | `grep -n "^## Handoff" docs/HANDOFF.md` | ✅ [verified] — Architect Read 2026-05-25, 5 handoffs at lines 13, 43, 67, 89, 134, 177 (Handoff 0/1/2/2.5/3/4). Appendix add preserves existing handoff numbering. |
| 17 | `README.md` Components/Subagents table at lines 109-115 has 3 rows (orchestrator/Quản đốc + architect + worker). P041 adds row for `advisory-watch` (Trinh sát). | `sed -n '109,118p' README.md` | ✅ [verified] — Architect Read 2026-05-25, table structure confirmed. Tools column for Trinh sát: `Read, Grep, Glob, WebFetch, WebSearch, Bash (scoped: parser scripts only)`. Cannot column: `Edit, Write, Task, Skill, arbitrary Bash` (read-only-output specialist; Bash scoped to parser invocation). |
| 18 | `docs/SETUP.md` has 6 numbered sections (Install Rust tools / Skills / Phiếu / Setup each project / Pre-commit / Canary). P040 added "Step 4h security init" reference. P041 either extends Step 4h or adds new Step 4i for `/advisory-scan` first-run. Architect recommends new "## Security pipeline" top-level section AFTER Quick Start, parallel to existing 6 — clearer than nesting 4h+4i+4j. | `grep -n "^### \|^## " docs/SETUP.md` | ⚠️ [unverified — Architect only Read first 60 lines] — Worker EXECUTE Glob full file structure + decide insertion point. Tầng 2 doc-structure decision; ANY working insertion that makes the Security flow discoverable is acceptable. |
| 19 | `CLAUDE.md` repo-structure tree (Architect Read in earlier session per system context) lists `agents/`, `templates/`, `scripts/`. P041 adds 3 mentions: `agents/advisory-watch.md`, `templates/advisory-inbox.md`, `.claude/commands/advisory-scan.md`. | `grep -n "^├──\|^│   ├──" CLAUDE.md` | ⚠️ [needs Worker verify] — Architect didn't fresh-Read `CLAUDE.md` tree in this DRAFT (Read it via system reminder context). Worker confirms exact tree structure + decides where to slot 3 new lines (Tầng 2 placement). |
| 20 | Slash command file `.claude/commands/advisory-scan.md` format: Claude Code slash command spec (frontmatter + body). Architect cannot verify exact spec without web fetch — defers contract details to Worker who can check Claude Code docs at EXECUTE. | (cannot verify from Architect envelope) | ⚠️ [needs Worker verify] — Slash command file format per Claude Code docs. Architect Read existing slash command if one exists in sos-kit (Anchor #2 says directory empty → no existing reference). Worker checks Claude Code docs format at EXECUTE; if format differs from skeleton in Task 3, Worker adapts (Tầng 2 file-format adjustment) and notes in Discovery. |

**Summary:** 16 ✅ verified + 4 ⚠️ (Anchor #5 Architect envelope blocks `.py` Read — Worker re-verifies P040 stub; Anchor #18 `docs/SETUP.md` full structure — Worker decides insertion; Anchor #19 `CLAUDE.md` tree — Worker confirms; Anchor #20 slash command format — Worker checks Claude Code docs). No ❌. Tầng 1 phiếu — Worker MUST CHALLENGE before EXECUTE per ORCHESTRATION.md Hard rule #7 / P036.

### Pre-phiếu snapshot (Worker auto first-step)

> **Worker EXECUTE FIRST ACTION** (before any code edit, before Task 0 grep verification): take a rollback point so failed mid-execute can revert.

```bash
# Run from project root (worktree root for phiếu workflow):
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/ — auto-cleaned on phieu-done"
```

If the phiếu hits ❌ mid-execute: `cp .backup/${PHIEU_ID}/settings.local.json .claude/` and `git reset --hard $(cat .backup/${PHIEU_ID}/main-head.txt)` (within phiếu worktree only — NEVER on main per safety rails).

---

## Debate Log

> Tầng 1 phiếu — Worker MUST CHALLENGE before EXECUTE (per ORCHESTRATION.md Hard rule #7 / P036). New public subagent contract + new slash command surface + new sentinel marker schema → mismatch costs all downstream phiếu (P042, future P0xx state-file phiếu, user projects depending on `advisory-inbox.md` template). 4 ⚠️ anchors documented — Worker CHALLENGE verifies stub + decides doc-structure insertion + checks Claude Code slash command spec.

**Phiếu version:** V2 — Reflect Quản đốc decisions (PyYAML ACCEPT, parser runs in subagent, Constraint #3 Bash scope clarification) per Worker CHALLENGE Turn 1.

### Turn 1 — Worker Challenge

*(Worker fills this when invoked in CHALLENGE mode. If no objections, write "Worker accepted V1 — no challenges. Ready for Chủ nhà approval." and skip to Final consensus.)*

**Anchor verification (recap from Task 0):**
- Anchor #N: ✅/⚠️/❌ + 1-line summary if ⚠️/❌

**Objections (Tầng 1 only — phiếu cần sửa):**
- [O1.1] Phiếu giả định X tại file Y, code thật là Z (cite `file:line`). Tác động: …
- [O1.2] …

**Proposed alternatives** (Worker recommends 1):
- A. … (Worker lean — vì …)
- B. …

**Status:** ✅ RESPONDED (V2)

### Turn 1 — Architect Response (phiếu V2)

Quản đốc decisions per Sếp 2026-05-25 — Architect ACCEPTs Worker CHALLENGE on all 3 fronts, refines phiếu to V2:

- **[O1 PyYAML new-dep]** → **ACCEPT (closed decision)**. PyYAML is the pragmatic choice — pnpm-lock.yaml v9 is structured YAML; hand-rolling fragile; PyYAML ubiquitous + battle-tested. Worker pre-flight: `python3 -c 'import yaml' || pip3 install pyyaml` (run as Bước 0 inside Trinh sát subagent OR document in `docs/SETUP.md` "Security" subsection — Worker decides best placement, Tầng 2). Constraint #6 rewritten from open escalation → closed decision with rationale.
- **[O2 Parser invocation location]** → **ACCEPT (REDESIGN)**. Parser runs INSIDE Trinh sát subagent, NOT in orchestrator main session. Rationale: orchestrator Bash is marker-only per LAYERS; spawning subagent for the whole scan = cleaner contract. Trinh sát frontmatter `tools:` MUST include `Bash` (scoped); Task 1 Bước 1 rewritten to invoke `python3 scripts/parsers/<parser>.py <lockfile-path>` directly. Task 3 slash command flow becomes spawn-only: (a) verify `.sos-stack.toml`, (b) spawn Trinh sát, (c) append returned rows. NO `python3` invocation in main session.
- **[O3 Constraint #3 "structurally read-only" contradicts Bash addition]** → **ACCEPT**. Constraint #3 rewritten: Trinh sát has Bash with **scoped purpose** — parser scripts only (`python3 scripts/parsers/*.py`) + WebFetch GHSA. NO Edit/Write. NO arbitrary Bash (`rm`, `git`, etc.). Scope documented explicitly in agent file body section "Bash usage" so future contributors aware. Tools list updated everywhere (frontmatter, README table, LAYERS table).

**Status:** ✅ RESPONDED — phiếu bumped to V2

*(Repeat Turn 2, Turn 3 if needed. Cap = 3.)*

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Chủ nhà: [date] — code execution may begin

---

## Nhiệm vụ

> Worker order: Pre-phiếu snapshot → Task 0 verify → Task 1 (agent file) → Task 2 (inbox template) → Task 3 (slash command) → Task 4 (pnpm parser) → Task 4b (optional package-lock parser) → Task 5 (doc consolidation) → Nghiệm thu. Tasks 1+2+3 independent of each other; Task 4 independent; Task 5 last (references files from 1-4).

### Task 1: `agents/advisory-watch.md` — NEW specialist subagent file

**File:** `agents/advisory-watch.md` (NEW; directory exists)

**Thêm (full file content — Worker writes verbatim, adapting `[needs Worker verify]` items at EXECUTE):**

````markdown
---
name: advisory-watch
description: Trinh sát — read-only-output specialist subagent. Reads `.sos-stack.toml` (written by `sos init security`), runs the configured parser per stack via scoped Bash to extract direct deps, queries GitHub Advisory Database (GHSA) + vendor advisory pages, matches advisories against resolved versions, optionally greps codebase for usage, and returns sentinel-wrapped advisory rows in a final report. The caller (slash command `/advisory-scan`) parses the sentinel block and appends rows to `docs/security/advisory-inbox.md` (or user-configured inbox path). KHÔNG patch lỗ. KHÔNG ghi luật. KHÔNG cầm Write/Edit tool. Bash scoped to parser scripts only.
tools: Read, Grep, Glob, WebFetch, WebSearch, Bash
model: sonnet
---

# Trinh sát — Advisory-watch specialist subagent

Em là **Trinh sát** trong sos-kit security pipeline. Vai trò: phát hiện advisory thế giới ngoài (CVE / GHSA / upstream security release) chạm stack mình, verify dính code thật, surface vào inbox cho Chủ nhà gate. Em là **specialist subagent**, không phải 1 trong 3 main roles (Chủ nhà / Kiến trúc sư / Thợ) — em ngồi cạnh chúng, được Quản đốc spawn qua slash command `/advisory-scan`.

Cặp đôi: **Giám sát** (boundary-check, P042) soi INVARIANT bên trong; em soi advisory bên ngoài.

## Bash usage (SCOPED — read this first)

Em có Bash tool nhưng **scope giới hạn**:

- ✅ **CHO PHÉP:** `python3 scripts/parsers/<parser>.py <lockfile-path>` — chạy parser script per stack từ `.sos-stack.toml`.
- ✅ **CHO PHÉP:** `python3 -c 'import yaml'` — pre-flight check PyYAML dep installed.
- ✅ **CHO PHÉP:** `pip3 install pyyaml` — nếu pre-flight thiếu (one-time install).
- ❌ **CẤM:** `rm`, `mv`, `cp` (mutate filesystem).
- ❌ **CẤM:** `git` (mutate VCS state).
- ❌ **CẤM:** Bất kỳ shell pipeline / network call ngoài parser scripts.

Future contributors: **KHÔNG mở rộng Bash scope** mà không bump `schema_version` + phiếu mới. Bash present here ONLY because parser invocation needs it; everything else stays read-only-output.

## Read-only-output contract (structural enforce)

- **Tools whitelist:** `Read, Grep, Glob, WebFetch, WebSearch, Bash` (Bash scoped per above).
- **KHÔNG có:** `Edit, Write, Task, Skill, AskUserQuestion`. Em không ghi file nào — output rows go through caller's slash command.
- **Output contract:** Em return structured rows trong final report (Bước 5 format), wrapped trong sentinel comments `<!-- advisory-start -->` / `<!-- advisory-end -->`. Caller (slash command `/advisory-scan`) parse + append vào inbox file. Em KHÔNG cầm Write — structural enforce qua tools allowlist.

> Mọi luật mới (handbook update, INVARIANT list change) phải ĐI QUA CHỦ NHÀ qua phiếu — em đề xuất, Chủ nhà gate.

## Vai trò bound (state machine ≈ Worker CHALLENGE)

Em là **CHALLENGE-mode equivalent** cho advisory bên ngoài: surface objection có bằng chứng rồi dừng. Em KHÔNG patch lỗ — đó là Thợ EXECUTE việc khác (phiếu mới).

| Layer | Tools | Em làm gì |
|-------|-------|----------|
| Phát hiện | Slash command `/advisory-scan` (manual hoặc cron) | Spawn em |
| Parser run (em) | `Bash` scoped | `python3 <parser> <lockfile>` mỗi stack |
| Advisory query (em) | `WebFetch`, `WebSearch` | Query GHSA + vendor pages |
| Code grep (em) | `Grep`, `Glob` | Confirm usage in source |
| Append inbox | Slash command (orchestrator main session) | Caller làm, KHÔNG em |
| Ghi luật | Chủ nhà (qua phiếu) | KHÔNG phải em |

## Workflow mỗi lần invoked

### Bước 0: PyYAML pre-flight

```bash
python3 -c 'import yaml' || pip3 install pyyaml
```

If install fails (no network, no pip permissions) → output empty report with warning "PyYAML required for pnpm-lock parsing; install + retry".

### Bước 1: Read `.sos-stack.toml` → run parsers → collect deps

1. `Read` file `.sos-stack.toml` ở project root.
2. Parse TOML structure: `schema_version` (must = 1; nếu ≠ → output report empty với warning), `[[stack]]` array.
3. Cho mỗi `[[stack]]` entry:
   - Extract `type`, `manifest`, `lock_file`, `lock_format`, `parser`.
   - Nếu `parser == ""` → skip stack với note "no parser available for `<lock_format>`; deferred to future P0xx".
   - Nếu `parser` file tồn tại — `Bash` invoke: `python3 <parser> <lock_file>` → capture JSON stdout.
   - Parse stdout JSON list-of-dicts (`name`, `version`, `ecosystem`, `source`).
   - Nếu output is `[]` (stub) hoặc empty → skip stack với note "parser stub returns empty; implementation deferred to future P0xx".
4. Aggregate all stacks' deps into in-memory list `{"stacks": [{"type": "node", "deps": [...]}, ...]}`.

### Bước 2: Query advisory database per (name, version, ecosystem)

Cho mỗi `(name, version, ecosystem)` triplet from Bước 1 output:

**Primary source — GitHub Advisory Database (GHSA):**

- Per-package search filter: `WebFetch url="https://github.com/advisories?query=ecosystem%3A<ecosystem>+<package>" prompt="Extract all advisory entries that match version <version>. Return: advisory ID, severity (from official GHSA reviewed badge ONLY), affected version range, summary, advisory URL."`
- Ecosystem mapping: `npm` → `ecosystem%3Anpm`, `pypi` → `ecosystem%3Apip`, `crates` → `ecosystem%3Arust`, `go` → `ecosystem%3Ago`.
- Per-org advisory pages (deeper coverage for top deps): `https://github.com/<org>/<repo>/security/advisories` — em derive `<org>/<repo>` from package metadata khi possible (e.g. `next` → `vercel/next.js`).

**Vendor pages (optional, secondary — only if agent caller flags `--include-vendor`):**

- nginx: `https://nginx.org/en/security_advisories.html`
- postgres: `https://www.postgresql.org/support/security/`
- Alpine: `https://secdb.alpinelinux.org/`

**WebSearch tertiary (ONLY khi GHSA + vendor miss + có dep cụ thể):** `"<dep> <version> CVE 2026"` — bound vào dep+version, KHÔNG search chung chung.

> ⛔ KHÔNG search "security news 2026" / "javascript vulnerabilities" chung chung — bound query luôn.
> ⛔ Match advisory version range against **resolved version** từ parser output, KHÔNG manifest caret-range. Parse advisory page "affected version range" text → SEMVER compare.

**OSV.dev API DEFERRED:** Tarot dogfood (P282 → P284 2026-05-24) proved WebFetch is GET-only; OSV's `POST /v1/query` returns 405. Bash scope ở P041 KHÔNG cho phép curl (parser scripts only) — OSV vẫn stays out cho đến khi phiếu mở rộng Bash scope. Trade-off: GHSA covers npm + PyPI primary, vendor pages cover Docker base. Acceptable for current scope.

### Bước 3: Verify dính code (Grep)

Cho mỗi advisory match resolved version:

1. `Grep` usage của dep trong codebase root:
   - npm dep `next` → pattern `from ['\"]next` trong file glob `**/*.{ts,tsx,js,jsx}`
   - Python dep `flask` → pattern `from flask|import flask` trong `**/*.py`
   - Rust crate `serde` → pattern `use serde|serde::` trong `**/*.rs`
   - Go module `gin` → pattern `gin-gonic/gin` trong `**/*.go`
2. Cho mỗi match, capture `file:line` + 1 dòng context.
3. Nếu **không có usage** trong source → row vẫn output với `file:line` = `indirect` (transitive risk vẫn cần Chủ nhà gate).

### Bước 4: Format structured rows (NO Write — return in report)

Row markdown format (8 pipe-separated columns — exact match for slash command append):

```markdown
| YYYY-MM-DD | <Advisory ID> | <Source URL> | <name@version-range> | <file:line> hoặc "indirect" | <Critical/High/Medium/Low> | open | - |
```

**KHÔNG tự Write vào file.** Em KHÔNG cầm Write tool. Return rows trong report Bước 5 wrapped trong sentinel comments. Caller append.

### Bước 5: Output final report cho caller

```markdown
## Advisory Scan Report — <YYYY-MM-DD>

**Stacks scanned (from `.sos-stack.toml`):**
- <type-1>: <N> direct deps parsed
- <type-2>: <K> direct deps parsed
- <skipped-type>: parser stub not implemented (deferred)

**Advisories found:** <total queried>
- Chạm stack (matched resolved version, output for append): <X>
- Không chạm (version mismatch, skipped): <Y>

**New rows for inbox append (status=open):**

<!-- advisory-start -->
| 2026-05-25 | GHSA-xxxx-yyyy | https://github.com/advisories/GHSA-xxxx-yyyy | next@<=15.5.17 | src/middleware.ts:42 | High | open | - |
| 2026-05-25 | GHSA-aaaa-bbbb | https://github.com/advisories/GHSA-aaaa-bbbb | next-auth@<=4.24.5 | indirect | Medium | open | - |
<!-- advisory-end -->

**Severity sourcing rule (P281 lesson 2026-05-24 — preserved verbatim):**

Severity column trong row PHẢI lấy từ **nguồn upstream official** ONLY:

| Ecosystem | Upstream official source |
|-----------|--------------------------|
| nginx | `https://nginx.org/en/security_advisories.html` (F5 CNA) |
| Python packages | PyPA Advisory Database |
| npm packages | GitHub Security Advisories (`https://github.com/advisories`, GHSA-prefixed reviewed badge) |
| Rust crates | RustSec Advisory Database (`https://rustsec.org/`) + GHSA |
| Go modules | Go vulnerability database (`https://pkg.go.dev/vuln/`) + GHSA |
| Docker base images | Respective official advisory page (postgres, alpine, nginx) |

**KHÔNG inflate** bằng cách lấy số CVSS cao nhất tìm được bên thứ ba (security researcher blog, NVD-rescore, alternative CNA). Nếu nguồn khác chấm khác official → ghi BOTH trong cùng cell nhưng RÕ ai là official, ai là third-party:

- ✅ ĐÚNG: `Medium (nginx.org official); CVSS v4.0=9.2 per [researcher X] (third-party rescore)`
- ❌ SAI: `High (CVSS v4.0=9.2)` ← gán nhầm cấp third-party thành official

**Lý do:** Severity drive priority decision (vá đêm nay vs vá tuần sau). False High = ép Chủ nhà panic vá khẩn không cần; false Low = ép Chủ nhà ignore lỗ thực. Anchor về upstream official protect khỏi cả hai.

**Inbox file:** Slash command `/advisory-scan` parses `<!-- advisory-start -->` ... `<!-- advisory-end -->` block above and appends rows to inbox (default `docs/security/advisory-inbox.md`, configurable).

**Next action:** Chủ nhà liếc inbox, mỗi row gạt "dismissed" hoặc tạo phiếu mới.
```

> Sentinel markers `<!-- advisory-start -->` / `<!-- advisory-end -->` BẮT BUỘC — slash command grep tìm 2 marker này để extract rows. Nếu không có row mới (0 advisory chạm) → vẫn output block empty: `<!-- advisory-start -->\n<!-- advisory-end -->` để slash command no-op cleanly.

## Anti-pattern em PHẢI tránh

- ❌ Phán "lỗi này nguy hiểm, phải fix ngay" — em surface evidence, Chủ nhà judge.
- ❌ Tự ghi vào `CLAUDE.md` / `.claude/agents/*.md` / docs guide.
- ❌ Cố Write vào inbox — em KHÔNG cầm Write. Return rows trong report block sentinel, caller append.
- ❌ Match advisory range chống manifest caret-range (`^15.5`) — phải resolved version từ parser output (`15.5.17`). False-positive killer.
- ❌ WebSearch chung chung không bound vào dep+version.
- ❌ Auto-decay row sau N ngày — Chủ nhà gạt tay.
- ❌ Patch lỗ trong cùng phiên gọi này — em là CHALLENGE-equivalent, không EXECUTE.
- ❌ Trộn vai với Giám sát (boundary-check, P042) — em soi NGOÀI (advisory thế giới), Giám sát soi TRONG (INVARIANT map).
- ❌ Emit sentinel marker `<!-- advisory-start/end -->` ngoài Bước 5 final report — slash command parse first match cặp marker. Nếu em emit trong Bước 1-4 body / example / explanation → slash dính nhầm. Marker CHỈ xuất hiện đúng 1 lần wrap rows block ở Bước 5.
- ❌ Scan transitive deps — bound vào `source = "direct"` từ parser output. Transitive để Dependabot lo.
- ❌ Bash invoke gì ngoài `python3 scripts/parsers/*.py` + `python3 -c 'import yaml'` + `pip3 install pyyaml`. Scope hard cap.

## Bounded scope

- Em **CHỈ** soi advisory phát hiện qua GHSA + vendor pages.
- Em **KHÔNG** re-scan toàn bộ history mỗi lần (incremental state DEFERRED — tarot P282 schema may port in follow-on phiếu).
- Em **KHÔNG** soi lỗi runtime / app logic — đó là Sentry MCP / Worker CHALLENGE.
- Em **KHÔNG** đề xuất refactor / kiến trúc — chỉ surface advisory.
- Em **CHỈ** support ecosystem có parser implementation thực sự. Stub parsers → skip stack với note "deferred".

## P041 implementation status

- ✅ **npm (pnpm-v9):** `scripts/parsers/pnpm_lock_v9.py` implemented in P041.
- ⚠️ **npm (npm-v3):** `scripts/parsers/package_lock_v3.py` — implemented IF P041 EXECUTE effort allowed; otherwise stub.
- ⏸️ **pypi (requirements-txt + pyproject-toml):** stubs only. Future phiếu fills.
- ⏸️ **crates (cargo-lock):** stub only. Future phiếu fills.
- ⏸️ **go (go-sum):** stub only. Future phiếu fills.

Worker EXECUTE updates this section to reflect actual ship state (Tầng 2 status text — Worker self-edits at end of EXECUTE).
````

**Lưu ý 1:**
- Persona name **Trinh sát** (Vietnamese) appears in body. Filename `advisory-watch.md` (English) matches sibling pattern `architect.md` / `worker.md` / `orchestrator.md`.
- Frontmatter `tools:` list LOAD-BEARING — Claude Code enforces tool allowlist structurally. Bash IS in the list (scoped per "Bash usage" body section); NO `Write` / `Edit` / `Task` / `Skill` — output stays read-only-output guaranteed by structural means.
- Sentinel markers `<!-- advisory-start -->` / `<!-- advisory-end -->` appear in agent body (Bước 5 example) AND in `templates/advisory-inbox.md` (Task 2) AND in `.claude/commands/advisory-scan.md` parse logic (Task 3). All 3 must match EXACTLY — they are the integration contract.
- Bước 1 design: agent invokes parser directly via scoped Bash. Slash command does NOT pre-run parsers — slash command is spawn-only. This is per Sếp REDESIGN 2026-05-25 (Worker CHALLENGE Turn 1).

**Validate (Task 1):**
- File exists at `agents/advisory-watch.md`.
- Frontmatter parses cleanly (YAML 3-key: `name`, `description`, `tools`, `model`).
- `grep -c "^name: advisory-watch$" agents/advisory-watch.md` = 1.
- `grep -c "Trinh sát" agents/advisory-watch.md` ≥ 5 (persona used throughout body).
- `grep -c "advisory-start\|advisory-end" agents/advisory-watch.md` = 2 (markers only in Bước 5 example block — count includes both opener + closer).
- `grep -c "INV-107\|extract-pnpm-versions\|tarot" agents/advisory-watch.md` = 0 (generic-able test — no tarot-specific paths).
- `grep -c "Bash usage" agents/advisory-watch.md` ≥ 1 (Bash scope section present).
- `grep -c "^tools:.*Bash" agents/advisory-watch.md` = 1 (Bash present in frontmatter).

---

### Task 2: `templates/advisory-inbox.md` — empty queue template

**File:** `templates/advisory-inbox.md` (NEW)

**Thêm (full file content):**

```markdown
# Advisory Inbox

> **Queue file for security advisories surfaced by Trinh sát (advisory-watch).**
> Generated rows are appended between the sentinel markers below by the `/advisory-scan` slash command (orchestrator-side).
> Chủ nhà reviews each row and either marks status `dismissed` or creates a follow-on phiếu to patch.
>
> **Schema (8 columns):** `date | advisory ID | URL | name@version-range | file:line or "indirect" | severity | status | notes`
>
> **Severity:** must reflect upstream official source (GHSA reviewed badge / vendor official page). NO third-party rescore inflation. See `agents/advisory-watch.md` Bước 5 "Severity sourcing rule" for full rationale.
>
> **Status values:** `open` (new, awaiting Chủ nhà review) | `dismissed` (reviewed, not actionable — e.g. unaffected code path, false positive) | `phieu-P<NNN>` (follow-on phiếu created — link the row to its tracking phiếu).

## Rows

<!-- advisory-start -->
<!-- advisory-end -->

## Example row (commented — for schema reference, NOT counted in scans)

<!--
| 2026-05-25 | GHSA-xxxx-yyyy | https://github.com/advisories/GHSA-xxxx-yyyy | next@<=15.5.17 | src/middleware.ts:42 | High (GHSA reviewed) | open | - |
-->

## How rows flow

1. User runs `/advisory-scan` (or cron triggers it).
2. Slash command verifies `.sos-stack.toml` exists; if missing → prompts user to run `sos init security` first.
3. Slash command spawns Trinh sát subagent.
4. Trinh sát runs each `[[stack]]` parser via scoped Bash (`python3 <parser> <lock_file>`), collects deps.
5. Trinh sát queries GHSA + vendor pages, greps codebase, returns sentinel-wrapped rows in final report.
6. Slash command extracts block between `<!-- advisory-start -->` and `<!-- advisory-end -->`, appends rows in this file BETWEEN those markers (preserve marker positions).
7. Chủ nhà reviews, gates per-row.

## Notes

- Empty `<!-- advisory-start --> / <!-- advisory-end -->` block = no advisories matched (clean scan). Re-run later as deps evolve.
- Multiple scans append cumulatively. Chủ nhà handles dedup by status updates (no auto-dedup in P041; state-file dedup deferred to follow-on phiếu).
- **Do NOT delete the sentinel markers.** They are the slash command's append point.
- **Do NOT rename this file's location randomly.** Default expected at `docs/security/advisory-inbox.md` in user project — copy this template there on first run.
```

**Lưu ý 2:**
- Sentinel markers `<!-- advisory-start -->` + `<!-- advisory-end -->` MUST appear exactly once each, in the order start-then-end, with the empty line between them serving as the append insertion point.
- The "Example row" block is `<!--` HTML-commented so it's visible in raw markdown but invisible in rendered view AND not parsed as a row by the slash command (slash matches only the active sentinel block).
- Template lives at `templates/advisory-inbox.md` — user copies to their project (typically `docs/security/advisory-inbox.md`). NOT auto-copied by P040's `sos init security` (P041 may add that copy step to slash command first-run if file missing — see Task 3).

**Validate (Task 2):**
- File exists at `templates/advisory-inbox.md`.
- `grep -c "advisory-start\|advisory-end" templates/advisory-inbox.md` = 2 (one each, in Rows section).
- Markdown lint clean (no broken headings, balanced HTML comments).

---

### Task 3: `.claude/commands/advisory-scan.md` — slash command

**File:** `.claude/commands/advisory-scan.md` (NEW; **Worker creates directory `.claude/commands/` — first file there**)

> **Slash command file format:** Claude Code slash commands are markdown files where the body becomes the prompt the model receives. Frontmatter (if used) configures behavior. Worker verifies exact spec at EXECUTE per Anchor #20 — if format differs, Worker adapts the skeleton below to spec while preserving the intent (verify `.sos-stack.toml`, spawn Trinh sát, append rows).
>
> **Per Sếp REDESIGN 2026-05-25:** orchestrator-side flow = spawn-only. Parser invocation happens INSIDE Trinh sát subagent (scoped Bash), NOT in main session. Quản đốc only verifies prerequisites, spawns subagent, parses returned rows, appends to inbox.

**Thêm (full file content — Worker adapts frontmatter to actual Claude Code slash command spec):**

````markdown
---
description: Scan stack for security advisories via GHSA + vendor pages. Verifies .sos-stack.toml exists, spawns Trinh sát subagent (which runs parsers + queries GHSA), appends results to advisory inbox.
---

# /advisory-scan

You are the orchestrator (Quản đốc) running the advisory-scan slash command. Execute these steps in order — DO NOT skip, DO NOT improvise. Parser invocation happens INSIDE the Trinh sát subagent, NOT in this main session.

## Step 0 — Verify prerequisites

1. Verify `.sos-stack.toml` exists at project root via `Glob(".sos-stack.toml")`.
   - **If missing:** STOP. Tell user: "No `.sos-stack.toml` found. Run `sos init security` first (foundation from P040 — detects stack + writes schema). Then re-run `/advisory-scan`."
2. Verify inbox file exists. Default path: `docs/security/advisory-inbox.md`.
   - **If missing:** Use `Read templates/advisory-inbox.md` then `Write docs/security/advisory-inbox.md` (mkdir-then-write via the file-write tool, NOT via Bash). Tell user the inbox was bootstrapped.

## Step 1 — Spawn Trinh sát subagent

Use `Task` tool with `subagent_type: "advisory-watch"`. Prompt format:

```
You are Trinh sát. Run your full workflow (Bước 0 PyYAML pre-flight → Bước 1 parse stacks → Bước 2 query advisories → Bước 3 grep code → Bước 4 format rows → Bước 5 final report).

Project root: <cwd>
`.sos-stack.toml` path: .sos-stack.toml (or absolute path if cwd unclear)

Return your final report with `<!-- advisory-start -->` ... `<!-- advisory-end -->` block as specified.
```

Wait for subagent return. Subagent handles parser invocation (scoped Bash) + GHSA query + code grep entirely on its own.

## Step 2 — Extract sentinel block from subagent output

Use `Grep` or string parsing to locate the block between `<!-- advisory-start -->` and `<!-- advisory-end -->` in the subagent's return.

- If block is empty (no rows between markers) → tell user "Scan complete. 0 new advisories." STOP.
- If block has N rows → continue to Step 3.

## Step 3 — Append rows to inbox

`Read` current inbox file. Locate the existing `<!-- advisory-start -->` and `<!-- advisory-end -->` markers (between them is the active rows region).

Insert subagent's new rows BETWEEN the existing markers (preserve markers; append below existing rows if any).

`Write` updated inbox file.

## Step 4 — Report to user

Tell user:
- N new advisories appended to `<inbox-path>`.
- Per-row summary (1-line each): `<advisory-ID> <severity> <name@version> — <file:line or indirect>`.
- Next action: review inbox, mark each row `dismissed` or create follow-on phiếu via `phieu <slug>`.

## Hard rules

- Trinh sát is the WORKHORSE. Parser invocation, advisory query, code grep all happen INSIDE the subagent (scoped Bash). Main session ONLY verifies prerequisites + spawns + appends.
- All `Write` happens in this slash command (orchestrator side), never inside the subagent.
- Sentinel markers `<!-- advisory-start -->` / `<!-- advisory-end -->` are LOAD-BEARING. Do not rename, do not duplicate, do not move.
- Schema version check (subagent handles via Bước 1) is a hard gate — schema v1 ONLY in P041. v2 = breaking change requires phiếu.
- If Trinh sát reports "all parsers stubs / PyYAML missing / no stacks" → relay the message verbatim, NOT a silent success.
````

**Lưu ý 3:**
- Slash command body is the literal prompt the model receives when user runs `/advisory-scan`. Format may differ slightly per Claude Code's actual spec — Worker adapts at EXECUTE (Anchor #20 ⚠️). Possibilities: (a) plain markdown body, no frontmatter (current skeleton works); (b) frontmatter `name:` + `description:` required; (c) `$ARGUMENTS` placeholder convention for user-passed args. Worker checks Claude Code docs / existing slash commands in sister projects (`~/tarot/.claude/commands/` if accessible) and conforms.
- **Main session (Quản đốc) does NOT invoke `python3 <parser>` directly.** That happens INSIDE Trinh sát subagent (scoped Bash). Main session is spawn-only + sentinel-block-extract + inbox-append. This is per Sếp REDESIGN 2026-05-25.
- The slash command is itself a documented contract — it IS the user-facing security feature. Wording inside this file matters more than internal helpers (Tầng 1 territory).

**Validate (Task 3):**
- Directory `.claude/commands/` exists.
- File `.claude/commands/advisory-scan.md` exists.
- `grep -c "advisory-start\|advisory-end" .claude/commands/advisory-scan.md` ≥ 4 (markers referenced in Step 2 + Step 3 + Hard rules).
- `grep -c "advisory-watch" .claude/commands/advisory-scan.md` ≥ 1 (Task subagent_type ref).
- `grep -c "python3 " .claude/commands/advisory-scan.md` = 0 (slash command MUST NOT invoke parser in main session — that's Trinh sát's job).
- `grep -c "tarot\|INV-107" .claude/commands/advisory-scan.md` = 0 (generic).

---

### Task 4: Implement `scripts/parsers/pnpm_lock_v9.py`

**File:** `scripts/parsers/pnpm_lock_v9.py` (was P040 stub returning `[]`)

**Tìm:** the stub `parse()` function body — currently per P040 spec:

```python
def parse(path: Path) -> list[dict]:
    # TODO(P041): implement <format> parsing.
    # Reference: tarot's `.claude/agents/advisory-watch.md` documents the parser
    # contract; port logic without tarot-specific path assumptions.
    _ = path  # silence unused-arg lint until P041 implements
    return []
```

**Worker first action: confirm stub matches P040 spec.** If stub drifted, log Discovery + escalate Tầng 1 (Architect cannot Read `.py` per envelope — Worker is sole verifier here).

**Thay bằng:** real implementation. Worker implements per the contract documented in the stub's docstring (`parse(path: Path) -> list[dict]` returning dicts with keys `name`, `version`, `ecosystem`, `source`). Implementation requirements:

1. **Input:** `Path` to `pnpm-lock.yaml` (lockfileVersion `'9.0'` or compatible v9 layout).
2. **Parse strategy:**
   - Use **PyYAML** (`import yaml`) — accepted dep per Constraint #6 + Sếp decision 2026-05-25. Pre-flight `python3 -c 'import yaml' || pip3 install pyyaml` runs in Trinh sát Bước 0 (OR document install requirement in `docs/SETUP.md` "Security" subsection — Worker decides best placement, Tầng 2).
   - Navigate `importers:` → `.` (project root) → `dependencies` + `devDependencies` keys.
   - For each entry: name (key), version (extract from `version:` field). Strip peer-suffix: `0.92.0(zod@4.3.6)` → `0.92.0` via regex `^([^\s(]+)`.
   - Source field: `"direct"` for all entries from `importers:` `dependencies` + `devDependencies` (these ARE direct deps by definition).
   - Ecosystem: `"npm"` (hardcoded — this parser handles npm/pnpm ecosystem).
3. **Output:** `list[dict]` per stub contract. Optional keys (`license`, `integrity`) — Worker may include if cheap to extract, omit if not.
4. **Edge cases (encode as inline comments referencing tarot P273/P282 lessons):**
   - `lockfileVersion` field check: must start with `'9.` — if older or `'10`, raise `ValueError("Unsupported lockfileVersion: <v>; this parser handles v9 only.")`.
   - `importers:` section may have multiple roots in monorepo (e.g. `importers: { ".": {...}, "packages/api": {...} }`). P041 handles ONLY root `.`; document monorepo limit in docstring + comment "monorepo multi-root deferred to follow-on phiếu".
   - `packages:` section (transitive deps) — DO NOT parse. P041 returns direct deps only per `source = "direct"` contract.
5. **CLI entry point:** preserve the `if __name__ == "__main__":` block from stub. Output JSON to stdout (Trinh sát subagent consumes this via scoped Bash):
   ```python
   import json
   deps = parse(Path(sys.argv[1]))
   print(json.dumps(deps))
   ```
6. **Tests:** Worker may add `scripts/parsers/tests/test_pnpm_lock_v9.py` (NEW dir) with 1-2 fixture-based tests (sample `pnpm-lock.yaml` fixture + expected deps list) — encouraged but Tầng 2 (Worker self-decides scope). Minimum: smoke test the parser against sos-kit's own `pnpm-lock.yaml` if one exists at repo root (Architect Glob 2026-05-25: no `pnpm-lock.yaml` in sos-kit root — sos-kit is bash, not Node — so smoke test fixture is the path).

**Lưu ý 4:**
- **PyYAML accepted dependency.** Pre-flight install (`pip3 install pyyaml`) is part of Trinh sát Bước 0. Constraint #6 below documents the decision rationale.
- Parser MUST handle empty/missing `dependencies` or `devDependencies` gracefully (return `[]` for that section, don't crash).
- Output JSON format: array of dicts. NOT object. Trinh sát parses via `json.loads(stdout)` after scoped-Bash invocation.
- Tarot P273 lesson: pnpm-lock v9 layout's flat regex pattern (`^  <name>@X.Y.Z:`) DOES NOT WORK for `importers:` section — that's why we navigate the 2-level YAML structure properly. Encode this lesson as a code comment so future maintainers don't "optimize" back to regex.

**Validate (Task 4):**
- `python -m py_compile scripts/parsers/pnpm_lock_v9.py` exits 0.
- `python scripts/parsers/pnpm_lock_v9.py /dev/null` exits cleanly with error OR returns `[]` JSON (graceful — `/dev/null` is not a valid YAML).
- Synthetic test: Worker creates a minimal `pnpm-lock.yaml` fixture in `/tmp/`:
  ```yaml
  lockfileVersion: '9.0'
  importers:
    .:
      dependencies:
        next:
          specifier: ^15.0.0
          version: 15.5.17
        react:
          specifier: ^18.0.0
          version: 18.2.0(zod@4.3.6)
      devDependencies:
        typescript:
          specifier: ^5.0.0
          version: 5.3.3
  ```
  `python scripts/parsers/pnpm_lock_v9.py /tmp/test-lock.yaml` outputs JSON containing 3 deps: `next@15.5.17`, `react@18.2.0` (peer-suffix stripped), `typescript@5.3.3`, all `ecosystem: "npm"`, `source: "direct"`.
- `grep -c "tarot\|extract-pnpm-versions" scripts/parsers/pnpm_lock_v9.py` = 0 (generic, no tarot refs).

---

### Task 4b (OPTIONAL): Implement `scripts/parsers/package_lock_v3.py`

**File:** `scripts/parsers/package_lock_v3.py` (was P040 stub)

**Skip rationale:** If Task 4 (pnpm parser) + Tasks 1-3 + Task 5 exceed estimated effort budget OR if dry-run nghiệm thu (Manual Testing below) passes with pnpm-only, Worker may DEFER package_lock_v3 to a follow-on phiếu. Worker logs decision in Discovery Report ("package_lock_v3 deferred — reason: <token cap / time cap / scope conservation>").

**If implementing:**

- Input: `Path` to `package-lock.json` (lockfileVersion 3, standard `npm install` output).
- Parse: JSON via `json.loads`. Navigate `packages` field — root entry is empty key `""`, has `dependencies` map. Direct deps from `package.json` mirror in `packages[""]["dependencies"]`.
- For each direct dep: name (key in `packages[""]["dependencies"]`), version (from `packages["node_modules/<name>"]["version"]` — npm's flat layout).
- Same output contract as pnpm parser: `list[dict]` with `name`, `version`, `ecosystem="npm"`, `source="direct"`.
- Edge cases: `packages["node_modules/<name>"]` may not exist for missing deps (incomplete install); skip with warning to stderr.

**Lưu ý 4b:**
- This is OPTIONAL — Worker explicitly logs "shipped" or "deferred" in Discovery Report. Both outcomes are acceptable.
- If deferring, Worker also updates `agents/advisory-watch.md` "## P041 implementation status" section to reflect actual ship state (Tầng 2 edit per agent body note).

**Validate (Task 4b):**
- Same as Task 4 but with package-lock.json fixture instead of pnpm-lock.yaml.

---

### Task 5: Doc consolidation — LAYERS / HANDOFF / README / SETUP / CLAUDE

**Worker order:** Edit each doc in this order. Each is small (1-3 paragraphs). Worker self-decides exact wording (Tầng 2) but MUST cover the Tầng 1 facts listed.

#### 5a. `docs/LAYERS.md` — add Trinh sát to access matrix

**File:** `docs/LAYERS.md`

**Tìm:** access matrix table at lines 21-28 (Architect Read 2026-05-25 confirms structure). Currently 4 columns: Chủ nhà / Quản đốc / Kiến trúc sư / Thợ.

**Thay bằng / Thêm:** Add a NEW small subsection AFTER the access matrix titled `### Specialist subagents (P041+)` with a mini-table:

| | Trinh sát (advisory-watch) | Giám sát (boundary-check, P042 — pending) |
|---|---|---|
| Role | Specialist subagent — soi advisory ngoài | Specialist subagent — soi INVARIANT trong |
| Spawned by | Quản đốc via `/advisory-scan` | Quản đốc via `/security-review` |
| Tools | Read, Grep, Glob, WebFetch, WebSearch, **Bash (scoped: parser scripts only)** | (P042 will spec) |
| Cannot | Edit, Write, Task, Skill, arbitrary Bash | (P042 will spec) |
| Output | Sentinel-wrapped advisory rows → caller appends to inbox | (P042 will spec) |

Plus 1 sentence in the "3 layers in detail" intro: "Specialist subagents (Trinh sát, Giám sát) are read-only-output verifiers that sit beside the 3 main roles; they don't replace them. Spawned by Quản đốc for narrow security audits."

**Lưu ý 5a:**
- Mini-table form is cleaner than expanding the main 4-column access matrix to 6 columns (which would make it unreadable in markdown).
- Worker decides exact insertion line (Tầng 2 doc structure).
- Bash scope ("parser scripts only") MUST appear in the Tools cell — this is the key contract per Constraint #3.

#### 5b. `docs/HANDOFF.md` — appendix for specialist-subagent handoff pattern

**File:** `docs/HANDOFF.md`

**Tìm:** end of file or after "## When to break the format" subsection.

**Thêm:** new appendix subsection `## Appendix — Specialist subagent handoffs (P041+)` describing the pattern:

- Trinh sát spawned by Quản đốc via `/advisory-scan` → returns sentinel-wrapped rows in final report → Quản đốc appends rows to inbox file → Chủ nhà reviews per-row at next session start.
- Same pattern will apply to Giám sát (P042 pending).
- Differs from the 5 main handoffs (0-4) because specialist subagent's output is data (advisory rows), not a phiếu or a code change. The handoff terminus is Chủ nhà's review queue, not a code commit.

**Lưu ý 5b:** 3-5 paragraph appendix. Worker self-writes wording (Tầng 2) but MUST capture: (a) Trinh sát is read-only-output, (b) Quản đốc spawn-only (does NOT invoke parser directly — Trinh sát does via scoped Bash), (c) Quản đốc mediates the inbox Write, (d) Chủ nhà gates each row, (e) parallel pattern for Giám sát coming P042.

#### 5c. `README.md` — Subagents table + Security mention

**File:** `README.md`

**Tìm:** Subagents table at lines 109-115 (Anchor #17 ✅).

**Thêm:** new row for Trinh sát:

```markdown
| **advisory-watch** (Trinh sát) | `agents/advisory-watch.md` | Read, Grep, Glob, WebFetch, WebSearch, Bash (scoped: parser scripts only) | Edit, Write, Task, Skill — read-only-output specialist (spawned by Quản đốc via `/advisory-scan`) |
```

**Thêm 2:** brief mention of security pipeline in Components section or Pipeline section. Suggested 1 sentence near the existing `sos init security` mention (line 70): "After `sos init security` writes `.sos-stack.toml`, run `/advisory-scan` to invoke Trinh sát (advisory-watch specialist subagent — P041) — surfaces GHSA + vendor advisories that hit your stack into `docs/security/advisory-inbox.md`."

**Lưu ý 5c:** Worker self-decides exact wording (Tầng 2). The Tầng 1 requirement: README documents Trinh sát exists, lists tools (including scoped Bash), links to slash command.

#### 5d. `docs/SETUP.md` — Security pipeline subsection

**File:** `docs/SETUP.md`

**Tìm:** Worker Glob full structure (per Anchor #18 ⚠️). Likely insertion: new top-level `## Security pipeline` section after "## Quick Start" (line 3-).

**Thêm:** new section content:

```markdown
## Security pipeline (P040 + P041)

Once your project has shipped its first version, optionally enable the security pipeline:

1. **Install PyYAML** (one-time, required by pnpm-lock parser):
   ```bash
   python3 -c 'import yaml' || pip3 install pyyaml
   ```
   (Trinh sát subagent also runs this check at Bước 0 — but pre-installing keeps the first scan smooth.)

2. **Detect stack** (one-time):
   ```bash
   sos init security
   ```
   Writes `.sos-stack.toml` documenting which package manifest + lock file your project uses. See P040 ship notes.

3. **Run advisory scan** (manual or via cron):
   In Claude Code session: `/advisory-scan`
   This spawns the Trinh sát subagent (read-only-output, scoped Bash for parser invocation) which queries GitHub Advisory Database + vendor pages, matches advisories against your resolved dep versions, and appends results to `docs/security/advisory-inbox.md`.

4. **Review inbox** (Chủ nhà):
   Open `docs/security/advisory-inbox.md`. For each row, either:
   - Mark status `dismissed` (false positive or unaffected code path), or
   - Create a follow-on phiếu via `phieu <slug>` to patch.

Currently implemented parsers: pnpm v9 (P041 ships) [+ npm v3 if Worker shipped Task 4b]. Other ecosystems (pip, cargo, go) have stubs only — implementation deferred to follow-on phiếu.
```

**Lưu ý 5d:** Worker self-decides exact wording + insertion point per Anchor #18. The Tầng 1 requirement: user finds `/advisory-scan` workflow discoverable from SETUP.md, and the PyYAML install requirement is documented somewhere (here OR pre-flight inside Trinh sát Bước 0 — Worker picks best UX).

#### 5e. `CLAUDE.md` — repo structure tree updates

**File:** `CLAUDE.md`

**Tìm:** repo structure tree (Architect via system context: tree under "## Repo structure"). Per Anchor #19 ⚠️ Worker Glob confirms structure.

**Thêm:** 3 lines in appropriate locations:
- `agents/` subtree: add `│   └── advisory-watch.md   # Trinh sát specialist subagent (P041 — scoped Bash, queries GHSA)`
- `templates/` subtree: add `│   └── advisory-inbox.md    # Empty queue template for security advisories (P041)`
- New top-level entry (alphabetically near `bin/`): `├── .claude/commands/         # Slash command files (P041+ — first: advisory-scan.md)`

**Lưu ý 5e:**
- Tree formatting MUST match existing ASCII box-drawing characters (`├──`, `│`, `└──`). Worker self-formats (Tầng 2 ASCII art).
- Comments after `#` brief — 1-line per entry — matches existing style.

**Validate (Task 5 all sub-steps):**
- `grep -c "Trinh sát\|advisory-watch" docs/LAYERS.md` ≥ 3.
- `grep -c "Trinh sát\|advisory-watch\|/advisory-scan" docs/HANDOFF.md` ≥ 2.
- `grep -c "advisory-watch\|Trinh sát" README.md` ≥ 2 (subagent table row + Security mention).
- `grep -c "/advisory-scan\|advisory-watch" docs/SETUP.md` ≥ 1.
- `grep -c "advisory-watch\|advisory-inbox\|\.claude/commands" CLAUDE.md` ≥ 2.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `agents/advisory-watch.md` | NEW — Task 1: full specialist subagent file (~200 lines, includes Bash usage scope section) |
| `templates/advisory-inbox.md` | NEW — Task 2: empty queue template with sentinel wrappers |
| `.claude/commands/advisory-scan.md` | NEW — Task 3: slash command orchestrator-side spawn-only caller (creates `.claude/commands/` directory) |
| `scripts/parsers/pnpm_lock_v9.py` | Task 4: implement `parse()` real logic with PyYAML (was P040 stub) |
| `scripts/parsers/package_lock_v3.py` | Task 4b OPTIONAL: implement `parse()` real logic (was P040 stub) — defer to follow-on phiếu if effort cap hit |
| `docs/LAYERS.md` | Task 5a: add Specialist subagents subsection + Trinh sát row (with scoped Bash note) |
| `docs/HANDOFF.md` | Task 5b: appendix for specialist-subagent handoff pattern |
| `README.md` | Task 5c: Subagents table row + Security mention |
| `docs/SETUP.md` | Task 5d: Security pipeline subsection (includes PyYAML install) |
| `CLAUDE.md` | Task 5e: repo structure tree updates (3 entries) |

(10 files total — 8 doc/agent files + 1 NEW slash command + 1-2 parser implementations.)

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `agents/orchestrator.md`, `agents/architect.md`, `agents/worker.md` | No edits. 3 main subagent contracts unchanged. |
| `phieu/TICKET_TEMPLATE.md` | No edits. Phiếu format unchanged. |
| `bin/sos.sh` | No edits. P040 already added `sos init security`; P041 doesn't extend the CLI. |
| `templates/.sos-stack.toml.example` | No edits. P040 schema unchanged. |
| `scripts/parsers/requirements_txt.py`, `pyproject_toml.py`, `cargo_lock.py`, `go_sum.py` | NO EDITS — stay as P040 empty stubs. P041 only implements pnpm (+ optionally package-lock). |
| `scripts/architect-guard.sh` | No edits. Architect-block on `.py` reads preserved. Trinh sát is a separate subagent — this hook doesn't affect Trinh sát's tool allowlist (which DOES include Bash, scoped). |
| `hooks/pre-commit` | No edits. No commit gating on inbox in P041. |
| `docs/PHILOSOPHY.md`, `docs/ORCHESTRATION.md` | No edits. Specialist subagent doesn't change 6 principles or state machine. |
| `phieu/RELAY_PROTOCOL.md`, `phieu/DISCOVERY_PROTOCOL.md`, `phieu/AUDIT_PROTOCOL.md` | No edits. Protocols unchanged. |
| `docs/BACKLOG.md` | No mid-execute edits. Orchestrator moves P041 from Active → Recently shipped post-merge. |
| `bootstrap/sos-rs/` | No edits. Rust port out of wave 1 scope. |

---

## Luật chơi (Constraints)

1. **Tier locked at 1 (móng nhà).** Must complete CHALLENGE round before EXECUTE (P036 Hard rule #7). New specialist subagent contract + new slash command + new sentinel marker schema → mismatch costs downstream phiếu (P042) + user projects. If CHALLENGE surfaces Tầng 2 sub-issues (e.g., exact ASCII tree formatting in CLAUDE.md), Worker self-decides and logs Discovery — only Tầng 1 issues (subagent tool allowlist, sentinel marker names, parser interface, slash command flow) need ACCEPT/DEFEND from Architect in RESPOND.
2. **Sentinel markers are LOAD-BEARING and FIXED.** `<!-- advisory-start -->` and `<!-- advisory-end -->` (lowercase + noun-form, per Sếp 2026-05-25) — these are NOT renaming-discretionary. Slash command parser, agent body example, inbox template all reference them verbatim. Renaming = breaking change requiring phiếu.
3. **Trinh sát is read-only-OUTPUT with scoped Bash.** Frontmatter `tools: Read, Grep, Glob, WebFetch, WebSearch, Bash` — Bash IS present, but **scoped** to parser script invocation only (`python3 scripts/parsers/*.py`, `python3 -c 'import yaml'`, `pip3 install pyyaml`). NO Edit/Write. NO arbitrary Bash (`rm`, `git`, `curl`, etc.). Scope documented explicitly in agent file body section "Bash usage" so future contributors aware. Worker MUST NOT add Bash invocations beyond the scope listed. Future expansion (e.g. OSV.dev POST via curl) requires new phiếu + scope review.
4. **`.sos-stack.toml` schema v1 is the contract from P040.** Trinh sát + slash command READ-ONLY consume `schema_version = 1`, `[[stack]]` entries with `type / manifest / lock_file / lock_format / parser` keys. Renaming a key = breaking change requiring `schema_version` bump + Sếp approval (touches P040 contract).
5. **Other 4 parsers stay stubs.** `requirements_txt.py`, `pyproject_toml.py`, `cargo_lock.py`, `go_sum.py` — P041 explicitly does NOT touch these. They ship as P040 stubs. Future phiếu (or follow-on P041-extension phiếu) fills them. Worker tempting "while I'm here, do pip too" → STOP. Scope discipline.
6. **PyYAML dependency ACCEPTED (closed decision per Sếp 2026-05-25).** Pnpm-lock v9 is structured YAML; hand-rolling parser is fragile; PyYAML is ubiquitous + battle-tested. Worker installs via pre-flight: `python3 -c 'import yaml' || pip3 install pyyaml` (runs in Trinh sát Bước 0 OR documented in `docs/SETUP.md` "Security" subsection — Worker decides best placement, Tầng 2). NO further escalation needed. Rationale closes Constraint #6 from open question (V1) → fixed decision (V2).
7. **Generic-able acceptance test.** `grep -rn "tarot\|INV-107\|extract-pnpm-versions" agents/advisory-watch.md templates/advisory-inbox.md .claude/commands/advisory-scan.md scripts/parsers/pnpm_lock_v9.py scripts/parsers/package_lock_v3.py` must return 0 hits at end of EXECUTE. This is the structural "strip tarot-specific" check.
8. **Slash command spec compliance.** `.claude/commands/advisory-scan.md` must conform to Claude Code's actual slash command format (Worker verifies at EXECUTE per Anchor #20). If Worker adapts the file's frontmatter or body structure to match spec, that's Tầng 2 file-format-detail (Worker self-decides + logs Discovery). Slash command MUST be spawn-only (no `python3 <parser>` invocation in main session — that's Trinh sát's job via scoped Bash). If Claude Code spec REQUIRES a structurally different invocation pattern that breaks spawn-only design, that's Tầng 1 — escalate.
9. **State persistence (`.advisory-scan-state` JSON) DEFERRED.** P041 = first-version scan with no dedup, no incremental window. Re-runs append cumulatively; Chủ nhà handles dedup via status updates. Tarot P282 schema may port in follow-on phiếu after P041 ships and real-use feedback accumulates.
10. **Vendor-page advisory queries OPTIONAL.** Agent body documents nginx / postgres / Alpine endpoints, but P041 default invocation is GHSA-only. Vendor expansion is a configurable behavior (agent receives `--include-vendor` flag from slash command); P041 ships without the flag enabled. Future phiếu opt-in if user requests.

---

## Nghiệm thu

### Automated

- [ ] All 9 (or 10 if Task 4b shipped) new/modified files exist at correct paths.
- [ ] `agents/advisory-watch.md` frontmatter parses as valid YAML (`name`, `description`, `tools`, `model`).
- [ ] `agents/advisory-watch.md` frontmatter `tools:` line contains `Bash` (per Constraint #3 V2 update).
- [ ] `agents/advisory-watch.md` body contains "Bash usage" section explicitly listing allowed + forbidden invocations.
- [ ] `python -m py_compile scripts/parsers/pnpm_lock_v9.py` exits 0.
- [ ] `python -m py_compile scripts/parsers/package_lock_v3.py` exits 0 (if Task 4b shipped — else stays a stub which already compiles).
- [ ] `grep -rn "tarot\|INV-107\|extract-pnpm-versions" agents/advisory-watch.md templates/advisory-inbox.md .claude/commands/advisory-scan.md scripts/parsers/pnpm_lock_v9.py 2>/dev/null` returns 0 hits (Generic-able test per Constraint #7).
- [ ] `grep -c "advisory-start\|advisory-end" agents/advisory-watch.md` = 2 (markers appear once each in Bước 5 example).
- [ ] `grep -c "advisory-start\|advisory-end" templates/advisory-inbox.md` = 2 (in Rows section).
- [ ] `grep -c "advisory-start\|advisory-end" .claude/commands/advisory-scan.md` ≥ 4 (multiple steps reference markers).
- [ ] `grep -c "python3 " .claude/commands/advisory-scan.md` = 0 (slash command MUST NOT invoke parser in main session — that's Trinh sát's job per Sếp REDESIGN).
- [ ] No `tomllib` ImportError on `python -c "import tomllib"` for Python ≥ 3.11; if Worker codepath uses `tomllib`, fallback for 3.10 documented.

### Manual Testing (dry-run)

- [ ] **Mono-stack Node + pnpm dry-run.** In a scratch dir with `package.json` + `pnpm-lock.yaml` containing 1-2 known-vulnerable deps (e.g. an old `lodash` ≤4.17.20 with known GHSA, or any historical CVE'd version):
  ```bash
  mkdir -p /tmp/sos-p041-test && cd /tmp/sos-p041-test
  # create package.json + intentionally-vulnerable pnpm-lock.yaml (Worker fabricates or copies)
  source ~/sos-kit/bin/sos.sh
  sos init security
  # back in Claude Code session:
  /advisory-scan
  cat docs/security/advisory-inbox.md
  ```
  Expect: inbox has ≥1 row appended between `<!-- advisory-start -->` and `<!-- advisory-end -->`, format matches 8-column schema, severity sourced from GHSA reviewed badge.
- [ ] **Empty advisory case.** Scratch dir with all-current deps (no known CVE). Run `/advisory-scan` → expect "0 new advisories" message, inbox sentinel block stays empty (markers preserved), no errors.
- [ ] **No `.sos-stack.toml` case.** Scratch dir without running `sos init security` first. Run `/advisory-scan` → expect graceful "run `sos init security` first" message, no crash.
- [ ] **No inbox file case.** Scratch dir with `.sos-stack.toml` but no `docs/security/advisory-inbox.md`. Run `/advisory-scan` → expect slash command bootstraps inbox from `templates/advisory-inbox.md` (per Step 0 of slash command), then proceeds.
- [ ] **Stub-only parser case.** Scratch dir with `pyproject.toml` only (no Node deps). Run `/advisory-scan` → expect "skipping pypi — parser not yet implemented" message; no crash; inbox unchanged.
- [ ] **PyYAML missing case.** Scratch env where `import yaml` fails AND `pip3 install pyyaml` fails (no network / no permissions). Run `/advisory-scan` → Trinh sát emits warning "PyYAML required for pnpm-lock parsing; install + retry", returns empty sentinel block, slash command relays message verbatim. No crash.

### Regression

- [ ] `sos init security` (from P040) still works identically — Architect Read P040 phiếu confirms this; P041 doesn't touch `bin/sos.sh`.
- [ ] All 4 OTHER parser stubs (`requirements_txt.py`, `pyproject_toml.py`, `cargo_lock.py`, `go_sum.py`) unchanged — `git diff scripts/parsers/{requirements_txt,pyproject_toml,cargo_lock,go_sum}.py` empty.
- [ ] `templates/.sos-stack.toml.example` unchanged (P040 schema not touched).
- [ ] `agents/orchestrator.md`, `agents/architect.md`, `agents/worker.md` unchanged.
- [ ] `phieu/TICKET_TEMPLATE.md` unchanged.
- [ ] `docs/PHILOSOPHY.md`, `docs/ORCHESTRATION.md` unchanged.
- [ ] Existing skills (`/init`, `/plan`, `/verify`, etc.) all still invoke per `docs/LAYERS.md` skills map.

### Docs Gate

- [ ] `CHANGELOG.md` — new entry at top: "P041: Trinh sát (advisory-watch) specialist subagent — generic port from tarot, strips tarot-specific paths. Adds `agents/advisory-watch.md` (read-only-output specialist, tools: Read/Grep/Glob/WebFetch/WebSearch/Bash-scoped-to-parsers), `templates/advisory-inbox.md` (queue with sentinel wrappers), `.claude/commands/advisory-scan.md` (orchestrator-side spawn-only caller). Implements `scripts/parsers/pnpm_lock_v9.py` (+ optionally `package_lock_v3.py`) with PyYAML. Other 4 parsers stay P040 stubs. Sentinel markers: `<!-- advisory-start --> / <!-- advisory-end -->`. State persistence + vendor-page expansion deferred to follow-on phiếu."
- [ ] `docs/LAYERS.md` — Specialist subagents subsection added (Task 5a).
- [ ] `docs/HANDOFF.md` — appendix added (Task 5b).
- [ ] `README.md` — Subagents table row + Security mention (Task 5c).
- [ ] `docs/SETUP.md` — Security pipeline subsection (Task 5d, includes PyYAML install).
- [ ] `CLAUDE.md` — repo tree updates (Task 5e).
- [ ] `docs/BACKLOG.md` — P041 row moved from Active sprint to "Recently shipped" (orchestrator handles post-merge, NOT Worker mid-execute).

### Discovery Report

- [ ] Write to `docs/discoveries/P041.md` (per-phiếu file, P038 pattern):
  - **Assumptions in phiếu — CORRECT** (per Task 0 verification results).
  - **Assumptions in phiếu — WRONG / adapted** — particularly: (a) Anchor #5 P040 stub state at HEAD (did it match P040 spec or drift?), (b) Anchor #18 SETUP insertion point Worker chose, (c) Anchor #19 CLAUDE tree placement Worker chose, (d) Anchor #20 slash command format Worker adapted.
  - **Scope expansions / contractions** — Task 4b shipped or deferred? PyYAML install placement (Trinh sát Bước 0 vs SETUP.md vs both)?
  - **Tarot port faithfulness** — Worker reports: which tarot patterns ported verbatim, which adapted, which dropped. Confirm Constraint #7 generic-able test passed.
  - **Sentinel marker rename impact** — `INBOX_APPEND_START/END` → `advisory-start/end` rename held cleanly across agent + template + slash command? Any drift?
  - **V2 redesign validation** — Did "parser-runs-in-subagent" (vs V1 "parser-runs-in-orchestrator") flow work cleanly in dry-run? Any Claude Code spec friction with scoped Bash? Sentinel block extraction across subagent boundary clean?
  - **Tier escalations (mid-execute)** — Constraint #8 (slash command spec incompat)? Other?
  - **CHALLENGE round value** — Was the CHALLENGE round (Worker → Architect) valuable for this Tầng 1 phiếu? V1→V2 caught 3 real issues (PyYAML open, parser location contradiction, Bash scope contradiction). Confirm in retro.
  - **Token + time cost** — Architect estimate: half-day. Actual? Wave-1 second-phiếu data point (P040 baseline).
  - **Trinh sát first-scan UX** — Dry-run nghiệm thu: did the slash command + agent loop produce a usable inbox row? Worth shipping to user projects, or rough-edges-need-follow-on?
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`.
