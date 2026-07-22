# PHIẾU P077c5: OA-02 scanner fix — Rust map/adopt CORRECTNESS (Rust beat Bash)

> **ID format:** P077c5 (sub-phiếu CUỐI của P077c — decomposition `docs/plans/P077c-decomposition.md` row c5).
> **Filename:** `docs/ticket/P077c5-oa02-scanner-fix.md`
> **Branch:** `fix/P077c5-oa02-scanner-fix`

---

> **Loại:** Bugfix (correctness — OA-02 🔴 false-green)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — scanner correctness + AGENT_MAP status semantics = surface contract Architect đọc để route; sai thì LAN sang mọi phiếu dùng map)
> **Lane:** Guarded
> **Ảnh hưởng:** `bootstrap/sos-rs/crates/sos-cli/src/commands/map.rs`, `adopt.rs` (verify only, likely no code change), `crates/sos-cli/tests/parity.rs`, `tests/golden/capture.sh`, `tests/golden/map.agent_map.golden`, `tests/golden/map.golden`, `tests/golden/adopt.gen.golden`, `tests/golden/adopt.golden`
> **Dependency:** P077c1 (Rust map parity + `map.agent_map.golden` + per-command harness), P077c4 (Rust adopt + map-within-adopt + `adopt.gen.golden`) — both SHIPPED

---

## Context

### Vấn đề hiện tại

OA-02 (`docs/retro/OUTSIDER_AUDIT_SYSTEM_GAPS_2026-07-22.md:93-131`, 🔴): `sos map` false-green về coverage. Brownfield Rust crate → map sinh surface `frontend` trỏ tới `templates/` (thư mục SOS Kit **vừa copy vào**), KHÔNG map `src/main.rs`, mà `doctor validate-map` vẫn PASS (validate-map kiểm soundness của entry đã khai, không kiểm completeness). Ba root cause (audit `:103-110`):
1. Scanner chỉ biết pattern `routes/handlers/views/controllers/api`, `models/entities/schema`, `services/lib`, `migrations`, `templates/components/static`, vài config — **không có generic source surface** (Rust `src/*.rs`, Go, Swift...).
2. Adopt copy kit assets (`[1/4]`) **trước** rồi mới gọi map (`[4/4]`) → scanner map chính asset của kit.
3. `status: draft_needs_review` (single verdict) giảm overclaim nhưng PASS từ validator tạo cảm giác map đáng tin hơn thực tế.

C1–c4 đã chứng minh Rust **==** Bash (parity, bug-for-bug). C5 làm Rust map **CORRECT** (cố ý **≠** Bash buggy) — đúng finding cốt lõi của decomposition (`docs/plans/P077c-decomposition.md:44-52`): parity-oracle và OA-02-oracle mâu thuẫn, nên c5 dùng **oracle khác** (correctness fixtures, KHÔNG "== Bash golden").

### Giải pháp — OA-02 fix 3-part (Rust map.rs + adopt inherit)

**Part 1 — Stack-aware source scanner.** Thêm generic source surface theo stack: Rust `src/**` (crate PHẢI map `src/main.rs`/`src/lib.rs`), Python `*.py` (mở rộng ngoài routes/models đã có), Node (`src/`, `*.ts`/`*.js` ngoài các dir đã match), Go (`*.go`, `cmd/`, `internal/`, `pkg/`), Swift (`Sources/`, `*.swift`), + monorepo/nested package awareness. Fresh scan trên Rust crate chuẩn PHẢI emit một `source`/`rust_src` surface chứa `src/main.rs`.

**Part 2 — Exclude managed kit assets.** Map KHÔNG được map kit-managed assets. **Cơ chế chốt (recommended): static exclude-list of kit-managed root dirs** trong scanner — loại `templates/`, `phieu/`, `scripts/`, `hooks/`, `.claude/` (và các root khác adopt copy vào — Worker verify chính xác set khỏi COPIED list `docs/discoveries/P077c4.md:122-128`) khỏi mọi surface pattern. Một cơ chế bắt 80% (WORKFLOW_V2.2 §0.1: một bệnh một cơ chế rẻ nhất) — giải CẢ standalone `sos map` (repo đã có kit assets từ lần adopt trước) LẪN map-within-adopt (templates/ vừa copy). **Alternative (survey-before-install / reorder adopt `[1/4]↔map`)** = restructure nặng hơn + phá parity fixture c4 froze theo current order → KHÔNG chọn default; escape hatch nếu exclude-list không đủ (xem Escape hatches).

**Part 3 — 3-verdict thay `draft_needs_review` đơn.** AGENT_MAP status field đổi sang verdict tường minh (audit `:122-126`):
- `PATH_VALID` — mọi entry ghi ra tồn tại (property validate-map assert).
- `COVERAGE_UNKNOWN` — chưa có oracle biết đã bao phủ source (default cho fresh machine scan — thay `draft_needs_review`).
- `COVERAGE_REVIEWED` — human/architect đã xác nhận load-bearing surfaces (chỉ human set, map KHÔNG tự set).

Fresh `sos map` emit `COVERAGE_UNKNOWN` (recommended field shape: `status: coverage_unknown` — Worker chốt exact key/value trong CHALLENGE). **`draft_needs_review` KHÔNG được dùng làm routing authority** — orchestrator/Architect đọc `COVERAGE_UNKNOWN` là tín hiệu "map chưa human-reviewed, đừng trust completeness".

### Bash-canonical tension — RECOMMEND Rust-only (escape-hatch ESCALATE nếu 🔴 không đợi được)

OA-02 là bug user gặp NGAY, nhưng orchestrator **lean Rust-only** (fix ship tới user ở P077e cutover — đúng plan). Lý do: (a) `bin/sos.sh` = MVP early, exposure hẹp (n=1 brownfield calibration); (b) fix cả Bash phá invariant xuyên suốt P077c "`bin/sos.sh` GIỮ canonical + KHÔNG đổi" (`docs/plans/P077c-decomposition.md:75`); (c) double-maintain 2 impl. **Default: Rust-only, KHÔNG đụng `bin/sos.sh`.** Đây KHÔNG phải Architect tự quyết đổi "Bash unchanged" — nếu Worker (CHALLENGE) HOẶC Sếp thấy 🔴 user-facing không đợi được tới P077e → đó là **scope-change owner-decision** → dừng, **ESCALATE gate** (Constraint 8), orchestrator hỏi Sếp qua AskUserQuestion. Architect KHÔNG unilaterally fix Bash.

### Scope
- CHỈ sửa: `map.rs` (3-part fix), `adopt.rs` (verify inherit — likely 0 code change), `parity.rs` (flip map/adopt → correctness oracle), `capture.sh` (map/adopt KHÔNG re-capture từ buggy Bash), correctness goldens (`map.agent_map.golden`, `map.golden`, `adopt.gen.golden`, `adopt.golden` re-froze).
- KHÔNG sửa: `bin/sos.sh` (Bash canonical GIỮ nguyên — invariant P077c), `sync.rs`/`new.rs` + `sync.*.golden`/`new.*.golden` (GIỮ parity — không dính OA-02), `~/doctor` repo (validate-map OFF-LIMITS — nếu cần đổi → FLAG).

---

## Task 0 — Verification Anchors

> Architect envelope: KHÔNG đọc được source (architect-guard block `map.rs`/`adopt.rs`). Anchors dưới nguồn từ c1/c4 discovery reports (đã cite `file:line`) + audit doc. Worker grep-verify mọi `[needs Worker verify]` trước EXECUTE.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Bash `sos_map` pattern list (OA-02 bug source) tại `bin/sos.sh:311-316`; scan sort `:296`, cap `head -25`; KHÔNG có generic `src/*.rs` surface | `grep -n` range `bin/sos.sh:279-352` | ✅ [verified via `docs/discoveries/P077c1.md:7`] |
| 2 | Rust `map.rs` port bug-for-bug: scanner + `hits.sort()` ~`:189`, 25-cap, KHÔNG có source surface | Worker đọc `map.rs`, xác nhận scanner block + surface pattern set | ⏳ [needs Worker verify] |
| 3 | adopt map-within-adopt = subprocess re-invoke `sos map <target>` (KHÔNG sửa map.rs); order-of-op `[1/4]` copy assets `bin/sos.sh:692-804` → `[4/4]` map `:932-955` | `grep -n run_map_subcommand` adopt.rs | ✅ [verified via `docs/discoveries/P077c4.md:159`, `:5`] |
| 4 | Re-froze scope: `map.agent_map.golden` (c1, python fixture routes/models) + AGENT_MAP.yaml content bên trong `adopt.gen.golden` (c4, 8 gen files) — cả hai chứa OA-02 pollution hiện tại | `ls tests/golden/`; đọc `adopt.gen.golden` header | ✅ [verified via `docs/discoveries/P077c4.md:89,126`] |
| 5 | `capture.sh` hiện capture MỌI golden từ Bash (`DOCTOR_BIN=/nonexistent/doctor`); map/adopt correctness golden KHÔNG được re-capture từ Bash buggy (else re-introduce OA-02) | Worker đọc `capture.sh` map/adopt branch | ⏳ [needs Worker verify — design: authored-expected hoặc capture-từ-corrected-Rust-once] |
| 6 | Harness: `PARITY_ENFORCED = &["map","sync","new","adopt"]`; `parity_map_enforced`/`parity_adopt_enforced` hiện assert stdout==golden + file==golden (Bash-captured). C5 flip 2 test này sang correctness-expected; `parity_sync_enforced`/`parity_new_enforced` GIỮ ==Bash | Worker đọc `parity.rs` PARITY_ENFORCED + 2 enforced test | ✅ [verified via `docs/discoveries/P077c4.md:36-38`, `:77`] |
| 7 | `doctor validate-map` — có assert/parse trên status string `draft_needs_review` không? Nếu CÓ → đổi 3-verdict phá validate-map → doctor change cần (OFF-LIMITS) | Worker READ-ONLY grep `draft_needs_review\|coverage\|status` trong `~/doctor/src/**` (KHÔNG edit doctor) | ⏳ [needs Worker verify — nếu CÓ → FLAG ESCALATE, Constraint 9] |
| 8 | Kit-managed root dirs adopt copy (exclude-list source): `templates/`, `phieu/`, `scripts/`, `hooks/`, `.claude/**` + remaps | đối chiếu COPIED list `docs/discoveries/P077c4.md:122-128` | ✅ [verified via c4 discovery] |
| 9 | AGENT_MAP.yaml `status:` field emit location trong `map.rs` (nơi hardcode `draft_needs_review`) | Worker grep `draft_needs_review` trong `map.rs` | ⏳ [needs Worker verify] |

**❌/⏳ handling:** #2,#5,#7,#9 là code-level Architect không đọc được → Worker grep/verify tại CHALLENGE. #7 đặc biệt: nếu validate-map dính status-string → dừng, FLAG owner (doctor OFF-LIMITS).

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Schema: 1 turn = Worker Challenge + Architect Response. Cap = 3.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Anchor verification (recap Task 0):**
- #2 ✅ `map.rs` confirmed bug-for-bug: `SURFACES` const (map.rs:34-83) has exactly the 6 Bash patterns, no Rust/generic source surface; sort-before-cap intact.
- #5 ✅ `capture.sh:336,340` currently runs real Bash and captures both stdout+file for `map` — confirms today's goldens are Bash-sourced/OA-02-polluted; must switch to authored/corrected-Rust mechanism.
- #7 ✅ **CRITICAL, resolved clean.** Read `~/doctor/src/cli/validate_map.rs` (read-only) in full — zero reference to a top-level `status` field anywhere (only walks `edit`/`read_shallow`/`read_deep`/`research_gate`/`contract_test` path/anchor categories). 3-verdict rename is SAFE — no doctor change, no Constraint 9 escalation.
- #9 ✅ Hardcode found at `map.rs:207` (HEAD const) + `map.rs:248` (println echo). Plus a 3rd un-addressed string: `UNMAPPED_STUB` (map.rs:211, `draft_unmapped`) — Task 3 only names 3 verdicts, doesn't map this branch → O1.1.

**Objections (Tầng 1):**
- [O1.1] `UNMAPPED_STUB` (map.rs:211, `status: draft_unmapped`) is a 3rd hardcoded status string Task 3 doesn't address — fold into `COVERAGE_UNKNOWN`, or keep as distinct 4th state?

**Worker accepted V1 — 1 objection (O1.1), resolved via Tầng-2 self-decide (see below), no Architect respond cycle needed.**

**Status:** ✅ RESOLVED

### O1.1 resolution (Tầng-2 self-decide, per orchestrator delegation)

Folded `UNMAPPED_STUB`'s `draft_unmapped` into `COVERAGE_UNKNOWN` — a zero-surface fresh scan is still "unreviewed coverage", just empty; it does NOT become a 4th verdict. Implemented at `map.rs:297` (`UNMAPPED_STUB` const) + `map.rs:321` (echo line). No 4th state survives.

### Final consensus
- Phiếu version: V1 (no revision needed — O1.1 resolved without Architect respond)
- Approved by Chủ nhà: 2026-07-22 (delegated via orchestrator: "CHALLENGE APPROVE V1 → EXECUTE mode... không escalate")

---

## Nhiệm vụ

### Task 1: Stack-aware source scanner (Part 1)

**File:** `bootstrap/sos-rs/crates/sos-cli/src/commands/map.rs`

**Tìm:** scanner block đang emit surface pattern set (routes_handlers/models_schema/services_logic/migrations/frontend/config_runtime — verify anchor #2). KHÔNG có generic source surface.

**Thay bằng / Thêm:** thêm stack-aware source surface(s) TRÊN cơ chế detect stack (manifest presence: `Cargo.toml`→Rust, `pyproject.toml`/`setup.py`→Python, `package.json`→Node, `go.mod`→Go, `Package.swift`→Swift):
- Rust: surface `rust_src` (hoặc `source`) = `src/**` (`src/main.rs`, `src/lib.rs`, nested modules). Crate chuẩn PHẢI xuất hiện `src/main.rs`.
- Python: mở rộng bắt `*.py` ngoài routes/models đã match.
- Node/Go/Swift: `src/`/`*.ts,*.js` · `*.go`/`cmd/`/`internal/`/`pkg/` · `Sources/`/`*.swift`.
- Monorepo/nested: nested packages có expected surfaces (audit acceptance `:130`).

**Lưu ý:** giữ sort-before-cap (anchor #2, deterministic — c1/c4 đã dựa vào). Exact surface naming = design point; Worker chốt tại CHALLENGE, ghi Discovery. KHÔNG phá determinism (cross-platform find-order residual risk documented c4 `:143`).

### Task 2: Exclude managed kit assets (Part 2)

**File:** `map.rs` (cùng scanner).

**Tìm:** vòng scan mọi surface pattern (nơi `templates/` match `frontend`).

**Thay bằng / Thêm:** static exclude-list kit-managed root dirs (anchor #8: `templates/`, `phieu/`, `scripts/`, `hooks/`, `.claude/` + adopt remaps — Worker verify exact set) — skip mọi path dưới các root này khỏi MỌI surface. Giải cả standalone-map lẫn map-within-adopt.

**Lưu ý:** exclude-list là cơ chế **default** (rẻ, bắt 80%). Nếu Worker thấy static list không đủ (repo có kit-managed dir tên khác, hoặc cần managed-manifest thật) → escape hatch B (survey/manifest), ghi Discovery, KHÔNG tự reorder adopt phase mà không flag.

### Task 3: 3-verdict status field (Part 3)

**File:** `map.rs` (anchor #9 — nơi hardcode `draft_needs_review`).

**Tìm:** hardcode `draft_needs_review` trong AGENT_MAP.yaml emit.

**Thay bằng:** `COVERAGE_UNKNOWN` (fresh machine scan). Document 3 verdict (PATH_VALID/COVERAGE_UNKNOWN/COVERAGE_REVIEWED) trong AGENT_MAP schema doc (Docs Gate). Map KHÔNG tự set COVERAGE_REVIEWED.

**Lưu ý:** exact field key/value (`status: coverage_unknown` recommended) = Worker chốt CHALLENGE. **BLOCKER check (anchor #7):** trước khi đổi, verify `~/doctor validate-map` có assert trên `draft_needs_review` không — nếu CÓ → FLAG ESCALATE (Constraint 9), KHÔNG edit doctor.

### Task 4: adopt inherit fix (verify, likely 0 code change)

**File:** `adopt.rs` (verify only).

**Tìm:** `run_map_subcommand()` re-invoke `sos map <target>` (anchor #3).

**Verify:** map-within-adopt tự hưởng fix Task 1-3 (adopt gọi cùng binary). Exclude-list (Task 2) loại `templates/` adopt copy `[1/4]` → `frontend` pollution biến mất. Nếu order-of-op cần đổi (`[1/4]↔map`) → escape hatch B + FLAG. Default: 0 code change adopt.rs.

### Task 5: Flip map/adopt harness → correctness oracle + re-froze goldens

**File:** `crates/sos-cli/tests/parity.rs`, `tests/golden/capture.sh`, correctness goldens.

**Tìm:** `parity_map_enforced` + `parity_adopt_enforced` (assert ==Bash golden); `capture.sh` map/adopt capture branch.

**Thay bằng:**
- `parity_map_enforced`/`parity_adopt_enforced`: assert Rust output == **correctness-expected** golden (KHÔNG ==Bash). Bash map/adopt vẫn buggy → KHÔNG so file-content với Bash.
- `capture.sh`: map/adopt correctness goldens KHÔNG re-capture từ Bash (else re-introduce OA-02). Mechanism = authored-expected HOẶC capture-từ-corrected-Rust-once-then-frozen-as-spec (Worker chốt anchor #5). Guard rõ trong capture.sh comment "map/adopt = correctness oracle, NOT Bash parity".
- Re-froze `map.agent_map.golden` + `map.golden` (verdict word đổi) + `adopt.gen.golden` (AGENT_MAP content) + `adopt.golden` (stdout verdict). `sync.*`/`new.*` KHÔNG đụng (GIỮ parity).

**Lưu ý:** giữ two-fixture pattern c1 (stdout + file-content). Negative test: sabotage → correctness assert fires (như c1 `:52-61`). PARITY_ENFORCED set giữ 4 tên nhưng semantics map/adopt đổi sang correctness — cân nhắc rename const/comment để không lừa reader (design point, ghi Discovery).

### Task 6: Acceptance fixtures (audit đòi `:127-130`)

**File:** `parity.rs` + `capture.sh` (fixture builders).

**Thêm:** fixture(s) chứng minh correctness:
- Rust crate (`Cargo.toml` + `src/main.rs`) → map PHẢI emit source surface chứa `src/main.rs`.
- Kit `templates/` present → KHÔNG thành `frontend` product surface (excluded).
- Repo có `sim/`, `tools/`, `migrations/`, nested packages → expected surfaces xuất hiện.

**Lưu ý:** đây là oracle chính của c5 (hard-fail). Fixture = synthetic self-contained (pattern c2/c3/c4 — không phụ thuộc real repo state).

### Task 7: Docs Gate

Xem section Docs Gate dưới.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-cli/src/commands/map.rs` | Task 1-3: stack-aware scanner + exclude kit assets + 3-verdict |
| `crates/sos-cli/src/commands/adopt.rs` | Task 4: verify inherit (likely 0 code change) |
| `crates/sos-cli/tests/parity.rs` | Task 5-6: flip map/adopt → correctness + acceptance fixtures |
| `crates/sos-cli/tests/golden/capture.sh` | Task 5: map/adopt KHÔNG re-capture Bash |
| `crates/sos-cli/tests/golden/map.agent_map.golden` | Task 5: re-froze correctness |
| `crates/sos-cli/tests/golden/map.golden` | Task 5: re-froze (verdict word) |
| `crates/sos-cli/tests/golden/adopt.gen.golden` | Task 5: re-froze (AGENT_MAP content) |
| `crates/sos-cli/tests/golden/adopt.golden` | Task 5: re-froze (stdout verdict) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `bin/sos.sh` | Bash canonical GIỮ nguyên bug OA-02 (invariant P077c) — `git diff bin/sos.sh` empty |
| `crates/sos-cli/src/commands/sync.rs`, `new.rs` | Parity GIỮ nguyên — không dính OA-02 |
| `tests/golden/sync.*.golden`, `new.*.golden` | Byte-identical (không re-froze) |
| `~/doctor/**` (validate-map) | OFF-LIMITS — chỉ READ verify anchor #7, KHÔNG edit; nếu cần đổi → ESCALATE |

---

## Luật chơi (Constraints)

1. **Correctness oracle, KHÔNG parity.** Map/adopt Rust cố ý **≠** Bash. KHÔNG assert ==Bash golden cho map/adopt file-content. Bash GIỮ buggy.
2. **`bin/sos.sh` KHÔNG đổi** (invariant xuyên P077c `:75`). `git diff bin/sos.sh` PHẢI empty.
3. **sync/new GIỮ parity** — không re-froze `sync.*`/`new.*`, không đổi `parity_sync_enforced`/`parity_new_enforced`.
4. **Exclude-list = default cơ chế Part 2** (một bệnh một cơ chế, WORKFLOW §0.1). Survey/manifest reorder = escape hatch B, cần FLAG.
5. **`draft_needs_review` KHÔNG routing authority** — thay bằng 3-verdict tường minh.
6. **Determinism giữ** — sort-before-cap, synthetic self-contained fixture (pattern c1-c4).
7. **Two-fixture (stdout + file) giữ** — negative test chứng minh correctness assert fires.
8. **ESCALATE gate — Bash-canonical.** Default Rust-only (KHÔNG fix Bash). Nếu Worker (CHALLENGE) HOẶC Sếp phán 🔴 user không đợi được P077e cutover → scope-change owner-decision → dừng, orchestrator AskUserQuestion Sếp. Architect KHÔNG tự đổi "Bash unchanged".
9. **ESCALATE gate — doctor/validate-map.** Nếu anchor #7 thấy validate-map assert trên `draft_needs_review` → đổi 3-verdict cần doctor change (OFF-LIMITS) → FLAG owner, KHÔNG tự edit `~/doctor`. Option: giữ backward-compat value HOẶC escalate doctor phiếu riêng.

---

## Nghiệm thu

### Automated
- [x] `cargo build --workspace` clean
- [x] `cargo test --workspace` — `parity_map_enforced`/`parity_adopt_enforced` (correctness), sync/new (parity) đều pass (8/8 `parity.rs`)
- [x] Acceptance fixtures (Task 6) hard-fail on regression — negative test: sabotaged `rust_src` marker name → 2/3 `oa02_*` fixtures FAILED loud; reverted, all green

### Manual Testing
- [x] Live: Rust crate (`Cargo.toml`+`src/main.rs`) → `sos map` → AGENT_MAP.yaml có source surface chứa `src/main.rs`, KHÔNG có `frontend→templates` (proven via `oa02_rust_crate_maps_src_main` + manual `/tmp` run)
- [x] Live: `sos map` emit `status: coverage_unknown` (không `draft_needs_review`)
- [x] Live brownfield adopt → `docs/AGENT_MAP.yaml` KHÔNG map copied `templates/` là frontend (OA-02 gone) — proven via `parity_adopt_enforced`'s re-froze `docs/AGENT_MAP.yaml` hash + `oa02_templates_excluded_from_frontend`

### Regression
- [x] `git diff bin/sos.sh` empty; sync/new goldens byte-identical
- [x] `bash scripts/trust-gate.sh` exit 0 (`bootstrap/sos-rs/**` không trong baseline — no rebaseline)

### Docs Gate
- [x] `CHANGELOG.md` — entry P077c5
- [x] `bootstrap/sos-rs/README.md` — command parity/status table: map/adopt = "CORRECTNESS oracle (OA-02 fixed, Rust beat Bash)"
- [x] `crates/sos-cli/tests/README.md` — "correctness oracle vs parity oracle" section (map/adopt khác sync/new)
- [x] `docs/plans/P077c-decomposition.md` — mark P077c5 SHIPPED (c5 done, P077c CLOSED)
- [x] `docs/retro/OUTSIDER_AUDIT_SYSTEM_GAPS_2026-07-22.md` — OA-02 mark addressed (Rust-only; Bash defer P077e)
- [x] 3-verdict AGENT_MAP.yaml format documented inline (map.rs HEAD const comment + README/tests-README) — no separate schema doc file exists in repo. validate-map contract KHÔNG đổi (anchor #7 clean) — no doctor FLAG needed.

### Discovery Report
- [x] `docs/discoveries/P077c5.md`:
  - Anchors #1-9 CORRECT/WRONG (file:line)
  - Part 2 mechanism dùng: exclude-list (default) hay escape hatch B (survey/manifest) — lý do
  - Bash-canonical decision: Rust-only proceed hay ESCALATE fired — outcome
  - validate-map (anchor #7): dính status-string không → doctor FLAG hay clean
  - Re-froze scope thực tế (goldens) + acceptance fixture kết quả
  - Exact 3-verdict field key/value chốt
  - Tier escalations (None nếu giữ Tầng 1)
- [x] Append 1-line index `docs/DISCOVERIES.md`
