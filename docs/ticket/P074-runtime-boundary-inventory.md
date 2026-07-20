# PHIẾU P074: Runtime boundary inventory — tách SOS Core khỏi Claude Code

> **ID:** P074
> **Filename:** `docs/ticket/P074-runtime-boundary-inventory.md`
> **Branch:** `docs/P074-runtime-boundary-inventory`

---

> **Loại:** Feature (architecture/docs)
> **Ưu tiên:** P1
> **Tầng:** 1 — chốt ownership boundary và migration order cho toàn bộ workflow/runtime surface; sai boundary sẽ làm Claude regression hoặc fork doctrine khi thêm Codex.
> **Ảnh hưởng:** new `docs/RUNTIME_BOUNDARY_INVENTORY.md`, new `docs/PORTABILITY_ARCHITECTURE.md`, `docs/BACKLOG.md`, `CHANGELOG.md`, discovery record.
> **Dependency:** None. P071 Linux/Windows acceptance là carry-over riêng, không block portability inventory.
> **Lane:** Normal (≤250 dòng, ≤5 anchors, ≤5 hard constraints)

```yaml
edit_allow:
  - docs/ticket/P074-runtime-boundary-inventory.md
  - docs/RUNTIME_BOUNDARY_INVENTORY.md
  - docs/PORTABILITY_ARCHITECTURE.md
  - docs/BACKLOG.md
  - CHANGELOG.md
  - docs/discoveries/P074.md
  - docs/DISCOVERIES.md

verify_read:
  - "**" # read-only; oracle input MUST be the paths returned by git ls-files at snapshot HEAD

contract_tests: []
```

---

## Context

### Vấn đề hiện tại

SOS Kit hiện là một meta-kit có doctrine tốt nhưng phần phân phối và enforcement đang gắn chặt với Claude Code: `CLAUDE.md`, `.claude/agents`, `.claude/settings.json`, Claude lifecycle hooks, `CLAUDE_PROJECT_DIR`, tool/model names và marker protocol. Nếu viết Codex support trực tiếp trên shape này, doctrine sẽ bị copy/fork và mỗi runtime phải duy trì một bản workflow riêng.

Chủ nhà đã duyệt đích đến: **một SOS Core độc lập runtime + adapter mỏng cho Claude Code và Codex**. Cursor, OpenCode và Antigravity không nằm trong vòng này vì chưa có môi trường dogfood thật.

### Giải pháp

Lập một inventory có bằng chứng cho các surface hiện tại, phân loại:

- `CORE` — doctrine, policy, workflow, phiếu, recipe, gate hoạt động độc lập agent runtime.
- `CLAUDE` — manifest, hook binding, tool/model/env và entrypoint chỉ Claude Code hiểu.
- `MIXED` — một file đang trộn semantics chung với serialization/runtime wiring; phải chẻ ở P075–P077.
- `GENERATED` — artifact adapter sinh ra khi install/sync; không phải doctrine source.

Kết quả của P074 là hai tài liệu: `docs/RUNTIME_BOUNDARY_INVENTORY.md` ghi current reality; `docs/PORTABILITY_ARCHITECTURE.md` chốt target boundary, dependency order và acceptance gates cho P075–P081. Không di chuyển file hoặc thay behavior trong phiếu này.

### Scope

- CHỈ tạo/sửa: `docs/RUNTIME_BOUNDARY_INVENTORY.md`, `docs/PORTABILITY_ARCHITECTURE.md`, `docs/BACKLOG.md`, `CHANGELOG.md`, `docs/discoveries/P074.md`, `docs/DISCOVERIES.md`, và chính phiếu P074.
- CHỈ thiết kế adapter cho Claude Code + Codex trong roadmap hiện tại.
- KHÔNG di chuyển/rename file, không sửa scripts/hooks/agents/skills/config, không viết Codex adapter.
- KHÔNG sửa `docs/WORKFLOW_V2.X.md`; doctrine change phải đi qua retro riêng.

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|------------|-----------|--------|
| 1 | Claude entry/config surfaces hiện tại gồm `CLAUDE.md`, `.claude/settings.json`, `.claude/agents`, `.claude/commands`, `.claude/skills` | `git ls-files CLAUDE.md '.claude/**'` | ✅ Các surface đều hiện diện; `.claude/agents` và skills có symlink trong working tree |
| 2 | Runtime hooks đang bind bằng Claude event names | Read `.claude/settings.json` | ✅ Có `SessionStart`, `PreToolUse`, `UserPromptSubmit`; matcher dùng Claude tool names |
| 3 | Guard scripts mang coupling qua `CLAUDE_PROJECT_DIR` | `rg -l 'CLAUDE_PROJECT_DIR' scripts hooks bin phieu templates` | ✅ Nhiều lifecycle/guard surface dùng biến Claude; cần inventory exact khi EXECUTE |
| 4 | Agent handbooks trộn core role semantics với Claude tool/model/background metadata | Read `agents/orchestrator.md`, `agents/architect.md`, `agents/worker.md` | ✅ `tools`, `model`, `background`, Claude task semantics và role doctrine cùng tồn tại |
| 5 | Rust `sos` port tồn tại nhưng còn được mô tả là skeleton; Bash CLI là MVP thực tế | Read `bootstrap/sos-rs/Cargo.toml`, `bootstrap/sos-rs/README.md`, `bin/sos.sh --help` | ✅ Rust package `sos` 0.1.0 có; README gọi skeleton; Bash CLI expose full command surface |

---

## Debate Log

**Phiếu version:** V2 (after Turn 1 Architect response)

### Turn 1 — Worker Challenge

**Oracle:** `doctor lane-check --ticket docs/ticket/P074-runtime-boundary-inventory.md` → `OK (Normal — 210 lines, 5 anchors, 5 constraints)`. Cả 5 anchors được chấp nhận: Claude hook events/tool matchers nằm tại `.claude/settings.json:3-57`; agent manifests trộn `tools`/`model`/`background` với role semantics tại `agents/{orchestrator,architect,worker}.md:1-17`; Rust skeleton và Bash-canonical status tại `bootstrap/sos-rs/README.md:1-27`; `bin/sos.sh --help` expose đủ command surface. `git ls-files -s '.claude/**'` còn xác nhận agent/skill registrations là tracked symlinks (mode `120000`).

**Objection O1 — Tầng 1, edit envelope tự mâu thuẫn.** Scope cho phép sửa chính phiếu (`:67-72`) và CHALLENGE bắt buộc ghi Debate Log, nhưng YAML `edit_allow` (`:17-23`) cùng bảng Files cần sửa (`:155-164`) đều bỏ `docs/ticket/P074-runtime-boundary-inventory.md`. Worker EXECUTE theo contract sẽ phải coi một trong các nguồn là sai. **Yêu cầu:** thêm exact ticket path vào `edit_allow` + Files cần sửa, hoặc tuyên bố Debate Log/Task-0 update là ngoại lệ schema rõ ràng.

**Objection O2 — Tầng 1, oracle coverage không khớp quyền đọc.** Task 1/acceptance đòi cover *toàn bộ tracked tree* và mọi runtime-token hit (`:108-118`, `:190-192`), nhưng `verify_read` (`:25-41`) bỏ các tracked root surfaces có hit thật: `README.md:5,14,26-30`, `INSTALL.md:33,39,46-64`, `SECURITY.md:5,11-21`, `CHANGELOG.md` và `.sos-trust-baseline:1`. Vì vậy inventory không thể vừa evidence-grounded vừa tuân envelope. **Yêu cầu:** mở `verify_read` thành tracked-tree read-only (`git ls-files`) hoặc bổ sung tất cả root/dotfile surfaces; khóa oracle tracked-only (ví dụ `git grep`) để untracked `.backup/`/retro dogfood không làm kết quả nondeterministic.

**Objection O3 — Tầng 1, ownership của Rust framework chưa được chốt.** Roadmap bắt P077 xây Rust adapter framework (`:131-143`), nhưng contract hiện tại nói repo này “Not a runtime binary source” (`CLAUDE.md:28-31`), còn Rust README nói sau parity sẽ chuyển `sos` sang repo riêng (`bootstrap/sos-rs/README.md:23-27`). Target architecture cần chọn rõ: P077 triển khai tạm trong `bootstrap/sos-rs` rồi extract, hay thay đổi product/repo contract để giữ Rust CLI tại đây; đồng thời gắn owning phiếu cho thay đổi contract đó. Nếu không, P077 không có edit owner/location hợp lệ.

**Status:** ❌ OBJECTIONS — cần Architect RESPOND, chưa được APPROVAL/EXECUTE.

### Turn 1 — Architect Response (phiếu V2)

- **O1 → ACCEPT.** Thêm chính `docs/ticket/P074-runtime-boundary-inventory.md` vào `edit_allow` và bảng Files cần sửa. Debate Log/anchor update không còn là ngoại lệ ngầm.
- **O2 → ACCEPT.** Mở `verify_read` cho toàn tree nhưng khóa evidence set vào tracked paths tại snapshot HEAD. Task 1 và Automated acceptance đổi sang `git ls-files` + `git grep`, không đọc untracked `.backup/` hoặc artifact ngoài snapshot.
- **O3 → DEFER TO CHỦ NHÀ.** Rust CLI ownership/location thay đổi product/repo contract hiện tại, nên Architect không tự chọn. Ba lựa chọn:
  - **A. Giữ Rust CLI vĩnh viễn trong sos-kit** — đổi contract repo thành monorepo chứa runtime source.
  - **B. Incubate P077 tại `bootstrap/sos-rs`, đạt parity/dogfood rồi extract sang repo `sos` riêng** — giữ contract hiện tại; **Architect từng khuyến nghị B tại Turn 1** vì ít đổi boundary nhất và khớp Rust README hiện hành; quyết định này đã bị supersede bởi lựa chọn A của Chủ nhà ở Final consensus.
  - **C. Extract sang repo `sos` riêng trước P077** — ownership sạch sớm nhưng tăng cross-repo coordination trước khi adapter contract chín.
  P074 vẫn inventory current reality và ghi decision record; không implement/extract theo lựa chọn nào. Chủ nhà phải chọn A/B/C trước APPROVAL_GATE để `docs/PORTABILITY_ARCHITECTURE.md` có target owner duy nhất.

**Status:** ✅ RESPONDED — Chủ nhà chọn A với monorepo module hóa và một binary/product entrypoint; P074 được phép EXECUTE docs-only.

### Final consensus

- Phiếu version: V2
- Total turns: 1
- Rust CLI ownership: ✅ **A — giữ trong `sos-kit` dưới Rust workspace module hóa**. Một repo/version/product; core và adapter vẫn là crate riêng, không trộn boundary.
- Tool distribution: ✅ `sos install` là entrypoint duy nhất; sister Rust tools được khóa version trong tool manifest và cài/kiểm tra tự động. Việc nhập source từng sister tool vào workspace không phải điều kiện của one-command UX.
- Approved by Chủ nhà: ✅ 2026-07-20 — EXECUTE P074 docs-only.

---

## Nhiệm vụ

### Task 1: Inventory current surfaces

**File:** new `docs/RUNTIME_BOUNDARY_INVENTORY.md`

Liệt kê các top-level surface và mọi file/pattern có runtime token (`Claude`, `.claude`, `CLAUDE_*`, tool/model/background/hook names). Mỗi row phải có:

```text
path/pattern | class | current responsibility | coupling evidence | target owner | migration phiếu
```

Inventory phải cover toàn bộ tracked tree bằng explicit row hoặc glob có precedence; mỗi `MIXED` row nêu rõ phần portable và phần runtime-specific.

Evidence set phải được sinh từ `git ls-files` tại snapshot HEAD. Runtime-token search dùng `git grep` trên đúng tracked set; không đưa untracked files hoặc build artifacts vào inventory.

### Task 2: Chốt target core/adapter boundary

**File:** new `docs/PORTABILITY_ARCHITECTURE.md`

Trong cùng doc, định nghĩa ownership:

- Portable core: role semantics, workflow, policy, phiếu, recipes, canonical skill bodies, universal Git gates và state schema.
- Claude adapter: `CLAUDE.md`, `.claude/**`, Claude agent manifests, hook serialization và tool/model/env mapping.
- Codex adapter: `AGENTS.md`, `.codex/**`, `.agents/skills`, Codex agent/hook/config/MCP/permission mapping.
- Generated artifacts: file install/sync sinh và quản lý bằng manifest/hash; không là source of truth.

### Task 3: Chốt migration order và gates

Ghi dependency chain P075→P081 và gate cho từng chặng:

```text
portable core
→ Claude behavioral parity
→ Rust adapter framework
→ Codex native adapter
→ sos-kit self-dogfood
→ dual-runtime brownfield dogfood
→ packaging
```

Nêu rõ Cursor/OpenCode/Antigravity là future adapters, không thuộc acceptance hiện tại.

Ghi Rust CLI ownership A đã được Chủ nhà chọn: P077 nâng `bootstrap/sos-rs` thành Rust workspace chính thức trong `sos-kit`; repo trở thành monorepo chứa runtime source. Việc sửa contract hiện hành trong `CLAUDE.md` thuộc P077, không thực hiện trong P074 docs-only.

### Task 4: Traceability

- Thêm `CHANGELOG.md` entry cho P074.
- Viết `docs/discoveries/P074.md` và index trong `docs/DISCOVERIES.md`.
- Nếu inventory phát hiện roadmap assumption sai, chỉ ghi discovery/backlog; không mở rộng refactor trong P074.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `docs/ticket/P074-runtime-boundary-inventory.md` | Debate/anchor updates trong lifecycle phiếu |
| `docs/RUNTIME_BOUNDARY_INVENTORY.md` | New: inventory tracked surfaces + classification evidence |
| `docs/PORTABILITY_ARCHITECTURE.md` | New: target boundary + dependency direction + migration gates |
| `docs/BACKLOG.md` | Runtime-portability sprint trace |
| `CHANGELOG.md` | P074 entry |
| `docs/discoveries/P074.md` | Discovery report |
| `docs/DISCOVERIES.md` | One-line index |

## Files KHÔNG sửa

| Surface | Verify only |
|---------|-------------|
| `.claude/**`, `CLAUDE.md`, `agents/**` | Phân loại coupling; không thay behavior |
| `scripts/**`, `hooks/**`, `bin/**`, `bootstrap/**` | Inventory; không refactor |
| `docs/WORKFLOW_V2.X.md` | Doctrine immutable trong production phiếu |

---

## Luật chơi

1. P074 là docs-only inventory: diff có executable/config/runtime file là scope violation.
2. Mỗi classification phải có evidence path/token từ tracked snapshot; không đọc/đếm untracked artifacts và không phân loại theo cảm giác.
3. Core target không được chứa runtime-specific tool, model, env var hoặc manifest schema.
4. Claude behavior hiện tại là golden oracle cho P076; P074 không “dọn” hay đổi behavior.
5. Runtime vòng này chỉ Claude Code + Codex; ba runtime chưa dogfood phải ở future scope.

---

## Nghiệm thu

### Automated

- [x] `docs/RUNTIME_BOUNDARY_INVENTORY.md` có đủ bốn class `CORE`, `CLAUDE`, `MIXED`, `GENERATED`.
- [x] `git ls-files` được đối chiếu inventory; không có tracked path chưa được cover.
- [x] Mọi kết quả `git grep -Il -E 'Claude|CLAUDE_|\.claude|sonnet|opus|AskUserQuestion|PreToolUse|SessionStart|UserPromptSubmit'` được map hoặc có exclusion reason (99 tracked paths tại EXECUTE snapshot).
- [x] `git diff --name-only` không chứa executable/config/runtime file ngoài docs scope.
- [x] Existing shell syntax/trust gate vẫn pass (regression-only; P074 không đổi chúng).

### Manual

- [x] Boundary doc trả lời được: xóa Claude adapter thì core nào còn hoạt động?
- [x] Mỗi `MIXED` surface có target split và owning phiếu P075/P076/P077.
- [x] Packaging chỉ xuất hiện sau hai dogfood gates P079/P080.

### Docs Gate

- [x] `CHANGELOG.md` có entry P074.
- [x] `docs/PORTABILITY_ARCHITECTURE.md` phân biệt rõ current reality và target architecture.

### Discovery Report

- [x] Viết `docs/discoveries/P074.md`.
- [x] Thêm one-line index vào `docs/DISCOVERIES.md`.
