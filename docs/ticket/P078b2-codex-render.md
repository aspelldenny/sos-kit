# PHIẾU P078b2: Codex adapter declarative render — `render()` sinh Codex-native artifacts (AGENTS.md + .codex/agents/*.toml + .agents/skills + .codex/config.toml)

> **Loại:** Feature
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — adapter render contract; artifact sai LAN sang b3 enforcement + P079 behavioral dogfood + Codex-Claude symmetry. Adapter-owned serialized-representation surface + provenance-map tới core ID = AUTO Tầng 1 dù phần lớn là string-format.)
> **Lane:** Guarded
> **Ảnh hưởng:** `crates/sos-adapter-codex/src/**` (render logic + template module + tests), `adapters/codex/MAPPING.md` (fill b2 render rows + reconcile skill set), `adapters/codex/README.md` (status line), `docs/PORTABILITY_ARCHITECTURE.md` (P078b2 status), `CHANGELOG.md`
> **Dependency:** P078b1 DONE (crate + trait impl + `detect()`/`verify()` + PARTIAL mechanism + `install --runtime codex` wired). Song song với P078b3 (enforcement) — b2 KHÔNG đụng hooks.json/rules/scripts.

---

## Context

### Vấn đề hiện tại

P078b1 dựng foundation: `CodexAdapter` implement trait `Adapter`, `detect()`/`verify()` live, nhưng `plan()`/`render()`/`uninstall()` = **minimal-honest stub** — chưa sinh byte artifact nào (`adapters/codex/README.md:57-59`, `docs/PORTABILITY_ARCHITECTURE.md:58` "render bytes thật = P078b2/b3"). Hệ quả: `install --runtime codex --dry-run` reach đúng engine code path nhưng plan rỗng → KHÔNG có Codex-native surface để Codex CLI đọc.

b2 = **declarative render** (KHÔNG enforcement): sinh 10 artifact declarative Codex-native, mỗi cái map tới stable core ID (KHÔNG duplicate semantics), PARTIAL honest nơi report đòi. Enforcement (hooks.json/rules/scripts guard-rewrite) = b3. Behavioral (Codex CLI thật chạy đúng output) = P079. b2 oracle = **STRUCTURAL** (fresh render → files valid + khớp report format + trỏ đúng core ID + PARTIAL-honest), chạy KHÔNG cần Codex cài.

### Giải pháp

Impl `CodexAdapter::render()` (+ wire `plan()`) sinh 10 declarative artifact:

| # | Artifact | Nội dung | Core ID pointer |
|---|----------|---------|-----------------|
| 1 | `AGENTS.md` (root) | orchestrator contract | `core/ROLES.md#orchestrator` + `core/WORKFLOW.md` |
| 2 | `.codex/agents/architect.toml` | subagent — sandbox workspace-write + **PARTIAL marker** | `core/ROLES.md#architect` |
| 3 | `.codex/agents/worker.toml` | subagent — sandbox workspace-write | `core/ROLES.md#worker` |
| 4 | `.codex/agents/advisory-watch.toml` | subagent — sandbox **read-only** (honest structural) | `core/ROLES.md#advisory_watch` |
| 5 | `.codex/agents/boundary-check.toml` | subagent — sandbox **read-only** (honest structural) | `core/ROLES.md#boundary_check` |
| 6 | `.agents/skills/idea/SKILL.md` | frontmatter name+description + body pointer | core skill semantics + `core/WORKFLOW.md` |
| 7 | `.agents/skills/forge/SKILL.md` | idem | idem |
| 8 | `.agents/skills/apply/SKILL.md` | idem | idem |
| 9 | `.agents/skills/retro/SKILL.md` | idem | idem |
| 10 | `.codex/config.toml` | `[mcp_servers.doctor]` + `[agents]` + sandbox/approval baseline | `core/POLICY.md` (authority/scope) + `core/ASSETS.md` (MCP registration) |

**Decision 1 — Content-source: crate-embedded template/format-string (KHÔNG đọc `core/**` lúc render, KHÔNG hardcode-mù).** Content sinh từ **template string trong crate** (`sos-adapter-codex`), field map từ core role ID pointer. Lý do: (a) Codex report format ổn định (`docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:6-13`) → format-string đúng khuôn; (b) render KHÔNG được đọc filesystem `core/**` lúc chạy (tạo runtime file-read coupling + không chạy được khi render tới target project khác cây); (c) template chỉ chứa **pointer** (`core/ROLES.md#architect`) + binding tối thiểu, **KHÔNG copy role semantics** — giống Claude adapter P076 provenance marker (`adapters/claude/README.md:28` inert HTML-comment trỏ `core/ROLES.md#<role_id>`). "Owner duy nhất" của semantics vẫn ở `core/**`; artifact là serialized pointer.

**Decision 2 — Map stable core ID, KHÔNG duplicate.** Mỗi `.codex/agents/*.toml` `developer_instructions` trỏ `core/ROLES.md#<role_id>` (6 ID chuẩn `core/ROLES.md:23-112`: owner/orchestrator/architect/worker/advisory_watch/boundary_check). AGENTS.md trỏ `core/{ROLES,WORKFLOW,POLICY,STATE}`. Skills trỏ core skill semantics + `core/WORKFLOW.md`. Render **pointer**, KHÔNG copy văn bản vai (`core/ROLES.md:120` sep-inv #5 + `core/ASSETS.md:51` "must not become the only copy of a semantic rule"). Mirror `adapters/codex/MAPPING.md:8-20` bảng đã seed b1.

**Decision 3 — PARTIAL honest TRONG render (in-artifact surface, surface #3 của 3-surface PARTIAL mechanism b1).** `.codex/agents/architect.toml`: `sandbox_mode=workspace-write` **KHÔNG** enforce "no Bash/Grep/Edit" (Codex 0.145.0 không có per-role tool allowlist — `adapters/codex/CAPABILITY.md:12-24` gap #1 PARTIAL). Render PHẢI note (TOML comment + trong `developer_instructions`) rằng envelope enforce qua PreToolUse hook (P078b3) + prose, **KHÔNG structural tool-removal như Claude**. KHÔNG render cái giả "architect tool-scoped" khi Codex không làm được (`core/POLICY.md:65` "Existence is not capability"). advisory-watch/boundary-check `sandbox_mode=read-only` = honest structural (đủ cho read-only specialist, `CAPABILITY.md:15-17`). Marker text = derive từ `verify()`/`CAPABILITY.md`, 1 tuyên bố nhất quán 3 surface.

**Decision 4 — Skill set = 4 (idea/forge/apply/retro), KHÔNG init.** Delegated scope chốt 4. `adapters/codex/MAPPING.md:15-19` (seed b1) liệt 5 skill **có init** (mirror Claude 5-LIVING). Reconcile: b2 render 4; annotate MAPPING init row (`:17`) = "deferred — Claude caller = `sos init` CLI, name-collision review pending; KHÔNG render b2". KHÔNG để MAPPING claim init rendered khi không (single-source drift — đúng bệnh retro v2.3). Nếu Chủ nhà muốn 5 → escape hatch bên dưới.

**Decision 5 — Oracle STRUCTURAL, KHÔNG behavioral.** b2 oracle = fresh render → (a) TOML parse OK (`.codex/agents/*.toml` + `config.toml`); (b) MD well-formed + YAML frontmatter valid (`SKILL.md` × 4 + AGENTS.md); (c) mỗi artifact chứa đúng core-ID pointer; (d) PARTIAL marker hiện diện trên architect.toml + read-only trên advisory/boundary; (e) khớp Codex report field shape per surface. Chạy trên máy KHÔNG cài codex (in-memory Artifact / temp dir, KHÔNG commit `.codex/` vào sos-kit). Behavioral (Codex CLI đọc chạy đúng) = P079.

**Decision 6 — Additive, render tới TARGET project.** `render()` OUTPUT khi `install --runtime codex` (tới project đích); **KHÔNG commit `.codex/`/`AGENTS.md`/`.agents/` vào chính sos-kit** (sos-kit chạy Claude). Test assert trên Artifact in-memory / temp dir. `install.sh`/`bin/sos.sh`/`sos-install/engine.rs` zero-touch (engine drive qua Plan/trait, b1 anchor #12 confirmed).

### Scope
- CHỈ sửa: `crates/sos-adapter-codex/src/**` (render + template module + unit tests; có thể tách `src/templates.rs` hoặc `src/render.rs` mới trong crate — Worker chốt module layout); `adapters/codex/MAPPING.md` (fill b2 rows + reconcile init); `adapters/codex/README.md` (status line b2); `docs/PORTABILITY_ARCHITECTURE.md` (P078b2 status); `CHANGELOG.md`.
- KHÔNG sửa: `crates/sos-core/src/**` (trait/type b1 đủ — Asset/Artifact/ManagedOperation.content sẵn; NẾU render buộc thêm field core type → STOP, escalate, KHÔNG tự thêm), `crates/sos-install/src/engine.rs` (engine drive qua Plan, KHÔNG đổi), `crates/sos-adapter-claude/**` (Claude render deferred P078c), `bin/sos.sh`/`install.sh` (additive), `core/**` docs semantics, `.codex/hooks.json`/`.codex/rules`/`scripts/codex` (= b3).
- KHÔNG commit artifact render (`.codex/`, `AGENTS.md`, `.agents/`) vào sos-kit repo (Decision 6).

---

## Task 0 — Verification Anchors

> Architect docs-only (KHÔNG đọc được `crates/**/src`). `[verified]` = đọc từ docs/report/discovery thật; `[needs Worker verify]` = Worker grep/mở src xác nhận TRƯỚC khi impl. Cite RANGE, KHÔNG count.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Trait `Adapter::render()` signature: nhận `Asset` → trả `Artifact` (hoặc `Result<Artifact>`); `Asset` @ `adapter.rs:70`, `Artifact` @ `:79` với field `target_path` + `content` | `grep -n "fn render\|struct Asset\|struct Artifact" crates/sos-core/src/adapter.rs` | ⏳ `[needs Worker verify]` — b1 CHALLENGE recap `docs/ticket/P078b1...:104-106` (`Asset:70`,`Artifact:79`); exact field names + có `Result` không = Worker chốt |
| 2 | `CodexAdapter::render()` hiện = stub passthrough (b1); wire lại để sinh artifact thật | `grep -n "fn render\|fn plan" crates/sos-adapter-codex/src/lib.rs` | ⏳ `[needs Worker verify]` — `docs/discoveries/P078b1.md:42` "render() stub uses exact same passthrough shape as ClaudeAdapter" |
| 3 | Quan hệ `plan()`↔`render()`↔engine: engine nhận `&Plan` (`ManagedOperation` với `content`); plan() enumerate Asset → render() fill content HAY engine gọi render() per-asset? | `grep -n "fn plan\|ManagedOperation\|render\|resolve_tools\|dry_run" crates/sos-install/src/engine.rs crates/sos-adapter-codex/src/lib.rs` | ⏳ `[needs Worker verify]` — b1 discovery `:50` engine=`resolve_tools()→dry_run()` nhận `&Plan`/`project_root` KHÔNG `Adapter`-typed → plan() phải sinh ManagedOperation.content; wiring chính xác = escape hatch |
| 4 | `ManagedOperation` có `content: String` + `target_path` (P077d2) — đủ chở render bytes | `grep -n "struct ManagedOperation\|content\|target_path\|path" crates/sos-core/src/adapter.rs` | ✅ `[verified]` `docs/PORTABILITY_ARCHITECTURE.md:52` (d2 amendment) + b1 anchor #2 |
| 5 | Codex report format per surface: AGENTS.md (contract, 32KiB, concat root→leaf); `.codex/agents/*.toml` required `name`+`description`+`developer_instructions`, override `model`/`sandbox_mode`/`mcp_servers`/`skills.config`; `SKILL.md` frontmatter `name`+`description` (chỉ 2 field mechanical); `config.toml` `[mcp_servers.<name>]` cmd/args/enabled_tools + `[agents] enabled` + `sandbox_mode`/`approval_policy` | Read `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:6-13` + `adapters/codex/README.md:14-27` | ✅ `[verified]` |
| 6 | 6 core role ID chuẩn: owner/orchestrator/architect/worker/advisory_watch/boundary_check | Read `core/ROLES.md:23-112` | ✅ `[verified]` |
| 7 | `adapters/codex/MAPPING.md:8-20` đã seed 13 row (10 b2-artifact + 3 b3); skill rows `:15-19` = **5** (có init) | Read `adapters/codex/MAPPING.md:8-20` | ✅ `[verified]` — init discrepancy vs scope-4 → Decision 4 |
| 8 | `verify()` 5-gap; architect envelope = PARTIAL (workspace-write KHÔNG structural tool-removal); advisory/boundary read-only OK | Read `adapters/codex/CAPABILITY.md:12-24,58-75` + `docs/discoveries/P078b1.md:30-34` | ✅ `[verified]` |
| 9 | `ClaudeAdapter::render()` vẫn stub (KHÔNG có reference render impl để mirror symmetric; b2 render Codex = genuinely new) | `grep -n "fn render" crates/sos-adapter-claude/src/lib.rs` | ⏳ `[needs Worker verify]` — `docs/discoveries/P078b1.md:40-42` mô tả stub |
| 10 | sandbox_mode value hợp lệ: `read-only` (advisory/boundary), `workspace-write` (architect/worker); advisory_watch cần network → read-only sandbox + network qua approval_policy/config? | Read report `:16` + `crates/sos-adapter-codex/**` nếu detect() đã encode; verify Codex read-only cho outbound | ⏳ `[needs Worker verify]` — escape hatch nếu read-only chặn network cho advisory-watch |
| 11 | Doctor MCP server config để mirror vào `config.toml [mcp_servers.doctor]` (command/args/enabled_tools) = PATH-relative `doctor` (B1 privacy) | Read `.mcp.json` (root) doctor server block | ⏳ `[needs Worker verify]` — mirror shape, PATH-relative giữ (KHÔNG absolute path) |
| 12 | Skill body source: `skills/{idea,forge,apply,retro}/SKILL.md` (Claude) = portable body để pointer, `caller:` frontmatter = adapter part | Read `adapters/claude/MAPPING.md:20-24` + `skills/*/SKILL.md` frontmatter | ✅ `[verified]` — render frontmatter `name`+`description` + body trỏ core, KHÔNG copy toàn văn |

**Anchor ❌:** không có. `[needs Worker verify]` = #1,#2,#3,#9,#10,#11 (Architect docs-only). **CRITICAL #1+#3:** quan hệ Asset/Artifact/plan/render/engine quyết render mechanism — Worker grep TRƯỚC khi impl; nếu trait shape KHÔNG cho per-asset render như Decision 1 mô tả → escape hatch (STOP + Discovery + đề xuất, KHÔNG tự đổi core trait).

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
**Worker accepted V1 — no challenges.** Anchor verification:
- #1 ✅ `Adapter::render(asset:&Asset,capabilities:&Capabilities)->Artifact` (`crates/sos-core/src/adapter.rs:147`), per-Asset, no `Result`.
- #2 ✅ `CodexAdapter::render()` confirmed exact passthrough stub (`crates/sos-adapter-codex/src/lib.rs:77-86` pre-change).
- #3 ✅ CRITICAL, resolved — `plan()` (adapter.rs:143) takes no Asset, but nothing prevents it from enumerating a fixed 10-Asset set internally and calling `render()` per-Asset, mapping `Artifact`→`ManagedOperation`. Engine (`crates/sos-install/src/engine.rs:100-143`) consumes `ManagedOperation` generically via `target_path`/`content` — zero engine change needed. Trait shape is coherent with Decision 1/3; **no escape hatch required**.
- #9 ✅ `sos-adapter-claude/src/lib.rs:28` render() also stub — no reference impl exists.
- #10 ⚠️ genuine open question (Codex sandbox_mode vs network semantics unconfirmed from discovery report) — correctly flagged as escape-hatch-during-impl, not blocking V1.
- #11 ✅ `.mcp.json` doctor server confirmed PATH-relative (`command:"doctor"`), mirror shape sound.

Ready for Chủ nhà approval gate.

**Status:** ✅ APPROVED V1 (Chủ nhà delegated sprint, 2026-07-22)

### Final consensus
- Phiếu version: V1
- Approved by Chủ nhà: 2026-07-22 — delegated sprint

---

## Nhiệm vụ

### Task 1: Định nghĩa bộ b2 Asset + template module (content-source)

**File:** `crates/sos-adapter-codex/src/**` (Worker chốt: template string trong `lib.rs` hay tách `src/templates.rs`/`src/render.rs` — module layout = Tầng 2 Worker call)

**Thêm:**
- Bộ **10 Asset** (Decision-map bảng Context): 1 AGENTS.md + 4 `.codex/agents/*.toml` + 4 `.agents/skills/*/SKILL.md` + 1 `.codex/config.toml`. Mỗi Asset mang đủ field để render() sinh `Artifact{target_path, content}` (target_path = đường dẫn declared trong bảng; verify field Asset thật `[needs Worker verify]` anchor #1).
- **Template string trong crate** cho từng surface — format-string khớp Codex report shape (anchor #5). KHÔNG đọc `core/**` filesystem lúc render (Decision 1). Template chứa **pointer** `core/ROLES.md#<role_id>` etc., KHÔNG copy semantics.

**Lưu ý:** template = string-format Rust literal (hoặc `format!`), KHÔNG template-engine ngoài (KHÔNG thêm dep nặng; nếu cần helper string thì workspace-inherited). Codex-only token (`.codex`, `AGENTS.md`, `sandbox_mode`) CHỈ trong crate này (dep-direction — core zero host token).

### Task 2: Impl `render()` — Asset → Artifact, map core ID, PARTIAL honest

**File:** `crates/sos-adapter-codex/src/lib.rs` (+ module Task 1)

**Tìm:** `CodexAdapter::render()` stub passthrough (anchor #2).

**Thay bằng:** render logic sinh content per surface:

1. **AGENTS.md** — orchestrator contract: load `SOS.md` + `core/{ROLES,WORKFLOW,POLICY,STATE}.md`; main-thread = orchestrator, **must NOT implement active ticket**; **no EXECUTE before exact-version approval**; spawn architect/worker per phase; pointer `core/ROLES.md#orchestrator` + `core/WORKFLOW.md`. Giữ concise (32KiB awareness, anchor #5).
2. **architect.toml** — `name="architect"`, `description`, `developer_instructions` trỏ `core/ROLES.md#architect`; `sandbox_mode="workspace-write"`; **PARTIAL marker** (TOML comment + trong developer_instructions): "envelope PARTIAL — Codex 0.145.0 no per-role tool allowlist; no-Bash/Grep/Edit KHÔNG structural, enforce qua PreToolUse hook (P078b3) + prose; KHÔNG structural tool-removal như Claude" (derive `CAPABILITY.md:12-24`).
3. **worker.toml** — trỏ `core/ROLES.md#worker`; `sandbox_mode="workspace-write"` (worker cần edit/run — honest); note chung "no per-role allowlist" ngắn.
4. **advisory-watch.toml** — trỏ `core/ROLES.md#advisory_watch`; `sandbox_mode="read-only"` (honest structural). Network cho advisory query: verify read-only sandbox cho outbound (anchor #10 escape hatch — nếu chặn thì note + config qua approval_policy).
5. **boundary-check.toml** — trỏ `core/ROLES.md#boundary_check`; `sandbox_mode="read-only"` (honest structural).
6-9. **`.agents/skills/{idea,forge,apply,retro}/SKILL.md`** — frontmatter `name`+`description` (2 field mechanical, anchor #5); body pointer core skill semantics + `core/WORKFLOW.md`. KHÔNG copy toàn văn Claude SKILL body (anchor #12).
10. **config.toml** — `[mcp_servers.doctor]` command/args/enabled_tools (PATH-relative `doctor`, anchor #11); `[agents] enabled=true`; `sandbox_mode`/`approval_policy` baseline; pointer `core/POLICY.md` + `core/ASSETS.md`.

**Lưu ý:** render() thuần (Asset→Artifact, no fs side-effect) để oracle unit-test trực tiếp. KHÔNG giả SOUND: architect.toml PHẢI mang PARTIAL marker (Decision 3), KHÔNG render "architect tool-scoped" giả. Provenance pointer = string trong content, mirror Claude P076 (`adapters/claude/README.md:28`).

### Task 3: Wire `plan()` để engine ghi 10 artifact khi `install --runtime codex`

**File:** `crates/sos-adapter-codex/src/lib.rs`

**Tìm:** `plan()` stub (b1 empty Plan).

**Thay bằng:** `plan()` enumerate bộ 10 Asset → render() từng cái → sinh `Plan` gồm 10 `ManagedOperation{target_path, content, ...}` để `sos-install::engine` ghi (non-clobber/rollback b1/d2 lo). Chính xác plan() gọi render() nội bộ HAY engine gọi render() per-asset = `[needs Worker verify]` anchor #3 (mirror ClaudeAdapter khi nó render — nhưng Claude stub, KHÔNG có reference → nếu trait shape mơ hồ = escape hatch).

**Lưu ý:** `install --runtime codex --dry-run` sau b2 phải LIỆT 10 artifact (path + preview). Full exit-0 vẫn có thể bị chặn bởi tool-manifest pin drift KHÔNG LIÊN QUAN (P077d3 step-5, giống `--runtime claude`, pre-existing — b1 discovery `:48-50`) → đó KHÔNG phải regression; verify plan/render reach đúng như claude path. `uninstall()` giữ stub-honest (removal thật ngoài scope b2 trừ khi trivial-mirror).

### Task 4: Structural oracle tests (Decision 5)

**File:** `crates/sos-adapter-codex/src/**` (`#[cfg(test)]` hoặc `tests/`)

**Thêm** test cho fresh render (in-memory Artifact / temp dir, KHÔNG commit `.codex/`):
- **TOML valid:** parse 4 `.codex/agents/*.toml` + `config.toml` (dùng `toml` crate nếu có trong workspace, else serde) — 0 parse error.
- **MD/frontmatter valid:** 4 `SKILL.md` frontmatter YAML parse OK có `name`+`description`; AGENTS.md well-formed (non-empty, có contract keyword).
- **Core-ID pointer:** mỗi artifact chứa đúng pointer (architect.toml chứa `core/ROLES.md#architect`, AGENTS.md chứa `core/WORKFLOW.md`, etc.).
- **PARTIAL-honest:** architect.toml content chứa PARTIAL marker text; advisory-watch.toml + boundary-check.toml chứa `sandbox_mode = "read-only"`; architect.toml + worker.toml chứa `workspace-write`.
- **Report-format match:** mỗi `.codex/agents/*.toml` có 3 required field `name`/`description`/`developer_instructions` (anchor #5); config.toml có `[mcp_servers.doctor]` + `[agents]`.
- **Artifact count:** render/plan sinh đúng 10 artifact (KHÔNG init → 4 skill, Decision 4).

**Lưu ý:** oracle chạy KHÔNG cần codex cài (Decision 5). KHÔNG test behavioral (Codex CLI chạy) = P079. Nếu `toml`/`serde_yaml` chưa trong workspace → dùng có sẵn hoặc hand-parse minimal (Worker chốt; KHÔNG thêm dep nặng chỉ cho test — verify `Cargo.toml` workspace deps trước).

### Task 5: Docs gate

**File:** `adapters/codex/MAPPING.md` — fill 10 b2-render row (Physical render column = "P078b2 DONE" cho 10 artifact); **reconcile init** (`:17`): annotate row init = "deferred (Claude caller `sos init` CLI, name-collision review) — KHÔNG render b2" (Decision 4). Foundation-coverage table (`:27-33`) thêm row render nếu hợp.

**File:** `adapters/codex/README.md` — status line (`:3-6`) bump: "P078b2 — 10 declarative artifact rendered (`render()`/`plan()` live, structural oracle); enforcement (hooks/rules/guards) = b3".

**File:** `docs/PORTABILITY_ARCHITECTURE.md` — thêm "P078b2 status" đoạn (sau `:58` P078b1): render 10 artifact, content-source = crate template, core-ID pointer, PARTIAL-honest architect, structural oracle, behavioral deferred P079. Migration table row P078 giữ IN PROGRESS (b3 còn).

**File:** `CHANGELOG.md` — entry P078b2.

**Lưu ý:** `core/**` docs semantics KHÔNG đổi. Regression: `grep -rn 'adapters/' core/` → zero (dep 1 chiều; b1 discovery `:64-66` từng suýt vi phạm — Worker tự chạy check này trên mọi prose thêm vào `core/**`, dù b2 KHÔNG dự định đụng core).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-adapter-codex/src/lib.rs` (+ optional `src/templates.rs`/`src/render.rs`) | Task 1,2,3,4: Asset set + template + render() + plan() wire + tests |
| `adapters/codex/MAPPING.md` | Task 5: fill 10 b2 render row + reconcile init |
| `adapters/codex/README.md` | Task 5: status line b2 |
| `docs/PORTABILITY_ARCHITECTURE.md` | Task 5: P078b2 status đoạn |
| `CHANGELOG.md` | Task 5: entry |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `crates/sos-core/src/adapter.rs` | Asset/Artifact/ManagedOperation đủ chở render — KHÔNG thêm field (nếu buộc → STOP escalate) |
| `crates/sos-install/src/engine.rs` | Engine ghi Plan qua trait KHÔNG đổi (b1 anchor #12) |
| `crates/sos-adapter-claude/src/lib.rs` | Claude render vẫn stub (P078c) — zero-touch |
| `bin/sos.sh`, `install.sh` | Zero-touch (additive) |
| `.codex/hooks.json`, `.codex/rules/**`, `scripts/codex/**` | = b3, KHÔNG b2 |
| `core/{ROLES,WORKFLOW,POLICY,STATE}.md` | Pointer đúng ID, KHÔNG sửa content |
| `.mcp.json` (root) | Đọc doctor server để mirror config.toml, KHÔNG sửa |

---

## Luật chơi (Constraints)

1. **Declarative-only** — b2 render artifact declarative; enforcement (hooks/rules/guard-rewrite) = b3. KHÔNG đụng `.codex/hooks.json`/`rules`/`scripts/codex`.
2. **Content-source = crate template** (Decision 1) — KHÔNG đọc `core/**` filesystem lúc render; template chứa pointer, KHÔNG copy semantics (`core/ASSETS.md:51`).
3. **Core ID pointer, KHÔNG duplicate** (Decision 2, `core/ROLES.md:120` sep-inv #5) — mỗi artifact trỏ `core/ROLES.md#<role_id>`/`core/WORKFLOW.md`/`core/POLICY.md`, render pointer không copy vai.
4. **PARTIAL honest trong render** (Decision 3, `core/POLICY.md:65`) — architect.toml MANG PARTIAL marker; advisory/boundary read-only honest; KHÔNG render giả "architect tool-scoped".
5. **Structural oracle** (Decision 5) — chạy KHÔNG cần codex cài; behavioral = P079.
6. **Additive + KHÔNG commit artifact** (Decision 6) — render tới target project; `.codex/`/`AGENTS.md`/`.agents/` KHÔNG commit vào sos-kit; `install.sh`/`bin/sos.sh`/`engine.rs` zero-touch.
7. **Dep-direction bất di** — Codex token CHỈ trong `sos-adapter-codex`; core zero host token; `dep_direction.rs` XANH; `grep -rn 'adapters/' core/` → zero.
8. **Skill = 4 (idea/forge/apply/retro)** (Decision 4) — KHÔNG init; MAPPING reconcile, KHÔNG để claim init rendered.
9. **KHÔNG thêm core-type field** — nếu render buộc → STOP escalate (b2 KHÔNG đụng `core/src`).
10. **Cite RANGE** (`core/POLICY.md:57-65`), KHÔNG count.

---

## Nghiệm thu

### Automated (oracle STRUCTURAL — Decision 5)
- [ ] `cargo build --workspace` xanh. `[oracle: cargo build --workspace SOUND]`
- [ ] `cargo test --workspace` xanh incl. render structural tests (TOML/MD/frontmatter valid + core-ID pointer + PARTIAL marker + report-format + count==10) + dep_direction guard. `[oracle: fresh-render structural — TOML/MD valid + match report format + core-ID pointer + PARTIAL-honest SOUND]`
- [ ] `cargo test --workspace` ×20 = 0 flaky. `[oracle: ×20 flaky-check SOUND]`
- [ ] dep-direction: `grep -rn "sos_adapter_codex" crates/sos-core/src/` → zero + `grep -rn 'adapters/' core/` → zero. `[oracle: dep-direction SOUND]`
- [ ] `install --runtime codex --dry-run` LIỆT 10 artifact (path + preview), reach đúng path như `--runtime claude`. `[oracle: dry-run structural — PARTIAL nếu env tool-pin drift chặn exit-0 giống claude, b1 precedent]`

**Ranh giới oracle (in rõ):** b2 KHÔNG verify Codex CLI đọc/chạy đúng output — đó behavioral, P079. b2 verify = artifact valid + khớp report format + trỏ đúng core ID + PARTIAL honest. Structural-vs-behavioral boundary = Decision 5.

### Manual Testing
- [ ] Đọc 1 `.codex/agents/architect.toml` render out: 3 required field + PARTIAL marker rõ (envelope PARTIAL, enforce b3-hook) + trỏ `core/ROLES.md#architect`.
- [ ] AGENTS.md render out: orchestrator contract (no-impl-active-ticket + no-EXECUTE-before-approval + spawn-per-phase) + trỏ core.
- [ ] `verify()` gaps ↔ architect.toml PARTIAL marker ↔ `CAPABILITY.md` khớp (1 tuyên bố 3 surface).

### Regression
- [ ] `install --runtime claude --dry-run` KHÔNG regress (Claude adapter zero-touch).
- [ ] P077c parity fixtures xanh.
- [ ] `crates/sos-core/**` diff rỗng (KHÔNG thêm core-type field).
- [ ] `.codex/`/`AGENTS.md`/`.agents/` KHÔNG xuất hiện trong `git status` của sos-kit (Decision 6).

### Docs Gate (Tầng 1)
- [ ] `adapters/codex/MAPPING.md` — 10 b2 render row filled + init reconciled
- [ ] `adapters/codex/README.md` — status line b2
- [ ] `docs/PORTABILITY_ARCHITECTURE.md` — P078b2 status đoạn
- [ ] `CHANGELOG.md` — entry P078b2

### Discovery Report
- [ ] Write `docs/discoveries/P078b2.md`:
  - Anchors #1,#2,#3,#9,#10,#11 — CORRECT/WRONG (`file:line`). ĐẶC BIỆT #1+#3: render mechanism thật — plan() gọi render() hay engine? Asset/Artifact field exact names.
  - Content-source: template layout thật (lib.rs inline hay module tách); có đọc core/** không (KHÔNG đúng chưa).
  - PARTIAL marker: architect.toml text ↔ verify()/CAPABILITY.md sync.
  - Skill decision: 4 rendered, init reconciled thế nào; MAPPING drift đóng chưa.
  - #10 escape: advisory-watch read-only + network — có chặn không, xử lý sao.
  - #11: doctor MCP config mirror — PATH-relative giữ chưa.
  - **Symmetry flag** (P078c Claude-render): render pattern b2 có bake Codex-only giả định nào cần reconcile khi Claude render thật.
  - Docs updated (list) / Tier escalations ("None" nếu không).
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`.
