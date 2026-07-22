# PHIẾU P078c: Tách adapter-render khỏi tool-manifest gate (unblock P079 dogfood)

> **Loại:** Bugfix (concern-conflation trong install flow)
> **Ưu tiên:** P1 (block P079 Codex self-dogfood)
> **Tầng:** 1 — đụng install engine flow + đổi behavior của OA-07 gate (contract surface: đổi TỪ "block render" SANG "render + loud report", đổi exit-code convention của `sos install`). Sếp-ratified.
> **Lane:** Guarded
> **Ảnh hưởng:** `crates/sos-cli/src/commands/install.rs` (flow reorder + `--require-tools` flag + exit-code), có thể `crates/sos-cli/src/main.rs` (exit-code plumbing). KHÔNG đổi render content, KHÔNG đổi tool-check logic core.
> **Dependency:** P077d2 (install engine) + P077d3 (tool-manifest gate) DONE. P078a/b (Codex adapter render) DONE — cung cấp `CodexAdapter.plan()` sinh AGENTS.md/.codex/**.

---

## Context

### Vấn đề hiện tại

Flow của `sos install` trong `crates/sos-cli/src/commands/install.rs` (per-runtime `run_codex()` / `run_claude()`) chạy 3 bước theo thứ tự (orchestrator verified, cần Worker re-verify tên hàm + call-order sau P078a/b restructure):

1. `adapter.plan(&capabilities)` — build render plan.
2. **`engine::resolve_tools()?`** — tool-manifest check (P077d3 / OA-07). Dấu `?` **HARD-FAIL TRƯỚC `apply()`**.
3. `engine::apply(...)` / `engine::dry_run(...)` — write adapter files.

Hệ quả (reproduce live per `docs/discoveries/P077d3.md`): máy có tool drift (doctor 0.1.1 < pinned 0.1.3, `inv-gate` MISSING) → `resolve_tools()` trả `Err` → `?` abort → **KHÔNG render/write file adapter nào** (scratch dir confirmed empty). → P079 dogfood KHÔNG install được Codex adapter chỉ vì sister-tools lệch version.

**Concern-conflation:** render adapter files (AGENTS.md / `.codex/**`) KHÔNG phụ thuộc sister-tools đúng version. Tool-manifest gate đo "workflow-ready" (tool nào chạy được lúc EXECUTE), KHÔNG nên gate "render files ra đĩa". Hai concern bị trộn ở dấu `?`.

### Giải pháp

Tách concern — **GIỮ nguyên tín hiệu OA-07**, chỉ đổi TỪ "block render" SANG "render + loud report":

1. **Reorder:** `apply()` / `dry_run()` (render + write) chạy TRƯỚC / độc lập tool-check. Files được ghi bất kể tool-drift.
2. **Tool-check → report, KHÔNG block-render:** required-tool drift/missing → in WARNING **loud** (giữ OA-07 intent: KHÔNG silent-run trên stale tools) + liệt kê tool + pointer `sos tools status`. KHÔNG `?`-propagate để abort render.
3. **Exit semantics (CHỐT — xem "Quyết định thiết kế"):** render OK + tool-drift → **exit 3** (DISTINCT "installed, tools-not-ready"); render fail → exit 1 (nguyên trạng); render OK + tools OK → exit 0.
4. **`--require-tools` opt-in (CHỐT: CÓ):** flag fail-fast cho CI/production — khôi phục hành vi cũ (tool-check TRƯỚC apply, abort exit 1, KHÔNG render). Default = render+warn+exit3.
5. **dry-run:** show CẢ plan CẢ tool-drift warning.
6. **Symmetric:** áp CẢ `run_claude()` + `run_codex()` (dù `ClaudeAdapter.plan()` còn stub, flow phải nhất quán).

### Quyết định thiết kế (Architect CHỐT — brief để Architect quyết)

- **Exit code 3 (distinct), KHÔNG exit-0+warning.** Lý do: CI/script pipe `sos install` cần đọc `$?` để phân biệt 3 trạng thái — (a) `0` = render OK + tools ready (an toàn EXECUTE ngay); (b) `3` = render OK nhưng tools stale (files đã ghi, an toàn, chỉ cần update tools trước khi chạy workflow); (c) `1` = render THẬT SỰ fail (nothing/partial written, phải retry). exit-0+warning nuốt trạng thái (b) vào (a) → script không phân biệt được → đúng bệnh OA-07 "silent-run stale tools" ở tầng exit-code. exit-1 nuốt (b) vào (c) → script tưởng install hỏng trong khi files đã ghi OK. → 3 exit code = 3 trạng thái, machine-readable.
- **`--require-tools` = CÓ (opt-in fail-fast).** Lý do: giữ dạng MẠNH NHẤT của OA-07 (fail-closed, không render khi tools chưa sẵn) làm lựa chọn cho CI/production install, trong khi default render+warn để unblock dogfood. Rẻ (chỉ gate `apply()` có điều kiện) + đúng brief "1 mode --require-tools cho CI". KHÔNG weaken OA-07: người cần fail-closed vẫn có nút.
- **OA-07 preserved cách nào:** (a) drift LUÔN surfaced loud (WARNING + tool list + pointer), không bao giờ nuốt; (b) exit non-zero (3) → script phát hiện được; (c) `sos tools status` UNCHANGED (vẫn exit 1 on drift — check riêng, đúng); (d) `--require-tools` khôi phục full fail-closed gate. Chỉ đổi "block render" → "render + loud report", tín hiệu drift sống nguyên.

### Scope
- CHỈ sửa `crates/sos-cli/src/commands/install.rs` (flow order + flag + exit signalling) + tối thiểu `crates/sos-cli/src/main.rs` nếu cần plumb exit-code 3.
- KHÔNG sửa `crates/sos-install/src/engine.rs` render/apply/dry_run logic (chỉ đổi CÁCH GỌI ở CLI, không đổi engine).
- KHÔNG sửa `crates/sos-install/src/tools.rs` (`check_tools`/`gate_required` core — logic drift-detect giữ nguyên; chỉ đổi CHỖ và CÁCH install.rs consume kết quả).
- KHÔNG đổi `sos tools status` (vẫn exit 1 on drift).
- KHÔNG đổi render content của bất kỳ adapter nào.

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `install.rs` có per-runtime fn (`run_codex()` / `run_claude()` hoặc tương đương) với flow `plan()` → `resolve_tools()?` → `apply()`/`dry_run()` | `grep -n "fn run_\|resolve_tools\|\.apply\|dry_run\|\.plan" crates/sos-cli/src/commands/install.rs` | ⏳ `[needs Worker verify]` — P078a/b có thể đã restructure `run()` d2 thành per-runtime fn; xác nhận tên hàm + đúng thứ tự 3 bước + vị trí dấu `?` |
| 2 | `engine::resolve_tools()` signature = `Result<Vec<ToolStatus>>`, hard-fail hiện qua `?` ở install.rs (nguồn: `docs/discoveries/P077d3.md` "Assumptions — adapted") | `grep -n "fn resolve_tools\|resolve_tools()?" crates/sos-install/src/engine.rs crates/sos-cli/src/commands/install.rs` | ⏳ `[needs Worker verify]` — xác nhận kiểu trả về + là dấu `?` này cần đổi thành capture-không-propagate |
| 3 | `engine::apply(...)` + `engine::dry_run(...)` là điểm write; `gate_required()`/`check_tools()` là core drift-check ở `tools.rs` (nguồn: P077d2/d3 discovery) | `grep -n "pub fn apply\|pub fn dry_run\|pub fn gate_required\|pub fn check_tools" crates/sos-install/src/engine.rs crates/sos-install/src/tools.rs` | ⏳ `[needs Worker verify]` — xác nhận signature để install.rs gọi tool-check SAU apply mà không cần đổi engine |
| 4 | Cách `sos install` trả exit-code (install.rs trả `anyhow::Result` → `main.rs` map `Err`→exit 1?) — cần biết để thêm exit 3 distinct | `grep -n "process::exit\|-> Result\|ExitCode\|fn main" crates/sos-cli/src/main.rs crates/sos-cli/src/commands/install.rs` | ⏳ `[needs Worker verify]` — Worker chọn cơ chế: custom `ExitCode`/`process::exit(3)` sau apply-success khi drift, hay return-type mang exit-hint. Contract = exit 3, cơ chế Tầng-2 |
| 5 | `sos tools status` exit 1 on drift, độc lập install (nguồn: `docs/discoveries/P077d3.md` "Manual testing") | `grep -n "Tools\|status" crates/sos-cli/src/commands/*.rs` + đọc dispatch | ⏳ `[unverified]` (từ discovery) — Task này KHÔNG đụng; chỉ regression-verify vẫn exit 1 |
| 6 | OA-07 intent = "đừng silent-run trên stale tools" (KHÔNG phải "block render") | Đọc `docs/retro/OUTSIDER_AUDIT_SYSTEM_GAPS_2026-07-22.md` OA-07 | ✅ `[verified]` — Architect đã đọc; evidence là version-drift → "repo contract giả định tool mới nhưng runtime dùng binary cũ". Intent = surface drift, không block ghi file |
| 7 | `CodexAdapter.plan()` sinh operations cho AGENTS.md + `.codex/**` (P078a/b) | `grep -rn "AGENTS.md\|.codex" crates/sos-adapter-codex/src/ crates/sos-cli/src/commands/install.rs` | ⏳ `[needs Worker verify]` — xác nhận đúng target paths để oracle "files-written-on-drift" kiểm được |

**❌/⚠️ handling:** nếu anchor #1 lộ install.rs vẫn là 1 `run()` chung (chưa per-runtime) → Worker CHALLENGE: fix áp cho `run()` chung, "symmetric claude+codex" tự động thỏa vì cùng 1 hàm; ghi Discovery. Nếu anchor #4 lộ không có đường trả exit-code distinct sạch → Worker đề xuất cơ chế ở CHALLENGE (Tầng-1 vì đổi exit-code contract).

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Anchor verification (recap from Task 0):**
- #1 ✅ `install.rs:24-125`: 2 separate fns `run_claude`/`run_codex`, both `plan()→resolve_tools()?→apply()/dry_run()`.
- #2 ✅ `engine.rs:308` `resolve_tools() -> Result<Vec<ToolStatus>>`, called via `?` at install.rs:47/93.
- #3 ✅ `engine.rs:186` `apply(project_root, plan, owner, source_version)` + `engine.rs:148` `dry_run(project_root, plan)` — neither depends on tool state; `engine.rs:183-185` doc comment explicitly confirms tool-resolve is a separate CLI-wired step.
- #4 ✅ `main.rs` returns `anyhow::Result<()>` uniformly; `tools.rs:51`/`launch.rs:34` already use `std::process::exit(1)` directly — same pattern trivially extends to `exit(3)`, no `main.rs` plumbing needed.
- #5 N/A this task (not touched).
- #6 ✅ verified against `docs/retro/OUTSIDER_AUDIT_SYSTEM_GAPS_2026-07-22.md`.
- #7 ✅ `templates.rs:87-103` confirms real target paths; `lib.rs:401` proves CodexAdapter not a stub (install.rs comment stale — Tầng-2 note).

**Objections:** None — accept V1. Bonus finding logged for EXECUTE: `tools::check_tools()`/`required_drift()`/`describe_failure()` already `pub` non-`Result`, so zero new engine/tools code needed for the capture-don't-propagate requirement.

**Status:** ✅ ACCEPTED — no challenges, ready for Chủ nhà approval gate.

### Final consensus
- Phiếu version: V1 (no revision needed — Worker accepted without objection)
- Approved by Chủ nhà: 2026-07-22 (CHALLENGE APPROVE V1 → EXECUTE delegated)

---

## Nhiệm vụ

### Task 1: Reorder `run_codex()` — render-before-toolgate

**File:** `crates/sos-cli/src/commands/install.rs`

**Tìm:** đoạn trong hàm install runtime=codex chạy `resolve_tools()?` (hoặc equivalent) TRƯỚC `apply()`/`dry_run()`, với dấu `?` propagate error làm abort. `[needs Worker verify anchor #1,#2]`

**Thay bằng / logic mới:**
1. Chạy `adapter.plan()` → `engine::apply(...)` (hoặc `dry_run()` nếu `--dry-run`) TRƯỚC. Files ghi ra đĩa bất kể tool-state.
2. SAU khi apply thành công (hoặc plan-printed cho dry-run): gọi tool-check (`resolve_tools()` / `check_tools()` — capture kết quả, KHÔNG `?`-propagate).
3. Nếu có required-tool drift/missing:
   - In WARNING **loud** ra stderr: dòng tiêu đề rõ ("⚠️ tools not workflow-ready — adapter files installed but ...") + liệt kê từng tool drift/missing (tên + expected + found), pointer `→ run 'sos tools status' / update tools before running the workflow`.
   - Set exit-status = **3** (installed, tools-not-ready).
4. Nếu tools OK → exit 0 như thường.
5. Nếu `--require-tools` set (Task 3) → tool-check chạy TRƯỚC apply, drift → abort exit 1, KHÔNG render (hành vi cũ, opt-in).

**Lưu ý:** giữ nguyên message/format của tool-drift report từ `tools.rs`/`sos tools status` nếu tái dùng được (đừng bịa format mới) — chỉ đổi CHỖ gọi + KHÔNG abort. Nếu apply THẬT SỰ lỗi (io error) → vẫn exit 1 với rollback nguyên trạng của engine (KHÔNG nuốt render-fail thành exit 3).

### Task 2: Symmetric cho `run_claude()`

**File:** `crates/sos-cli/src/commands/install.rs`

**Tìm:** hàm install runtime=claude/auto (cùng shape 3 bước). `[needs Worker verify anchor #1]`

**Thay bằng:** áp Y HỆT logic Task 1 (render-before-toolgate + report + exit 3 + `--require-tools`). Dù `ClaudeAdapter.plan()` còn stub (render empty/minimal per P077d2), flow phải nhất quán để hai runtime đối xứng.

**Lưu ý:** nếu install.rs thực chất là 1 hàm chung (không per-runtime) → sửa 1 chỗ, ghi Discovery rằng "symmetric tự thỏa vì shared path". Đừng nhân bản code chỉ để có 2 hàm.

### Task 3: Thêm `--require-tools` flag

**File:** `crates/sos-cli/src/commands/install.rs` (+ clap definition nơi `--runtime`/`--dry-run` khai báo — `[needs Worker verify]` có thể ở `main.rs`).

**Thêm:** boolean flag `--require-tools` (default false). Khi set → khôi phục fail-fast: tool-check TRƯỚC apply, required drift/missing → abort exit 1, KHÔNG render. Doc-comment: "Fail-closed: abort install if required tools are not pin-current (CI/production). Default: render adapter files then warn on drift (exit 3)."

**Lưu ý:** `--require-tools` + `--dry-run` cùng lúc → tool-check báo drift nhưng KHÔNG cần abort (dry-run không mutate gì) — Worker chốt hành vi nhất quán (lean: dry-run luôn show-both, `--require-tools` chỉ đổi exit-code của dry-run thành 1 nếu muốn CI gate cả dry-run; Tầng-2 self-decide, log Discovery).

### Task 4: Exit-code 3 plumbing

**File:** `crates/sos-cli/src/commands/install.rs` + `crates/sos-cli/src/main.rs` `[needs Worker verify anchor #4]`

**Thêm:** cơ chế trả exit 3 distinct cho "render OK + tool-drift". Worker chọn cơ chế sạch nhất với codebase (`std::process::ExitCode`, `process::exit(3)` sau apply-success, hoặc return-type mang hint). Contract cứng: **exit 3 = installed-but-tools-not-ready**, phân biệt với exit 1 (render fail) + exit 0 (all ready).

**Lưu ý:** đây là exit-code CONTRACT (Tầng-1) — nếu Worker thấy exit 3 va chạm convention có sẵn của repo (vd exit 2 = clap parse error, exit 1 = generic) → CHALLENGE đề xuất mã khác, miễn distinct + non-zero + documented.

### Task 5: dry-run show-both

**File:** `crates/sos-cli/src/commands/install.rs`

**Xác nhận:** nhánh `--dry-run` in CẢ plan (per-target verb, đã có từ P077d2) CẢ tool-drift warning. `[needs Worker verify]` nhánh dry-run hiện có gọi tool-check chưa — nếu chưa, thêm tool-check report vào (không abort).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-cli/src/commands/install.rs` | Task 1-5: reorder render-before-toolgate, report-not-block, `--require-tools`, exit 3, dry-run show-both |
| `crates/sos-cli/src/main.rs` | Task 4 (nếu cần): exit-code 3 plumbing / clap flag wiring `[needs Worker verify]` |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `crates/sos-install/src/engine.rs` | `apply()`/`dry_run()`/`resolve_tools()` logic KHÔNG đổi — chỉ đổi cách install.rs gọi |
| `crates/sos-install/src/tools.rs` | `check_tools()`/`gate_required()` drift-detect KHÔNG đổi; tái dùng report format nếu được |
| `crates/sos-cli/src/commands/*.rs` (tools status dispatch) | `sos tools status` vẫn exit 1 on drift (regression) |
| `crates/sos-adapter-codex/src/**` | render content AGENTS.md/.codex/** KHÔNG đổi |

---

## Luật chơi (Constraints)

1. **KHÔNG weaken OA-07 cho use-case workflow-ready.** Drift PHẢI báo loud (WARNING + tool list + pointer), exit non-zero. Chỉ đổi "block render" → "render + report". Nếu nuốt drift (exit 0 im lặng) = vi phạm, phiếu FAIL.
2. **KHÔNG đổi engine/tools core logic.** Chỉ đổi ORDER + fatal→report ở tầng CLI. `git diff crates/sos-install/` chỉ được rỗng hoặc thuần additive (nếu cần expose 1 hàm non-`?` variant).
3. **Additive-ish:** KHÔNG đổi render content, KHÔNG đổi tool-check detect logic.
4. **`sos tools status` UNCHANGED** — exit 1 on drift (check riêng, đúng).
5. **Symmetric:** claude + codex cùng flow (hoặc shared path chứng minh tự-symmetric).
6. **Dep direction giữ nguyên** — `dep_direction.rs` guard vẫn green (không đưa host token vào sos-core/sos-install).
7. Cite ranges khi port format, KHÔNG cite counts.

---

## Nghiệm thu

### Automated
- [x] `cargo build --workspace` clean
- [x] `cargo test --workspace` green
- [x] `cargo test --workspace` × 20 TRUE parallel (`seq 1 20 | xargs -P 20 -I{} cargo test --workspace`) — 0 flaky
- [x] `cargo clippy --workspace` — no NEW warning (pre-existing `sync.rs:102` OK)
- [x] `dep_direction.rs` `sos_core_stays_host_neutral` guard — green
- [x] `git diff bin/sos.sh install.sh` — empty (Bash legacy path untouched)

### Manual Testing (Oracle)
- [x] **Oracle chính:** `sos install --runtime codex` trên máy tool-drift (doctor 0.1.1 < 0.1.3, inv-gate MISSING) → **VẪN write adapter files** (17 file, `AGENTS.md` + `.codex/**` xuất hiện ở target — `find` confirm) + drift báo **loud** (WARNING + tool list: 6 required tool) + **exit 3** (distinct).
- [x] `sos install --runtime claude` symmetric — render (ClaudeAdapter stub → `.sos-manifest.toml` only) + drift warning + exit 3.
- [x] `sos install --runtime codex --require-tools` trên máy drift → abort **exit 1**, KHÔNG render (scratch dir empty, `find` confirm 0 file).
- [x] `sos install --runtime codex --dry-run` trên máy drift → in CẢ 17 would-CREATE plan CẢ drift warning, zero mutation (`find` confirm 0 file), exit 3.
- [x] `sos install --runtime codex` trên máy tools-current (mock qua fake PATH binaries in pinned versions) → render + **exit 0**, no warning.
- [x] `--require-tools --dry-run` (Tầng-2 CHỐT) → gate trước, exit 1, zero output/files (bonus scenario tested).
- [ ] Render THẬT lỗi (io error inject/parent occupied) → covered by existing `sos-install` rollback fixtures (`install.rs` tests), not independently re-injected this phiếu (apply()/rollback path untouched by P078c).

### Regression
- [x] `sos tools status` vẫn exit 1 on drift (OA-07 dedicated check — KHÔNG đụng)
- [x] P077d2 install fixtures (`crates/sos-install/tests/install.rs` 6 test) pass
- [x] P077d3 tools fixtures (`crates/sos-install/tests/*` 6 test) pass
- [x] P078a/b Codex adapter tests pass (c1-f / P078a / P078b render fixtures)
- [x] `crates/sos-cli/tests/parity.rs` 8/8 + goldens untouched

### Docs Gate (Tầng 1 — BẮT BUỘC)
- [x] `docs/PORTABILITY_ARCHITECTURE.md` — install flow "render-before-toolgate" (reorder + exit-code 3 convention + `--require-tools` opt-in); cập nhật/nối sau P078b3 status line
- [x] `docs/retro/OUTSIDER_AUDIT_SYSTEM_GAPS_2026-07-22.md` — OA-07 section: nối note "resolved-differently P078c: render decoupled từ tool-gate; drift surfaced qua loud-warn + exit 3 thay vì block; `--require-tools` giữ fail-closed opt-in" (audit body KHÔNG rewrite)
- [x] `CHANGELOG.md` — entry `[P078c]` trên `[P077d3]` (actually placed above `[P078b3]`, top of v2.3 forge section — newest-on-top)
- [x] `SECURITY.md` — updated (not N/A): new section "Install: tool-version drift (OA-07) is workflow-safety, not a trust boundary" — chose to update because SECURITY.md already had adjacent Codex-adapter sections a reader could reasonably wonder about.

### Discovery Report
- [x] Write `docs/discoveries/P078c.md`
  - Anchors #1-7 CORRECT / WRONG (file:line citations) — đặc biệt #1 (per-runtime vs shared `run()`), #4 (exit-code mechanism)
  - Exit-code cơ chế đã chọn (ExitCode / process::exit / return-hint)
  - `--require-tools` + `--dry-run` interaction đã chốt
  - SECURITY.md verdict (updated / N/A + lý do)
  - Docs updated (list) / Tier escalations (None nếu giữ Tầng 1)
- [x] Append 1-line index vào `docs/DISCOVERIES.md`
