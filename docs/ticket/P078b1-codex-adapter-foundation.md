# PHIẾU P078b1: Codex adapter foundation — crate + `detect()` + PARTIAL-declaration mechanism + CLI wire

> **Loại:** Feature
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — adapter contract surface; sai thì LAN sang b2/b3 render, engine drive, Claude symmetry. Adapter contract + core Finding-status type = AUTO Tầng 1.)
> **Lane:** Guarded
> **Ảnh hưởng:** `crates/sos-adapter-codex/**` (NEW crate), `crates/sos-core/src/adapter.rs` (additive Finding-status nếu thiếu), `crates/sos-cli/**` (composition root + `install --runtime codex`), `Cargo.toml` (workspace member), `adapters/codex/**` (NEW declarative boundary docs)
> **Dependency:** P077 DONE (a–f), P078a DONE (`core/STATE.md`). Đầu chuỗi P078b — b2 (declarative render) + b3 (enforcement + guard-rewrite) build lên foundation này.

---

## Context

### Decomposition (Architect chốt — 3 phiếu, KHÔNG 1)

P078b = build lớn nhất còn lại của sprint portability. Khác P078a (1 docs file cohesive), đây là **code across nhiều surface + 1 khối security-guard-rewrite** với risk-profile khác nhau. Repo precedent (P077 a–f, P077d1–d3) favors decompose cho Rust build lớn. Chia theo seam rủi ro:

- **P078b1 (phiếu này) — foundation:** tạo crate `sos-adapter-codex`, `impl Adapter for CodexAdapter` với `detect()` thật (structural Capabilities) + `plan/render/uninstall` minimal-honest (render thật = b2/b3), **`verify()` thiết lập cơ chế tuyên bố PARTIAL** + declare 5 known capability gaps; wire vào `sos-cli` composition root; un-stub `install --runtime codex`; declarative boundary docs `adapters/codex/{README,MAPPING,CAPABILITY}.md` (symmetric `adapters/claude/`). Oracle = compile + trait-bound + dep-direction + `install --runtime codex --dry-run` không còn error.
- **P078b2 — declarative render:** `render()` cho `AGENTS.md` + `.codex/agents/<role>.toml` (4) + `.agents/skills/<name>/SKILL.md` (4) + `.codex/config.toml`. Mỗi artifact map tới core ID (ROLES/WORKFLOW/POLICY/STATE) + inline PARTIAL marker nơi report đòi (architect envelope). Oracle structural = fresh render → valid TOML/MD parse + core-ID map + PARTIAL label.
- **P078b3 — enforcement + guard-rewrite:** `.codex/hooks.json` + `.codex/rules/<name>.rules` (Starlark) + `scripts/codex/*` guard REWRITE (parse `apply_patch` `tool_input.command`, architect read-restriction inspect shell cmd). Security-surface block — CHALLENGE riêng. Oracle structural = valid JSON/Starlark + guard fire-test.

Founder delegated sprint → Architect chốt decompose. b1 de-risk foundation trước khi 2 render phiếu song song build lên (đúng pattern P077d1 carve contract trước engine).

### Vấn đề hiện tại

`sos-adapter-codex` **CHƯA tồn tại** (`crates/README.md:41` "sos-adapter-codex not created — that's P078"; target tree `docs/PORTABILITY_ARCHITECTURE.md:33`). Hệ quả:
- `install --runtime codex` hiện **error rõ** "not yet available, P078" (`crates/README.md:64`) — engine đã sẵn sàng drive bất kỳ adapter qua trait `Adapter`, chỉ thiếu Codex impl.
- Chưa có nơi Codex adapter tuyên bố **capability gaps** mà Codex CLI 0.145.0 KHÔNG biểu diễn được (report `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:15-22`: no per-role tool allowlist → architect envelope PARTIAL; no repo slash commands; skill-level allowed-tools not mechanical; no native ticket-version approval; no Read/Glob path interception; enforcement bypassable). `core/ROLES.md:120` separation-invariant #5: "Capability absence must be explicit; an integration cannot simulate success with prose." `core/POLICY.md:57-65` oracle vocab: SOUND/PARTIAL/MISSING. Codex adapter PHẢI report gaps visibly, KHÔNG giả SOUND.

Không có foundation này thì b2/b3 không có crate + trait impl + PARTIAL-mechanism để render lên.

### Giải pháp

Carve foundation adapter (như P077d1 carve trait trước engine), 4 phần:

1. **Crate `sos-adapter-codex`** — deps CHỈ `sos-core` (dep-direction adapter→core one-way). `impl Adapter for CodexAdapter`.
2. **`detect()` structural** — trả `Capabilities` từ static facts Codex 0.145.0 (report). KHÔNG cần runtime-probe codex để oracle pass (behavioral = P079). Nếu detect() shell `codex --version` để enrich → runtime path đó `[needs Worker verify]` + phải fail-safe (không có codex → trả Capabilities mặc-định-known, KHÔNG panic), vì oracle P078b chạy trên máy KHÔNG cần codex.
3. **`verify()` = cơ chế tuyên bố PARTIAL (doctrine-critical).** `verify()` trả `Findings` gồm 5 known gaps, mỗi `Finding` mang status theo oracle vocab **SOUND/PARTIAL/MISSING** (`core/POLICY.md:57-65`). Nếu type `Finding` chưa có field status enum → thêm (additive, neutral vocab) vào `crates/sos-core/src/adapter.rs`. Đây là **machine surface** của "capability absence explicit". `plan/render/uninstall` = minimal-honest stub (render bytes thật = b2/b3; KHÔNG giả render).
4. **Wire + un-stub** — thêm `sos-adapter-codex` vào `sos-cli` composition root; `install --runtime codex` construct `CodexAdapter` drive engine (dry-run chạy dù plan tối thiểu, KHÔNG còn error string).

**PARTIAL mechanism — Architect chốt (Decision 3) cả 3 surface:**
- **Machine:** `CodexAdapter::verify()` → `Findings` với status SOUND/PARTIAL/MISSING (b1 này).
- **Human frozen doc:** `adapters/codex/CAPABILITY.md` — liệt kê 5 gaps + threat/backstop (Git/CI), seeded từ verify() (b1 này).
- **In-artifact:** inline provenance marker trong artifact render (vd `architect.toml` comment "envelope PARTIAL: PreToolUse-hook + prose, KHÔNG structural tool-removal") — populate khi render (b2/b3).

Ba surface = 1 tuyên bố, KHÔNG mô phỏng SOUND. `verify()` là single source; CAPABILITY.md + inline marker derive từ nó.

**Claude symmetry — Architect chốt (Decision 2): CHỈ Codex.** `ClaudeAdapter::render()` giữ stub (Claude `.claude/**` đã committed + install qua generated-symlink manifest P076/P077, hoạt động). Flesh-out `ClaudeAdapter::render()` symmetric = refactor riêng với golden-parity oracle (`docs/golden/P076-claude-baseline.md`) — follow-up phiếu (đề xuất P078c), KHÔNG scope P078b. **Ràng buộc cứng:** mọi shared type (`ManagedOperation`, `Capabilities`, `Finding`) + render pattern b1 thiết lập KHÔNG được bake Codex-only assumption → phải áp được cho ClaudeAdapter sau. Đó là lý do Finding-status enum (nếu thêm) phải neutral.

**Oracle boundary — Architect chốt (Decision 5): STRUCTURAL, KHÔNG behavioral.** P078b oracle = crate compile + trait implementable + `install --runtime codex --dry-run` exit-0-non-error + verify() trả đúng 5-gap Findings. **Behavioral (Codex CLI chạy đúng adapter output qua 1 phiếu thật) = P079, KHÔNG P078b.** Structural oracle chạy được trên máy KHÔNG cài codex → P078b executable độc lập. Nêu rõ ranh giới này trong Nghiệm thu.

### Scope
- CHỈ sửa: tạo `crates/sos-adapter-codex/**`; `crates/sos-core/src/adapter.rs` (additive Finding-status enum NẾU thiếu — KHÔNG đổi 5-method trait, KHÔNG đổi ManagedManifest 6-field); `crates/sos-cli/**` (composition root dep + `install --runtime codex` construct); workspace `Cargo.toml` (member); tạo `adapters/codex/{README,MAPPING,CAPABILITY}.md`.
- KHÔNG sửa: `bin/sos.sh`, `install.sh`, `crates/sos-adapter-claude/**` (Claude render deferred — chỉ được đụng nếu additive Finding-status buộc ClaudeAdapter update để compile; nếu vậy = minimal compile-fix + note Discovery), `crates/sos-install/engine.rs` logic (engine drive qua trait, KHÔNG đổi), P077c parity fixtures, `core/**` docs semantics.
- KHÔNG render artifact Codex thật (AGENTS.md/.codex/**/scripts/codex) — đó là b2/b3.

---

## Task 0 — Verification Anchors

> Architect docs-only (không đọc được `crates/**/src`). `[verified]` = đọc từ docs/report thật; `[needs Worker verify]` = Worker grep/mở src xác nhận TRƯỚC khi impl. Cite RANGE, không count.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Trait `Adapter` sống ở `crates/sos-core/src/adapter.rs`, 5 method `detect/plan/render/verify/uninstall`, types `Capabilities`/`Plan`/`ManagedOperation`/`Asset`/`Artifact`/`Findings`/`Finding`/`RemovalPlan`/`RemovalStep` | `grep -n "trait Adapter\|fn detect\|fn plan\|fn render\|fn verify\|fn uninstall\|struct Finding\|enum " crates/sos-core/src/adapter.rs` | ⏳ `[needs Worker verify]` — strong evidence `crates/README.md:48`, `docs/PORTABILITY_ARCHITECTURE.md:48,63-69` |
| 2 | `ManagedOperation` có field `content: String` (P077d2 amendment) — render populate bytes | `grep -n "content" crates/sos-core/src/adapter.rs` | ✅ `[verified]` `crates/README.md:58` + `docs/PORTABILITY_ARCHITECTURE.md:52` (d2 status) |
| 3 | `Finding`/`Findings` shape — CÓ field status/severity biểu diễn SOUND/PARTIAL/MISSING chưa? | `grep -n "struct Finding\|enum.*Status\|Sound\|Partial\|Missing\|severity\|status" crates/sos-core/src/adapter.rs` | ⏳ `[needs Worker verify]` — nếu CHƯA có → Task 2 thêm enum neutral (vocab `core/POLICY.md:57-65`); nếu ĐÃ có → dùng lại, KHÔNG tạo trùng |
| 4 | `install --runtime codex` hiện error "not yet available, P078" tại `crates/sos-cli/src/commands/install.rs` | `grep -rn "not yet available\|codex\|P078" crates/sos-cli/src/commands/install.rs` | ⏳ `[needs Worker verify]` — `crates/README.md:64` mô tả |
| 5 | `sos-cli` composition root deps = sos-core + sos-install + sos-adapter-claude + sos-hooks (thêm sos-adapter-codex) | `grep -A15 "\[dependencies\]" crates/sos-cli/Cargo.toml` | ⏳ `[needs Worker verify]` — `crates/README.md:24` mô tả |
| 6 | dep-direction guard `crates/sos-core/tests/dep_direction.rs` scan CHỈ `sos-core/src/**` cho forbidden token `sos_adapter/sos_install/sos_hooks/sos_cli` — thêm crate adapter mới KHÔNG regress guard | `grep -n "sos_adapter\|sos-core/src\|forbidden" crates/sos-core/tests/dep_direction.rs` | ⏳ `[needs Worker verify]` — `crates/README.md:37` + `docs/ticket/P077d1...:76` mô tả |
| 7 | Workspace member list ở root `Cargo.toml` — crate mới phải thêm vào `members` | `grep -n "members\|crates/" Cargo.toml` | ⏳ `[needs Worker verify]` |
| 8 | `ClaudeAdapter` (`crates/sos-adapter-claude/src/lib.rs`) impl `Adapter` với stub bodies — pattern reference cho CodexAdapter | `grep -n "impl Adapter\|struct ClaudeAdapter\|todo\|fn detect" crates/sos-adapter-claude/src/lib.rs` | ⏳ `[needs Worker verify]` — `crates/README.md:50` mô tả stub |
| 9 | Codex 0.145.0 capability facts: no per-role built-in tool allowlist; `sandbox_mode=read-only` khả dụng (advisory/boundary OK); hooks + multi_agent stable/enabled; `apply_patch` (tool_name="apply_patch", patch ở `tool_input.command`); reads via shell KHÔNG Read/Glob | Read `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:15-22,32` | ✅ `[verified]` |
| 10 | 5 known gaps để verify() declare: (a) per-role tool allowlist NONE → architect envelope PARTIAL; (b) repo slash commands NONE; (c) skill-level allowed-tools not mechanical; (d) native ticket-version approval NONE; (e) Read/Glob path interception NONE + enforcement bypassable (untrusted-repo/user-disable) | Read report `:15-22` | ✅ `[verified]` |
| 11 | `adapters/claude/{README,MAPPING}.md` = declarative-boundary pattern để mirror sang `adapters/codex/` | Read `adapters/claude/README.md` + `adapters/claude/MAPPING.md` | ✅ `[verified]` — Architect đọc, mirror shape (mỗi row artifact→core ID, dep 1 chiều) |
| 12 | engine drive adapter THUẦN qua trait (zero host token) — CodexAdapter chỉ cần impl trait đúng, engine không cần đổi | Read `crates/README.md:56-64` (d2 engine status) | ✅ `[verified]` — engine `sos-install::engine` trait-driven, `install.rs` construct adapter |

**Anchor ❌:** không có. #1,#3,#4,#5,#6,#7,#8 = `[needs Worker verify]` (Architect docs-only). **CRITICAL #3:** quyết Finding-status có sẵn hay thêm mới — Worker grep TRƯỚC; nếu thêm mới = touch core adapter.rs (shared với ClaudeAdapter) → giữ ClaudeAdapter compile.

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

Rollback: `git reset --hard $(cat .backup/${PHIEU_ID}/main-head.txt)` (trong worktree phiếu, KHÔNG main).

---

## Debate Log

> Schema: 1 turn = Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

*(Worker CHALLENGE mode. Verified anchors #1,#3,#4,#5,#6,#7,#8 against real code.)*

**Anchor verification (recap Task 0):**
- #1 ✅ `crates/sos-core/src/adapter.rs:118-136` — trait `Adapter` 5 methods (detect/plan/render/verify/uninstall) + all 9 types present (`Capabilities:33`, `ManagedOperation:53`, `Plan:62`, `Asset:70`, `Artifact:79`, `Finding:87`, `Findings:94`, `RemovalStep:102`, `RemovalPlan:109`).
- #2 ✅ `ManagedOperation.content:56` confirmed.
- #3 ✅ CRITICAL confirmed — `struct Finding { target_path, message }` (`adapter.rs:87-90`) has **NO status field, no enum**. Task 2's plan to ADD `FindingStatus{Sound,Partial,Missing}` is correct and necessary, not redundant.
- #4 ✅ `crates/sos-cli/src/commands/install.rs:26` exact string `"codex adapter not yet available (P078) — use --runtime claude for now"`.
- #5 ✅ `crates/sos-cli/Cargo.toml` deps = clap/anyhow/sha2/walkdir/chrono + sos-core/sos-install/sos-adapter-claude/sos-hooks. Matches.
- #6 ✅ `crates/sos-core/tests/dep_direction.rs:18` `FORBIDDEN_TOKENS = ["sos_adapter","sos_install","sos_hooks","sos_cli"]`, substring-scans `sos-core/src/**` only (`:34-37`). `"sos_adapter"` substring-matches `sos_adapter_codex` too — guard already covers the new crate without changes.
- #7 ⚠️ **Correction, non-blocking** — root `Cargo.toml:4` is `members = ["crates/*"]` (a glob), NOT an explicit list. Task 1's "Lưu ý: thêm crate vào workspace members" is unnecessary — `crates/sos-adapter-codex/` is auto-included once created. No Architect action needed; Worker will just skip that sub-step and note it in Discovery (self-closed via oracle = reading the file, SOUND).
- #8 ✅ `crates/sos-adapter-claude/src/lib.rs:1-56` — exact stub pattern confirmed (Capabilities::default(), Plan::default(), Artifact passthrough, Findings::default(), RemovalPlan::default(), plus a trait-bound test at :51-55 to mirror).

**Additional structural check (not in Task 0 table):** `crates/sos-cli/src/commands/install.rs:37-79` (`run_claude`) shows `sos_install::engine::{dry_run, apply, resolve_tools}` take `&Plan`/`project_root` only — zero `Adapter`-typed parameter into the engine. Confirms anchor #12: engine is plan-driven, not adapter-coupled — `run_codex` can mirror `run_claude` 1:1 with zero engine changes.

**Objections (Tầng 1 only):** None. Anchor #7 is a Tầng 2 factual correction with a sound oracle (reading `Cargo.toml`), self-closed here — does not require Architect response or phiếu scope change.

**Status:** ✅ ACCEPTED — no Tầng 1 objections. Ready for Chủ nhà approval gate.

### Final consensus
- Phiếu version: V1 (no amendments needed)
- Approved by Chủ nhà: 2026-07-22 (delegated sprint, CHALLENGE APPROVE V1 → EXECUTE)

---

## Nhiệm vụ

### Task 1: Tạo crate `sos-adapter-codex`

**File:** `crates/sos-adapter-codex/Cargo.toml` (NEW) + `crates/sos-adapter-codex/src/lib.rs` (NEW)

**Thêm:**
- `Cargo.toml`: package `sos-adapter-codex`, deps CHỈ `sos-core = { path = "../sos-core" }` (+ serde/anyhow nếu cần, workspace-inherited). **KHÔNG** dep `sos-install`/`sos-hooks`/`sos-cli` (dep-direction).
- `src/lib.rs`: `pub struct CodexAdapter { … }` + `impl Adapter for CodexAdapter` — mirror stub pattern của `ClaudeAdapter` (anchor #8, `[needs Worker verify]` exact signatures).

**Lưu ý:** thêm crate vào workspace `members` (`Cargo.toml` root, anchor #7). Dep-direction guard (anchor #6) scan CHỈ `sos-core/src` → crate mới không regress miễn `sos-core` không import `sos_adapter_codex`.

### Task 2: `Finding` status enum (additive NẾU thiếu) — cơ chế PARTIAL machine surface

**File:** `crates/sos-core/src/adapter.rs`

**Tìm:** định nghĩa `struct Finding` / `struct Findings` (anchor #3).

**Thêm bằng:** NẾU `Finding` CHƯA mang status → thêm neutral enum:
```
pub enum FindingStatus { Sound, Partial, Missing }
```
+ field `status: FindingStatus` trên `Finding`. Vocab khớp `core/POLICY.md:57-65` (SOUND: oracle đóng claim; PARTIAL: chỉ cover named dimension, residual explicit; MISSING: no oracle). NẾU đã có enum tương đương → dùng lại, KHÔNG tạo trùng (ghi Discovery tên thật).

**Lưu ý:**
- Enum PHẢI neutral (KHÔNG token host) — nó ở `sos-core`, dep-direction guard forbidden-token scan enforce.
- Thêm field = touch shared type → `ClaudeAdapter` (dùng `Finding`?) có thể phải update để compile. Nếu vậy: minimal compile-fix (default `status`), note Discovery. KHÔNG flesh-out Claude render.
- Đây là mục backward-symmetry rủi ro nhất — giữ `Finding` áp được cho CẢ 2 adapter (Decision 2 ràng buộc).

### Task 3: `CodexAdapter::detect()` — structural Capabilities

**File:** `crates/sos-adapter-codex/src/lib.rs`

**Thêm:** `detect()` trả `Capabilities` phản ánh Codex 0.145.0 facts (anchor #9): per-role tool-allowlist = absent; sandbox read-only = available; hooks + multi_agent = available; apply_patch model; shell-based reads.

**Lưu ý:**
- Structural oracle KHÔNG đòi runtime-probe. NẾU detect() shell `codex --version` để enrich → path đó `[needs Worker verify]` + **fail-safe**: codex vắng mặt → trả Capabilities known-default (KHÔNG panic, KHÔNG error) vì oracle chạy trên máy không cài codex (Decision 5). Runtime-probe verify thật = P079.
- Capabilities shape = `[needs Worker verify]` (đọc type ở adapter.rs). Giữ neutral fields đã có; KHÔNG thêm Codex-only field vào core type.

### Task 4: `CodexAdapter::verify()` — declare 5 known gaps (PARTIAL/MISSING)

**File:** `crates/sos-adapter-codex/src/lib.rs`

**Thêm:** `verify()` trả `Findings` gồm 5 gap (anchor #10), mỗi Finding status theo Task 2 enum:
- architect envelope: **PARTIAL** — no per-role built-in tool allowlist; enforced via PreToolUse-hook + prose + sandbox read-only, KHÔNG structural tool-removal như Claude.
- repo-distributed slash commands: **MISSING** — replacement = repo skill `$name`.
- skill-level allowed-tools: **PARTIAL** — not mechanical.
- native ticket-version approval: **MISSING** — build via persisted approved-version + PreToolUse guard (b3).
- Read/Glob path interception: **MISSING** — Codex reads via shell; architect read-restriction = inspect shell cmd (b3). + enforcement-weakness: **PARTIAL** — config ignored nếu repo untrusted / user disable → retain Git/CI backstop.

**Lưu ý:** verify() = single source of PARTIAL truth (`core/ROLES.md:120` sep-inv #5). Text mỗi Finding ngắn + cite report. KHÔNG giả SOUND. `plan/render/uninstall` = minimal-honest stub (`todo!`/empty Plan) — render bytes thật b2/b3; KHÔNG giả render (Existence ≠ capability, `core/POLICY.md:65`).

### Task 5: Wire `sos-cli` composition root + un-stub `install --runtime codex`

**File:** `crates/sos-cli/Cargo.toml` + `crates/sos-cli/src/commands/install.rs`

**Tìm:** (a) `[dependencies]` block (anchor #5); (b) nhánh `codex =>` error "not yet available, P078" (anchor #4).

**Thay bằng:** (a) thêm `sos-adapter-codex = { path = "../sos-adapter-codex" }`; (b) construct `CodexAdapter` + drive engine y như nhánh `claude` (engine trait-driven, anchor #12) — `--dry-run` phải chạy exit-0-non-error dù plan tối thiểu.

**Lưu ý:** engine (`sos-install`) KHÔNG đổi — nó drive bất kỳ `impl Adapter`. `install.sh`/`bin/sos.sh` zero-touch (additive). `--runtime auto` detect logic: nếu auto hiện chỉ chọn claude → để nguyên (Codex auto-detect = khi detect() runtime-probe thật, P079); b1 chỉ cần `--runtime codex` explicit chạy.

### Task 6: Declarative boundary docs `adapters/codex/`

**File:** `adapters/codex/README.md` + `adapters/codex/MAPPING.md` + `adapters/codex/CAPABILITY.md` (NEW, mirror `adapters/claude/`)

**Thêm:**
- `README.md`: adapter Codex owns serialized representations (`AGENTS.md`, `.codex/**`, scripts/codex guards) → core source ID; dep-direction adapter→core one-way; note render vật lý b2/b3 (declarative-first như P076).
- `MAPPING.md`: bảng artifact → core ID (rows sẽ fill khi render b2/b3; b1 seed shape + "Physical render" column = P078b2/b3). Mỗi future artifact có non-empty core source ID (`core/ASSETS.md:51`).
- `CAPABILITY.md`: **frozen human-readable 5-gap declaration** seeded từ verify() (Task 4) — mỗi gap: what/why-PARTIAL-or-MISSING/backstop (Git+CI). Đây là human surface của "capability absence explicit"; verify() là machine source.

**Lưu ý:** KHÔNG `core/** → adapters/**` reference (dep 1 chiều — regression check). CAPABILITY.md nội dung derive từ report `:15-22` + Task 4 verify().

### Task 7: Docs Gate updates

**File:** `docs/PORTABILITY_ARCHITECTURE.md` — thêm "P078b1 status" line (như P077 status pattern §43-56): crate `sos-adapter-codex` created, detect/verify live, PARTIAL-declaration mechanism established, render deferred b2/b3, `install --runtime codex` un-stubbed. Cập nhật migration table §160 row P078 nếu cần (in-progress).

**File:** `crates/README.md` — Module layout: `sos-adapter-codex` KHÔNG còn "not created" (line 41); thêm subsection "Codex adapter foundation (P078b1)" mô tả crate + PARTIAL mechanism (như d1/d2/d3 subsections).

**File:** `core/ASSETS.md` — adapter-owned classes (§40-51): confirm Codex serialized representations covered (README hiện ghi Claude; thêm note Codex adapter-owned = `AGENTS.md`/`.codex/**`/scripts/codex). NẾU §32-33 (agents/skills migration owner "P076-P078") cần bump status → cập nhật, else N/A explicit.

**File:** `CHANGELOG.md` — entry P078b1.

**Lưu ý:** `core/**` docs semantics KHÔNG đổi (STATE/ROLES/POLICY/WORKFLOW content untouched — chỉ ASSETS.md adapter-owned note). Nếu Worker thấy `docs/BACKLOG.md` P078 item nên tick-partial → guidance, KHÔNG bắt buộc (curation, single-role).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-adapter-codex/Cargo.toml` | NEW — Task 1 |
| `crates/sos-adapter-codex/src/lib.rs` | NEW — Task 1,3,4: CodexAdapter + detect + verify + stub plan/render/uninstall |
| `crates/sos-core/src/adapter.rs` | Task 2: additive `FindingStatus` enum (NẾU thiếu) |
| `Cargo.toml` (root workspace) | Task 1: thêm member |
| `crates/sos-cli/Cargo.toml` | Task 5: dep sos-adapter-codex |
| `crates/sos-cli/src/commands/install.rs` | Task 5: construct CodexAdapter, un-stub codex nhánh |
| `crates/sos-adapter-claude/src/lib.rs` | Task 2 ONLY-IF: minimal compile-fix nếu Finding-status buộc (KHÔNG render) |
| `adapters/codex/README.md` | NEW — Task 6 |
| `adapters/codex/MAPPING.md` | NEW — Task 6 (seed shape) |
| `adapters/codex/CAPABILITY.md` | NEW — Task 6 (5-gap frozen) |
| `docs/PORTABILITY_ARCHITECTURE.md` | Task 7: P078b1 status |
| `crates/README.md` | Task 7: Codex foundation subsection |
| `core/ASSETS.md` | Task 7: Codex adapter-owned note |
| `CHANGELOG.md` | Task 7: entry |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `crates/sos-install/src/engine.rs` | Engine drive CodexAdapter qua trait KHÔNG đổi (trait-driven, anchor #12) |
| `crates/sos-core/tests/dep_direction.rs` | Guard XANH sau thêm crate + enum (core zero adapter import) |
| `bin/sos.sh`, `install.sh` | Zero-touch (additive) |
| `crates/sos-adapter-claude/src/lib.rs` | Claude render vẫn stub (Decision 2) — chỉ đụng nếu Task 2 buộc compile-fix |
| P077c parity fixtures (`crates/sos-cli/tests/**`) | Không regress |
| `core/{STATE,ROLES,POLICY,WORKFLOW}.md` | Semantics referenced đúng, KHÔNG sửa content |

---

## Luật chơi (Constraints)

1. **Additive** — `install.sh`/`bin/sos.sh` zero-touch; `sos-adapter-codex` crate mới; `install --runtime codex` un-stub (từ error → chạy). KHÔNG đổi engine logic.
2. **Dep-direction bất di** — `sos-adapter-codex` deps CHỈ `sos-core`; core zero adapter import. Guard `dep_direction.rs` phải XANH.
3. **Core zero runtime token** (`core/POLICY.md` portable-core) — `FindingStatus` enum + bất kỳ type thêm vào `sos-core` PHẢI neutral (không `.codex`/`AGENTS`/`CLAUDE_*`). Codex token CHỈ trong `sos-adapter-codex`.
4. **Symmetric shared types (Decision 2)** — `Finding`/`Capabilities`/`ManagedOperation` giữ áp được cho CẢ Claude + Codex. Codex-only field KHÔNG được vào core type.
5. **PARTIAL, KHÔNG giả SOUND** (`core/ROLES.md:120` sep-inv #5, `core/POLICY.md:65`) — verify() declare 5 gap thật; plan/render stub = honest-empty, KHÔNG mô phỏng render. Existence ≠ capability.
6. **Structural oracle boundary (Decision 5)** — b1 oracle chạy KHÔNG cần codex cài. Behavioral (codex chạy đúng) = P079. detect() runtime-probe fail-safe.
7. **Claude render deferred (Decision 2)** — KHÔNG flesh-out `ClaudeAdapter::render()` (follow-up P078c).
8. **Cite RANGE** khi reference doc/code (vd `core/POLICY.md:57-65`), KHÔNG count.

---

## Nghiệm thu

### Automated (oracle STRUCTURAL — Decision 5)
- [x] `cargo build --workspace` xanh (crate mới compile, ClaudeAdapter vẫn compile sau Finding-status). `[oracle: cargo build --workspace SOUND]`
- [x] `cargo test --workspace` xanh incl. dep_direction guard + trait-bound test mới. `[oracle: cargo test SOUND]`
- [x] `cargo test --workspace` ×20 = 0 flaky. `[oracle: ×20 flaky-check SOUND]`
- [x] ⚠️ PARTIAL — `sos install --runtime codex --dry-run` KHÔNG còn "not yet available" (grep-confirmed 0 hit) và reach ĐÚNG code path như `--runtime claude`, nhưng full exit-0 trên dev machine này bị chặn bởi tool-manifest pin drift KHÔNG LIÊN QUAN (P077d3 step-5 gate) — GIỐNG HỆT xảy ra với `--runtime claude` (byte-identical output verified), pre-existing, KHÔNG phải regression từ ticket này. `[oracle: dry-run structural — PARTIAL, env-blocked]`
- [x] Unit test: `CodexAdapter::verify()` trả đúng 5 Findings, status PARTIAL/MISSING khớp anchor #10 (test `verify_reports_exactly_five_gaps_none_sound` asserts count==5 + never Sound). `[oracle: verify-gap-count structural SOUND]`
- [x] dep-direction guard xanh (`grep -rn "sos_adapter_codex" crates/sos-core/src/` → zero). `[oracle: dep-direction SOUND]`

**Ranh giới oracle (in rõ):** P078b1 KHÔNG verify Codex CLI chạy đúng output — đó là behavioral, P079. b1 verify = crate implementable + Findings đúng + engine drive được + valid Rust. Structural-vs-behavioral boundary Decision 5.

### Manual Testing
- [x] `adapters/codex/CAPABILITY.md` đọc độc lập: 5 gap + backstop rõ ràng (what/why-status/backstop mỗi mục).
- [x] `verify()` Findings text ↔ CAPABILITY.md ↔ report `:15-22` khớp (1 tuyên bố 3 surface).

### Regression
- [x] `install --runtime claude --dry-run` vẫn chạy như trước (byte-identical tool-gate output before/after — ClaudeAdapter không regress bởi Finding-status).
- [x] P077c parity fixtures xanh (8/8 `parity.rs` tests pass).
- [x] `grep -rn 'adapters/' core/` → zero (dep 1 chiều) — **fixed mid-EXECUTE**: an early draft of the `core/ASSETS.md` note referenced `adapters/codex/...` path literally, caught by this exact regression check, rewritten to describe classes without a `adapters/` path reference.

### Docs Gate (Tầng 1)
- [x] `docs/PORTABILITY_ARCHITECTURE.md` — P078b1 status line + migration table row
- [x] `crates/README.md` — Codex foundation subsection (line 41 "not created" gỡ)
- [x] `core/ASSETS.md` — Codex adapter-owned note
- [x] `CHANGELOG.md` — entry P078b1

### Discovery Report
- [ ] Write `docs/discoveries/P078b1.md`:
  - Anchors #1,#3,#4,#5,#6,#7,#8 — CORRECT/WRONG (file:line). ĐẶC BIỆT #3: Finding-status có sẵn hay thêm? (tên enum thật + ClaudeAdapter có phải compile-fix không)
  - CodexAdapter type/method exact names chốt
  - detect() runtime-probe: shell codex hay pure-static? fail-safe path
  - PARTIAL mechanism: 5 gap declared đúng chưa; verify()↔CAPABILITY.md sync
  - **Symmetry flag:** mọi chỗ shared type suýt bake Codex-only (cho P078c Claude-render reconcile)
  - **Cross-ref flag (founder-facing):** P078b thêm product-source crate `sos-adapter-codex` → BACKLOG line 23 open doctrine question (orchestrator-guard gate `crates/**/src` via worker-marker?) nay có surface mới; note cho Chủ nhà quyết
  - Docs updated (list) / Tier escalations ("None" nếu không)
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`
