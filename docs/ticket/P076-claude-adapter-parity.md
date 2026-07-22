# PHIẾU P076: Claude Code adapter parity — declarative boundary

> **Loại:** Feature (architecture / boundary extraction)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — adapter boundary + role/hook wiring là contract surface; sai thì LAN sang mọi host integration + P077 renderer. AUTO Tầng 1.)
> **Lane:** Guarded
> **Ảnh hưởng:** `adapters/claude/**` (mới), `agents/*.md` (annotate provenance), `skills/**/SKILL.md` (annotate), `docs/LAYERS.md`, `docs/HANDOFF.md`, `core/ASSETS.md`, `CLAUDE.md` repo-tree
> **Dependency:** P075 (merged `462a1e8`). Golden oracle frozen `docs/golden/P076-claude-baseline.md`.

---

## ⚠️ ESCALATE GATE — founder confirm TRƯỚC khi Worker EXECUTE

Phiếu chốt hướng **(B) declarative boundary** (không di chuyển file vật lý — chờ P077 renderer). Lý do đầy đủ ở "Giải pháp" + "Phân tích CRUX" dưới. **Physical extraction (A) đã được chứng minh KHÔNG khả thi trong P076** mà không (a) vỡ golden parity — flip git-mode `.claude/settings.json` `100644`→`120000`, hoặc (b) viết temp renderer mà plan cấm.

**Một câu hỏi owner-decision duy nhất cần Sếp phán (orchestrator hỏi qua AskUserQuestion):**

> DoD của plan viết: *"Claude-specific source/wiring có owner duy nhất **dưới adapter boundary**."* P076 giao owner-duy-nhất theo dạng **DECLARATIVE** (mapping manifest `adapters/claude/MAPPING.md` — mỗi Claude artifact trỏ về 1 core source ID; file vật lý vẫn nằm nguyên chỗ, golden diff = 0). Physical move sang `adapters/claude/**` defer P077 (khi renderer tồn tại).
>
> **Sếp confirm:** (B) declarative boundary = ĐỦ cho P076 DoD? Hay Sếp muốn physical move NGAY (→ phải kéo P077 renderer lên sớm = đổi order P076→P077 đã chốt trong architecture)?

- Nếu Sếp chọn **(B) đủ** → phiếu EXECUTE nguyên văn dưới đây.
- Nếu Sếp chọn **physical now** → đây là quyết định đổi order/scope (plan "Điểm phải dừng hỏi founder" #4); phiếu này REWRITE, orchestrator mở lại P076/P077 sequencing. **Architect KHÔNG tự quyết đổi order.**

Không phải "dừng hỏi founder" về semantics — không có semantic conflict giữa golden và core. Đây thuần là scope-definition của một chữ ("dưới adapter boundary" = declarative vs physical).

---

## Context

### Vấn đề hiện tại

P075 đã tách semantic core (`SOS.md`, `core/**`) nhưng wiring riêng của Claude Code vẫn trộn với portable semantics ở nhiều surface (`RUNTIME_BOUNDARY_INVENTORY.md` dòng 18-26):

- `CLAUDE` thuần: `.claude/settings.json`, `.claude/commands/**`, `templates/claude-settings.local.json`.
- `GENERATED`: `.claude/agents/**`, `.claude/skills/**` (symlink registration).
- `TRANSITIONAL_MIXED`: `agents/*.md` (role doctrine + Claude frontmatter), `skills/**/SKILL.md` (workflow body + Claude caller/invocation), lifecycle guard scripts (policy intent + Claude event binding).

Chưa có **owner duy nhất** cho các Claude artifact này, và chưa có mapping tường minh từ mỗi artifact về core source ID. `adapters/` chưa tồn tại (verify: Glob `adapters/**` = rỗng).

### Giải pháp — (B) Declarative boundary + mapping manifest + golden parity-proof

P076 **KHÔNG di chuyển file vật lý** và **KHÔNG viết renderer** (cả hai thuộc P077). Thay vào đó:

1. **Tạo `adapters/claude/`** chứa boundary doc + mapping manifest. Manifest liệt kê mỗi Claude artifact → class ownership → **core source ID** ổn định (`core/ROLES.md#<role_id>`, `core/POLICY.md`, `core/WORKFLOW.md`). Đây là "owner duy nhất" ở dạng khai báo (ASSETS.md dòng 51: *"An adapter-owned artifact must identify the portable source or policy it represents"* — nghĩa vụ mapping, không phải mandate vị trí file).
2. **Annotate provenance** trên các MIXED file — thêm 1 marker trỏ về core source ID, **đặt ở body/comment, KHÔNG đụng frontmatter field mà golden probe đọc** (name/model/tools/caller). Giữ golden section 3-4 diff = 0.
3. **Chứng minh golden parity** — Worker re-run mọi probe trong `docs/golden/P076-claude-baseline.md` section 1-9 + fire-test section 10; diff = 0 (vì không đổi runtime file nào). Đây là oracle chính.

Physical file-move + render defer P077 (renderer `sos-adapter-claude::render()` per `PORTABILITY_ARCHITECTURE.md` dòng 32, 144).

### Phân tích CRUX — vì sao (A) bị chặn, (B) là plan-as-written

| Ràng buộc | Hệ quả cho (A) |
|---|---|
| Golden section 1: `.claude/settings.json` = git mode `100644` (real file, KHÔNG symlink); `.claude/commands/**` = `100644` | Move sang `adapters/claude/` + re-symlink → mode flip `100644`→`120000` → **golden tree diff ≠ 0** → parity FAIL |
| MIXED `agents/*.md`, `skills/**` = portable body + Claude frontmatter chung 1 file | Tách + recombine thành cái Claude đọc **cần renderer**; renderer = P077 (`PORTABILITY_ARCHITECTURE.md` dòng 32, 56-64) |
| Plan step 3 (dòng 25): "Không viết temporary renderer mà P077 sẽ phải vứt bỏ" | (A) không có renderer → hoặc vỡ parity hoặc đẻ temp renderer (cấm) |

→ (A) buộc: vỡ parity HOẶC temp renderer. Cả hai vi phạm plan. **(B) là đọc duy nhất không tự mâu thuẫn của plan** (step 3: "adapter chỉ tham chiếu/render", renderer ở P077). Chọn (B) = ở trong scope; chọn (A) = kéo P077 lên = owner-decision (ESCALATE GATE trên).

### Scope

- **CHỈ Claude adapter boundary (declarative).**
- **KHÔNG** Rust workspace / crate (P077). **KHÔNG** Codex (P078). **KHÔNG** renderer tạm dưới bất kỳ hình thức. **KHÔNG** di chuyển/move file vật lý. **KHÔNG** sửa contract "Not a runtime binary source" trong `CLAUDE.md` (P077 sở hữu — `PORTABILITY_ARCHITECTURE.md` dòng 41).

---

## Task 0 — Verification Anchors

> Architect docs-only (không Bash/Grep). Anchor `[verified]` = đã Read doc xác nhận. `[needs Worker verify]` = Worker BẮT BUỘC grep/bash trước khi apply/nghiệm thu.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `core/ROLES.md` định nghĩa stable role IDs `owner`/`orchestrator`/`architect`/`worker`/`advisory_watch`/`boundary_check` + capability vocabulary — dùng làm core source ID cho mapping `[verified]` (đã Read core/ROLES.md dòng 22-113) | `grep -nE '^## \`(owner\|orchestrator\|architect\|worker\|advisory_watch\|boundary_check)\`' core/ROLES.md` | ⏳ TO VERIFY |
| 2 | `core/POLICY.md` + `core/WORKFLOW.md` tồn tại (core source ID cho policy/workflow mapping) `[verified]` (Glob `core/*.md` trả cả 5) | `ls core/POLICY.md core/WORKFLOW.md` | ⏳ TO VERIFY |
| 3 | `adapters/` chưa tồn tại — P076 tạo mới, không đụng cây cũ `[verified]` (Glob `adapters/**` = rỗng) | `test ! -e adapters` | ⏳ TO VERIFY |
| 4 | `.claude/settings.json` là REAL file git mode `100644` (KHÔNG symlink) → physical move sẽ flip mode → chứng minh (A) vỡ parity `[verified]` (golden section 1 dòng 15) | `git ls-files --stage .claude/settings.json` → expect `100644` | ⏳ TO VERIFY |
| 5 | `.claude/commands/**` = `advisory-scan.md` + `security-review.md`, mode `100644` `[verified]` (golden section 1 dòng 13-14, section 5) | `git ls-files --stage .claude/commands/` | ⏳ TO VERIFY |
| 6 | `.claude/agents/**` + `.claude/skills/**` = symlink mode `120000` trỏ `../../agents/**`, `../../skills/**` `[verified]` (golden section 1-2) | `git ls-files --stage .claude/agents .claude/skills` → expect `120000` | ⏳ TO VERIFY |
| 7 | `agents/*.md` frontmatter (name/model/tools) = golden section 3 dòng 38-42; annotate provenance PHẢI ở body/comment để không đổi các field này `[needs Worker verify]` | `grep -nE '^(name\|model\|tools):' agents/*.md` diff vs golden section 3 | ⏳ TO VERIFY |
| 8 | `skills/*/SKILL.md` frontmatter (name/caller) = golden section 4 dòng 47-51; annotate KHÔNG đổi caller value `[needs Worker verify]` | `grep -nE '^(name\|caller):' skills/*/SKILL.md` diff vs golden section 4 | ⏳ TO VERIFY |
| 9 | `templates/claude-settings.local.json` tồn tại = permission template Claude syntax (`CLAUDE`/ADAPTER_OWNED per inventory dòng 21) `[needs Worker verify]` (không Read được, ref từ inventory + CLAUDE.md) | `test -f templates/claude-settings.local.json` | ⏳ TO VERIFY |
| 10 | `.claude/settings.json` hook wiring bind SessionStart/PreToolUse/UserPromptSubmit → scripts (`session-start-banner`, `architect-guard`, `block-env-edit`, `orchestrator-guard`, `block-unsafe-merge`, `idea-smell`) `[verified]` (golden section 6 dòng 61-116) | re-read golden section 6 | ⏳ TO VERIFY |
| 11 | Không có renderer/render infra sống trong repo (rendering = P077) — do đó (A) không có công cụ render `[needs Worker verify]` | `grep -rIl -E 'render\(|renderer' bootstrap/ bin/ 2>/dev/null` → expect none binding `.claude/**` | ⏳ TO VERIFY |
| 12 | Edit BODY của `agents/*.md` / `skills/**` (thêm HTML-comment provenance) KHÔNG đổi output golden probe section 3-4 (probe chỉ đọc frontmatter field) `[needs Worker verify]` — Worker confirm annotation placement | re-run golden section 3-4 probe sau annotate → diff = 0 | ⏳ TO VERIFY |
| 13 | Golden baseline section 1-9 probes reproducible bytewise (oracle chính) `[needs Worker verify]` | re-run toàn bộ probe → diff vs `docs/golden/P076-claude-baseline.md` = 0 | ⏳ TO VERIFY |

**Oracle:** Anchor #4,5,6,7,8,12,13 = `[oracle: golden baseline diff]` — SOUND. Parity claim đóng được bằng re-run probe + `git ls-files --stage` diff. Anchor #1,2,3 = `[oracle: filesystem]`. Anchor #11 = `[oracle: grep, partial]` — grep chứng minh vắng mặt render binding, không chứng minh ý định P077 (đó là `[design]`, plan-fixed).

---

## Debate Log

> Guarded lane — full RESPOND, no cap-3 shortcut. Worker CHALLENGE bắt buộc (Tầng 1).

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge
*(Worker điền khi CHALLENGE. Nếu không objection: "Worker accepted V1 — no challenges. Ready for Chủ nhà approval.")*

**Anchor verification (recap Task 0):**
- Anchor #N: ✅/⚠️/❌ + 1-line

**Objections (Tầng 1):**
- [O1.1] …

**Proposed alternatives:**
- A. …

**Status:** ⏳ AWAITING ARCHITECT RESPONSE

### Final consensus
- Phiếu version: V1 (no revision needed)
- Total turns: 1 (Worker CHALLENGE — accepted, no objections; 1 non-blocking completeness note re: `.mcp.json` row, folded into EXECUTE)
- Approved by Chủ nhà: 2026-07-22 (ESCALATE GATE (B) declarative confirmed) — EXECUTE completed same session, commit `404450b`

---

## Nhiệm vụ

### Task 1: Tạo boundary doc `adapters/claude/README.md`

**File:** `adapters/claude/README.md` (MỚI — tạo cả thư mục `adapters/claude/`)

**Thêm:** Doc khai báo Claude adapter boundary. Nội dung bắt buộc:
- Adapter sở hữu **serialized representation only** (ASSETS.md dòng 42-51): host entry instructions, agent/skill registration, lifecycle event binding, tool/capability map, permission config, MCP registration.
- **Dependency direction:** adapter → core (một chiều); core KHÔNG import/name adapter (core/README.md dòng 12-16). 
- **Transition note (P076):** boundary hiện là **declarative** — file vật lý chưa move; mỗi artifact khai owner qua `MAPPING.md`. Physical render/move = P077 (`sos-adapter-claude::render()`).
- Trỏ `SOS.md` + `core/README.md` là semantic source of truth.

**Lưu ý:** Đây là adapter-owned doc, không phải runtime binary → KHÔNG vi phạm "Not a runtime binary source" (đó nói về Rust source, P077).

### Task 2: Tạo mapping manifest `adapters/claude/MAPPING.md`

**File:** `adapters/claude/MAPPING.md` (MỚI)

**Thêm:** Bảng — mỗi Claude artifact → class → **core source ID** → migration ticket. Tối thiểu các row (dùng path THẬT sau khi verify anchor #4-9, KHÔNG bịa path):

| Artifact (physical, verify) | Class | Core source ID | Physical move |
|---|---|---|---|
| `.claude/settings.json` | ADAPTER_OWNED | lifecycle binding của policy intent → `core/POLICY.md` (+ guard scripts) | P077 |
| `.claude/commands/advisory-scan.md`, `.claude/commands/security-review.md` | ADAPTER_OWNED | command entry → `core/WORKFLOW.md` (security-review flow) | P077 |
| `templates/claude-settings.local.json` | ADAPTER_OWNED | permission template → `core/POLICY.md` (authority/scope) | P077 |
| `.claude/agents/**` (5 symlink) | GENERATED | registration của `agents/*.md` | P077 install manifest |
| `.claude/skills/**` (5 symlink) | GENERATED | registration của `skills/*/SKILL.md` | P077 install manifest |
| `agents/architect.md` | TRANSITIONAL_MIXED | `core/ROLES.md#architect` | P077 render, P078 |
| `agents/worker.md` | TRANSITIONAL_MIXED | `core/ROLES.md#worker` | P077 render |
| `agents/orchestrator.md` | TRANSITIONAL_MIXED | `core/ROLES.md#orchestrator` | P077 render |
| `agents/advisory-watch.md` | TRANSITIONAL_MIXED | `core/ROLES.md#advisory_watch` | P077 render |
| `agents/boundary-check.md` | TRANSITIONAL_MIXED | `core/ROLES.md#boundary_check` | P077 render |
| `skills/{idea,retro,init,apply,forge}/SKILL.md` | TRANSITIONAL_MIXED | portable body → core skill semantics; caller/invocation = adapter part | P077 render, P078 |
| lifecycle guard scripts (list per inventory dòng 26) | TRANSITIONAL_MIXED | policy intent → `core/POLICY.md`; event binding = adapter | P076/P077 |

**Lưu ý:** Path THẬT lấy từ anchor #4-9 verify (`[needs Worker verify]`). Nếu path khác golden → DISCOVERY_REPORT, đừng bịa. Mỗi row PHẢI có core source ID (không để trống — ASSETS.md dòng 51).

### Task 3: Annotate provenance trên MIXED role/skill files

**File:** `agents/architect.md`, `agents/worker.md`, `agents/orchestrator.md`, `agents/advisory-watch.md`, `agents/boundary-check.md`, `agents/README.md`; `skills/idea/SKILL.md`, `skills/retro/SKILL.md`, `skills/init/SKILL.md`, `skills/apply/SKILL.md`, `skills/forge/SKILL.md`

**Tìm:** vị trí BODY (sau frontmatter block `---`, KHÔNG trong frontmatter) — ngay dưới dòng H1 tiêu đề hoặc cuối file.

**Thêm:** 1 marker provenance dạng HTML comment, ví dụ (agents):
```
<!-- SOS-ADAPTER-PROVENANCE: role semantics canonical → core/ROLES.md#architect; Claude frontmatter/capability = adapter-claude (adapters/claude/MAPPING.md). Physical render → P077. -->
```
Skills tương tự trỏ core skill semantics + adapter caller.

**Lưu ý — PARITY-CRITICAL (anchor #7,8,12):**
- Marker PHẢI ở BODY, **TUYỆT ĐỐI KHÔNG** thêm/sửa/xóa frontmatter key `name`/`model`/`tools`/`caller`/`background` (golden probe section 3-4 đọc đúng các field này). Sai = golden diff ≠ 0 = phiếu FAIL.
- Marker KHÔNG đổi hành vi runtime (comment inert). `.claude/agents/*.md` là symlink → tự động phản chiếu, mode `120000` không đổi.
- Sau annotate BẮT BUỘC re-run golden section 3-4 probe → diff = 0 mới pass.

### Task 4: DOCS GATE Tầng-1 doc updates

**File:** `docs/LAYERS.md`

**Tìm:** access matrix / role capability table.

**Thêm:** Note (không đổi VALUE envelope — capabilities/tools GIỮ NGUYÊN, đó là mục tiêu parity): role semantics nay có canonical source `core/ROLES.md#<role_id>`; Claude capability serialization mapping ở `adapters/claude/MAPPING.md`. Access matrix values KHÔNG đổi.

**File:** `docs/HANDOFF.md`

**Tìm:** Handoff 2 (Kiến trúc sư → Thợ) section.

**Thêm:** Note adapter boundary declared — handbook `agents/*.md` = MIXED, portable role → core/ROLES.md, Claude wiring → adapters/claude. Format handoff KHÔNG đổi.

**File:** `core/ASSETS.md`

**Tìm:** bảng "Transitional mixed assets" row `agents/**` + `skills/**` (dòng 32-33).

**Thêm:** cột/note: P076 DECLARED adapter boundary (mapping `adapters/claude/MAPPING.md`); physical render P077. (Không đổi Migration owner range.)

**File:** `CLAUDE.md`

**Tìm:** "Repo structure" tree block (mục `agents/` … `bin/`).

**Thêm:** entry `adapters/` với `└── claude/` — mô tả: "Claude adapter boundary (declarative) — MAPPING.md trỏ artifact → core source ID; physical render P077. KHÔNG runtime binary."

**Lưu ý:** **KHÔNG** sửa clause "Not a runtime binary source" / "Not a runtime binary source." contract — P077 sở hữu (`PORTABILITY_ARCHITECTURE.md` dòng 41). Chỉ thêm inventory row cho `adapters/`.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `adapters/claude/README.md` | Task 1: MỚI — boundary doc |
| `adapters/claude/MAPPING.md` | Task 2: MỚI — artifact → core source ID manifest |
| `agents/{architect,worker,orchestrator,advisory-watch,boundary-check}.md`, `agents/README.md` | Task 3: body provenance marker (frontmatter BẤT BIẾN) |
| `skills/{idea,retro,init,apply,forge}/SKILL.md` | Task 3: body provenance marker |
| `docs/LAYERS.md` | Task 4: note core source + adapter mapping (values BẤT BIẾN) |
| `docs/HANDOFF.md` | Task 4: Handoff 2 boundary note |
| `core/ASSETS.md` | Task 4: transitional-mixed note P076 declared |
| `CLAUDE.md` | Task 4: repo-tree add `adapters/` |
| `CHANGELOG.md` | Discovery gate: P076 entry |

## Files KHÔNG sửa (verify only — parity surface)

| File | Verify gì |
|------|----------|
| `.claude/settings.json` | KHÔNG đụng — golden section 1 mode `100644` + section 6 wiring bất biến. Move = P077. |
| `.claude/commands/**` | KHÔNG đụng — mode `100644` bất biến |
| `.claude/agents/**`, `.claude/skills/**` | KHÔNG đụng — symlink `120000` + target bất biến |
| `templates/claude-settings.local.json` | KHÔNG đụng (permission template) — chỉ reference trong MAPPING |
| `bootstrap/sos-rs/**`, `bin/sos.sh`, `install.sh` | KHÔNG đụng — P077 |
| `scripts/*guard*.sh`, `scripts/idea-smell.sh`, `scripts/session-start-banner.sh`, `scripts/block-unsafe-merge.sh` | KHÔNG đụng script body — chỉ reference trong MAPPING; guard behavior/exit code bất biến (golden section 10 fire-test) |
| frontmatter mọi `agents/*.md` + `skills/*/SKILL.md` | name/model/tools/caller GIỮ NGUYÊN bytewise |

---

## Luật chơi (Constraints)

1. **Zero physical move.** Không file nào rời vị trí. Boundary thuần declarative. (Physical move = P077.)
2. **Zero renderer.** Không viết bất kỳ render/generate logic nào (Bash/Rust/script). Vi phạm = out-of-scope P077.
3. **Golden parity = acceptance oracle.** Sau EXECUTE, golden section 1-9 probe re-run + section 10 fire-test → diff = 0 HOẶC chỉ các thay đổi liệt kê tường minh dưới. Không diff-0 = phiếu CHƯA XONG.
4. **Chỉ thay đổi có chủ đích được phép** (không tính vào "regression"): thêm file mới `adapters/claude/**`; thêm body provenance marker (inert comment) vào MIXED files; doc updates Task 4. TẤT CẢ các thứ này KHÔNG được đổi output golden probe section 1-9 (probe không đọc body agents/skills, không đọc adapters/, chỉ đọc frontmatter field + .claude tree + settings + CLI surface + MCP + doctor).
5. **Frontmatter bất biến.** Golden section 3-4 field (name/model/tools/caller) diff = 0 bắt buộc.
6. **KHÔNG đụng CLAUDE.md runtime-source contract** — P077 owns.
7. **Core KHÔNG import adapter.** MAPPING trỏ một chiều adapter→core; không thêm reference core→adapters.
8. **Scope gate:** CHỈ Claude. KHÔNG Codex, KHÔNG Rust crate, KHÔNG npm.

---

## Nghiệm thu

### Automated / Oracle (chính)
- [ ] **Golden parity — section 1** (tracked `.claude` tree + git modes): re-run `git ls-files --stage .claude/` → diff vs `docs/golden/P076-claude-baseline.md` section 1 = **0**.
- [ ] **section 2** (symlink topology): re-run symlink probe → diff = 0.
- [ ] **section 3** (role frontmatter name/model/tools) → diff = 0. ← Task 3 parity-critical.
- [ ] **section 4** (skills name/caller) → diff = 0. ← Task 3 parity-critical.
- [ ] **section 5** (commands) → diff = 0.
- [ ] **section 6** (settings.json hook wiring) → diff = 0.
- [ ] **section 7** (sos CLI surface) → diff = 0.
- [ ] **section 8** (MCP servers) → diff = 0.
- [ ] **section 9** (doctor connectivity WIRED J1-J6) → diff = 0.

### Manual Testing (golden section 10 fire-test — live)
- [ ] Mỗi guard (`architect-guard`, `orchestrator-guard`, `block-env-edit`, `block-unsafe-merge`, `idea-smell`) VALID + INVALID payload → exit code + allow/block bất biến.
- [ ] SessionStart banner render BACKLOG.
- [ ] `sos new` greenfield → buildable; `sos adopt` brownfield → non-clobber, hooks wired; `sos sync` giữ golden outcome.
- [ ] Agents/skills/commands discoverable trong Claude session.

### Regression
- [ ] `adapters/claude/MAPPING.md` mỗi row có core source ID hợp lệ (anchor #1,2 verify các ID tồn tại).
- [ ] Không thêm reference core→adapters (constraint 7): `grep -rn 'adapters/' core/ SOS.md` → expect none (trừ RUNTIME_BOUNDARY/PORTABILITY doc-refs đã có).

### Docs Gate
- [ ] `CHANGELOG.md` — entry P076 (adapter boundary declarative + parity oracle).
- [ ] `docs/LAYERS.md` — access-matrix note (values bất biến).
- [ ] `docs/HANDOFF.md` — Handoff 2 boundary note.
- [ ] `core/ASSETS.md` — transitional-mixed P076-declared note.
- [ ] `CLAUDE.md` — repo-tree `adapters/` row.
- [ ] Xác nhận KHÔNG sửa "Not a runtime binary source" contract.

### Discovery Report
- [ ] Write `docs/discoveries/P076.md`:
  - Anchors CORRECT/WRONG (file:line citations) — đặc biệt #7,8,9,12 (frontmatter/path/annotation-placement).
  - Golden parity result: liệt kê TỪNG section 1-9 diff = 0 hay có delta (nếu delta → giải thích tại sao có chủ đích hoặc là regression).
  - (A)-vs-(B) decision: ghi Sếp confirm (B) tại ESCALATE GATE (date).
  - Docs updated (Tầng-1 list) hoặc "None".
  - Tier escalations (none expected).
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`.
