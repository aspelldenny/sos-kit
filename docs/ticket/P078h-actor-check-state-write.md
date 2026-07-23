# PHIẾU P078h: actor-check state-write hardening — block marked-actor approval advance (#3) + path normalize (#4)

---

> **Loại:** Bugfix (security — actor-check / approval-gate guard logic)
> **Ưu tiên:** P1 — 2 lỗ trong actor-check guard ship ở P078e (`dd4594d`); self-approve hole = security integrity.
> **Tầng:** 1 — chạm approval-gate / actor-check guard logic (security boundary, mở rộng P078e/d2a) = AUTO Tầng 1 dù nhỏ. LOC KHÔNG quyết.
> **Lane:** Guarded — security guard logic, no-cap (real-payload fixture matrix + negative-test bắt buộc, không always-green). SECURITY → Worker CHALLENGE bắt buộc.
> **Ảnh hưởng:** `crates/sos-adapter-codex/src/templates.rs` (guard content-fn: actor-check block-all + path-normalize); `crates/sos-adapter-codex/src/lib.rs` test module.
> **Dependency:** None — độc lập, SONG SONG với **P078g** (install/arm-hooks, gap#1). Nền = P078e SHIPPED (`dd4594d`, actor-check exemption + marker precedent). Spec = `docs/adapters/P079-ROUND3-FINDINGS-2026-07-23.md` (gap #3 + #4, round-3 live-confirmed).

---

## Context

### Vấn đề hiện tại

P079 **round-3** live-dogfood (`docs/adapters/P079-ROUND3-FINDINGS-2026-07-23.md`) — sau P078e ship actor-check exemption — phơi 2 lỗ trong guard logic:

**Gap #3 — Actor-check chỉ so state CŨ, cho advance self-approve (SECURITY hole):**
Với marker `.sos-state/worker-active` PRESENT (marked actor = subagent, không phải main-thread), state đã approved `V3/V3` VẪN đổi được → `V4/V4` bằng state-only patch. Live repro (findings §B4 + §"distinct actor-check logic bypass", `:159-175`):
```text
MANUAL_MARKER=present
patch: completed
-version=V3
-approved_version=V3
+version=V4
+approved_version=V4
```
- **Root cause:** actor-check exemption (P078e) chỉ chặn **create / first-approval** (nhánh state-alone + marker-absent → ALLOW; marker-set → KHÔNG exempt). Nhưng khi marker SET, guard **fall-through** về so **pre-patch state** — thấy `V3 == approved_version V3` (đã approved) → coi hợp lệ → ALLOW advance. Gỡ riêng state-file exemption KHÔNG đủ: fallback version-match vẫn cho advance (findings `:162-165`).
- **FIX:** marked actor (`.sos-state/worker-active` **HOẶC** `.sos-state/architect-active` PRESENT = không phải main-thread) → **từ chối MỌI ghi `.sos-state/ticket-state.env`**, BẤT KỂ pre-patch approval state (create / update / advance V3→V4 đều BLOCK). KHÔNG fall-through về so V==V. Main-thread (cả 2 marker ABSENT) → giữ exemption như P078e (deadlock fix không regress).

**Gap #4 — Path-match exact-string, false-block absolute path (fragile):**
State-file exemption so **exact relative string** `.sos-state/ticket-state.env`. Live: một lần Codex emit **absolute patch path** → bị chặn nhầm (false-block); cùng thao tác với relative header thì lọt đúng (findings §"Additional usability observation" `:206-212`).
- **FIX:** normalize repo-absolute patch path → relative repo-root **TRƯỚC** khi apply state-file-allow + multi-path (d2a all-path) match. Giảm prompt-shape dependence + giảm nguy cơ path-trick bypass multi-path guard.

**⚠️ Codex caveat (giữ nguyên P078e, ghi lại CAPABILITY + Discovery):** gap #2 (findings — custom-role SubagentStart marker KHÔNG fire trên Codex 0.145.0, upstream `openai/codex#21753`) NGOÀI phạm vi — KHÔNG fix được. Nghĩa: trên **Codex** actor-check là **best-effort** (real subagent path → markers absent → actor-check KHÔNG kích hoạt); **FULL trên Claude** (markers fire in-subagent). Git backstop (P078g arm-hooks) + human-review-at-commit là net. Gap #3 fix vẫn đúng cho manual-marker + Claude path; KHÔNG khai full-protection Codex (ship≠chạy).

### Giải pháp

Additive, 1 area (`sos-adapter-codex`), 2 sửa guard content-fn trong `templates.rs` + test:
- **Task 1 (gap #3):** actor-check → marked-actor block-ALL ticket-state.env write (KHÔNG fall-through version-match). Main-thread giữ exemption.
- **Task 2 (gap #4):** normalize absolute→relative repo-root path TRƯỚC state-file-allow + multi-path match.
- **Task 3:** real-payload fixture matrix + negative-test.

### Scope

- **CHỈ sửa:** `crates/sos-adapter-codex/src/templates.rs` (guard content-fn: actor-check block-all + path-normalize); `crates/sos-adapter-codex/src/lib.rs` test module; docs (CAPABILITY/SECURITY/CHANGELOG/Discovery/BACKLOG).
- **KHÔNG sửa:** `crates/sos-install/**`, `crates/sos-cli/**`, engine (= **P078g** arm-hooks surface); d2a multi-path guard core (nương KHÔNG regress, chỉ chèn normalize trước); SubagentStart marker touch (d2b); `sos-core`, `sos-adapter-claude`; `scripts/orchestrator-guard.sh` (precedent, đọc-không-sửa).

---

## Task 0 — Verification Anchors

> Architect docs-only. Round-3 fact = `[verified: P079-ROUND3-FINDINGS-2026-07-23.md]`. P078e-shipped code-site = `[verified: P078e phiếu Task 0]` (P078e Worker đã grep-confirm) nhưng LINE cụ thể có thể drift post-P078e/P078f → Worker re-grep, đánh `[needs Worker verify]`.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Guard content-fn ở `crates/sos-adapter-codex/src/templates.rs` render actor-check exemption (P078e): state-alone + BOTH markers absent → exempt; marker SET → KHÔNG exempt. `[needs Worker verify exact fn+line range]` | `rg -n "worker-active\|architect-active\|ticket-state\|approved_version\|STATE_FILE" crates/sos-adapter-codex/src/templates.rs` → tìm actor-check + exemption branch | ⏳ TO VERIFY |
| 2 | **Gap #3 root cause:** khi marker SET, guard KHÔNG BLOCK ngay mà fall-through về version-match so pre-patch (`$version == $approved_version` → coi approved → ALLOW advance). `[verified: findings :159-175; needs Worker verify exact fall-through branch trong templates.rs]` | Đọc findings §B4 `:159-175`; `rg -n "approved_version\|version=\|-eq\|==\|BLOCK\|exit 2" templates.rs` → xác nhận marker-set path KHÔNG có early-BLOCK, rơi version-compare | ⏳ TO VERIFY |
| 3 | Marker path chính xác `.sos-state/worker-active` + `.sos-state/architect-active` (khớp SubagentStart touch d2b + P078e actor-check + `lib.rs` helper `with_worker_active`/`with_architect_active`). `[verified: P078e Task 0 anchor #3]` | `rg -n "worker-active\|architect-active" crates/sos-adapter-codex/src/` → filename+dir khớp cả touch + check + test | ⏳ TO VERIFY |
| 4 | **Gap #4:** state-file-allow so **exact relative string** `.sos-state/ticket-state.env`; absolute patch path → không khớp → false-block. `[verified: findings :206-212; needs Worker verify path-extract + compare site]` | `rg -n "ticket-state.env\|head -n1\|while read\|for .*path\|repo\|realpath\|pwd" templates.rs` → xác nhận compare = exact-string, chưa normalize | ⏳ TO VERIFY |
| 5 | d2a multi-path guard (all-path block-if-any) parse mọi apply_patch path block-if-state-coupled; path-normalize phải chèn TRƯỚC nó, KHÔNG regress. `[verified: P078e/d2a]` | `rg -n "for\|while read\|block\|multi\|all.path" templates.rs` → xác nhận all-path loop; normalize áp mỗi path trước match | ⏳ TO VERIFY |
| 6 | Test harness `run_guard()` `lib.rs` (P078e `:582-616` `setup: impl FnOnce(&PathBuf)` closure hỗ trợ ghi marker + payload fixture). `[verified: P078e Task 0 anchor #5]` | `rg -n "run_guard\|setup:\|FnOnce\|fixtures" crates/sos-adapter-codex/src/lib.rs` → harness hỗ trợ marker+payload | ⏳ TO VERIFY |

**Nếu ❌ (gap #3/#4 KHÔNG tồn tại như spec — guard đã block-all marked-actor / đã normalize path) → Worker DISCOVERY_REPORT + dừng, bug không như spec KHÔNG "sửa mò".**
**Nếu anchor #2/#3 sai (marker path lệch → actor-check no-op; hoặc fall-through KHÔNG như findings mô tả) → ESCALATE NGAY: security regression, KHÔNG land, báo orchestrator.**

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

> Cap = 3 turns. SECURITY → Worker CHALLENGE bắt buộc. **Append-only.**

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Anchor verification (grep-confirmed — all 6, no discrepancy from spec):**
- #1/#2/#3 ✅ Guard fn `guard_approval_gate_sh()` = `templates.rs:667-828`. `ACTOR_IS_MAIN_THREAD` set `:789-791` (marker paths exactly `.sos-state/{worker,architect}-active`, matches `lib.rs:672-680` helpers + `run_guard()` harness `lib.rs:634-668`). **Gap #3 fall-through CONFIRMED REAL:** exemption `if :801-805` requires `ACTOR_IS_MAIN_THREAD==1`; marker-SET falls to `:807-824` which reads pre-patch `STATE_FILE` and only blocks if `APPROVED != VERSION` (`:815`) → already-approved `V3/V3` passes equality → `exit 0` at `:825` = genuinely ALLOWS advance. Bug real, not already fixed.
- #4 ✅ exact-string compare `:783` `STATE_FILE=".sos-state/ticket-state.env"` vs `:803`; no normalize anywhere (`rg normalize|realpath` = 0). `REPO_ROOT` already computed `:746` (`git rev-parse --show-toplevel || pwd`) — reusable, no strip util, Worker adds inline.
- #5 ✅ d2a all-path loop builds `NON_TICKET_PATHS` `:772-781` — normalize insert per-path inside/before this loop so both state-file-allow (`:803`) + multi-path block consume normalized paths.
- #6 ✅ `run_guard()`/`with_worker_active`/`with_architect_active`/`synthetic_apply_patch` present; fixture `tests/fixtures/codex-apply-patch-payloads.jsonl` still wired `include_str!` `lib.rs:624`.

**Objections (Tầng 1 only):** Worker accepted V1 — no challenges. Spec matches code exactly.

**Status:** ✅ ACCEPTED V1 — no Architect response needed

### Final consensus
- Phiếu version: V1 (accepted as-drafted)
- Total turns: 1 (Worker accepted, 0 objections)
- Approved by Chủ nhà: 2026-07-23 — self-approved by Quản đốc under delegated sprint authority (P076→P081, self-approve in-scope; Guarded/Tầng-1 security guard, no owner-decision, no Codex run required). EXECUTE insertion points: early-BLOCK after `templates.rs:791`; normalize inside `for RAW_PATH` loop `:773-779` reusing `REPO_ROOT` `:746`; tests in `mock_payload_oracle` module `lib.rs:602+`.

---

## Nhiệm vụ

### Task 1: gap #3 — marked-actor → BLOCK MỌI ghi `.sos-state/ticket-state.env` (KHÔNG fall-through version-match)

**File:** `crates/sos-adapter-codex/src/templates.rs` — guard content-fn, actor-check + exemption branch (P078e-shipped, `[needs Worker verify exact fn+line]`).

**Tìm:** nhánh actor-check hiện tại: state-alone patch → nếu BOTH marker absent → exempt ALLOW; nếu marker SET → KHÔNG exempt → **fall-through** về version-match so pre-patch (`$version == $approved_version` → coi approved → ALLOW advance V3→V4). Đây là lỗ gap #3.

**Thay bằng / Thêm:** thêm **early-BLOCK cho marked actor** TRƯỚC mọi version-compare:
- Nếu `.sos-state/worker-active` **HOẶC** `.sos-state/architect-active` PRESENT **VÀ** patch touch `.sos-state/ticket-state.env` (bất kỳ path nào trong patch, sau normalize gap #4) → **BLOCK NGAY** (exit non-zero, message rõ "control-plane write by marked actor: blocked"). KHÔNG so pre-patch version, KHÔNG fall-through.
- Main-thread (BOTH marker ABSENT) → giữ nguyên P078e exemption (state-alone → ALLOW create+update; bundle → all-path #6 BLOCK). Deadlock fix KHÔNG regress.
- `[needs Worker verify]` cú pháp exact: `[ -f .sos-state/worker-active ] || [ -f .sos-state/architect-active ]` guard-block đặt TRƯỚC version-match branch.

**Lưu ý:** (1) khác P078e: P078e chỉ "không exempt" (rồi rơi version-match); P078h phải **BLOCK dứt khoát** — version-match KHÔNG được là escape hatch cho marked actor. (2) **⚠️ Codex caveat:** trên Codex real subagent path markers absent (upstream gap #2) → early-BLOCK không kích hoạt → best-effort; FULL trên Claude (markers fire) + manual-marker. Git backstop (P078g) + human-review net. KHÔNG khai full-protection Codex. (3) Chỉ áp cho ghi `.sos-state/ticket-state.env`; marked-actor ghi product code (path khác, không kèm state) KHÔNG phải phạm vi task này (orchestrator-guard / approval-gate khác xử).

### Task 2: gap #4 — normalize absolute→relative repo-root path TRƯỚC state-file-allow + multi-path match

**File:** `crates/sos-adapter-codex/src/templates.rs` — path-extract site (nơi đọc apply_patch path header, `[needs Worker verify]`).

**Tìm:** chỗ so path patch với exact relative string `.sos-state/ticket-state.env` (state-file-allow) + d2a all-path loop. Hiện so exact-string → absolute path false-block.

**Thay bằng / Thêm:** chèn bước **normalize mỗi extracted path → relative repo-root** TRƯỚC khi so:
- Nếu path absolute (bắt đầu `/`) và nằm dưới repo-root → strip repo-root prefix → relative. (Repo-root = guard chạy từ đâu; `[needs Worker verify]` cách guard biết repo-root — `pwd` / `git rev-parse --show-toplevel` / env).
- Áp normalize cho MỌI path TRƯỚC cả state-file-allow (Task 1) VÀ d2a all-path multi-path check → 1 chỗ normalize, cả 2 consumer dùng path đã chuẩn hoá.
- Path ngoài repo-root / relative sẵn → giữ nguyên.

**Lưu ý:** (1) normalize phải đặt sao cho d2a multi-path guard KHÔNG regress — bundle (ticket-state.env + code) sau normalize vẫn multi-path → BLOCK. (2) đừng để path-trick (`../`, symlink) bypass: nếu normalize không resolve được an toàn về repo-root → xử conservative (coi như path lạ → KHÔNG exempt / để guard block). `[needs Worker verify]` guard có sẵn util normalize chưa hay phải thêm inline shell.

### Task 3: tests — real-payload fixture matrix (gap #3 + #4) + negative-test

**File:** `crates/sos-adapter-codex/src/lib.rs` test module (`run_guard()` + `setup` closure ghi marker + fixture, P078e-established).

**Thêm test (mock apply_patch payload REAL b3 shape + marker fixture, qua `run_guard`):**
1. **gap #3 core — advance BLOCK:** `.sos-state/worker-active` SET + state already-approved `V3/V3` + patch state-only `V4/V4` → **BLOCK**. (Phải FAIL trước fix / PASS sau fix — case răng gap #3.)
2. **marked-actor any-write BLOCK:** worker-active SET + ticket-state.env write (create HOẶC update HOẶC advance) → **BLOCK** (cả 3 biến thể).
3. **architect-context BLOCK:** `.sos-state/architect-active` SET + ticket-state.env-alone → **BLOCK** (symmetric).
4. **main-thread ALLOW (no-regress P078e deadlock fix):** BOTH marker ABSENT + state `V1/empty` → approval update `V2/V2` → **ALLOW**.
5. **gap #4 — absolute path normalize:** patch dùng **absolute** path cho ticket-state.env → normalize → xử ĐÚNG như relative (marked-actor → BLOCK; main-thread → ALLOW). So sánh cặp absolute vs relative cho cùng kết quả.
6. **bundle BLOCK (multi-path no-regress):** patch ticket-state.env **kèm** code path (`src/**`) → **BLOCK** (kể cả absolute path sau normalize).
7. **negative-test (răng, bắt buộc — ghi Discovery):** revert Task 1 early-BLOCK → test #1 (advance) BLOCK→ALLOW = FAIL (early-BLOCK load-bearing); revert Task 2 normalize → test #5 (absolute) false-block → main-thread ALLOW→BLOCK = FAIL.

**Lưu ý:** fixture = REAL b3 apply_patch shape (d2a `tests/fixtures/codex-apply-patch-payloads.jsonl` nếu còn, `[needs Worker verify]`), KHÔNG string bịa. Marker fixture = file thật temp `.sos-state` (mirror SubagentStart touch). Absolute-path fixture = repo-root prefix thật của temp dir. KHÔNG always-green (gap #3 test phải đỏ trước fix).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-adapter-codex/src/templates.rs` | Task 1: marked-actor early-BLOCK mọi ticket-state.env write (không fall-through version-match). Task 2: normalize absolute→relative path trước state-file-allow + multi-path |
| `crates/sos-adapter-codex/src/lib.rs` | Task 3: fixture matrix (advance BLOCK + any-write BLOCK + architect BLOCK + main-thread ALLOW + absolute-normalize + bundle BLOCK + negative) |
| `adapters/codex/CAPABILITY.md` | actor-check: block-all-write marked-actor + path-normalize; **Codex caveat** (best-effort Codex real-subagent markers absent / FULL Claude + manual-marker; git backstop + human-review net) |
| `SECURITY.md` | control-plane write: marked actor BLOCKED mọi write (không escape qua already-approved); path normalized trước match; Codex best-effort caveat |
| `CHANGELOG.md` | entry P078h (actor-check advance-block #3 + path-normalize #4) |
| `docs/discoveries/P078h.md` | Discovery Report (mới) |
| `docs/BACKLOG.md` | mark P078h DONE (active sprint) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `templates.rs` — d2a multi-path guard (all-path block-if-any) | #6 SHIPPED — nương; normalize chèn TRƯỚC, bundle vẫn BLOCK KHÔNG regress |
| `templates.rs` — P078e main-thread exemption branch | Deadlock fix KHÔNG regress: main-thread state-alone vẫn ALLOW (test #4) |
| `templates.rs` — SubagentStart marker touch (d2b) | Marker path khớp actor-check (anchor #3); KHÔNG đổi output |
| `scripts/orchestrator-guard.sh` (actor-marker precedent `:596`) | Đọc semantics — KHÔNG sửa |
| `crates/sos-install/**`, `crates/sos-cli/**`, engine | **P078g** (arm-hooks) — KHÔNG chạm (song song) |
| `crates/sos-core/**`, `crates/sos-adapter-claude/**` | Untouched |

---

## Luật chơi (Constraints)

1. **Marked-actor = BLOCK dứt khoát, không escape hatch:** worker/architect marker PRESENT + touch ticket-state.env → BLOCK NGAY, KHÔNG fall-through version-compare BẤT KỂ pre-patch approved. Version-match KHÔNG được là đường vòng cho marked actor advance self-approve.
2. **Main-thread exemption KHÔNG regress:** BOTH marker ABSENT → giữ P078e (state-alone create+update ALLOW; bundle → all-path BLOCK). Test #4 chứng minh deadlock fix sống.
3. **Path-normalize 1 chỗ, cả 2 consumer:** normalize absolute→relative repo-root TRƯỚC state-file-allow + d2a multi-path; conservative với path-trick (`../`/symlink không resolve an toàn → coi path lạ, KHÔNG exempt). Multi-path guard KHÔNG regress (bundle sau normalize vẫn BLOCK).
4. **Marker-path khớp:** actor-check đọc ĐÚNG `.sos-state/worker-active` + `.sos-state/architect-active` khớp SubagentStart touch (d2b) + P078e. Sai path → no-op → security regression → ESCALATE, KHÔNG land.
5. **Codex caveat honest:** KHÔNG khai actor-check = full protection trên Codex (real-subagent markers absent, upstream gap #2 `openai/codex#21753`). Ghi best-effort Codex / FULL Claude + manual-marker; git backstop (P078g) + human-review-at-commit. Nhất quán honest-MISSING (d2b).
6. **Oracle real, không always-green:** Task 3 fixture = REAL b3 apply_patch shape + marker fixture thật qua `run_guard`; gap #3 test PHẢI đỏ trước fix. Negative-test bắt buộc (revert early-BLOCK → advance ALLOW = FAIL; revert normalize → absolute false-block). KHÔNG string-contains thô.
7. **Additive + dep-direction:** adapter→core, no core→adapter import, KHÔNG đổi public signature. Độc lập P078g — KHÔNG chạm install/cli/engine.
8. **Dựa round-3 findings THẬT:** bug không khớp spec → DISCOVERY + báo, KHÔNG sửa mò. Lane Guarded declared.

---

## Nghiệm thu

### Automated
- [ ] `cargo check` clean
- [ ] `cargo test -p sos-adapter-codex` pass (advance BLOCK + any-write BLOCK + architect BLOCK + main-thread ALLOW + absolute-normalize + bundle BLOCK)
- [ ] **Oracle (real-payload guard fixtures):**
  - worker-active SET + approved V3/V3 → patch V4/V4 → **BLOCK** (gap #3, đỏ trước fix)
  - marker present + any ticket-state.env write (create/update/advance) → **BLOCK**
  - markers ABSENT (main-thread) + approval update → **ALLOW** (P078e deadlock no-regress)
  - absolute patch path cho ticket-state.env → normalize → xử đúng như relative (gap #4)
  - bundle (ticket-state.env + code) → **BLOCK** (multi-path no-regress)
- [ ] **Negative-test:** revert Task 1 early-BLOCK → advance BLOCK→ALLOW = FAIL; revert Task 2 normalize → absolute main-thread ALLOW→BLOCK = FAIL. Ghi Discovery
- [ ] Flake gate: `cargo test -p sos-adapter-codex` ×20 → 0-flaky
- [ ] Dep-direction guard xanh (adapter→core)

### Manual Testing
- [ ] (nếu có Claude) spawn worker → thử advance approved state ticket-state.env → BLOCK (early-BLOCK fire in-subagent Claude) — best-effort defer nếu không có instance
- [ ] (nếu có Codex 0.145.0) manual-marker repro findings §B4 → advance BLOCK (manual-marker path; real-subagent markers absent = best-effort caveat)

### Regression
- [ ] P078e main-thread deadlock fix (V1→V2 approval ALLOW) KHÔNG đổi
- [ ] d2a multi-path guard (#6) + bootstrap create KHÔNG regress (bundle vẫn BLOCK, kể cả absolute path)
- [ ] 3 startup render (d1) + SubagentStart marker (d2b) output KHÔNG đổi

### Docs Gate
- [ ] `adapters/codex/CAPABILITY.md` — actor-check block-all + normalize + Codex caveat
- [ ] `SECURITY.md` — control-plane write marked-actor BLOCKED (no version escape) + path normalized; Codex best-effort. **Trust-gate:** verify SECURITY.md content plain ASCII (INV-TRUST-02); rebaseline chỉ nếu SURFACE_GLOBS đổi — ghi Discovery có/không
- [ ] `CHANGELOG.md` — entry P078h
- [ ] `docs/discoveries/P078h.md`
- [ ] `docs/BACKLOG.md` — P078h DONE

### Discovery Report
- [ ] Write to `docs/discoveries/P078h.md`
  - Anchor #1–6 — CORRECT / WRONG (file:line thật cho actor-check fall-through site + path-extract site + marker path + run_guard harness)
  - **Gap #3 note (bắt buộc):** early-BLOCK marked-actor thay fall-through version-match; version-match KHÔNG còn là escape hatch cho advance self-approve
  - **Gap #4 note:** normalize 1 chỗ, cả state-file-allow + multi-path dùng; conservative path-trick
  - **Codex caveat (bắt buộc):** real-subagent markers absent (upstream gap #2) → best-effort Codex / FULL Claude + manual-marker; git backstop P078g + human-review. KHÔNG giả full-protection Codex
  - **Marker path verify:** actor-check đọc đúng `.sos-state/{worker,architect}-active` khớp SubagentStart touch
  - Negative-test kết quả (revert early-BLOCK → advance ALLOW; revert normalize → absolute false-block)
  - trust-gate SECURITY.md rebaseline — có chạy không, tại sao
  - Docs updated (CAPABILITY/SECURITY hoặc "N/A" explicit)
  - Tier escalations (None expected — marker-path lệch / exemption mở advance khi marker SET → escalate)
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`
