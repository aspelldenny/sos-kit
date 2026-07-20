# PHIẾU P075: Portable SOS Core — role, workflow, policy và asset ownership trung lập runtime

> **ID:** P075
> **Filename:** `docs/ticket/P075-portable-sos-core.md`
> **Branch:** `feat/P075-portable-core`

---

> **Loại:** Feature (architecture/docs)
> **Ưu tiên:** P1
> **Tầng:** 1 — đây là semantic contract mà mọi adapter và Rust engine sẽ phụ thuộc; sai boundary sẽ fork workflow giữa các runtime.
> **Ảnh hưởng:** new `SOS.md`, new `core/**`, backlog/changelog/discovery trace.
> **Dependency:** P074 complete (`3c08fdb`)
> **Lane:** Normal (≤250 dòng phiếu, ≤5 anchors, ≤5 hard constraints)

```yaml
edit_allow:
  - SOS.md
  - core/README.md
  - core/ROLES.md
  - core/WORKFLOW.md
  - core/POLICY.md
  - core/ASSETS.md
  - docs/ticket/P075-portable-sos-core.md
  - docs/BACKLOG.md
  - CHANGELOG.md
  - docs/discoveries/P075.md
  - docs/DISCOVERIES.md

verify_read:
  - docs/PORTABILITY_ARCHITECTURE.md
  - docs/RUNTIME_BOUNDARY_INVENTORY.md
  - docs/WORKFLOW_V2.2.md
  - docs/LAYERS.md
  - docs/HANDOFF.md
  - docs/ORCHESTRATION.md
  - docs/PHILOSOPHY.md
  - agents/**
  - skills/**
  - phieu/**
  - recipes/**
  - configs/**
  - templates/**

contract_tests: []
```

---

## Context

### Vấn đề hiện tại

P074 xác nhận doctrine chung đang trộn với runtime wiring trong gần 4.000 dòng docs/agents/skills. Copy toàn bộ rồi search-replace tên runtime sẽ tạo một doctrine fork mới, trong khi giữ nguyên shape hiện tại sẽ buộc adapter sau tiếp tục hiểu tool/model/event cụ thể.

### Giải pháp

Tạo một contract trung lập, nhỏ và có cấu trúc:

- `SOS.md`: entrypoint + precedence của portable core.
- `core/ROLES.md`: role IDs, responsibilities, authority và capability requirements.
- `core/WORKFLOW.md`: state machine, transitions, gates và delivery unit.
- `core/POLICY.md`: authority tiers, information envelopes, oracle/scope/safety rules.
- `core/ASSETS.md`: ownership catalog cho phiếu, recipes, gates, templates và transitional assets.
- `core/README.md`: boundary/import rules cho adapter và engine.

Core dùng capability vocabulary (`read_files`, `edit_files`, `run_commands`, `ask_human`, `delegate_work`, `track_tasks`) thay vì tên tool/model/event/env/manifest của một host. Existing runtime behavior giữ nguyên trong P075; P076 mới map contract vào adapter đầu tiên và chạy golden parity.

### Scope

- CHỈ tạo portable contract Markdown và sửa trace docs được liệt kê trong `edit_allow`.
- KHÔNG sửa/di chuyển `CLAUDE.md`, `.claude/**`, `agents/**`, `skills/**`, scripts, hooks, CLI hoặc Rust source.
- KHÔNG viết adapter, renderer, manifest schema hay installer code.
- KHÔNG rewrite historical docs; chúng là evidence/provenance.

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|---|---|---|
| 1 | P074 đã chốt core không chứa runtime event/tool/model/env/path | Read `docs/PORTABILITY_ARCHITECTURE.md` Ownership | ✅ Lines 45-53 |
| 2 | Canonical role files trộn semantics với runtime frontmatter/tool names | `git grep -n -E '^(tools|model|background):|AskUserQuestion|TaskCreate' -- agents/` | ✅ Hits ở architect/worker/specialists |
| 3 | Workflow lifecycle chung hiện có DRAFT, CHALLENGE, APPROVAL, EXECUTE, DISCOVERY và merge/cleanup | Read `docs/HANDOFF.md`, `docs/ORCHESTRATION.md`, `agents/worker.md` | ✅ Các state/handoff đều hiện diện |
| 4 | Portable assets hiện có ở `phieu/**`, `recipes/**`, configs/templates và universal Git gates | Cross-check `docs/RUNTIME_BOUNDARY_INVENTORY.md` với `git ls-files` | ✅ Inventory đã map tracked tree |
| 5 | P075 gate yêu cầu zero runtime token trong `SOS.md` + `core/**` | Run negative grep sau EXECUTE | ✅ Zero hits |

---

## Debate Log

**Phiếu version:** V2 (after Turn 1 challenge)

### Turn 1 — Worker Challenge

**Anchor verification:**

- #1 ✅ P074 forbids runtime event/tool/model/env/path in core.
- #2 ✅ All spawnable handbooks carry runtime frontmatter/tool names.
- #3 ✅ Existing docs contain the complete DRAFT→CHALLENGE→APPROVAL→EXECUTE→DISCOVERY→merge lifecycle.
- #4 ✅ P074 inventory covers the tracked asset tree.
- #5 ⏳ Post-create negative oracle; cannot run before core exists.

**Objection O1.1 — `core/ASSETS.md` task conflicts with zero-token constraint.** V1 asked the core file to “catalog path hiện tại” including `ADAPTER_OWNED`, but exact adapter paths contain forbidden runtime names. That would make acceptance fail by design and leak host serialization back into core.

**Recommendation:** keep exact current-path mapping in P074 inventory; `core/ASSETS.md` defines portable asset classes, abstract host-registration ownership and links to inventory for migration evidence.

**Status:** ✅ OBJECTION — Architect response below.

### Turn 1 — Architect Response (phiếu V2)

- **O1.1 → ACCEPT.** Task 4 now forbids exact host path duplication. `core/ASSETS.md` owns semantic asset classes and portable source paths only; P074 remains the path-level inventory.

**Status:** ✅ RESPONDED — V2 ready for delegated approval.

### Final consensus

- Phiếu version: V2
- Total turns: 1
- Approved by Chủ nhà: ✅ portability sprint delegated 2026-07-20 (“dừng khi có vấn đề hoặc cần founder”); O1.1 closed mechanically, no founder decision required.

---

## Nhiệm vụ

### Task 1: Root portable entrypoint và boundary

**Files:** `SOS.md`, `core/README.md`

Định nghĩa portable core là semantic source of truth; adapter chỉ map capability và serialize artifacts, không được thay role/workflow/policy. Ghi precedence và transitional rule để P075 không tuyên bố existing runtime files đã generated khi chúng chưa phải.

### Task 2: Role contract

**File:** `core/ROLES.md`

Định nghĩa stable IDs `owner`, `orchestrator`, `architect`, `worker`, `advisory_watch`, `boundary_check`. Mỗi role có responsibilities, authority, required/forbidden capabilities, inputs/outputs. Không chứa host tool/model/event/env/path.

### Task 3: Workflow + policy contract

**Files:** `core/WORKFLOW.md`, `core/POLICY.md`

- State machine: `INTAKE → DRAFT → CHALLENGE? → APPROVAL → EXECUTE → DISCOVERY → REVIEW → DELIVERED` cùng `BLOCKED`/return transitions.
- Challenge bắt buộc cho Tầng 1, optional cho Tầng 2; human giữ scope/vision/security/irreversible authority.
- Mỗi phiếu là một delivery unit: gate xanh → commit → push/merge → mới execute phiếu tiếp theo.
- Giữ oracle-first, edit/verify asymmetry, information envelopes, non-clobber và escalation semantics.

### Task 4: Asset ownership

**File:** `core/ASSETS.md`

Định nghĩa asset classes `PORTABLE`, `TRANSITIONAL_MIXED`, `ADAPTER_OWNED`, `GENERATED`; catalog portable source paths và canonical owner. Host-registration classes chỉ được mô tả trừu tượng, không lặp exact runtime paths/tokens; exact current-path evidence link về P074 inventory. Catalog là content ownership, chưa phải install manifest schema.

### Task 5: Traceability

Sửa `docs/BACKLOG.md`, `CHANGELOG.md`; viết `docs/discoveries/P075.md` và index. Không sửa runtime documentation ngoài trace.

---

## Files cần sửa

| File | Thay đổi |
|---|---|
| `SOS.md` | New portable entrypoint/precedence |
| `core/README.md` | New boundary/import rules |
| `core/ROLES.md` | New neutral role contract |
| `core/WORKFLOW.md` | New neutral state machine |
| `core/POLICY.md` | New authority/envelope/oracle/safety policy |
| `core/ASSETS.md` | New asset ownership catalog |
| `docs/ticket/P075-portable-sos-core.md` | Lifecycle evidence/acceptance |
| `docs/BACKLOG.md`, `CHANGELOG.md` | P075 status/release trace |
| `docs/discoveries/P075.md`, `docs/DISCOVERIES.md` | Discovery report/index |

## Files KHÔNG sửa

| Surface | Verify only |
|---|---|
| `CLAUDE.md`, `.claude/**`, `agents/**`, `skills/**` | Golden runtime behavior cho P076 |
| `docs/WORKFLOW_V2.2.md`, `docs/LAYERS.md`, `docs/HANDOFF.md`, `docs/ORCHESTRATION.md`, `docs/PHILOSOPHY.md` | Extract semantics; không rewrite history/current runtime docs |
| `scripts/**`, `hooks/**`, `bin/**`, `bootstrap/**` | Không đổi executable/runtime behavior |

---

## Luật chơi

1. `SOS.md` và mọi file dưới `core/**` phải pass zero runtime-token grep.
2. Core chỉ nói capability/semantics; không chứa host manifest, permission schema, event, env, tool hay model name.
3. Không copy nguyên handbook hiện tại; contract phải nhỏ, normative và tránh historical narrative.
4. P075 không thay current runtime behavior hoặc tuyên bố adapter generation đã hoạt động.
5. Mỗi semantic rule phải có đúng một canonical location; file khác link thay vì paraphrase thành bản thứ hai.

---

## Nghiệm thu

### Automated

- [x] Negative grep zero hits trên `SOS.md core/**` cho runtime names/paths/events/tool/model/env tokens.
- [x] `git diff --name-only` chỉ chứa `edit_allow` docs paths.
- [x] `docs-gate --all`, discovery gate, lane-check và `git diff --check` pass.
- [x] Existing shell syntax + trust gate pass (regression only).

### Manual

- [x] Một adapter author có thể map host capabilities mà không sửa core semantics.
- [x] Xóa toàn bộ adapter surfaces vẫn đọc được roles, lifecycle, authority và asset ownership.
- [x] Không có hai file core cùng định nghĩa lại một policy.

### Regression

- [x] `CLAUDE.md`, `.claude/**`, `agents/**`, `skills/**`, scripts/hooks/CLI/Rust source không đổi.

### Docs Gate

- [x] `CHANGELOG.md` có entry P075; `docs/BACKLOG.md` chuyển P075 done.

### Discovery Report

- [x] `docs/discoveries/P075.md` đúng schema và được index trong `docs/DISCOVERIES.md`.
