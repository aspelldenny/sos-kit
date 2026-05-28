# Workflow v2.2 — Soul Signature SOS Kit

> **Status:** SPEC — materialized 2026-05-28 from `~/sos-kit/docs/retro/WORKFLOW_V2.2_RETRO_advisory-inbox.md` (CLOSED).
> **Home:** `~/sos-kit/docs/WORKFLOW_V2.2.md` (durable doctrine — không rotate).
> **Pilot vòng 1:** `advisory-inbox` (Rust, oracle SOUND) — done. 13 phiếu, 14 PR, 69 test pass.
> **Pilot vòng 2 (pending):** repo Python/TS THẬT, partial-oracle — chưa pick.
> **Replaces:** `~/sos-kit/docs/WORKFLOW_V2.1.md` (giữ historical, không xóa).

---

## §0. Diff vs v2.1 — TL;DR

v2.1 trả lời "có an toàn / có trigger / có durable chưa". v2.2 trả lời **"có RẺ hơn chưa, và có chống mù khi oracle PARTIAL chưa"**.

| Gap v2.1 phơi qua pilot | Fix v2.2 |
|--------------------------|----------|
| Lane label, KHÔNG cắt tải (8/13 Guarded, P003 643 dòng phiếu cho 6 test) | §1 Lane budgets `[gate]` |
| Oracle-resolvable objection bị debate LLM (50-100k token/vòng) | §2 Oracle-first routing với trục claim-match |
| Discovery N/A matrix (half rows "không áp dụng") | §3 Sparse discovery |
| Architect đọc 113k+ token/draft (tarot sẽ thảm họa hơn) | §4 AGENT_MAP shape + validator |
| edit/verify scope không tách (worker sờ chân voi) | §5 Asymmetric: edit gate, verify guidance |
| State cầm tay (echo counter, manual rm marker) | §6 State auto `[hook]` |
| 6 Sub-mech prose để agent nhớ | §7 Sub-mech → hook + doctor binary |
| Boundary-check rubric mù với INV-LOCAL-* (canary 1 miss) | §8 Rubric injection `[hook]` |

**Mảng v2.2 CHƯA chứng minh:** partial-oracle (Python/TS), brownfield migration, multi-stakeholder, cross-repo orchestration. Pilot vòng 2 sẽ test.

---

## §0.1. Doctrine principles — 3 luật cứng

**Mọi mục v2.2 dưới đây tuân thủ 3 luật. Vi phạm luật = không phải v2.2 fix, là drift.**

### Luật 1 — Mỗi fix phải gắn cờ cơ chế

```
[gate]     — block trên exit ≠ 0, mechanical kiểm được
[hook]     — one-liner nổ đúng lúc (PreToolUse / pre-commit / session-start)
[guidance] — judgment thật, không cơ chế hóa được — giữ tối giản
```

KHÔNG fix nào là prose để agent nhớ. Prose = drift, đã chứng minh qua DISCOVERIES.md tarot 265KB (rule rotate prose, 5x over soft-cap, không fire).

### Luật 2 — Một bệnh, một cơ chế rẻ nhất bắt 80%

Cấm 3 tầng `gate + hook + guidance` cho một bệnh. Pattern thấy-phát-hiện → mọc-cơ-chế là phản xạ over-engineer.

**Precedent:** N1-Fix sau canary 1: Orchestrator đề xuất 3 tầng. Sếp cắt về 1 [hook] sau canary 2. Subagent đọc semantic được nếu được CHỈ phải canh — không cần [gate] verdict validation (sân khấu) + [guidance] bắt tự grep (prose).

### Luật 3 — Mechanical mới gate, judgment giữ guidance

Mechanical (grep, count, exit code) → `[gate]` / `[hook]`.
Judgment (INV-101..108, câu hỏi vàng, "có over-engineer không", "tone đúng character không") → `[guidance]`.

Ép judgment thành hook giả → phán bừa. Giữ judgment ở advisory mode (boundary-check post comment, KHÔNG block merge).

---

## §1. Lane budgets `[gate]`

### Bệnh
P003 Normal lane chạy full DRAFT → CHALLENGE → RESPOND → SURGICAL → EXECUTE cho 6 test. Lane field là label, không enforce.

### Cơ chế

```
Normal lane (budget):
  - phiếu ≤ 250 dòng
  - ≤ 5 anchors
  - ≤ 5 hard constraints
  - KHÔNG full Sub-mech matrix
  - KHÔNG full provenance block
  - KHÔNG full RESPOND nếu objection oracle-đóng-claim (§2)

Guarded lane: full quyền (dài, full RESPOND, Sub-mech matrix).

Fast lane: ≤ 100 dòng phiếu, KHÔNG architect, worker tự execute, oracle-first only.
```

### Override — phải ĐAU (chọn 1)
- **(a) Chủ nhà duyệt** override, KHÔNG orchestrator tự duyệt.
- **(b)** Override-rate **hard-fail ngưỡng**: vượt 20% trong 50 PR gần nhất → STOP cả sprint.

Cả hai option ngăn drift "gặp phiếu dài, gõ reason, qua".

### Enforcement
- `[gate]` pre-CHALLENGE: `doctor lane-check --ticket <path>` đếm dòng + anchor + constraint vs lane field. Vượt budget không có cờ override → block.
- Token cap đi kèm (N2 ship cùng nhịp):
  - Fast: 30k token / subagent
  - Normal: 80k token / subagent
  - Guarded: 150k token / subagent

Vượt cap → orchestrator AskUserQuestion "scope creep?".

### Verification
```bash
doctor lane-check --ticket docs/ticket/P020-foo.md
# exit 0 = budget OK
# exit 1 = budget exceeded + reason
# exit 2 = ticket missing lane field
```

---

## §2. Oracle-first routing `[guidance]` + `[gate]`

### Bệnh
Ba objection đắt nhất sprint (P003/P011/P013) đều oracle phán được trong 30s nhưng route qua Architect RESPOND tốn 50-100k token/vòng.

### Cơ chế

**Worker CHALLENGE hỏi 2 câu, KHÔNG 1:**
1. Loại objection? (mechanical / shape / design)
2. **Oracle nào phán đúng CLAIM, và SOUND hay PARTIAL?**

**Routing:**

```
[mechanical + oracle SOUND đóng-claim]   → Worker fix trực tiếp. KHÔNG Architect.
[shape + oracle SOUND đóng-claim]        → Worker compile/probe, fix tại chỗ. KHÔNG Architect.
[shape + oracle PARTIAL]                  → Worker chạy oracle như SÀNG. Phần chạm contract cần contract-test HOẶC Architect short.
[shape + architecture-impact]             → Architect short respond.
[design / security / cross-cutting]       → Architect full respond.
```

### Oracle SOUND vs PARTIAL — table

| Stack | Oracle | Verdict |
|-------|--------|---------|
| Rust | `cargo check`, `cargo clippy -- -D warnings` | SOUND cho shape |
| Rust | `cargo test` | PARTIAL cho logic |
| JSON | schema validator (e.g. ajv, jsonschema) | SOUND cho structural |
| CLI | `<binary> --help` về sự tồn tại của flag | SOUND |
| CLI | `<binary> --help` về behavior của flag | PARTIAL |
| Grep | exact line/literal match | SOUND |
| Next.js | `pnpm install --frozen-lockfile && next build` env sạch | PARTIAL (đỡ partial hơn `pnpm test`) |
| Next.js | `pnpm test` một mình | PARTIAL-weak |
| TypeScript | `tsc --noEmit` strict | PARTIAL — `any` = vùng MÙ |
| Python | `mypy --strict` | PARTIAL (nếu type có) |
| Python | `pytest` | PARTIAL |

**Doctrine:** *Compiler/--help/schema phán đúng CLAIM thì đừng cho LLM debate. Oracle PARTIAL chỉ là vòng SÀNG, không phải vòng QUYẾT.*

### Oracle đóng CLAIM ≠ oracle chạy được

Critical distinction (round 3 ChatGPT fix):

```
P011 import path Parameters sai
  CLAIM = "path này tồn tại không"
  Oracle: cargo check
  Verdict: SOUND, ĐÓNG ĐƯỢC → Worker self-fix.

P013 --report - có nhận stdin
  CLAIM = "flag behavior thế nào"
  Oracle: --help / smoke test
  Verdict: SOUND, ĐÓNG ĐƯỢC → Worker self-fix.

P003 str::find vs regex
  CLAIM = "docs wording 'regex' có buộc dùng regex crate không"
  Oracle: cargo check
  Verdict: ❌ Compiler CÂM với docs wording — KHÔNG đóng được.
  → Cần docs-precedence rule HOẶC Architect short.
```

### Worker checklist trước bỏ debate `[guidance]`

```
Worker BẮT BUỘC ghi 3 thứ trong discovery report:
  - Claim: "what the objection actually asks"
  - Oracle: "<tool/command that judges this claim>"
  - Soundness: SOUND | PARTIAL | NONE for this claim

Thiếu 3 thứ → không được self-close objection.
```

### Enforcement
- `[guidance]` worker.md handbook.
- `[gate]` discovery-report parser ép có 3 field trên khi worker close objection ngoài RESPOND cycle.

---

## §3. Sparse Discovery `[guidance]`

### Bệnh
Discovery N/A matrix — half rows "Sub-mech X N/A, không áp dụng". Agent chứng minh "đã nhớ catalog".

### Cơ chế

```
## Sub-mechanism fired

- B (capability): cargo check/test/clippy/fmt pass
- D (persistence): doctrine sync to CLAUDE.md

## Not fired
none required by classifier
```

KHÔNG bảng A-F với N/A. Hook §7 tự nổ đúng cái liên quan.

---

## §4. AGENT_MAP shape + validate-map `[gate]` + `[guidance]`

### Khi nào cần
- KHÔNG cần repo < 10 docs (advisory-inbox).
- CẦN repo > 10 docs HOẶC docs > 500KB total (tarot 1.8MB).

### Shape

```yaml
# AGENT_MAP.yaml
version: 1

surfaces:
  <surface_name>:
    load_bearing: true | false
    edit: [glob patterns of files allowed to edit]
    read_shallow: [docs Sếp đọc khi touch surface KHÔNG load-bearing]
    read_deep: [docs Sếp đọc khi touch surface load-bearing]
    research_gate: [docs BẮT BUỘC đọc — special class]
    contract_test: [test files chạm boundary, không mock]
    blast: "câu mô tả 'đổi cái này gãy đâu'"

never_default_read:
  - <docs nặng KHÔNG nạp default — CHANGELOG/DISCOVERIES/BACKLOG/Archive>
```

### Doctrine
- Tách `read_shallow` vs `read_deep` cho cùng surface (sửa case "đổi 1 dòng copy mà đọc 327KB").
- `never_default_read` = ăn 50%+ token cháy mà KHÔNG đổi lấy gì (log + idea, không doctrine).
- `blast` = cho architect quyền DỪNG đọc khi thấy hết vùng nổ.

### validate-map `[gate]` — BẮT BUỘC

```bash
doctor validate-map --map docs/AGENT_MAP.yaml
# Check 1: mọi path trong edit/read_shallow/read_deep/research_gate/contract_test tồn tại (test -e)
# Check 2: mọi anchor #section còn trong file đích
# Check 3: contract_test files compile/parse (per stack)
# Exit 0 = OK | 1 = drift | 2 = parse error
```

Chạy pre-commit. Map drift còn tệ hơn không map vì nó chủ động nói architect "leaf, đọc nông" trong khi load-bearing, và partial-oracle không vớt.

### Ranh giới mini-map pilot

Repo Python pilot dù nhỏ vẫn có mini-map 3 surface ĐỂ test cơ chế. Nhưng ghim rõ:
- Mini-map validate được **VALIDATOR MECHANISM** (path tồn tại, anchor còn).
- KHÔNG validate được **giá trị blast-radius / load-bearing flag** — vì greenfield surface là BỊA.

Cái sau chỉ kiểm được trên repo có lịch sử thật (tarot AGENT_MAP nếu build).

---

## §5. Edit-scope vs verify-scope — asymmetric

### Bệnh
v2.1 spec "scope hẹp chống ngứa nghề" nhưng worker hẹp về edit lẫn verify → sờ chân voi. Phải tách 2 thứ.

### Cơ chế

**Ticket template thêm 3 field:**

```yaml
edit_allow:           # [gate] — mechanical, grep diff
  - src/parser/**
verify_read:          # [guidance] — worker self-report
  - src/parser/**
  - docs/PARSER.md
  - tests/parser.rs
contract_tests:       # [gate] — must pass before EXECUTE
  - tests/parser_boundary.rs
```

### Enforcement

- **`edit_allow` = `[gate]`** pre-commit: `git diff --name-only` vs `edit_allow` glob → block file touched outside allow.
- **`verify_read` = `[guidance]`** worker.md: worker self-declare "đã đọc" trong discovery. KHÔNG enforce được agent đã đọc thật.
- **`contract_tests` = `[gate]`** pre-EXECUTE: test pass mới được commit.

### Asymmetric reason
"Agent edit ngoài allow" grep từ git diff (verifiable). "Agent đã đọc đủ file verify" không grep được → ép gate giả sẽ phán bừa (Luật 3).

---

## §6. State auto `[hook]`

### Bệnh
`echo "5" > .phieu-counter` gõ tay. `touch/rm .sos-state/architect-active` manual. Quên rm = kẹt state vĩnh viễn. DISCOVERIES.md tarot 265KB không rotate.

### Cơ chế

**`.phieu-counter` — atomic increment `[hook]`:**

```bash
# Function phieu() trong ~/.zshrc — KHÔNG echo gõ tay
phieu() {
  local counter_dir="$HOME/<repo>/.phieu-lock"
  mkdir "$counter_dir" 2>/dev/null || { echo "lock busy"; return 1; }
  trap "rm -rf $counter_dir" EXIT
  local n=$(($(cat ~/<repo>/.phieu-counter) + 1))
  echo $n > ~/<repo>/.phieu-counter
  # ... create branch + ticket file
}
```

`mkdir` atomic guarantee — không race. Trap cleanup khi exit (kể cả crash).

**`.sos-state/architect-active` — lock with TTL `[hook]`:**

```json
{
  "pid": 12345,
  "started_at": "2026-05-28T15:30:00Z",
  "ticket_id": "P020",
  "ttl_minutes": 30
}
```

Session-start hook check:
- Lock tồn tại + pid sống + TTL chưa hết → block architect mới.
- Lock tồn tại + (pid chết HOẶC TTL hết) → stale, clean up VỚI LOG.
- Lock không có → free.

KHÔNG dựa "agent tự rm lúc end" (crash thì kẹt).

**Rotate cap `[hook]`:**

```bash
# Pre-commit hook
doctor rotate-check
# Đếm dòng DISCOVERIES.md/CHANGELOG.md
# Soft cap 1000 dòng → warn
# Hard cap 1500 dòng → block (force archive trước)
```

Cấu hình trong `.sos-stack.toml`:
```toml
[rotate]
discoveries_soft = 1000
discoveries_hard = 1500
changelog_soft = 1500
changelog_hard = 2500
archive_dir = "docs/Archive/"
```

---

## §7. 6 Sub-mech: prose → hook

### Cơ chế

Toàn bộ "Layer 2 capability check" trong CLAUDE.md (hiện prose) → cơ chế chạy:

| Sub-mech | Hook | Trigger |
|----------|------|---------|
| **A** (trigger gap) | `block-unsafe-merge.sh`, `grep "if: *false" .github/workflows/`, `[[ -x hook ]]` check | PreToolUse `Bash gh pr merge`, session-start |
| **B** (capability) | `grep "export " src/app/**/route.ts \| grep -v "GET\|POST\|..."` → block illegal exports | pre-commit if touch `route.ts` |
| **B** (post-bump) | `pnpm install --frozen-lockfile && next build` | CI gate after `package.json` change |
| **C** (migration) | `doctor migrate-check --source <file> --target <state>` (`jq length` vs `wc -l` diff) | per phiếu touch state schema |
| **D** (persistence) | `doctor doctrine-check` (commit msg has `home:` field if touch CLAUDE.md / handbook) | pre-commit |
| **E** (env drift) | `pnpm install --frozen-lockfile` before any build | CI gate, manual pre-deploy |
| **F** (runtime state) | `doctor runtime-scan` (`.git/config`, `~/.ssh/config`, `.env*`, `.mcp.json` for token leaks) | SessionStart + pre-commit |

### Doctor binary — Rust, sống trong sos-kit golden template

Repo `~/doctor/`. Subcommands:

```
doctor lane-check     # §1 lane budget
doctor validate-map   # §4 AGENT_MAP path/anchor
doctor rotate-check   # §6 dòng cap
doctor migrate-check  # Sub-mech C
doctor doctrine-check # Sub-mech D
doctor runtime-scan   # Sub-mech F
```

1 binary, 6 subcmd, không 6 binary riêng. Pattern khớp 6 binary đã ship (vps/ship/guard/quality-gate/advisory-cron/advisory-inbox).

### Ranh giới

**Mechanical** gate được → §7. **Judgment** (INV-101..108, câu hỏi vàng, character voice) KHÔNG grep được → ở lại boundary-check advisory mode.

Đừng ép judgment thành hook giả.

---

## §8. Boundary-check rubric injection `[hook]`

### Bệnh (canary 1+2 evidence)
- Canary 1: subagent miss INV-LOCAL-002 atomic write degrade — CHÍNH INV subagent vừa verify clean ở P006 1 sprint trước.
- Canary 2: inject INV-LOCAL-* vào prompt → catch chính xác với reasoning sâu (userspace buffer vs fsync syscall).

Subagent đọc semantic được NẾU được CHỈ phải canh. Không đọc nếu không biết phải canh.

### Cơ chế — MỘT hook, hết

```
Skill /security-review (hoặc orchestrator pre-spawn boundary-check) BẮT BUỘC:
  1. Read docs/security/INVARIANTS.md
  2. Extract block matching `^## INV-LOCAL-` hoặc `^### INV-LOCAL-`
  3. Paste verbatim vào prompt cho boundary-check subagent
```

Implementation: < 20 dòng code trong skill / slash command.

### KHÔNG làm
- ❌ `[guidance]` "boundary-check.md handbook lệnh subagent tự grep INVARIANTS.md" — prose để nhớ.
- ❌ `[gate]` "verdict validation grep đủ INV expected" — sân khấu, bắt subagent "đề cập đủ vai" không tăng giá trị.

**Doctrine (canary 2 refined):** *Subagent đọc semantic được nếu được CHỈ phải canh. Một bệnh, một cơ chế.*

### Verification
Canary 2 (2026-05-28) — repro test:
1. PR có vi phạm subtle thuộc INV-LOCAL-* — orchestrator inject INV-LOCAL-* vào prompt → subagent FLAG.
2. Same PR — orchestrator KHÔNG inject → subagent MISS.

→ Inject mechanism active = correct behavior. Re-test mỗi quarter trên 1 PR canary.

---

## §9. Hook tiering — 3 tier

### Bệnh
Nếu chuyển hết prose → hook (lane budget + sparse discovery + validate-map + rotate cap + .phieu-counter + .sos-state + 6 sub-mech + rubric inject), pre-commit chạy 8-10 check/commit → workflow chậm → người bắt đầu `--no-verify` → Sub-mech A precedent phản tác dụng.

### Cơ chế

| Tier | Examples | Bypass policy |
|------|----------|---------------|
| **Tier 1 [block, no-bypass]** | Token leak (INV-009), unsafe merge, `.git/config` plaintext token, edit_allow violation, contract_tests fail | `--no-verify` BLOCKED bằng wrapper script. Chỉ Chủ nhà override với reason explicit. |
| **Tier 2 [warn, review weekly]** | Lane budget exceed, sparse discovery missing field, validate-map drift, rotate soft-cap, oracle SOUND/PARTIAL checklist | Warn nhưng pass. Orchestrator review weekly aggregated count. >X violations/week → trigger lane budget tune. |
| **Tier 3 [advisory]** | Commit msg format, doctrine home: field, naming convention | Informational. KHÔNG block. |

### Enforcement
- Tier 1 hooks call `set -e` + `exit 1` + reason.
- Tier 2 hooks log to `.sos-state/violations.jsonl` + echo warning + `exit 0`.
- Tier 3 hooks echo info + `exit 0`.

Orchestrator weekly digest tail `.sos-state/violations.jsonl` → Tier 2 pattern.

---

## §10. Watchlist sensors `[gate]` + `[hook]` + `[guidance]`

**Cấu trúc:** mỗi mảng MỘT sensor rẻ nhất bắt 80% ca. Pilot vòng 2 phơi M nào NỔ → M đó lên doctrine v2.3. M không nổ = giả định, vứt.

| ID | Vùng chưa test | Sensor (1 cơ chế) | Cờ |
|----|----------------|-------------------|-----|
| **N2** | Token cháy/subagent | Cap theo lane (Fast 30k / Normal 80k / Guarded 150k) → vượt = AskUserQuestion | `[gate]` |
| **N3** | Hook cross-repo fail (P013 bug đã lộ) | block-unsafe-merge detect `-R <owner>/<repo>` flag → invoke gh đúng repo | `[hook]` |
| **M1** | Real legacy data ≠ fixture (P013 hit format thứ 4) | Migration phiếu thiếu file snapshot trong `fixtures/` từ real export → block | `[hook]` |
| **M2** | Branch stale / rebase mid-flight | `git merge-base --is-ancestor origin/main HEAD` pre-EXECUTE | `[hook]` |
| **M3** | NEEDS_REVIEW path chưa chạy lần nào | Verdict NEEDS_REVIEW → orchestrator AskUserQuestion, KHÔNG tự skip | `[hook]` |
| **M4** | Hotfix lane / interrupt mid-sprint | Hotfix lane scope cứng (prod-down/security/user-blocking), security-review POST-merge | `[guidance]` |
| **M5** | CI flake / retry policy | Max 2 retry + 1-line flake reason, >2 = bug thật return Worker | `[guidance]` |
| **M6** | Counter/marker race (team) | Counter `mkdir` atomic; architect-active PID-tagged | `[hook]` (đã ship §6) |

### Phân biệt cốt lõi

**Fix ≠ Arm a sensor.**
- Fix M2 = viết cả policy rebase conflict khi chưa gặp conflict nào (over-engineer giả định).
- Arm M2 = cài đúng 1 dòng is-ancestor để khi stale THẬT thì nổ + Sếp THẤY, rồi fix dựa ca thật.

Việc của v2.2 KHÔNG phải fix hết watchlist. Là cài cảm biến để Python pilot DẠY cái nào là thật.

---

## §11. Out of scope v2.2

Ghi rõ KHÔNG làm gì, để tránh scope creep:

- ❌ Auto-PR fix dep (Sếp giữ van người-gate — feature có chủ đích).
- ❌ SBOM / CycloneDX export (solo + 30 dep, scale không cần — bias filter).
- ❌ EPSS / KEV signals (< 2 dòng/scan, prioritize không cần).
- ❌ Cross-repo orchestrator (P013 install advisory-inbox vào tarot là exception cross-repo phiếu, KHÔNG generalize).
- ❌ Multi-developer concurrency (advisory-inbox + tarot solo — defer M6 stress).
- ❌ AGENT_MAP load_bearing validator advanced (chỉ check path/anchor, không check "load_bearing flag khớp số caller").
- ❌ Subagent diff lớn (500+ dòng) test — defer organic Python pilot.
- ❌ Sub-mechanism G "Spec myopia" full fix — observe Python pilot first (em raise Round 2, defer).

---

## §12. Verification checklist — khi spec coi là đã ship

Mỗi mục có verify command. Không tick được = không ship.

| §  | Mục | Verify command | Expected |
|----|-----|----------------|----------|
| §1 | Lane budgets | `doctor lane-check --ticket <phiếu Normal vượt 250 dòng>` | exit 1 |
| §1 | Override hard-fail | `doctor lane-override-rate` (compute 50 PR rolling) | < 20% trong steady state |
| §2 | Oracle checklist enforced | grep `discovery.md` for 3 field claim/oracle/sound | all present |
| §3 | Sparse discovery | grep `discovery.md` for "N/A, không áp dụng" | 0 hits |
| §4 | validate-map runs | `doctor validate-map --map docs/AGENT_MAP.yaml` pre-commit | exit 0 trên clean repo |
| §5 | edit_allow enforce | `git diff --name-only` vs `edit_allow` glob | no file outside |
| §6 | counter atomic | concurrent `phieu foo` × 3 → no duplicate numbers | unique IDs |
| §6 | lock TTL | architect-active lock + sleep > TTL → next check | stale, clean up |
| §6 | rotate-check | DISCOVERIES.md > 1000 dòng pre-commit | warn |
| §6 | rotate-check hard | DISCOVERIES.md > 1500 dòng pre-commit | block |
| §7 | doctor binary | `which doctor && doctor --version` | binary exists |
| §7 | runtime-scan | `doctor runtime-scan` on repo with token in .git/config | flag |
| §8 | rubric inject | Spawn boundary-check on canary PR with INV-LOCAL-* | inject in prompt |
| §9 | tier 1 no-bypass | `git commit --no-verify` on Tier 1 violation | blocked by wrapper |
| §10 | sensors armed | All M1-M6 + N2-N3 sensors có code path | spot-check |

---

## §13. Migration v2.1 → v2.2

### Sos-kit golden template work (nhịp 3)

```
1. Add ~/sos-kit/docs/WORKFLOW_V2.2.md (file này — done)
2. Update ~/sos-kit/agents/architect.md với §2 oracle checklist
3. Update ~/sos-kit/agents/worker.md với §2 oracle 3-field + §5 asymmetric
4. Update ~/sos-kit/agents/orchestrator.md với §1 lane budget + §8 rubric inject + §10 sensor arm
5. Update ~/sos-kit/agents/boundary-check.md để CHỈ canh 5 generic INV (rubric inject từ caller, không tự grep)
6. Build ~/doctor/ Rust binary với 6 subcmd (§1, §4, §6, §7)
7. Ship hooks scripts/ trong template:
   - block-unsafe-merge.sh (đã có)
   - block-env-edit.sh (đã có)
   - pre-commit-rotate-check (mới)
   - pre-commit-edit-allow (mới)
   - pre-commit-doctrine-home (mới)
   - session-start-runtime-scan (mới)
8. AGENT_MAP shape template trong configs/AGENT_MAP.example.yaml
9. Update .docs-gate.toml để reflect Tier 1/2/3 settings
10. CLAUDE.md template update — reference WORKFLOW_V2.2.md, remove duplicate doctrine prose
```

### Per-repo bootstrap

```
1. Copy hooks + scripts từ template
2. Symlink .claude/agents → ~/sos-kit/agents (or copy snapshot)
3. cargo install --path ~/doctor (binary cache)
4. doctor init (generate .sos-stack.toml + AGENT_MAP.yaml stub)
5. Add to .mcp.json:
     "doctor": { "command": "~/.cargo/bin/doctor", "args": ["serve"] }
6. First commit: `doctor verify-setup` exit 0
```

### Pilot vòng 2 — repo Python/TS

**Tiêu chí:**
- Sếp THẬT SỰ cần (không bịa).
- Partial-oracle stack (Python `mypy --strict` + pytest, hoặc Next.js standalone).
- Greenfield OK (test mini-map mechanism); brownfield tốt hơn (test legacy migration).

**Candidate:** chưa pick. Pending Sếp decide.

**Test giả thuyết:**
- v2.2 có chống mù khi compiler không vớt không?
- AGENT_MAP shape có thay được tấm lưới rustc không?
- Hook tier 1/2/3 có giảm bypass không?
- Sensors M1-M6 cái nào nổ thật?

---

## §14. Provenance (forge log)

| Round | Tác giả | Input → Output |
|-------|---------|----------------|
| 1 | Claude Web | 6 chẩn đoán + đơn thuốc gắn cờ cơ chế |
| 2 | Orchestrator advisory-inbox | §6 watchlist M1-M6 + N1-N3 (vùng pilot không dạy) |
| 2b | Claude Web | 1-sensor-rẻ-nhất, arm-not-fix, N1 canary nâng cao |
| 3 | ChatGPT | 5 patch: oracle-đóng-claim, structured override, blind canary, mini-map pilot, lock PID/TTL |
| 3b | Claude Web | Override phải ĐAU; canary 2-PR; mini-map test validator-mechanism only |
| 4 | Orchestrator tarot | Thứ tự nhịp unified, doctor=Rust, edit/verify asymmetric |
| 4b | Sếp + Orchestrator tarot | Canary 1+2 chạy. N1-Fix cắt 3 tầng → 1 hook. Luật vàng 2 "một bệnh một cơ chế" thêm |

Full retro trace: `~/sos-kit/docs/retro/WORKFLOW_V2.2_RETRO_advisory-inbox.md` (CLOSED 2026-05-28).

---

## §15. Câu hỏi mở v2.2 → v2.3 (defer)

Pilot vòng 2 Python sẽ trả lời:

1. Lane budget 250 dòng — cứng hay tune?
2. Oracle SOUND/PARTIAL table — Go/Java/Ruby khi gặp xếp đâu?
3. AGENT_MAP load_bearing flag drift — có cần advanced validator?
4. Hook tier threshold — Tier 2 warn weekly có đủ không, hay cần tier 1.5 daily?
5. Subagent consistency — 2 spawn cùng diff verdict identical?
6. Sub-mechanism G "Spec myopia" (P010 → P011) — fire trong Python pilot không?
7. Cross-repo phiếu (P013 pattern) — generalize hay exception?
8. Brownfield migration overhead — tarot AGENT_MAP build vs maintenance ROI?

---

**End of Workflow v2.2 spec.**

Next action: Sếp duyệt → nhịp 3 (sos-kit template work) → nhịp 4 (Python pilot pick + bootstrap).
