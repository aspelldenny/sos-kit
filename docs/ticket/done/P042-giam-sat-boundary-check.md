# PHIẾU P042: Giám sát (boundary-check) generic agent

> **Loại:** Feature (new specialist subagent + new slash command + new INVARIANTS template — second specialist subagent in wave 1, mirrors P041 Trinh sát pattern)
> **Ưu tiên:** P1 (wave 1 final — security pipeline both sides shipped; user-visible `/security-review` command)
> **Tầng:** 1 (móng nhà — new public subagent contract, new slash command surface, new `INVARIANTS-template.md` 5-INV schema; downstream user projects + future phiếu depend on these interfaces)
> **Ảnh hưởng:** `agents/boundary-check.md` (NEW), `templates/INVARIANTS-template.md` (NEW), `.claude/commands/security-review.md` (NEW), `docs/LAYERS.md` (Giám sát row in Specialist subagents table — currently "(P042 will spec)" placeholders), `docs/HANDOFF.md` (appendix — Giám sát entry, currently "Upcoming: Giám sát (P042 — pending)"), `README.md` (Subagents table row + Security section sentence), `docs/SETUP.md` (Security pipeline subsection extension), `CLAUDE.md` (repo structure tree — 2 new lines: `agents/boundary-check.md` + `templates/INVARIANTS-template.md`; `.claude/commands/` entry already added by P041)
> **Dependency:** P040 SHIPPED — `.sos-stack.toml` schema (not directly read by Giám sát, but conceptually paired). P041 SHIPPED — established the specialist-subagent pattern (Trinh sát) Giám sát mirrors. P043 SHIPPED — Quản đốc persona codify.

---

## Context

### Vấn đề hiện tại

Wave 1 status (2026-05-25): P040 stack-detect shipped, P041 Trinh sát (advisory-watch, soi ADVISORY ngoài) shipped, P043 Quản đốc persona codify shipped. Wave 1 GOAL per BACKLOG line 13: "Done when: 4 phiếu shipped … `sos init security` detect stack đúng → `/advisory-scan` chạy zero-workaround → `/security-review <PR>` post advisory comment." P042 is the FINAL phiếu — ships the "soi INVARIANT trong" half (Giám sát / boundary-check, paired with Trinh sát).

Gap thực tế:
- Subagent file `agents/boundary-check.md` không tồn tại trong sos-kit (Architect Glob 2026-05-25: `agents/` chỉ có `orchestrator.md`, `architect.md`, `worker.md`, `advisory-watch.md` (P041), `README.md`).
- `.claude/commands/` directory tồn tại sau P041 với 1 file (`advisory-scan.md`); P042 adds second file `security-review.md`.
- `templates/INVARIANTS-template.md` không tồn tại — user project cần skeleton 5 generic INV + placeholder cho project-specific extensions.
- Tarot source `~/tarot/.claude/agents/boundary-check.md` (Architect Read 2026-05-25, 151 lines) chứa 7 INV mixed generic + tarot-specific. P042 strips 7→5 generic.
- `docs/LAYERS.md` Specialist subagents table (added by P041, lines 36-42) has Giám sát column with all cells showing `(P042 will spec)` placeholders — P042 fills them.
- `docs/HANDOFF.md` appendix (added by P041, lines 270-316) has stub subsection "Upcoming: Giám sát (boundary-check, P042 — pending)" (line 307-309) — P042 expands it to full handoff entry.

### Giải pháp

4 deliverables + 1 doc consolidation:

1. **`agents/boundary-check.md` (NEW)** — generic specialist subagent file. Frontmatter `tools: Read, Grep, Glob, Bash` (Bash **scoped** to PR-diff capture only — `git diff`, `git show`, `git log`, `grep` for code analysis). Body: 5 INV explanation + invocation contract + ADVISORY mode emphasis. Persona name **Giám sát** (Vietnamese). Filename `boundary-check.md` (English) matches sibling pattern (`advisory-watch.md`, `architect.md`, `worker.md`, `orchestrator.md`).

2. **`templates/INVARIANTS-template.md` (NEW)** — skeleton 5 INV (generic, project-agnostic) + placeholder section "User-added INV (project-specific)" for user to extend. Format: numbered list with `**Statement**`, `**Rubric soi diff**`, `**Output format**` per INV (mirrors tarot pattern lines 30-109 but generic).

3. **`.claude/commands/security-review.md` (NEW)** — slash command. Orchestrator-side spawn-only flow per Sếp wave-1 design (parallels P041's `/advisory-scan` slash command): (a) determine PR/branch/commit-range to review, (b) capture diff via `gh pr diff` or `git diff` (main session — Quản đốc has Bash for marker ops, but PR-diff capture is a single-purpose external call documented as allowed in this slash command's "Hard rules" — Worker verifies feasibility at EXECUTE per Anchor #18), (c) spawn Giám sát subagent passing diff content in prompt, (d) parse Giám sát's sentinel-wrapped advisory block, (e) post to PR via `gh pr comment` if PR context exists, OR write to local file as fallback. **ADVISORY mode — KHÔNG block merge.**

4. **5 INV generic** (port from tarot, strip 2 tarot-specific):
   - **INV-1: ENV VAR changes** — PR adds new `process.env.<KEY>` / `os.environ.get(<KEY>)` / `std::env::var(<KEY>)` etc. PHẢI có corresponding update tới `.env.example` (or equivalent env-template-doc per stack). Generic across npm/python/rust/go.
   - **INV-2: EXTERNAL SERVICE call** — PR adds new HTTP/external-API call without explicit timeout AND error-handling AND (optional) retry. Generic risk: hung connections / silent failures / cascading outage.
   - **INV-3: CROSS-USER data access** — PR adds API route/handler that reads or mutates user-scoped data (DB query, cache key, session state) PHẢI có explicit ownership-binding (`where userId = session.user.id` or equivalent). Generic data-leak risk.
   - **INV-4: WEBHOOK signature verify** — PR adds inbound webhook handler PHẢI verify signature/HMAC + replay protection (nonce or timestamp window) before reading request body. Generic injection/spoofing risk.
   - **INV-5: DEPENDENCY major bump** — PR bumps any dep's major version PHẢI cite changelog/migration-guide review in PR description. Generic breaking-change risk; complements Trinh sát's GHSA scan (Trinh sát flags known CVEs, Giám sát flags discipline of audit-before-bump).

5. **Doc consolidation** — `docs/LAYERS.md` Giám sát column fill (replace `(P042 will spec)` placeholders); `docs/HANDOFF.md` "Upcoming: Giám sát" stub → full entry; `README.md` Subagents table row + Security section sentence; `docs/SETUP.md` Security pipeline subsection extension (Step 5 = `/security-review`); `CLAUDE.md` repo tree add 2 entries.

**Generic-able invariants** (strip-tarot test):
- NO `INV-102 nginx` references — tarot has `nginx/conf.d/soulsign.conf` specific to tarot infra. Drop entirely.
- NO `INV-105 users.credits` references — tarot has Prisma `$transaction` requirement on credit ledger. Drop entirely.
- NO `SURFACE_MAP.md` references — tarot doc, kit-level neutral.
- NO 10-deps allowlist (tarot has critical-deps list in INV-107). P042 INV-5 generic = "any dep major bump" not allowlist-bounded.
- NO project-specific path hardcodes (`docs/security/INVARIANTS.md` — kit ships template at `templates/INVARIANTS-template.md`, user copies to wherever they want).
- Sentinel markers: tarot uses `<!-- SECURITY_REVIEW_START/END -->` (caps + verb-form). P042 uses `<!-- security-review-start --> ... <!-- security-review-end -->` (lowercase + noun-form, matches P041 `<!-- advisory-start/end -->` convention).

### Scope

- CHỈ sửa / tạo:
  - `agents/boundary-check.md` — NEW specialist subagent file (~180 lines target, mirrors P041's `advisory-watch.md` structure).
  - `templates/INVARIANTS-template.md` — NEW skeleton with 5 generic INV + user-added section.
  - `.claude/commands/security-review.md` — NEW slash command.
  - `docs/LAYERS.md` — fill Giám sát column in Specialist subagents table (lines 36-42 per P041 ship).
  - `docs/HANDOFF.md` — replace "Upcoming: Giám sát (boundary-check, P042 — pending)" stub (lines 307-309) with full handoff entry + table row in summary section.
  - `README.md` — Subagents table row for Giám sát; Security section sentence mentioning `/security-review` alongside `/advisory-scan` (currently P041's line 72 only mentions `/advisory-scan`).
  - `docs/SETUP.md` — Security pipeline subsection extension. Worker verifies actual SETUP.md current state via Glob (Architect didn't fresh-Read this file in DRAFT, Anchor #15 ⚠️).
  - `CLAUDE.md` — repo structure tree adds `agents/boundary-check.md` + `templates/INVARIANTS-template.md` lines.

- KHÔNG sửa:
  - `agents/orchestrator.md`, `agents/architect.md`, `agents/worker.md`, `agents/advisory-watch.md` — 4 existing subagent contracts unchanged. P042 adds a 5th specialist (Giám sát) without touching siblings.
  - `phieu/TICKET_TEMPLATE.md` — phiếu format unchanged.
  - `bin/sos.sh` — `sos init security` from P040 unchanged. No new subcommand in P042.
  - `templates/.sos-stack.toml.example` — P040 schema unchanged (Giám sát does NOT read `.sos-stack.toml`; it reads PR diff directly).
  - `templates/advisory-inbox.md` — P041 template unchanged.
  - `.claude/commands/advisory-scan.md` — P041 slash command unchanged.
  - `scripts/parsers/*` — Giám sát uses git/diff directly, no parser dep.
  - `scripts/architect-guard.sh` — Architect-block hook unchanged. Giám sát is a separate subagent with its own tool allowlist; envelope hook doesn't affect it.
  - `hooks/pre-commit` — NO commit gating in P042. ADVISORY only.
  - `docs/PHILOSOPHY.md`, `docs/ORCHESTRATION.md` — no edits.
  - `phieu/RELAY_PROTOCOL.md`, `phieu/DISCOVERY_PROTOCOL.md`, `phieu/AUDIT_PROTOCOL.md` — protocols unchanged.
  - `docs/BACKLOG.md` — orchestrator handles move from Active → Recently shipped post-merge, NOT Worker mid-EXECUTE.
  - `bootstrap/sos-rs/` — Rust port out of wave 1 scope.

### Skills consulted (optional)

<!-- None for P042 — no `/frontend-design` / `/security-review` skill invoked by Quản đốc before DRAFT. (Note ironic naming: this phiếu IS adding the `/security-review` slash command but no pre-existing skill is consulted.) Leave blank. -->

---

## Task 0 — Verification Anchors

> **Architect humility note:** Architect Read `phieu/TICKET_TEMPLATE.md` + `docs/BACKLOG.md` (Active sprint confirmed P042 line 18) + `~/tarot/.claude/agents/boundary-check.md` (tarot source — explicit Read succeeded, 151 lines) + `phieu/done/P041-trinh-sat-advisory-watch.md` (pattern reference + giọng) + `agents/advisory-watch.md` (P041 shipped — frontmatter + body structure reference) + `agents/architect.md` (lines 1-30) + `agents/worker.md` (lines 1-15) + `docs/LAYERS.md` (full, Specialist subagents table P041-shipped state confirmed lines 32-42) + `docs/HANDOFF.md` (full, appendix P041-shipped + "Upcoming: Giám sát" stub at 307-309 confirmed) + `README.md` (lines 60-180 — Subagents table P041 state confirmed at 109-122, Security section sentence at 72) + `.claude/commands/advisory-scan.md` (P041 slash command — pattern reference) + `templates/advisory-inbox.md` (P041 template — sentinel + structure reference) end-to-end trong DRAFT session 2026-05-25. Architect did NOT fresh-Read `docs/SETUP.md` full structure, `CLAUDE.md` tree section, or any code/script file — Worker re-verifies at EXECUTE per ⚠️ anchors below.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Tarot source `~/tarot/.claude/agents/boundary-check.md` exists + readable. 7 INV present: INV-101 (env var → secret table), INV-102 (nginx block), INV-103 (external service), INV-104 (cross-user resource), INV-105 (`users.credits` $transaction), INV-106 (webhook → signature), INV-107 (npm major bump). | `cat ~/tarot/.claude/agents/boundary-check.md \| grep -E '^### INV-'` | ✅ [verified] — Architect Read 2026-05-25, 151 lines. 7 INV headings confirmed at lines 32, 44, 54, 66, 76, 88, 98. |
| 2 | Tarot INV-102 (nginx block → `SURFACE_MAP.md §2.1 + §3` map update) at lines 44-52 IS tarot-specific (mentions `nginx/conf.d/soulsign.conf` line 47). P042 DROPS this INV (kit-level neutral — sos-kit doesn't assume any specific reverse-proxy). | (manual review of tarot lines 44-52) | ✅ [verified] — Architect re-read 2026-05-25; nginx path is hardcoded tarot infra. DROP confirmed. |
| 3 | Tarot INV-105 (`users.credits` field + `$transaction`) at lines 76-85 IS tarot-specific (mentions `users.credits` line 76, Prisma `$transaction` line 84). P042 DROPS this INV (kit-level neutral — sos-kit doesn't assume Prisma, doesn't assume credit-ledger domain model). | (manual review of tarot lines 76-85) | ✅ [verified] — Architect re-read 2026-05-25; INV-105 is credit-ledger concurrency, domain-bound. DROP confirmed. |
| 4 | Remaining 5 tarot INV (101 env var, 103 external service, 104 cross-user, 106 webhook signature, 107 npm major bump) ARE generic across stacks. P042 keeps these 5, renumbers INV-1 → INV-5, strips tarot-specific path refs (`SURFACE_MAP.md §2.3`, `next-auth`, `@payos/node`, etc.). | (manual mapping tarot → P042) | ✅ [verified] — Mapping: tarot INV-101 → P042 INV-1 (env var); INV-103 → INV-2 (external service); INV-104 → INV-3 (cross-user); INV-106 → INV-4 (webhook); INV-107 → INV-5 (dep major bump). 5 INV generic confirmed. |
| 5 | `agents/` directory currently has 5 files: `orchestrator.md`, `architect.md`, `worker.md`, `advisory-watch.md` (P041), `README.md`. `boundary-check.md` does NOT exist. | `ls agents/` | ✅ [verified] — Architect Glob 2026-05-25 confirms 4 .md files + README.md. `boundary-check.md` = NEW. |
| 6 | `.claude/commands/` directory exists post-P041 with 1 file: `advisory-scan.md`. P042 adds `security-review.md` (second file in this dir). Worker does NOT need to mkdir. | `ls .claude/commands/` | ✅ [verified] — Architect Read confirms `.claude/commands/advisory-scan.md` exists (read it in DRAFT). Directory exists. P042 file = NEW second file. |
| 7 | `templates/` currently has 5 files after P041: `BACKLOG_template.md`, `claude-settings.local.json`, `.docs-gate.toml`, `.sos-stack.toml.example`, `advisory-inbox.md`. `INVARIANTS-template.md` does NOT exist. | `ls templates/` | ⚠️ [needs Worker verify] — Architect did not fresh-Glob `templates/` in this DRAFT; relies on P041 phiếu's Anchor #3 (`templates/` had 4 files pre-P041) + P041 shipped `advisory-inbox.md` → 5 total. Worker confirms exact count at EXECUTE. |
| 8 | Tarot subagent frontmatter (line 1-6): `name: boundary-check`, `description: Read-only judgment invariant gate (INV-101 → INV-107 trong docs/security/INVARIANTS.md). Soi PR diff post-push ...`, `tools: Read, Grep, Glob`, `model: sonnet`. P042 generic version: same `name: boundary-check`, generic description (no tarot ref), `tools: Read, Grep, Glob, Bash` (Bash ADDED — see Anchor #9), `model: sonnet`. | `head -7 ~/tarot/.claude/agents/boundary-check.md` | ✅ [verified] — Architect Read tarot frontmatter at lines 1-6 confirmed. P042 ADDS `Bash` to tools list (rationale Anchor #9). |
| 9 | **Bash scope decision.** Tarot does NOT have Bash in tools list (line 4: `tools: Read, Grep, Glob`) — tarot subagent assumes diff is pre-captured and embedded in spawn prompt (tarot's `/security-review` slash command captures diff via `gh pr diff` in main session, passes diff content to subagent via Task prompt). P042 follows tarot pattern OR adds Bash to subagent? **Architect decision: ADD Bash, scoped to git/grep ops only.** Rationale: (a) subagent can re-grep diff for cross-INV correlation; (b) subagent can `git log` to check whether PR description mentions changelog (INV-5); (c) parallels P041 Trinh sát which has scoped Bash for parser invocation — consistent specialist-subagent pattern. Scope: `git diff`, `git show`, `git log`, `grep`. Forbidden: `Edit/Write` (no tools), `rm/mv/cp` (mutate fs), `gh pr comment` (slash command's job, not subagent). | (design decision) | ✅ [design DECIDED] — Bash present, scoped, documented explicitly in agent body section "Bash usage" mirroring P041 advisory-watch line 14-25. Slash command STILL captures diff in main session and passes to subagent (defense-in-depth: subagent shouldn't need to call `gh`, but `git diff` re-confirm on already-checked-out worktree is acceptable scope). |
| 10 | Tarot sentinel markers `<!-- SECURITY_REVIEW_START --> ... <!-- SECURITY_REVIEW_END -->` at lines 118 + 130. P042 RENAMES to lowercase noun-form: `<!-- security-review-start --> ... <!-- security-review-end -->` (matches P041 `<!-- advisory-start/end -->` convention per Sếp 2026-05-25). Markers are LOAD-BEARING (slash command grep-extracts). | `grep "SECURITY_REVIEW" ~/tarot/.claude/agents/boundary-check.md` | ✅ [verified] — tarot markers confirmed at lines 118, 130. P042 RENAMES. Agent body + slash command + (NO inbox file — Giám sát outputs PR comment, not inbox; differs from Trinh sát here, see Anchor #14) all must use new marker name consistently. |
| 11 | Tarot verdict format (lines 117-130) is sentinel-wrapped block with 7 INV one-per-line + final `Verdict: APPROVE | NEEDS_REVIEW (≥1 ⚠️)` line. P042 same structure but 5 INV. Silent-when-clean rule (tarot line 135 "P275 silent-when-clean"): `APPROVE + 0 FLAG → exit silently, KHÔNG post comment`. P042 preserves silent-when-clean (generic anti-approve-fatigue rule). | `sed -n '117,135p' ~/tarot/.claude/agents/boundary-check.md` | ✅ [verified] — tarot format confirmed. P042 inherits structure + silent-when-clean rule verbatim (generic, not tarot-specific). |
| 12 | `docs/LAYERS.md` Specialist subagents table at lines 36-42 (per P041 ship) has Giám sát column with all cells showing `(P042 will spec)` placeholders: Role, Spawned by, Tools, Cannot, Output. P042 fills these 5 cells. | `sed -n '36,42p' docs/LAYERS.md` | ✅ [verified] — Architect Read 2026-05-25 confirms 4 placeholders (`(P042 will spec)`) in Giám sát column. P042 fills with concrete values from agent file frontmatter + body. |
| 13 | `docs/HANDOFF.md` appendix at lines 270-316 (per P041 ship) has subsection "### Upcoming: Giám sát (boundary-check, P042 — pending)" at lines 307-309 (3 lines: heading + 1 sentence "Same structural pattern as Trinh sát, but scans INVARIANT map internally rather than querying external advisory databases. P042 will spec the Giám sát tools, input (`INVARIANTS.md`), and output format. Likely same sentinel-marker-based inbox pattern."). P042 REPLACES this 3-line stub with full handoff entry (3-5 paragraphs). Also adds row to summary table at line 313-315. | `sed -n '307,316p' docs/HANDOFF.md` | ✅ [verified] — Architect Read 2026-05-25; stub confirmed at lines 307-309, summary table at 313-315 has only Trinh sát row. P042 expands subsection + adds Giám sát row to summary table. **Architect note:** the stub text says "scans INVARIANT map internally" + "Likely same sentinel-marker-based **inbox** pattern" — but P042 differs from Trinh sát: Giám sát's terminus is PR comment (or local file fallback), NOT an inbox file. Worker updates HANDOFF stub to reflect actual P042 design. |
| 14 | **Giám sát terminus differs from Trinh sát.** Trinh sát: rows → `<inbox>.md` append (persistent queue, Chủ nhà reviews session-by-session). Giám sát: ADVISORY comment → PR (or local file fallback if no PR context). Giám sát does NOT have a persistent inbox file (a PR's comment thread IS the queue — review-bounded lifecycle). | (design decision) | ✅ [design DECIDED] — Architect: PR comment thread is the natural review queue for security review (already where merge decision happens). NO `templates/security-review-inbox.md` shipped — different shape from P041. Worker: if PR context missing (e.g., user runs `/security-review` against a local branch with no PR), slash command falls back to writing the sentinel block to a local file (Worker decides filename — Tầng 2; suggested `docs/security/last-review.md` but not load-bearing). |
| 15 | `docs/SETUP.md` full structure (sections, headings, P040 + P041 insertion points). Architect did NOT fresh-Read this file in DRAFT — last Read was via P041 phiếu context (P041 added "## Security pipeline (P040 + P041)" subsection per Task 5d, line ~700-728 in phiếu spec). Worker re-Globs full SETUP.md structure to confirm P041's Security pipeline subsection is at expected line range + decides where to slot `/security-review` step. | `grep -n "^### \|^## " docs/SETUP.md` | ⚠️ [needs Worker verify] — Worker reads + decides insertion. Tầng 2 doc placement; ANY working extension to the P041 "Security pipeline" subsection (adding Step 5 = `/security-review`) is acceptable. |
| 16 | `CLAUDE.md` repo structure tree section. P041 added 3 lines (per P041 phiếu Task 5e): `agents/advisory-watch.md`, `templates/advisory-inbox.md`, `.claude/commands/` entry. P042 adds 2 lines: `agents/boundary-check.md` + `templates/INVARIANTS-template.md`. (`.claude/commands/` directory entry already exists from P041; P042 does NOT add another top-level dir entry — just one more file inside, which by tree convention may or may not be listed individually.) | `grep -n "^├──\|^│   ├──\|advisory-watch\|advisory-inbox\|\.claude/commands" CLAUDE.md` | ⚠️ [needs Worker verify] — Worker confirms P041's 3 tree entries shipped + decides where to slot 2 new lines (Tầng 2 ASCII placement). May choose to add `security-review.md` as a sub-entry under `.claude/commands/` if P041 expanded that subtree, or leave it as a directory-level note. |
| 17 | `README.md` Subagents table at lines 109-118 (per P041 ship) has 4 rows: orchestrator, architect, worker, advisory-watch. P042 adds 5th row for boundary-check (Giám sát). Tools cell: `Read, Grep, Glob, Bash (scoped: git diff / show / log + grep)`. Cannot cell: `Edit, Write, Task, Skill, arbitrary Bash, gh pr comment`. | `sed -n '109,120p' README.md` | ✅ [verified] — Architect Read 2026-05-25 confirms 4 rows + structure. P042 adds 5th row matching format. |
| 18 | **Slash command Bash usage for diff capture.** Slash commands run inside main Claude Code session (Quản đốc). Per `docs/LAYERS.md` line 27 access matrix: Quản đốc has `Bash (marker file ops only)`. **Question:** is `gh pr diff` / `git diff` considered marker-file-ops or general-shell? Architect interpretation: PR-diff capture is a single-purpose external read (no fs mutation, no state change) — should be allowed for this slash command's stated purpose. Worker verifies at EXECUTE: if Quản đốc's Bash allowlist actually blocks `gh`/`git`, slash command falls back to instructing user to paste diff manually OR Worker escalates Tầng 1 (slash command can't function without diff access). | (Claude Code spec / hook config) | ⚠️ [needs Worker verify] — Worker checks Quản đốc Bash allowlist at EXECUTE (likely in `.claude/settings.local.json` or similar). If blocked, fallback path = user pastes diff into slash command prompt. Tầng 1 IF blocked AND no fallback works. |
| 19 | `agents/boundary-check.md` filename = English (matches sibling pattern `advisory-watch.md` per P041, `architect.md`, `worker.md`, `orchestrator.md`). Persona name **Giám sát** appears in agent body throughout. | (naming convention) | ✅ [design DECIDED] — Architect: English filename, Vietnamese persona body. Confirmed consistent with P041. |
| 20 | Slash command file format `.claude/commands/security-review.md` — should match P041's `.claude/commands/advisory-scan.md` format (frontmatter with `description:` + body with numbered steps + Hard rules section). Worker can copy-adapt structure from P041 file directly. | (file format) | ✅ [verified — pattern reference] — Architect Read `.claude/commands/advisory-scan.md` in DRAFT (60 lines, frontmatter `description:` only, body with `## Step 0/1/2/3/4` + `## Hard rules`). P042 mirrors exact structure, swaps content for security-review flow. |

**Summary:** 16 ✅ verified/decided + 4 ⚠️ (Anchor #7 templates count — Worker re-Globs; Anchor #15 SETUP.md structure — Worker decides insertion; Anchor #16 CLAUDE.md tree — Worker confirms + slots; Anchor #18 Quản đốc Bash allowlist for `gh pr diff` — Worker checks at EXECUTE, may force slash command fallback). No ❌. Tầng 1 phiếu — Worker MUST CHALLENGE before EXECUTE per ORCHESTRATION.md Hard rule #7 / P036.

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

> Tầng 1 phiếu — Worker MUST CHALLENGE before EXECUTE (per ORCHESTRATION.md Hard rule #7 / P036). New public subagent contract + new slash command surface + new INVARIANTS-template schema → mismatch costs future user projects + downstream phiếu (any project that copies `templates/INVARIANTS-template.md` as starting point). 4 ⚠️ anchors documented — Worker CHALLENGE verifies template directory count + doc insertion points + Quản đốc Bash allowlist for `gh pr diff`.

**Phiếu version:** V1 (initial draft)

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

**Status:** ⏳ AWAITING ARCHITECT RESPONSE

### Turn 1 — Architect Response

*(Architect fills this when invoked in RESPOND mode. Cannot read source code — relies on Worker's `file:line` citations.)*

- [O1.1] → ACCEPT / DEFEND / REFRAME (Tầng 2) / DEFER TO CHỦ NHÀ → action taken

**Status:** ✅ RESPONDED — phiếu bumped to V2

*(Repeat Turn 2, Turn 3 if needed. Cap = 3.)*

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Chủ nhà: [date] — code execution may begin

---

## Nhiệm vụ

> Worker order: Pre-phiếu snapshot → Task 0 verify → Task 1 (agent file) → Task 2 (INVARIANTS template) → Task 3 (slash command) → Task 4 (doc consolidation: LAYERS / HANDOFF / README / SETUP / CLAUDE) → Nghiệm thu. Tasks 1+2+3 are independent of each other; Task 4 last (references files from 1-3).

### Task 1: `agents/boundary-check.md` — NEW specialist subagent file

**File:** `agents/boundary-check.md` (NEW; directory exists, sibling of `advisory-watch.md`)

**Thêm (full file content — Worker writes verbatim, adapting `[needs Worker verify]` items at EXECUTE):**

````markdown
---
name: boundary-check
description: Giám sát — read-only-output specialist subagent. Soi PR diff (or branch/commit-range diff) chống 5 generic boundary invariants (env var / external service / cross-user / webhook / dep major bump). Return sentinel-wrapped advisory verdict for caller (slash command `/security-review`) to post as PR comment OR write to local fallback file. ADVISORY mode — KHÔNG block merge. Companion to Trinh sát (advisory-watch, P041): Trinh sát soi advisory NGOÀI (external CVE/GHSA), Giám sát soi INVARIANT TRONG (boundary discipline). KHÔNG patch lỗ. KHÔNG ghi luật. KHÔNG cầm Write/Edit/gh tool. Bash scoped to git/grep ops only.
tools: Read, Grep, Glob, Bash
model: sonnet
---

# Giám sát — Boundary-check specialist subagent

Em là **Giám sát** trong sos-kit security pipeline. Vai trò: soi PR diff (or arbitrary diff range) chống 5 generic boundary invariants, surface advisory verdict cho Quản đốc post lên PR comment (or local file fallback). Em là **specialist subagent**, không phải 1 trong 3 main roles (Chủ nhà / Kiến trúc sư / Thợ) — em ngồi cạnh chúng, được Quản đốc spawn qua slash command `/security-review`.

Cặp đôi: **Trinh sát** (advisory-watch, P041) soi advisory thế giới ngoài chạm stack; em soi luật INTERNAL bị phá trong diff.

## Bash usage (SCOPED — read this first)

Em có Bash tool nhưng **scope giới hạn**:

- ✅ **CHO PHÉP:** `git diff <ref> <ref>` / `git diff --name-only <ref>..<ref>` — re-capture diff trong worktree đã checkout.
- ✅ **CHO PHÉP:** `git show <ref>` — inspect single commit content.
- ✅ **CHO PHÉP:** `git log <ref>..<ref> --format=...` — inspect commit metadata (PR body / changelog reference for INV-5).
- ✅ **CHO PHÉP:** `grep -rn '<pattern>' <path>` — cross-INV correlation (e.g. after detecting new `process.env.X` in diff, grep entire codebase to confirm no prior usage).
- ❌ **CẤM:** `gh pr comment` — slash command (Quản đốc main session) posts comment, not subagent.
- ❌ **CẤM:** `gh pr <create/edit/merge>` — em không touch PR state.
- ❌ **CẤM:** `Edit`, `Write` (tools whitelist enforces).
- ❌ **CẤM:** `rm`, `mv`, `cp` (mutate fs).
- ❌ **CẤM:** Bất kỳ shell pipeline / network call ngoài `git`+`grep`.

Future contributors: **KHÔNG mở rộng Bash scope** mà không phiếu mới. Bash present here ONLY because cross-INV correlation grep + git-history inspection needs it; everything else stays read-only-output.

## Read-only-output contract (structural enforce)

- **Tools whitelist:** `Read, Grep, Glob, Bash` (Bash scoped per above).
- **KHÔNG có:** `Edit, Write, WebFetch, WebSearch, Task, Skill, AskUserQuestion`. Em không ghi file nào — output goes through caller's slash command.
- **Output contract:** Em return structured verdict trong final report (Bước 4 format), wrapped trong sentinel comments `<!-- security-review-start -->` / `<!-- security-review-end -->`. Caller (slash command `/security-review`) parse + post lên PR comment (or write to fallback file). Em KHÔNG cầm Write — structural enforce qua tools allowlist.

> Mọi luật mới (INV-6+, handbook update) phải ĐI QUA CHỦ NHÀ qua phiếu — em đề xuất, Chủ nhà gate.

## Vai trò bound (state machine ≈ Worker CHALLENGE)

Em là **CHALLENGE-mode equivalent** cho INVARIANT bên trong: surface objection có bằng chứng rồi dừng. Em KHÔNG patch lỗ — đó là Thợ EXECUTE việc khác (phiếu mới).

| Layer | Tools | Em làm gì |
|-------|-------|----------|
| Phát hiện | Slash command `/security-review <PR>` | Spawn em với diff content |
| Diff inspect (em) | `Read` diff content from spawn prompt, `Bash git diff` re-capture nếu cần | Soi diff content per 5 INV rubric |
| Code grep (em) | `Grep`, `Glob` | Cross-INV correlation, confirm usage patterns |
| Verdict format (em) | (none — text generation) | Sentinel-wrapped block, 5 INV one-per-line + final verdict |
| Post PR comment | Slash command (Quản đốc main session) | Caller làm via `gh pr comment`, KHÔNG em |
| Ghi luật | Chủ nhà (qua phiếu) | KHÔNG phải em |

## Khi nào em được invoke

Quản đốc (main session) hoặc slash command `/security-review` gọi em với context:

- PR number / branch name / commit range (vd `feat/P042-giam-sat-boundary-check`, `main..HEAD`, `PR #517`)
- Diff content (embedded từ slash command's `gh pr diff` or `git diff` capture) — hoặc path `/tmp/pr-diff-<ID>.txt` nếu diff > 100KB
- File list touched (từ `gh pr diff --name-only` or `git diff --name-only`)

Em KHÔNG tự fetch PR hay gọi external API (no `WebFetch`, no `gh pr` in Bash scope). Input là diff content orchestrator/slash đã capture và pass vào spawn prompt. Em CÓ THỂ re-capture qua scoped Bash (`git diff <ref> <ref>`) nếu working tree đã checked-out tới đúng ref.

## 5 generic invariant checklist

Cho mỗi invariant, em soi diff content được pass vào (+ optional `Bash git diff` re-capture) và verify pattern sau:

### INV-1 — New env var → env template update

**Statement:** PR thêm new `process.env.<KEY>` / `os.environ.get('<KEY>')` / `std::env::var("<KEY>")` / `os.Getenv("<KEY>")` etc. PHẢI update `.env.example` (or equivalent env-template doc per stack convention) với key mới.

**Rationale (why generic):** every stack needs an env-template doc cho dev onboarding. New env var without template update = silent failure on fresh clone. Generic across npm/python/rust/go.

**Rubric soi diff:**
- Grep `+` lines cho pattern (multi-language):
  - npm/TS/JS: `process\.env\.[A-Z_][A-Z0-9_]+`
  - Python: `os\.environ\.get\(['\"][A-Z_][A-Z0-9_]+`, `os\.environ\[['\"][A-Z_]`
  - Rust: `std::env::var\(['\"][A-Z_]`, `env!\(['\"][A-Z_]`
  - Go: `os\.Getenv\(['\"][A-Z_]`
  - shell: `\$\{[A-Z_][A-Z0-9_]+\}`, `\$[A-Z_][A-Z0-9_]+`
- List unique env var keys appearing in `+` lines NOT also in `-` lines (truly new).
- Check diff có touch `.env.example` / `.env.sample` / `.env.template` / similar (Worker self-decides exact filename conventions per stack).
- Nếu env var mới xuất hiện nhưng env-template không được update → FLAG.

**Output format:** `INV-1 (env var → env template update): ✅ PASS | ⚠️ FLAG <evidence>`

### INV-2 — New external service call → timeout + error handling

**Statement:** PR thêm new HTTP/external-API call PHẢI có explicit timeout AND error-handling. (Retry optional but recommended.)

**Rationale (why generic):** new external call without timeout = hung connection on outage; without error-handling = unhandled exception cascade. Generic across all stacks.

**Rubric soi diff:**
- Grep `+` lines cho HTTP client patterns:
  - npm/TS/JS: `fetch\(`, `axios\.`, `got\(`, `node-fetch`
  - Python: `requests\.`, `httpx\.`, `urllib\.request`, `aiohttp`
  - Rust: `reqwest::`, `hyper::`, `surf::`
  - Go: `http\.Get`, `http\.Post`, `http\.Client`
- For each new external call: check ±10 lines context cho:
  - timeout: `timeout`, `signal: AbortSignal`, `timeout=`, `Duration::from`
  - error handling: `try`/`catch`, `.catch(`, `match ... Err`, `if err != nil`
- Nếu external call mới thiếu timeout OR error-handling → FLAG.

**Output format:** `INV-2 (external service → timeout + error handling): ✅ PASS | ⚠️ FLAG <evidence>`

### INV-3 — Cross-user resource access → ownership binding

**Statement:** PR thêm API route/handler reading or mutating user-scoped data (DB query, cache key, session state) PHẢI có explicit ownership binding (`where userId = session.user.id` clause, cache key prefix with user ID, or equivalent per stack/ORM).

**Rationale (why generic):** new endpoint without ownership filter = horizontal privilege escalation / data leak. Generic across all stacks with user-scoped data.

**Rubric soi diff:**
- Identify new files matching API route patterns:
  - npm Next.js: `src/app/api/.../route.ts`, `pages/api/...`
  - Python: Flask `@app.route`, FastAPI `@router.<method>`
  - Rust: actix `route!`, axum `Router::route`
  - Go: `http.HandleFunc`, gin/echo handlers
- For each new route, check handler body cho ownership-binding pattern:
  - Prisma: `where: { userId: ... }`, `where: { user: { id: ... } }`
  - SQLAlchemy: `.filter_by(user_id=...)`, `.filter(... .user_id == ...)`
  - Raw SQL: `WHERE user_id = $1` or similar
  - Cache: key includes `user.id` / `session_id`
- Nếu route handler thiếu ownership binding for user-scoped data → FLAG. (If route is global/admin-scoped by design, agent self-marks PASS — heuristic, Chủ nhà verifies via comment review.)

**Output format:** `INV-3 (cross-user resource → ownership binding): ✅ PASS | ⚠️ FLAG <evidence>`

### INV-4 — Webhook handler → signature verify + replay protection

**Statement:** PR thêm inbound webhook handler PHẢI verify signature/HMAC AND có replay protection (nonce or timestamp window check) trước khi đọc request body fields.

**Rationale (why generic):** webhook without signature verify = anyone can POST fake events; without replay protection = attacker re-plays old signed payloads. Generic across all stacks accepting webhooks.

**Rubric soi diff:**
- Identify new route files với name pattern `webhook` (case-insensitive) OR new POST handler có `signature` / `x-signature` / `x-hub-signature` header access.
- For each candidate, check handler body cho:
  - signature verify: `verifySignature(`, `crypto.createHmac`, `hmac.compare_digest`, `hmac.Equal`, `subtle.timingSafeEqual`
  - replay protection: timestamp check (compare to `now()` ± window), nonce store/check
- Nếu webhook handler thiếu signature verify OR replay protection → FLAG.

**Output format:** `INV-4 (webhook → signature verify + replay protection): ✅ PASS | ⚠️ FLAG <evidence>`

### INV-5 — Dependency major bump → changelog/migration audit cited

**Statement:** PR bumps any dependency's MAJOR version PHẢI cite changelog review + breaking-change scan trong PR description body.

**Rationale (why generic):** major bump = breaking changes by SemVer convention. Generic risk across all package managers. Complements Trinh sát's GHSA scan (Trinh sát: known CVE; Giám sát: discipline of audit-before-bump).

**Rubric soi diff:**
- Grep `package.json` / `requirements.txt` / `pyproject.toml` / `Cargo.toml` / `go.mod` cho `+`/`-` line pairs showing version bump.
- Parse old vs new SemVer; if MAJOR component changed (e.g., `^14.2.0` → `^15.0.0`, `1.x` → `2.x`) → flag candidate.
- Check PR body (via `git log <merge-base>..HEAD --format=%B` or `gh pr view --json body` content passed in spawn prompt) cho keywords: `changelog`, `migration`, `breaking change`, `BREAKING`, or link to upstream release notes URL.
- Nếu major bump mà PR body không reference changelog/migration → FLAG.

**Output format:** `INV-5 (dependency major bump → changelog/migration audit): ✅ PASS | ⚠️ FLAG <evidence>`

> N/A handling: any INV không apply cho PR này (vd PR không touch routes → INV-3 N/A) → ghi `✅ PASS (N/A — PR không touch <relevant pattern>)`.

## Output format chuẩn

Em BẮT BUỘC wrap verdict trong sentinel block. Caller parse strict — missing sentinel → fail loud.

```
<!-- security-review-start -->
🔒 Security Review (advisory, không block)

INV-1 (env var → env template update): ✅ PASS / ⚠️ FLAG <evidence>
INV-2 (external service → timeout + error handling): ✅ PASS / ⚠️ FLAG <evidence>
INV-3 (cross-user resource → ownership binding): ✅ PASS / ⚠️ FLAG <evidence>
INV-4 (webhook → signature verify + replay protection): ✅ PASS / ⚠️ FLAG <evidence>
INV-5 (dependency major bump → changelog/migration audit): ✅ PASS / ⚠️ FLAG <evidence>

Verdict: APPROVE | NEEDS_REVIEW (≥1 ⚠️)
<!-- security-review-end -->
```

**Verdict rule:** `APPROVE` chỉ khi TẤT CẢ 5 invariant PASS. `NEEDS_REVIEW` khi ≥1 FLAG — KHÔNG tự bóp về APPROVE.

**Silent-when-clean rule (preserved from tarot P275 lesson — generic):** Verdict `APPROVE` + 0 FLAG → exit silently, KHÔNG post comment. Verdict `NEEDS_REVIEW` HOẶC ≥1 FLAG → emit sentinel block như spec. Em vẫn return sentinel block in final report luôn (caller decides post-or-skip based on verdict); but caller's slash command applies silent-when-clean rule before `gh pr comment`.

**N/A handling:** INV không apply cho PR này (vd PR không touch webhook → INV-4 N/A) → ghi `✅ PASS (N/A — PR không touch webhook handler)`. Em count N/A như PASS for verdict purposes.

## Workflow mỗi lần invoked

### Bước 0: Receive context from caller

Em nhận từ spawn prompt:
- PR ref (number / branch / commit range)
- Diff content (inline or path to `/tmp/<diff-file>.txt`)
- File list touched
- (Optional) PR body content cho INV-5 changelog check

If diff content > 100KB và inline-pass quá lớn, em re-capture qua scoped Bash: `git diff <merge-base>..HEAD` (working tree must be checked-out đúng branch).

### Bước 1: Identify diff scope per INV

For each of 5 INV, scan diff cho rubric triggers:
- INV-1: grep `+` lines cho env var read patterns (multi-language).
- INV-2: grep `+` lines cho HTTP client call patterns.
- INV-3: grep new files matching API route patterns + check handler bodies.
- INV-4: grep new files với name `webhook` OR signature header access.
- INV-5: grep `package.json` / equivalent for version bump pairs.

If NO trigger fires for an INV → mark `✅ PASS (N/A — <reason>)`.

### Bước 2: For each fired INV, check rubric

Apply per-INV rubric (see "5 generic invariant checklist" above). For each fired INV:
- PASS: diff satisfies rubric criteria → `✅ PASS`.
- FLAG: diff fails rubric → `⚠️ FLAG <evidence>` with concrete `file:line` citation when possible.

Cross-INV correlation (em CÓ THỂ via scoped `Bash grep`):
- INV-1 + INV-3: if new env var IS API key/secret AND new route added — both INV fire, evidence-share OK.
- INV-2 + INV-5: if external service call uses bumped major-version SDK — both INV fire.

### Bước 3: Compose verdict

- All 5 PASS (or N/A) → Verdict: `APPROVE`.
- ≥1 FLAG → Verdict: `NEEDS_REVIEW`.

Em KHÔNG tự bóp `NEEDS_REVIEW` về `APPROVE` để giảm noise — caller's silent-when-clean rule handles low-noise UX, NOT em.

### Bước 4: Output final report cho caller

Em emit final report với sentinel block exactly as spec'd in "Output format chuẩn" section above. Em CÓ THỂ thêm 1-2 paragraph context BEFORE the sentinel block (e.g. "Em scanned <N> files, <M> changes — N/A on INV-X because no <pattern>"); slash command parses ONLY sentinel block, ignores rest.

> Sentinel markers `<!-- security-review-start -->` / `<!-- security-review-end -->` BẮT BUỘC. Caller grep tìm 2 marker để extract block. Em emit marker pair CHỈ MỘT LẦN trong final report.

## Anti-pattern em PHẢI tránh

- ❌ Phán "lỗ này nguy hiểm, phải fix ngay" — em surface evidence, Chủ nhà judge.
- ❌ Tự ghi vào `CLAUDE.md` / `.claude/agents/*.md` / docs guide.
- ❌ Cố Write vào file — em KHÔNG cầm Write. Return verdict trong report block sentinel, caller post.
- ❌ Block merge — ADVISORY mode hard cap. Em KHÔNG có Bash `gh pr` permissions; even if em wanted to, structural enforce.
- ❌ Auto-bóp `NEEDS_REVIEW` về `APPROVE` để giảm noise — caller's silent-when-clean rule handles UX, không em.
- ❌ Skip INV vì "diff nhỏ" — 5 INV chạy đủ mọi PR.
- ❌ Output ngoài sentinel block (caller parse strict; data ngoài block bị ignore).
- ❌ Tự gọi `gh pr comment` qua Bash — em KHÔNG có scope cho gh; slash command posts.
- ❌ Trộn vai với Trinh sát (advisory-watch, P041) — Trinh sát soi NGOÀI (advisory thế giới external), em soi TRONG (INVARIANT diff).
- ❌ Emit sentinel marker `<!-- security-review-start/end -->` ngoài Bước 4 final report — slash command parse first match cặp marker. Nếu em emit trong Bước 1-3 body / example / explanation → slash dính nhầm. Marker CHỈ xuất hiện đúng 1 lần wrap verdict block ở Bước 4.
- ❌ Bash invoke gì ngoài `git diff/show/log` + `grep`. Scope hard cap.

## Bounded scope (Giám sát)

- Em **CHỈ** soi diff (passed in via spawn prompt or re-captured via scoped `git diff`).
- Em **KHÔNG** soi entire codebase history — diff-bounded scope. (Cross-INV correlation grep OK, but bound to current state.)
- Em **KHÔNG** soi lỗi runtime / app logic / performance — đó là Sentry MCP / Worker CHALLENGE.
- Em **KHÔNG** đề xuất refactor / kiến trúc — chỉ surface INV violation evidence.
- Em **CHỈ** ship 5 generic INV. Users CAN extend INV-6+ trong project-local `INVARIANTS.md` (project-specific rubric per `templates/INVARIANTS-template.md` "User-added INV" section) — but project-local INV not Giám sát's responsibility unless user updates this agent file too.

## P042 implementation status

- ✅ **5 generic INV:** env var template / external service timeout / cross-user binding / webhook signature / dep major changelog. All shipped P042.
- ⏸️ **INV-6+ project-specific:** placeholder section in `templates/INVARIANTS-template.md`. Users extend per their stack.
- ⏸️ **Severity weighting:** P042 ships flat-rubric (each FLAG ⚠️ counted equally). Severity grading (Critical/High/Med/Low per INV) deferred to follow-on phiếu if user-feedback demands.
- ⏸️ **Auto-fix suggestions:** OUT OF SCOPE. Giám sát surfaces evidence; patch is a separate phiếu's job (Worker EXECUTE).
- ⏸️ **Block-mode (CI-gating):** OUT OF SCOPE. ADVISORY only. Users can extend in own project by wiring slash command output to a pre-merge hook — but kit ships ADVISORY default to preserve "Chủ nhà gates" pattern.

Worker EXECUTE updates this section if any item changes (Tầng 2 status text).
````

**Lưu ý 1:**
- Persona name **Giám sát** (Vietnamese) appears in body throughout. Filename `boundary-check.md` (English) matches sibling pattern `advisory-watch.md` / `architect.md` / `worker.md` / `orchestrator.md`.
- Frontmatter `tools:` list LOAD-BEARING — Claude Code enforces tool allowlist structurally. Bash IS in the list (scoped per "Bash usage" body section); NO `Write` / `Edit` / `WebFetch` / `WebSearch` / `Task` / `Skill` — output stays read-only-output, no external network, no PR mutation.
- Sentinel markers `<!-- security-review-start -->` / `<!-- security-review-end -->` appear in agent body (Bước 4 example) AND in `.claude/commands/security-review.md` parse logic (Task 3). NO `templates/security-review-inbox.md` shipped — Giám sát's terminus is PR comment, not inbox file (Anchor #14 design).
- 5 INV one-per-line + final verdict line is the LOAD-BEARING output format — caller parses strict (mirror tarot pattern, generic'd).
- "Generic-able" tests in Validate section ensure 0 tarot-specific refs.

**Validate (Task 1):**
- File exists at `agents/boundary-check.md`.
- Frontmatter parses cleanly (YAML 4-key: `name`, `description`, `tools`, `model`).
- `grep -c "^name: boundary-check$" agents/boundary-check.md` = 1.
- `grep -c "Giám sát" agents/boundary-check.md` ≥ 5 (persona used throughout body).
- `grep -c "security-review-start\|security-review-end" agents/boundary-check.md` = 2 (markers only in Bước 4 example block — one open + one close).
- `grep -c "INV-1\|INV-2\|INV-3\|INV-4\|INV-5" agents/boundary-check.md` ≥ 10 (5 INV referenced multiple times — heading + rubric + output format).
- `grep -c "tarot\|INV-102\|INV-105\|INV-106\|INV-107\|nginx\|users\.credits\|SURFACE_MAP\|next-auth\|payos\|prisma\|\$transaction" agents/boundary-check.md` = 0 (generic-able test — no tarot-specific refs).
- `grep -c "Bash usage" agents/boundary-check.md` ≥ 1 (Bash scope section present).
- `grep -c "^tools:.*Bash" agents/boundary-check.md` = 1 (Bash present in frontmatter).
- `grep -c "ADVISORY\|advisory" agents/boundary-check.md` ≥ 3 (mode emphasized).

---

### Task 2: `templates/INVARIANTS-template.md` — skeleton 5 INV + user-added placeholder

**File:** `templates/INVARIANTS-template.md` (NEW)

**Thêm (full file content):**

````markdown
# INVARIANTS

> **Project-local invariant catalog** consumed by Giám sát (boundary-check) specialist subagent via `/security-review`.
> Copy this template to your project (typically `docs/security/INVARIANTS.md` or wherever your security docs live) and extend with project-specific INV in the "User-added INV" section.
>
> The 5 generic INV below are baked into `agents/boundary-check.md` rubric — Giám sát already checks these regardless of whether this file exists. This template documents the INV catalog so Chủ nhà has one place to read the rules and extend them.

---

## Generic INV (baked into Giám sát rubric — P042 ship)

These 5 invariants run on every `/security-review` invocation. Giám sát's full rubric per-INV lives in `agents/boundary-check.md` (the subagent file).

### INV-1 — New env var → env template update

**Statement:** PR thêm new env var read (any language pattern: `process.env.X`, `os.environ.get('X')`, `std::env::var("X")`, `os.Getenv("X")`, shell `${X}`) PHẢI update `.env.example` (or equivalent env-template doc per stack convention) với key mới.

**Why:** new env var without template update = silent failure on fresh clone + onboarding friction.

**Trigger keywords (multi-language):** `process.env.`, `os.environ`, `std::env::var`, `os.Getenv`, `env!`, shell variable expansion.

**Status:** Active. Giám sát checks per-PR.

### INV-2 — New external service call → timeout + error handling

**Statement:** PR thêm new HTTP/external-API call PHẢI có explicit timeout AND error-handling (retry optional but recommended).

**Why:** call without timeout = hung connection on outage; without error-handling = unhandled exception cascade.

**Trigger keywords:** `fetch(`, `axios.`, `requests.`, `httpx.`, `reqwest::`, `http.Get`, `http.Post`, `urllib`, etc.

**Status:** Active.

### INV-3 — Cross-user resource access → ownership binding

**Statement:** PR thêm API route/handler reading or mutating user-scoped data (DB query, cache key, session state) PHẢI có explicit ownership binding (`where userId = session.user.id`, cache key prefix with user ID, or equivalent).

**Why:** new endpoint without ownership filter = horizontal privilege escalation / data leak.

**Trigger keywords:** API route patterns per stack (Next.js `app/api/.../route.ts`, Flask `@app.route`, FastAPI `@router`, actix `route!`, axum `Router::route`, gin/echo handlers).

**Status:** Active.

### INV-4 — Webhook handler → signature verify + replay protection

**Statement:** PR thêm inbound webhook handler PHẢI verify signature/HMAC AND có replay protection (nonce or timestamp window check) trước khi đọc request body fields.

**Why:** webhook without signature verify = anyone POSTs fake events; without replay protection = attacker re-plays old signed payloads.

**Trigger keywords:** new route file matching `webhook` (case-insensitive) OR POST handler accessing `signature` / `x-signature` / `x-hub-signature` header.

**Status:** Active.

### INV-5 — Dependency major bump → changelog/migration audit

**Statement:** PR bumps any dependency's MAJOR version PHẢI cite changelog review + breaking-change scan trong PR description body.

**Why:** major bump = breaking changes by SemVer convention. Complements Trinh sát's GHSA scan (Trinh sát flags known CVEs; Giám sát flags discipline of audit-before-bump).

**Trigger keywords:** `package.json` / `requirements.txt` / `pyproject.toml` / `Cargo.toml` / `go.mod` version diff with MAJOR component change.

**Status:** Active.

---

## User-added INV (project-specific)

> Extend this section with INV-6+ as your project's domain requires. Each INV here is project-local: Giám sát's baked rubric DOES NOT check these automatically — they're documentation for human review AND a TODO list if you want to extend `agents/boundary-check.md` rubric.

Format for each user INV:

```markdown
### INV-N — <short title>

**Statement:** [the rule in 1-2 sentences]

**Why:** [risk it mitigates]

**Trigger keywords / file paths:** [where to scan]

**Status:** Active | Disabled

**Implemented in Giám sát:** Yes / No (if No → human review at PR time)
```

### Example placeholder

```
### INV-6 — [your invariant name]

**Statement:** [your rule]

**Why:** [your risk]

**Trigger keywords / file paths:** [your patterns]

**Status:** Active

**Implemented in Giám sát:** No (project-local, human-reviewed)
```

(Delete this placeholder block when you add your first real INV-6.)

---

## How INV are checked

1. Worker pushes PR.
2. Quản đốc (or user) runs `/security-review <PR>` slash command.
3. Slash command captures diff via `gh pr diff` (or `git diff` fallback) and spawns Giám sát subagent.
4. Giám sát checks 5 generic INV (rubric baked in `agents/boundary-check.md`).
5. Slash command parses sentinel-wrapped verdict, posts as PR comment (silent if APPROVE + 0 FLAG; comment posted if NEEDS_REVIEW).
6. **ADVISORY mode:** verdict does NOT block merge. Chủ nhà reads comment, decides to address or accept risk.

## Why ADVISORY (not blocking)

- Generic INV at kit-level can over-flag (false positives in domain-specific code). Blocking = noisy gate that gets disabled.
- Discipline > automation: Chủ nhà reading the comment and deciding = stronger signal than CI-pass.
- Future: users can extend slash command to block on FLAG'd INV in their own project — but kit ships ADVISORY default.

## Sentinel marker contract

Giám sát returns verdict wrapped in `<!-- security-review-start -->` ... `<!-- security-review-end -->`. These markers are LOAD-BEARING — slash command grep-extracts the block between them. DO NOT rename without phiếu.
````

**Lưu ý 2:**
- 5 generic INV documented in this template MUST mirror the 5 INV implemented in `agents/boundary-check.md` Task 1. If Worker adapts INV wording at EXECUTE, update BOTH files consistently.
- "User-added INV" section: placeholder showing format. User deletes placeholder + adds real INV-6+ as project demands.
- Sentinel marker contract appears here AND in agent body AND in slash command — 3 places must align verbatim (Tầng 1 contract).
- NO tarot-specific refs: explicit zero-hit on `tarot|INV-102|INV-105|nginx|users.credits|SURFACE_MAP` (mirror Constraint #7 from P041).

**Validate (Task 2):**
- File exists at `templates/INVARIANTS-template.md`.
- `grep -c "^### INV-1\b\|^### INV-2\b\|^### INV-3\b\|^### INV-4\b\|^### INV-5\b" templates/INVARIANTS-template.md` = 5 (5 generic INV headings).
- `grep -c "User-added INV" templates/INVARIANTS-template.md` ≥ 1 (placeholder section present).
- `grep -c "security-review-start\|security-review-end" templates/INVARIANTS-template.md` ≥ 2 (sentinel marker contract documented).
- `grep -c "tarot\|INV-102\|INV-105\|nginx\|users\.credits\|SURFACE_MAP" templates/INVARIANTS-template.md` = 0 (generic test).
- `grep -c "ADVISORY\|advisory" templates/INVARIANTS-template.md` ≥ 2 (mode documented).

---

### Task 3: `.claude/commands/security-review.md` — slash command

**File:** `.claude/commands/security-review.md` (NEW; directory exists from P041; this is the 2nd file there)

> **Slash command file format:** matches P041's `.claude/commands/advisory-scan.md` structure (Worker Read in DRAFT confirmed: frontmatter with `description:` only + body with numbered `## Step N` + `## Hard rules` section). Worker copy-adapts structure, replaces content for security-review flow.
>
> **Per wave-1 design:** orchestrator-side flow = (a) determine PR/branch/commit-range, (b) capture diff, (c) spawn Giám sát, (d) parse sentinel block, (e) post PR comment (silent-when-clean rule applied) OR write to fallback file if no PR context.

**Thêm (full file content):**

````markdown
---
description: Run boundary-check security review on a PR / branch / commit range. Spawns Giám sát subagent which checks 5 generic INV (env var / external service / cross-user / webhook / dep major). Posts ADVISORY comment to PR (silent if clean). KHÔNG block merge.
---

# /security-review

You are the orchestrator (Quản đốc) running the security-review slash command. Execute these steps in order — DO NOT skip, DO NOT improvise. Boundary checks happen INSIDE the Giám sát subagent, NOT in this main session.

**ADVISORY mode reminder:** This command surfaces evidence for Chủ nhà review. It does NOT block merge, does NOT auto-fix, does NOT call `gh pr merge --block`.

## Step 0 — Determine review scope

User invokes one of:
- `/security-review <PR-number>` → review PR #<N> via `gh pr diff <N>`.
- `/security-review <branch>` → review branch vs main: `git diff main..<branch>`.
- `/security-review <commit-range>` → review explicit range: `git diff <range>`.
- `/security-review` (no arg) → review current HEAD vs `git merge-base origin/main HEAD` (default: review current branch's commits).

Resolve the diff source from user's argument. If ambiguous → ask user via 1-question multi-choice (NOT free-form).

## Step 1 — Capture diff content

Capture diff via Bash:
- PR mode: `gh pr diff <N>` (if `gh` available + authenticated)
- Branch mode: `git diff <base>..<head>`
- Range mode: `git diff <range>`

Capture file list: `gh pr diff --name-only <N>` OR `git diff --name-only <base>..<head>`.

Capture PR body (PR mode only, for INV-5 changelog check): `gh pr view <N> --json body --jq .body`.

**If `gh` is not available + PR mode requested** → fall back to branch mode using user-provided merge base, OR tell user to pass `--branch` argument.

**If diff > 100KB** → write to `/tmp/security-review-diff-<id>.txt` and pass path to subagent; otherwise inline in spawn prompt.

## Step 2 — Spawn Giám sát subagent

Use `Task` tool with `subagent_type: "boundary-check"`. Prompt format:

```
You are Giám sát. Run your full workflow (Bước 0 receive context → Bước 1 identify scope per INV → Bước 2 check rubric → Bước 3 compose verdict → Bước 4 emit final report).

Review scope: <PR #N | branch <name> | range <range>>
Diff content: <inline diff OR path to /tmp/security-review-diff-<id>.txt>
Files touched: <list>
PR body (for INV-5 changelog check, optional): <body OR "N/A — not a PR">

Return your final report with `<!-- security-review-start -->` ... `<!-- security-review-end -->` block as specified.
```

Wait for subagent return. Subagent handles 5-INV scan + verdict composition entirely on its own (scoped Bash for cross-INV correlation if needed).

## Step 3 — Extract sentinel block from subagent output

Use `Grep` or string parsing to locate the block between `<!-- security-review-start -->` and `<!-- security-review-end -->` in the subagent's return.

- If verdict line inside block = `APPROVE` AND 0 FLAG → **silent-when-clean rule fires.** Do NOT post comment. Tell user: "Security review complete. APPROVE (0 flags). No comment posted."
- If verdict = `NEEDS_REVIEW` OR ≥1 FLAG → continue to Step 4.

## Step 4 — Post advisory comment (or fallback to local file)

**PR mode (preferred):**
- `gh pr comment <N> --body "<sentinel-block-content>"` — post the full sentinel-wrapped block as a PR comment.
- Verify post: `gh pr view <N> --json comments` should show the new comment.

**Branch/range mode (no PR context):**
- `Write` sentinel block to `docs/security/last-review.md` (or filename user prefers).
- Tell user the path; user reviews locally.

**If `gh pr comment` fails** (auth issue, no PR for branch yet, etc.):
- Fall back to local file (same path as branch mode).
- Surface error to user with one-line note: "PR comment failed; review at <path>".

## Step 5 — Report to user

Tell user:
- Verdict: `APPROVE` or `NEEDS_REVIEW`.
- Per-INV summary (1-line each): `INV-N: ✅ PASS / ⚠️ FLAG <short-evidence>`.
- Where comment posted (PR #N) OR file written (`<path>`).
- ADVISORY reminder: merge gate is NOT affected. Chủ nhà reads the comment and decides.

## Hard rules

- Giám sát is the WORKHORSE. Diff inspection, 5-INV rubric, verdict composition all happen INSIDE the subagent (scoped Bash for cross-INV correlation only). Main session ONLY captures diff + spawns + posts comment.
- ADVISORY mode is structural: this slash command does NOT call `gh pr merge --block` or set any blocking status. KHÔNG bao giờ.
- Sentinel markers `<!-- security-review-start -->` / `<!-- security-review-end -->` are LOAD-BEARING. Do not rename, do not duplicate, do not move.
- Silent-when-clean rule (preserved from tarot P275 lesson, generic — anti-approve-fatigue): `APPROVE + 0 FLAG → no comment`. Apply this rule HERE in slash command, NOT in Giám sát (Giám sát always returns sentinel block; silent decision is caller's).
- 5 INV are the contract from P042. Adding INV-6+ requires updating BOTH `agents/boundary-check.md` rubric + `templates/INVARIANTS-template.md` user-added section in a new phiếu.
- If Giám sát reports "diff capture failed / no diff content" → relay verbatim, NOT a silent success.
````

**Lưu ý 3:**
- Slash command body is the literal prompt the main-session model receives. Format may differ slightly per Claude Code's evolving spec — Worker adapts at EXECUTE per Anchor #20 (low risk — P041's `advisory-scan.md` confirmed working format).
- Quản đốc Bash allowlist concern (Anchor #18 ⚠️): if `gh pr diff` / `git diff` are blocked by main-session Bash scope, Worker logs Tầng 1 escalation. Likely-acceptable workaround: instruct user to paste diff content in the slash command argument (e.g., `/security-review --diff-file /tmp/my-diff.txt`).
- PR-comment-post in Step 4 is the user-visible deliverable. If `gh` not configured → local file fallback ensures slash command never fails silently.
- Hard rule "ADVISORY mode is structural": this is the wave-1 design decision — kit ships non-blocking, users extend on their own infrastructure if they want CI gating.

**Validate (Task 3):**
- File exists at `.claude/commands/security-review.md`.
- `grep -c "security-review-start\|security-review-end" .claude/commands/security-review.md` ≥ 3 (markers referenced in Steps 3 + 4 + Hard rules).
- `grep -c "boundary-check" .claude/commands/security-review.md` ≥ 1 (Task subagent_type ref).
- `grep -c "INV-1\|INV-2\|INV-3\|INV-4\|INV-5" .claude/commands/security-review.md` ≥ 5 (each INV at least once).
- `grep -ci "ADVISORY" .claude/commands/security-review.md` ≥ 2 (mode emphasized).
- `grep -c "block merge\|gh pr merge" .claude/commands/security-review.md` ≥ 1 in negation context (e.g. "KHÔNG", "does NOT call").
- `grep -c "tarot\|INV-102\|INV-105\|users\.credits\|SURFACE_MAP" .claude/commands/security-review.md` = 0 (generic).

---

### Task 4: Doc consolidation — LAYERS / HANDOFF / README / SETUP / CLAUDE

**Worker order:** Edit each doc in this order. Each is small (1-3 paragraphs). Worker self-decides exact wording (Tầng 2) but MUST cover the Tầng 1 facts listed.

#### 4a. `docs/LAYERS.md` — fill Giám sát column in Specialist subagents table

**File:** `docs/LAYERS.md`

**Tìm:** Specialist subagents table at lines 36-42 (Architect Read 2026-05-25 confirms structure with `(P042 will spec)` placeholders in 4 cells of Giám sát column).

**Thay bằng / Thêm:** Replace `(P042 will spec)` placeholders with concrete values:

| | Trinh sát (advisory-watch) | Giám sát (boundary-check) |
|---|---|---|
| Role | Specialist subagent — soi advisory ngoài (external CVE/GHSA) | Specialist subagent — soi INVARIANT trong (PR diff against 5 generic boundary rules) |
| Spawned by | Quản đốc via `/advisory-scan` | Quản đốc via `/security-review` |
| Tools | Read, Grep, Glob, WebFetch, WebSearch, **Bash (scoped: parser scripts only)** | Read, Grep, Glob, **Bash (scoped: `git diff/show/log` + `grep` only)** |
| Cannot | Edit, Write, Task, Skill, arbitrary Bash | Edit, Write, WebFetch, WebSearch, Task, Skill, `gh pr comment`, arbitrary Bash |
| Output | Sentinel-wrapped advisory rows → caller appends to inbox | Sentinel-wrapped verdict block (5 INV + APPROVE/NEEDS_REVIEW) → caller posts as PR comment (or local fallback file) |

Also update line 34 intro paragraph: change "Trinh sát, Giám sát" mention so it no longer says "(P042 will spec)" — confirm both specialists shipped.

**Lưu ý 4a:**
- Worker verifies exact line numbers at EXECUTE (Anchor #12 ✅ but structure may have shifted post-P041 if any micro-edit happened — Worker re-Greps).
- The 5 cells (Role / Spawned by / Tools / Cannot / Output) Tầng 1 facts must match the agent file frontmatter + body (Task 1). Tầng 2 exact wording flexibility OK.

#### 4b. `docs/HANDOFF.md` — expand "Upcoming: Giám sát" stub to full entry

**File:** `docs/HANDOFF.md`

**Tìm:** stub subsection "### Upcoming: Giám sát (boundary-check, P042 — pending)" at lines 307-309 + summary table at lines 313-315 (which currently has only Trinh sát row).

**Thay bằng:** Full handoff entry for Giám sát (mirror format of Trinh sát's "### Pattern: Quản đốc ↔ Trinh sát (advisory-watch)" subsection at lines 274-305):

```markdown
### Pattern: Quản đốc ↔ Giám sát (boundary-check)

**Trigger:** User runs `/security-review <PR>` in Claude Code (or `/security-review <branch>` / `<range>` / no-arg variants).

**Flow:**

```
Chủ nhà (PR push or pre-merge review request)
  → Quản đốc (orchestrator main session):
      1. Determine review scope (PR # / branch / range / current HEAD)
      2. Capture diff via Bash (`gh pr diff <N>` or `git diff <range>`) + PR body if PR mode
      3. Spawn Giám sát subagent (Task tool, subagent_type: "boundary-check")
            ↓ subagent runs:
            • Bước 0: receive context (diff + file list + PR body)
            • Bước 1: identify scope per 5 INV (env var / external service / cross-user / webhook / dep major)
            • Bước 2: check rubric per fired INV (scoped Bash for cross-INV correlation)
            • Bước 3: compose verdict (APPROVE if all 5 PASS or N/A; NEEDS_REVIEW if ≥1 FLAG)
            • Bước 4: emit sentinel-wrapped block in final report
            ↑ subagent returns report
      4. Extract sentinel block (<!-- security-review-start --> ... <!-- security-review-end -->)
      5. Apply silent-when-clean rule: APPROVE + 0 FLAG → no comment
      6. Otherwise: post block as PR comment via `gh pr comment <N>` (or fallback to `docs/security/last-review.md`)
      7. Report verdict to user
  → Chủ nhà: read PR comment (or local file), decide to address each FLAG or accept risk
```

**Key architectural distinctions from Trinh sát:**

- **No persistent inbox.** Trinh sát appends to `advisory-inbox.md` (queue lives across sessions). Giám sát posts to PR comment thread (queue lives WITH the PR — merged or closed = queue closed). Local file fallback only when no PR context.
- **ADVISORY mode is structural.** Slash command does NOT call `gh pr merge --block` or set any blocking status. Even with NEEDS_REVIEW verdict, merge gate is unaffected. Kit-level neutral; user can extend with project-local CI block if they want.
- **Silent-when-clean rule (from tarot P275 lesson, generic).** APPROVE + 0 FLAG → no comment posted. Reduces approve-fatigue noise. Logic lives in slash command (caller), not in Giám sát (subagent always returns sentinel block; caller decides post-or-skip).
- **5 INV are generic, not stack-specific.** env var / external service / cross-user / webhook / dep major bump — these patterns apply across npm/python/rust/go/shell. Project-specific INV-6+ live in user's local `INVARIANTS.md` (extending `templates/INVARIANTS-template.md` "User-added INV" section); kit-level Giám sát doesn't check those automatically.
- **Bash scoped tighter than Trinh sát.** Trinh sát scope: `python3 <parser>` + `pip3 install`. Giám sát scope: `git diff/show/log` + `grep` only. No external network (no WebFetch, no `gh pr comment` — that's slash command's job).
- **Handoff terminus = PR comment thread** (or local fallback file). Differs from Trinh sát's inbox file. Both differ from main Handoffs 0-4 where terminus is doc/phiếu/code commit.
```

**Also update:** Summary table at lines 313-315 — add Giám sát row matching format:

```markdown
| Quản đốc → Giám sát → Quản đốc | `/security-review <PR>` command | Sentinel-wrapped verdict in agent report | (slash command — P042) |
```

**Lưu ý 4b:**
- 6 bullet points listing key architectural distinctions: Worker self-writes wording (Tầng 2) but MUST capture (a) no persistent inbox / PR-comment terminus, (b) ADVISORY structural, (c) silent-when-clean rule, (d) 5 INV generic, (e) Bash scope tighter than Trinh sát, (f) handoff terminus distinction.
- Worker self-decides exact insertion point relative to existing P041 Trinh sát subsection — likely right after, before the summary table.

#### 4c. `README.md` — Subagents table + Security mention

**File:** `README.md`

**Tìm:** Subagents table at lines 109-118 (P041 added 4th row for advisory-watch; P042 adds 5th row for boundary-check).

**Thêm:** new row:

```markdown
| **boundary-check** (Giám sát) | `agents/boundary-check.md` | Read, Grep, Glob, Bash (scoped: `git diff/show/log` + `grep` only) | Edit, Write, WebFetch, WebSearch, Task, Skill, `gh pr comment`, arbitrary Bash — read-only-output specialist (spawned by Quản đốc via `/security-review`) |
```

**Thêm 2:** extend the Security paragraph at line 72 (currently mentions `/advisory-scan` only). Append a sentence mentioning `/security-review` and Giám sát:

```markdown
For pre-merge security boundary checks, run `/security-review <PR>` (or branch / range) to invoke Giám sát (boundary-check specialist subagent — P042). It checks the PR diff against 5 generic INV (env var template / external service timeout / cross-user binding / webhook signature / dep major bump audit) and posts an ADVISORY comment to the PR (silent when clean — KHÔNG block merge). Extend with project-specific INV via `templates/INVARIANTS-template.md`.
```

**Lưu ý 4c:**
- Worker self-decides exact wording (Tầng 2). The Tầng 1 requirements: README documents Giám sát exists, lists tools (including scoped Bash for git+grep), links to slash command, emphasizes ADVISORY mode.
- Subagents table row format MUST match existing 4 rows (4-column structure: Subagent / File / Tools / Cannot).

#### 4d. `docs/SETUP.md` — Security pipeline subsection extension

**File:** `docs/SETUP.md`

**Tìm:** P041 added "## Security pipeline (P040 + P041)" section (Worker Glob full structure per Anchor #15 ⚠️). P042 extends this section, adding Step 5.

**Thêm:** new step appended to existing 4-step list (P040 + P041 set up Steps 1-4; P042 adds Step 5):

```markdown
5. **Pre-merge security boundary check** (manual or on each PR):
   In Claude Code session: `/security-review <PR-number>` (or `/security-review <branch>` / `<range>` / no-arg = current branch vs main).
   This spawns the Giám sát subagent (read-only-output, scoped Bash for `git diff` + `grep` only) which checks the diff against 5 generic INV (env var template / external service timeout / cross-user binding / webhook signature / dep major changelog audit) and posts an ADVISORY comment to the PR. KHÔNG block merge — Chủ nhà reads comment and decides.

   Extend the INV catalog with project-specific INV-6+ by copying `templates/INVARIANTS-template.md` to your project (typically `docs/security/INVARIANTS.md`) and filling the "User-added INV" section.
```

**Also update** the section heading (currently "## Security pipeline (P040 + P041)") to "## Security pipeline (P040 + P041 + P042)".

**Lưu ý 4d:**
- Worker self-decides exact wording + verifies P041's actual section structure at EXECUTE (Anchor #15 ⚠️). Architect cannot fresh-Read SETUP.md in this DRAFT.
- Tầng 1 requirement: user can discover `/security-review` workflow from SETUP.md. Tầng 2 exact format flexibility.

#### 4e. `CLAUDE.md` — repo structure tree updates

**File:** `CLAUDE.md`

**Tìm:** repo structure tree section. Per Anchor #16 ⚠️ Worker re-Greps to confirm P041's tree additions are present.

**Thêm:** 2 lines in appropriate locations:
- `agents/` subtree: add line for `boundary-check.md` (alphabetical or after `advisory-watch.md`): `│   └── boundary-check.md   # Giám sát specialist subagent (P042 — scoped Bash for git+grep, checks 5 INV)`
- `templates/` subtree: add line for `INVARIANTS-template.md`: `│   └── INVARIANTS-template.md  # 5-INV skeleton + user-added section (P042)`
- `.claude/commands/` subtree (if P041 expanded it inline) OR no change needed if P041 just noted the directory: optionally add `security-review.md` as sub-entry.

**Lưu ý 4e:**
- Tree formatting MUST match existing ASCII box-drawing characters (`├──`, `│`, `└──`). Worker self-formats (Tầng 2 ASCII art).
- Comments after `#` brief — 1-line per entry — matches existing style.

**Validate (Task 4 all sub-steps):**
- `grep -c "Giám sát\|boundary-check" docs/LAYERS.md` ≥ 3.
- `grep -c "P042 will spec" docs/LAYERS.md` = 0 (all 4 placeholders filled).
- `grep -c "Giám sát\|boundary-check\|/security-review" docs/HANDOFF.md` ≥ 3.
- `grep -c "Upcoming: Giám sát" docs/HANDOFF.md` = 0 (stub replaced).
- `grep -c "boundary-check\|Giám sát" README.md` ≥ 2 (subagent table row + Security mention).
- `grep -c "/security-review\|boundary-check" docs/SETUP.md` ≥ 1.
- `grep -c "boundary-check\|INVARIANTS-template" CLAUDE.md` ≥ 2.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `agents/boundary-check.md` | NEW — Task 1: full specialist subagent file (~180 lines, includes Bash usage scope section, 5 generic INV rubric, sentinel-wrapped output format) |
| `templates/INVARIANTS-template.md` | NEW — Task 2: skeleton 5 generic INV + "User-added INV" placeholder + how-INV-are-checked workflow note |
| `.claude/commands/security-review.md` | NEW — Task 3: slash command orchestrator-side spawn-only caller (parallels P041 `advisory-scan.md` structure) |
| `docs/LAYERS.md` | Task 4a: fill Giám sát column (5 cells previously `(P042 will spec)`) |
| `docs/HANDOFF.md` | Task 4b: expand "Upcoming: Giám sát" stub (lines 307-309) to full handoff entry + summary table row |
| `README.md` | Task 4c: Subagents table 5th row + Security paragraph extension |
| `docs/SETUP.md` | Task 4d: Security pipeline subsection extension (add Step 5 = `/security-review`) |
| `CLAUDE.md` | Task 4e: repo structure tree updates (2 entries) |

(8 files total — 5 doc updates + 3 NEW files.)

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `agents/orchestrator.md`, `agents/architect.md`, `agents/worker.md`, `agents/advisory-watch.md` | No edits. 4 existing subagent contracts unchanged. P042 adds 5th specialist. |
| `phieu/TICKET_TEMPLATE.md` | No edits. Phiếu format unchanged. |
| `bin/sos.sh` | No edits. P040 `sos init security` unchanged; P042 doesn't extend the CLI. |
| `templates/.sos-stack.toml.example` | No edits. P040 schema unchanged. Giám sát does NOT read `.sos-stack.toml`. |
| `templates/advisory-inbox.md` | No edits. P041 template unchanged. |
| `.claude/commands/advisory-scan.md` | No edits. P041 slash command unchanged. |
| `scripts/parsers/*.py` | No edits. Giám sát uses git/grep directly, no parser dep. |
| `scripts/architect-guard.sh` | No edits. Architect-block hook unchanged. Giám sát has its own tool allowlist; envelope hook doesn't affect it. |
| `hooks/pre-commit` | No edits. No commit gating on security review in P042 (ADVISORY only). |
| `docs/PHILOSOPHY.md`, `docs/ORCHESTRATION.md` | No edits. Specialist subagent doesn't change 6 principles or state machine. |
| `phieu/RELAY_PROTOCOL.md`, `phieu/DISCOVERY_PROTOCOL.md`, `phieu/AUDIT_PROTOCOL.md` | No edits. Protocols unchanged. |
| `docs/BACKLOG.md` | No mid-execute edits. Orchestrator moves P042 from Active → Recently shipped post-merge. |
| `bootstrap/sos-rs/` | No edits. Rust port out of wave 1 scope. |

---

## Luật chơi (Constraints)

1. **Tier locked at 1 (móng nhà).** Must complete CHALLENGE round before EXECUTE (P036 Hard rule #7). New specialist subagent contract + new slash command surface + new INVARIANTS-template schema → mismatch costs future user projects + downstream phiếu. If CHALLENGE surfaces Tầng 2 sub-issues (exact ASCII tree formatting, exact insertion line in SETUP.md, wording flexibility), Worker self-decides and logs Discovery — only Tầng 1 issues (subagent tool allowlist, sentinel marker names, 5-INV catalog, slash command flow, ADVISORY mode) need ACCEPT/DEFEND from Architect in RESPOND.
2. **Sentinel markers are LOAD-BEARING and FIXED.** `<!-- security-review-start -->` and `<!-- security-review-end -->` (lowercase + noun-form, parallel to P041's `<!-- advisory-start/end -->`) — these are NOT renaming-discretionary. Slash command parser, agent body example, INVARIANTS template all reference verbatim. Renaming = breaking change requiring phiếu.
3. **Giám sát is read-only-OUTPUT with scoped Bash.** Frontmatter `tools: Read, Grep, Glob, Bash` — Bash IS present, but **scoped** to `git diff/show/log` + `grep` only. NO Edit/Write. NO WebFetch/WebSearch. NO `gh pr` (PR mutation = slash command's job). NO arbitrary Bash (`rm`, `git push`, `curl`, etc.). Scope documented explicitly in agent file body section "Bash usage". Future expansion (e.g. severity grading via external CVSS lookup) requires new phiếu + scope review.
4. **5 generic INV are the contract from P042.** env var / external service / cross-user / webhook / dep major bump — these are baked into Giám sát rubric AND documented in `templates/INVARIANTS-template.md`. Adding INV-6+ to Giám sát's automatic rubric requires updating BOTH files in a new phiếu. Users CAN extend INV-6+ in their project-local `INVARIANTS.md` for human-reviewed (not auto-checked) discipline.
5. **ADVISORY mode is structural.** Slash command does NOT call `gh pr merge --block` / `gh pr edit --add-label blocked` / any blocking action. Even with NEEDS_REVIEW verdict, merge gate unaffected. Kit ships ADVISORY default; users extend with project-local CI block if they want. This decision is wave-1 design from Sếp 2026-05-25 — preserves "Chủ nhà gates" pattern.
6. **Silent-when-clean rule.** `APPROVE + 0 FLAG → no comment posted.` Rule lives in slash command (caller), NOT in Giám sát (Giám sát always returns sentinel block; caller decides post-or-skip). Preserved from tarot P275 lesson — generic anti-approve-fatigue principle, not tarot-specific.
7. **Generic-able acceptance test.** `grep -rn "tarot\|INV-102\|INV-105\|INV-106\|INV-107\|nginx\|users\.credits\|SURFACE_MAP\|next-auth\|payos\|prisma\|\$transaction\|next-auth" agents/boundary-check.md templates/INVARIANTS-template.md .claude/commands/security-review.md` must return 0 hits at end of EXECUTE. Structural "strip tarot-specific" check (mirror P041 Constraint #7 pattern).
8. **Slash command spec compliance.** `.claude/commands/security-review.md` must conform to Claude Code's actual slash command format (Worker mirrors P041's `advisory-scan.md` proven format). Tầng 2 frontmatter / body structure adjustments OK + log to Discovery. If Claude Code spec REQUIRES a structurally different pattern that breaks spawn-only design → Tầng 1 escalate.
9. **PR comment fallback (no-PR-context).** If slash command runs against branch/range with no associated PR (`gh pr view` returns "no PR found"), slash command falls back to writing sentinel block to `docs/security/last-review.md` (Tầng 2 exact filename Worker self-decides). Slash command MUST NOT silently fail or skip post.
10. **Severity grading DEFERRED.** P042 ships flat-rubric (each FLAG ⚠️ counted equally, no Critical/High/Medium/Low per INV). Severity grading deferred to follow-on phiếu if user-feedback demands; tarot version had implicit severity via INV-specific severity column but kit-level neutral version simpler.
11. **Auto-fix suggestions OUT OF SCOPE.** Giám sát surfaces evidence; patch is a separate phiếu's job (Worker EXECUTE). Slash command MUST NOT call `gh pr edit --body <patched-version>` or similar.

---

## Nghiệm thu

### Automated

- [ ] All 8 new/modified files exist at correct paths.
- [ ] `agents/boundary-check.md` frontmatter parses as valid YAML (`name`, `description`, `tools`, `model`).
- [ ] `agents/boundary-check.md` frontmatter `tools:` line contains `Bash` (per Constraint #3).
- [ ] `agents/boundary-check.md` body contains "Bash usage" section explicitly listing allowed + forbidden invocations.
- [ ] `agents/boundary-check.md` body contains 5 INV headings (INV-1 through INV-5), NOT 7 (tarot's count).
- [ ] `templates/INVARIANTS-template.md` body contains 5 generic INV headings + "User-added INV" section.
- [ ] `.claude/commands/security-review.md` exists in directory (`.claude/commands/` already exists from P041; new file is 2nd entry).
- [ ] `grep -rn "tarot\|INV-102\|INV-105\|INV-106\|INV-107\|nginx\|users\.credits\|SURFACE_MAP\|next-auth\|payos\|prisma\|\$transaction" agents/boundary-check.md templates/INVARIANTS-template.md .claude/commands/security-review.md 2>/dev/null` returns 0 hits (Generic-able test per Constraint #7).
- [ ] `grep -c "security-review-start\|security-review-end" agents/boundary-check.md` = 2 (markers appear once each in Bước 4 example).
- [ ] `grep -c "security-review-start\|security-review-end" templates/INVARIANTS-template.md` ≥ 2 (sentinel contract documented).
- [ ] `grep -c "security-review-start\|security-review-end" .claude/commands/security-review.md` ≥ 3 (markers referenced in Steps 3 + 4 + Hard rules).
- [ ] `grep -c "gh pr merge\|block merge" .claude/commands/security-review.md` should appear only in NEGATION context (e.g., "does NOT call `gh pr merge --block`").
- [ ] `grep -c "ADVISORY\|advisory" agents/boundary-check.md` ≥ 3 (mode emphasized).
- [ ] `grep -c "ADVISORY\|advisory" .claude/commands/security-review.md` ≥ 2 (mode emphasized).
- [ ] `docs/LAYERS.md` Giám sát column has NO `(P042 will spec)` strings remaining.
- [ ] `docs/HANDOFF.md` "Upcoming: Giám sát" stub has been replaced (no longer present).

### Manual Testing (dry-run)

- [ ] **PR-mode dry-run.** Use the current P042 phiếu's own branch + open a PR. From Claude Code session: `/security-review <PR>`. Expect: slash command captures diff, spawns Giám sát, returns verdict with 5 INV lines + APPROVE/NEEDS_REVIEW. Since P042 itself only touches docs/templates (no env var / external call / route / webhook / dep bump), expected verdict = `APPROVE` + 5 PASS (likely N/A on each). Silent-when-clean rule fires → no PR comment posted.
- [ ] **Branch-mode dry-run (no PR).** From a local branch with no associated PR: `/security-review <branch>`. Expect: diff captured via `git diff main..<branch>`, Giám sát returns verdict, slash command falls back to writing sentinel block to local file (e.g., `docs/security/last-review.md`). No `gh pr comment` attempted.
- [ ] **Intentional FLAG trigger.** Create a scratch branch that intentionally adds `process.env.NEW_SECRET_KEY` to a file without updating `.env.example`. Run `/security-review <branch>`. Expect: INV-1 fires ⚠️ FLAG with file:line evidence. Verdict = `NEEDS_REVIEW`. Comment posted (no PR) OR written to local file.
- [ ] **Empty diff case.** Run `/security-review` against current HEAD with no diff (clean branch). Expect: Giám sát returns 5 PASS with N/A on each. APPROVE verdict. Silent-when-clean → no comment.
- [ ] **No `gh` available case.** In env without `gh` CLI installed, run `/security-review <PR>`. Expect: graceful fallback to `git diff` mode OR user-message asking to provide diff manually. No crash.
- [ ] **5-INV smoke test.** For each INV individually, craft minimal diff that triggers the INV (env var no `.env.example`, fetch() with no timeout, route handler with no `where userId`, webhook with no signature verify, `"next": "^14.0.0"` → `"next": "^15.0.0"` without PR body changelog). Run `/security-review` on each. Expect each respective INV to fire ⚠️ FLAG with file:line evidence. Verify 5 separate dry-runs each produce exactly the intended INV's flag.
- [ ] **Generic-able verification.** `grep -rn "tarot\|INV-102\|INV-105\|nginx\|users\.credits\|SURFACE_MAP" agents/boundary-check.md templates/INVARIANTS-template.md .claude/commands/security-review.md` = 0 hits.

### Regression

- [ ] `sos init security` (from P040) unchanged — Worker confirms `bin/sos.sh` not touched.
- [ ] `/advisory-scan` (from P041) unchanged — Worker confirms `.claude/commands/advisory-scan.md` not touched, `agents/advisory-watch.md` not touched, `templates/advisory-inbox.md` not touched, `scripts/parsers/*` not touched.
- [ ] `agents/orchestrator.md`, `agents/architect.md`, `agents/worker.md` unchanged.
- [ ] `phieu/TICKET_TEMPLATE.md` unchanged.
- [ ] `docs/PHILOSOPHY.md`, `docs/ORCHESTRATION.md` unchanged.
- [ ] Existing skills (`/init`, `/plan`, `/verify`, etc.) all still invoke per `docs/LAYERS.md` skills map.

### Docs Gate

- [ ] `CHANGELOG.md` — new entry at top: "P042: Giám sát (boundary-check) specialist subagent — generic port from tarot, strips 7 INV → 5 generic (drop tarot's nginx + users.credits). Adds `agents/boundary-check.md` (read-only-output specialist, tools: Read/Grep/Glob/Bash-scoped-to-git-and-grep), `templates/INVARIANTS-template.md` (5-INV skeleton + user-added section), `.claude/commands/security-review.md` (orchestrator-side spawn-only caller, posts PR comment in ADVISORY mode — KHÔNG block merge). 5 generic INV: env var template / external service timeout / cross-user binding / webhook signature / dep major changelog audit. Sentinel markers: `<!-- security-review-start --> / <!-- security-review-end -->`. Silent-when-clean rule preserved from tarot P275 lesson. Wave 1 final phiếu shipped — P040+P041+P042+P043 complete."
- [ ] `docs/LAYERS.md` — Giám sát column filled (Task 4a).
- [ ] `docs/HANDOFF.md` — "Pattern: Quản đốc ↔ Giám sát" subsection + summary table row (Task 4b).
- [ ] `README.md` — Subagents table 5th row + Security paragraph extension (Task 4c).
- [ ] `docs/SETUP.md` — Security pipeline Step 5 added + section heading updated to "(P040 + P041 + P042)" (Task 4d).
- [ ] `CLAUDE.md` — repo tree updates (Task 4e).
- [ ] `docs/BACKLOG.md` — P042 row moved from Active sprint to "Recently shipped" (orchestrator handles post-merge, NOT Worker mid-execute). Wave 1 sprint "Done when" criteria all met → mark sprint complete.

### Discovery Report

- [ ] Write to `docs/discoveries/P042.md` (per-phiếu file, P038 pattern):
  - **Assumptions in phiếu — CORRECT** (per Task 0 verification results — especially Anchors #1, #2, #3, #4, #8, #10 confirming tarot mapping).
  - **Assumptions in phiếu — WRONG / adapted** — particularly: (a) Anchor #7 `templates/` directory count (Worker re-Globs), (b) Anchor #15 SETUP.md insertion point Worker chose, (c) Anchor #16 CLAUDE tree placement Worker chose, (d) Anchor #18 Quản đốc Bash allowlist for `gh pr diff` — did it work as-is or need fallback?
  - **Scope expansions / contractions** — Was `INVARIANTS-template.md` shipped to `templates/` or somewhere else? Was the fallback file path `docs/security/last-review.md` used during dry-run (or a different filename)?
  - **Tarot port faithfulness** — Worker reports: which tarot patterns ported (sentinel marker rename + silent-when-clean + verdict-format), which dropped (INV-102, INV-105, INV-107 10-deps allowlist, SURFACE_MAP refs), which adapted (5-INV generic vs 7-INV tarot). Confirm Constraint #7 generic-able test passed.
  - **Sentinel marker rename impact** — `SECURITY_REVIEW_START/END` → `security-review-start/end` rename held cleanly across agent + template + slash command? Any drift?
  - **ADVISORY mode validation** — Did dry-run confirm NO merge block? Did silent-when-clean rule fire correctly on APPROVE? Did NEEDS_REVIEW cases post comment (or write fallback) as expected?
  - **5-INV rubric quality** — During smoke tests for each individual INV trigger: did each INV fire correctly? Any false positives / false negatives observed? Worker notes specific improvement candidates for follow-on phiếu (severity grading deferred per Constraint #10).
  - **CHALLENGE round value** — Was the CHALLENGE round (Worker → Architect) valuable for this Tầng 1 phiếu? Did Worker catch real Tầng 1 issues (Quản đốc Bash allowlist, fallback path, sentinel rename impact) before EXECUTE?
  - **Token + time cost** — Architect estimate: half-day. Actual? Wave-1 final-phiếu data point (P040 + P041 baselines).
  - **Giám sát first-run UX** — Dry-run nghiệm thu: did the slash command + agent loop produce a usable PR comment? Worth shipping to user projects as-is, or rough edges need follow-on?
  - **Wave 1 retrospective summary** — All 4 phiếu (P040 + P041 + P042 + P043) shipped. "Done when" criteria from BACKLOG line 13 verified: `sos init security` works zero-workaround, `/advisory-scan` works zero-workaround, `/security-review <PR>` posts advisory comment zero-workaround. Confirm sprint can close.
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`.
