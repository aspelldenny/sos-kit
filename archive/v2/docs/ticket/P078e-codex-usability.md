# PHIẾU P078e: Codex adapter usability — approval-transition deadlock (#1) + actor-check chống worker self-approve

---

> **Loại:** Bugfix (security — approval-gate exemption)
> **Ưu tiên:** P1 — gap#1 CHẶN Codex adapter chạy được thật (unattended workflow deadlock)
> **Tầng:** 1 — Task 1 chạm approval-gate exemption + actor-check (security boundary, mở rộng d2a) = AUTO Tầng 1 dù nhỏ. LOC KHÔNG quyết.
> **Lane:** Normal (re-declared V2 — sau khi BỎ Task 2 arm-hooks + install-smoke tests sang P078f, P078e còn DUY NHẤT Task 1 approval-exemption + actor-check + test, phạm vi 1 crate `sos-adapter-codex`; anchor/constraint trim theo — 5 anchor + 5 constraint). **Token budget: ~50k** (Architect docs-only; Worker đọc `templates.rs` approval-gate content-fn + `orchestrator-guard.sh:596` actor-marker precedent + `lib.rs` test module). **SECURITY (Task 1) → Worker CHALLENGE bắt buộc dù Normal lane.** Worker chạy lane-check lại khi EXECUTE; actor-check balloon → khai lại Guarded + escalate.
> **Ảnh hưởng:** `crates/sos-adapter-codex/src/templates.rs` (approval-gate content-fn exemption + actor-check); `crates/sos-adapter-codex/src/lib.rs` test module.
> **Dependency:** P078d (d1+d2a+d2b) SHIPPED. Spec = `docs/adapters/P079-ROUND2-FINDINGS-2026-07-23.md` (gap#1 round-2 live-confirmed). d2a multi-path all-path guard (#6) + create-when-missing bootstrap exemption = nền (mở rộng, không tái tạo). **Actor-marker precedent:** `orchestrator-guard.sh:596` (`[ -f worker-active ] && continue`).
> **Split-out:** gap#2 `sos install` arm Git hooks → **P078f** (port `scripts/install-hooks.sh` security-arming: F09 hijack-guard + chmod + `.bak` rename). Xem `docs/plans/P078d-decomposition.md`.

---

## Context

### Vấn đề hiện tại

P079 **round-2** live-dogfood (sau d1/d2a/d2b PASS) phơi gap#1 chặn adapter chạy thật; gap#2 (arm-hooks) TÁCH sang P078f. Spec `docs/adapters/P079-ROUND2-FINDINGS-2026-07-23.md`:

**Gap #1 — Approval transition DEADLOCK (CRITICAL, chặn unattended workflow):**
Bootstrap tạo state `version=V1, approved_version=empty`. Bước approval hợp lệ ĐẦU TIÊN (ghi `version=V2, approved_version=V2` vào `.sos-state/ticket-state.env`) bị approval-gate **BLOCK** vì V1 chưa-approved → vòng luẩn quẩn: *approve = ghi approved_version, nhưng chính ghi đó bị gate chặn vì chưa approved*. Unattended thì kẹt cứng.

- **Root cause:** d2a chỉ exempt **create-when-missing** (`.sos-state/ticket-state.env` thiếu → cho tạo path đơn độc), KHÔNG exempt **UPDATE** (state đã tồn tại) → nhánh update rơi version-match `V1 != approved` → BLOCK.
- **FIX (V2 — actor-gated, sửa theo Worker CHALLENGE O1.1):** approval-gate exempt write tới `.sos-state/ticket-state.env` (create HOẶC update) **CHỈ KHI actor KHÔNG phải subagent** — `.sos-state/worker-active` VÀ `.sos-state/architect-active` đều ABSENT (mirror `orchestrator-guard.sh:596`). Nghĩa: **chỉ orchestrator/main-thread ghi approval được**; worker-context (marker set) → BLOCK. (a) gỡ deadlock (orchestrator approve OK), (b) chặn **worker self-approve LẶP LẠI** — path-set-only (không actor-check) cho worker ghi `approved_version` ticket-state.env đơn độc ngay sau code-patch merged → self-approve thường trực (multi-path #6 chỉ chặn bundle-code, KHÔNG chặn ghi ticket-state.env đơn độc patch riêng).
- **An toàn 2 lớp:** (1) actor-check (worker/architect marker absent) — chỉ main-thread; (2) d2a multi-path all-path #6 — bundle `ticket-state.env` + code khác → BLOCK. Exemption ăn CHỈ khi patch touch **DUY NHẤT** `.sos-state/ticket-state.env` VÀ actor là main-thread.

**⚠️ Codex caveat (bắt buộc ghi CAPABILITY + Discovery):** trên **Codex**, in-subagent enforcement MISSING (#4/d2b — SubagentStart marker không fire in-subagent) → worker-subagent write KHÔNG bị gate BẤT KỂ actor-check. Nên actor-check là **best-effort main-thread + đúng-hoàn-toàn Claude** (guard fire in-subagent). Trên Codex, approval-record integrity vẫn **boundary-enforced ở commit** (human review commit đổi `approved_version`) — nhất quán honest-MISSING d2b. **KHÔNG khai actor-check = full protection trên Codex** (ship≠chạy claim). Round-2 deadlock repro (orchestrator ghi V1→V2, marker absent → ALLOW) vẫn fix; worker-context (marker set) → BLOCK.

### Giải pháp

Additive, 1 area (`sos-adapter-codex`): **Task 1** — mở rộng approval-gate exemption trong `templates.rs` content-fn (create-when-missing → any-write khi ticket-state-alone) **+ actor-check gate** (exempt CHỈ khi worker/architect marker absent). Nương d2a all-path guard cho lớp safety 2, KHÔNG tái implement guard.

### Decomposition — 1 phiếu (P078e), 1 task (V2 — sau tách P078f)

- **security-isolation (P085 #4):** V1 bundle Task 1 (approval-exemption) + Task 2 (arm-hooks). Worker CHALLENGE O2.1 chứng minh gap#2 arm-hooks = `install-hooks.sh` mang **security-arming non-trivial** (F09 hijack-guard + `chmod +x` + `.bak` rename) → chạm **ngưỡng-tách** chính V1 tự khai. → **split Task 2 → P078f.**
- **P078e V2 = CHỈ Task 1** (approval-exemption + actor-check + test). Oracle = `cargo test -p sos-adapter-codex` (mock-payload guard + actor-marker fixture). 1 crate, security-isolated. external-blocker? KHÔNG.

### Scope

- **CHỈ sửa:** `crates/sos-adapter-codex/src/templates.rs` (approval-gate exemption + actor-check); `crates/sos-adapter-codex/src/lib.rs` test module; docs (CAPABILITY/SECURITY/CHANGELOG/Discovery).
- **KHÔNG sửa:** d2a multi-path guard logic (#6 SHIPPED, nương KHÔNG regress), d2a create-when-missing branch (mở rộng, không xoá), 3 startup render-fn (d1), SubagentStart marker (#4 d2b), `crates/sos-install/**` + `crates/sos-cli/**` + `scripts/install-hooks.sh` (= P078f), `sos-core`/`sos-adapter-claude`.

---

## Task 0 — Verification Anchors

> Architect docs-only. Round-2 fact = `[verified: P079-ROUND2-FINDINGS-2026-07-23.md]`. Code-site = `[needs Worker verify]`. d2a-shipped = `[verified: d2a]`. Actor-marker precedent = `[verified: Worker CHALLENGE O1.1]`. Worker grep-verify TRƯỚC khi sửa.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | approval-gate content-fn (d2a-confirmed `templates.rs:619-622` path-extract, `:634-637` missing-file branch) hiện exempt CHỈ create-when-missing; nhánh UPDATE (state tồn tại) rơi version-match `V1!=approved` → BLOCK `[needs Worker verify]` | `rg -n "ticket-state\|approved_version\|missing\|exit 2\|-f .*STATE" crates/sos-adapter-codex/src/templates.rs` → xác nhận exemption chỉ ở `[ ! -f ]`, update-nhánh không exempt | ✅ Confirmed — pre-fix exemption chỉ trong `if [ ! -f "$STATE_FILE" ]`; update-branch rơi thẳng version-match BLOCK |
| 2 | Round-2 deadlock repro: state `V1, approved_version=empty` → approval write `V2, approved_version=V2` bị BLOCK `[verified: P079-ROUND2-FINDINGS-2026-07-23.md:9-11]` | Đọc round-2 findings §Gap#1; optional live-repro nếu có Codex 0.145.0 | ✅ Confirmed — mock-payload test `approval_gate_allows_v1_to_v2_transition_when_main_thread_round2_repro` reproduces + fix verified ALLOW |
| 3 | Actor-marker: precedent `orchestrator-guard.sh:596` `[ -f worker-active ] && continue`; marker path thật `.sos-state/worker-active` + `.sos-state/architect-active` (SubagentStart touch, d2b). actor-check phải đọc ĐÚNG path này `[verified: Worker CHALLENGE O1.1; needs Worker verify exact marker path trong codex-adapter context]` | `rg -n "worker-active\|architect-active\|-active\|orchestrator-guard" scripts/ crates/sos-adapter-codex/src/templates.rs` → xác nhận marker filename+dir khớp SubagentStart touch (d2b) ↔ actor-check mới | ✅ Confirmed — same paths `.sos-state/worker-active`/`.sos-state/architect-active` used by `lib.rs` `with_worker_active`/`with_architect_active` test helpers (d2a-established) |
| 4 | **Actor-check chặn worker self-approve + coupled all-path:** exemption ăn CHỈ khi worker/architect marker ABSENT + ticket-state-alone. Mock: (a) marker-absent + ticket-state.env-alone → **ALLOW** (orchestrator approve); (b) worker-marker-SET + ticket-state.env-alone → **BLOCK** (chặn self-approve); (c) bundle ticket-state.env + code (marker absent) → **BLOCK** (all-path #6 d2a-active) `[needs Worker verify]` | `rg -n "head -n1\|while read\|for .*path\|BLOCK\|-active" templates.rs` → xác nhận all-path #6 active + actor-check thêm đúng | ✅ Confirmed — 5 mock-payload tests added (round-2 ALLOW, worker-marker BLOCK, architect-marker BLOCK, pre-approval code-edit BLOCK regression, bundle BLOCK even main-thread); negative-test (actor-check removed → self-approve test flips ALLOW) confirmed then reverted |
| 5 | Test harness: `run_guard()` `lib.rs:582-616` render guard → feed apply_patch payload + set/clear `.sos-state/{worker,architect}-active` marker fixture → assert exit `[verified: d2a (15+ test); needs Worker verify marker-fixture support]` | `[verified: d2a]`; nếu `run_guard` chưa hỗ trợ marker-set → Worker mở rộng harness (temp `.sos-state` dir set trước feed payload) | ✅ Confirmed — harness's `setup: impl FnOnce(&PathBuf)` closure already supports writing `.sos-state/{worker,architect}-active` inline, no harness extension needed |

**Nếu ❌ (approval đã exempt update sẵn / actor-check đã có / marker path khác spec) → Worker DISCOVERY_REPORT + điều chỉnh; bug KHÔNG tồn tại như spec → dừng + báo.**
**Nếu anchor #3/#4 sai (marker path không khớp SubagentStart touch → actor-check no-op / exemption vẫn mở self-approve khi marker SET) → ESCALATE NGAY: security regression, KHÔNG land, báo orchestrator.**

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

> Cap = 3 turns. SECURITY (Task 1) → Worker CHALLENGE bắt buộc. **Append-only.**

**Phiếu version:** V2 (bumped by Architect RESPOND Turn 1 — 2 objection ACCEPT + gap#2 split → P078f; anchor/constraint trimmed to Normal-lane cap 5/5 post-split)

### Turn 1 — Worker Challenge (against V1)

**Objections (Tầng 1 only):**
- **[O1.1] Gap#1 exemption thiếu actor-check (SECURITY):** V1 exemption (any-write khi ticket-state.env đơn độc) gate theo **path-set, KHÔNG theo actor**. Cùng `apply_patch` tool Worker dùng → ghi `approved_version` → **worker self-approve LẶP LẠI** (d2a chấp nhận 1-lần bootstrap; mở rộng mọi update = self-approve thường trực). Multi-path #6 chặn bundle nhưng KHÔNG chặn "worker ghi ticket-state.env đơn độc patch riêng sau code patch merged". Precedent: `orchestrator-guard.sh:596` actor-distinguishing đã có.
- **[O2.1] Gap#2 arm-hooks TO hơn "bare git config":** `scripts/install-hooks.sh` = F09 hijack-guard (TTY confirm / non-TTY abort khi hooksPath custom) + `chmod +x` + `.git/hooks/{pre-commit,pre-push}` rename-`.bak`. Non-trivial security-arming. Chạm ngưỡng-tách chính phiếu V1 tự khai.

**Status:** ✅ RESPONDED (Architect Turn 1 below)

### Turn 1 — Architect Response (phiếu V2)

- **[O1.1] → ACCEPT:** approval-exemption thêm **actor-check** — exempt ticket-state.env write CHỈ khi `.sos-state/worker-active` VÀ `architect-active` đều ABSENT (mirror `orchestrator-guard.sh:596`), chỉ orchestrator/main-thread approve; worker-context → BLOCK. Fix deadlock + chặn worker self-approve. **Codex caveat ghi (CAPABILITY + Discovery):** in-subagent enforcement MISSING (#4/d2b) → actor-check best-effort main-thread + đúng-hoàn-toàn Claude; Codex dựa human-review-at-commit. KHÔNG khai full-protection Codex. Action: Context §Gap#1 actor-gated + caveat; Task 1 thêm actor-check; anchor #3 (marker path) + #4 (self-approve BLOCK) verify actor-check; Task 2 tests thêm case worker-marker-SET → BLOCK.
- **[O2.1] → ACCEPT (split):** gap#2 arm-hooks → **P078f** (port `install-hooks.sh` security-arming). **P078e = CHỈ Task 1.** Đúng P085 #4 (security-isolation) + ngưỡng-tách V1 tự khai. Action: BỎ Task 2 (arm-hooks) + install-smoke tests; sửa header/Scope/Files/Nghiệm thu; `docs/plans/P078d-decomposition.md` += P078f.
- **Lane reconcile (post-CHALLENGE lane-check):** anchor/constraint trim leftover-Task-2 → **5 anchor + 5 constraint** → Normal-lane cap pass.

**Status:** ✅ RESPONDED — phiếu bumped to V2. 0 DEFER → sẵn sàng Worker CHALLENGE round 2 (verify consensus) hoặc approval gate.

### Turn 2 — Worker EXECUTE verify (V2)

Task 0 anchors #1–5 re-verified against real code post-V2 (Result column above updated) — all ✅, no drift from Architect's Turn 1 accept. Actor-check implemented exactly as specced (`.sos-state/worker-active`/`architect-active` both-absent gate). 5 new mock-payload tests added, all pass; negative-test (actor-check removed) confirmed the self-approve-BLOCK test flips to ALLOW, then reverted. `cargo build/test --workspace` green, `sos-adapter-codex` 53/53, ×20 = 0 flaky, dep-direction green. Additive confirmed — engine.rs/install.rs/sos-core/sos-adapter-claude untouched. No new objections. **Worker accepted V2 — Task 1 SHIPPED.**

**Status:** ✅ RESPONDED — no challenge, EXECUTE complete.

### Final consensus
- Phiếu version: V2
- Total turns: 2
- Approved by Chủ nhà: sprint-delegated (SECURITY-adjacent CHALLENGE completed, Architect ACCEPT both objections, Worker EXECUTE authorized directly) — 2026-07-23 — code executed

---

## Nhiệm vụ

### Task 1: approval-gate — exempt write tới `.sos-state/ticket-state.env` (create HOẶC update) **CHỈ khi actor = main-thread** (worker/architect marker absent), coupled all-path

**File:** `crates/sos-adapter-codex/src/templates.rs` — approval-gate content-fn (d2a-confirmed `:619-622` path-extract, `:634-637` missing-file branch, `[needs Worker verify]` version-match branch).

**Tìm:** nhánh xử `.sos-state/ticket-state.env`: hiện `[ ! -f STATE ]` (missing) → exempt tạo path đơn độc (d2a); state TỒN TẠI → version-match BLOCK nếu chưa approved (chỗ deadlock update transition).

**Thay bằng / Thêm:** mở rộng exemption từ *create-when-missing* → **any-write-when-ticket-state-alone-AND-actor-is-main-thread**:
- Sau d2a all-path parse (đã land): nếu patch touch **DUY NHẤT** `.sos-state/ticket-state.env` **VÀ** actor-check pass → **ALLOW** (create + update đều exempt).
- **Actor-check (MỚI, mirror `orchestrator-guard.sh:596`):** exemption CHỈ ăn khi `.sos-state/worker-active` **VÀ** `.sos-state/architect-active` đều ABSENT (`[ ! -f .sos-state/worker-active ] && [ ! -f .sos-state/architect-active ]`). Marker nào SET → **KHÔNG exempt** → rơi version-match → BLOCK (chặn worker self-approve). `[needs Worker verify]` exact marker path — KHỚP filename+dir SubagentStart touch (d2b `.sos-state/{architect,worker}-active`).
- Patch touch ticket-state.env **kèm** path khác → **BLOCK** (all-path #6, KHÔNG nới). Product-edit (path khác, không kèm state) → giữ version-match hiện tại.

**Lưu ý:** (1) an toàn nhờ **2 lớp**: actor-check (chính) + d2a all-path #6 (bundle → BLOCK) — ghi coupling vào Discovery. (2) **⚠️ Codex caveat bắt buộc:** in-subagent hook không fire → actor-check best-effort main-thread + đúng-hoàn-toàn Claude; KHÔNG khai full-protection Codex (human-review-at-commit backstop). (3) KHÔNG mở exemption path nào khác ngoài `.sos-state/ticket-state.env`. `[needs Worker verify]` cú pháp nhánh check (path parse + `-f STATE` + `-f *-active` + version compare).

### Task 2: tests — approval update-transition (actor-gated) + regression

**File:** `crates/sos-adapter-codex/src/lib.rs` test module (`run_guard()` `:582-616`, `[needs Worker verify]` harness set/clear `.sos-state/{worker,architect}-active` marker fixture).

**Thêm test (mock-payload, `run_guard`):**
1. **update ALLOW (round-2 repro, main-thread):** marker ABSENT + state `V1, approved_version=empty` + patch update CHỈ ticket-state.env → `V2, approved_version=V2` → **ALLOW** (deadlock gỡ).
2. **create ALLOW (no-regress d2a):** marker ABSENT + state missing + patch tạo CHỈ ticket-state.env → ALLOW.
3. **worker self-approve BLOCK (actor-check, MỚI — O1.1):** `.sos-state/worker-active` SET + patch update CHỈ ticket-state.env `approved_version=V2` → **BLOCK**. Case răng O1.1.
4. **architect-context BLOCK:** `.sos-state/architect-active` SET + ticket-state.env-alone → **BLOCK** (symmetric).
5. **bundle BLOCK (all-path, marker absent):** patch ticket-state.env **kèm** code-path (`src/**`/`.env`) → **BLOCK**.
6. **Regression pre-approval code-edit BLOCK:** product code-edit đơn độc, state chưa approved → BLOCK.

**Negative-test (răng, bắt buộc — ghi Discovery):** revert actor-check → test #3 (worker self-approve) BLOCK→ALLOW = FAIL (actor-check load-bearing); revert update-exemption → test #1 ALLOW→BLOCK = FAIL.

**Lưu ý:** fixture apply_patch = REAL b3 shape (d2a `tests/fixtures/codex-apply-patch-payloads.jsonl`), KHÔNG string bịa. Marker fixture = tạo/xoá file thật temp `.sos-state` (mirror SubagentStart touch). `run_guard` chưa hỗ trợ marker-set → Worker mở rộng harness.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-adapter-codex/src/templates.rs` | Task 1: approval-gate exempt any-write ticket-state-alone (create+update) **CHỈ khi worker/architect marker absent** (actor-check), coupled all-path #6 |
| `crates/sos-adapter-codex/src/lib.rs` | Task 2: test update ALLOW (main-thread) + create no-regress + **worker/architect self-approve BLOCK** + bundle BLOCK + pre-approval BLOCK + negative |
| `adapters/codex/CAPABILITY.md` | Approval-gate actor-check + **Codex caveat** (in-subagent MISSING → best-effort main-thread; human-review-at-commit backstop; full trên Claude) |
| `SECURITY.md` | Control-plane write (ticket-state.env) exempt CHỈ main-thread (actor-gated) + all-path #6; worker self-approve BLOCKED (Claude) / best-effort (Codex) |
| `CHANGELOG.md` | entry P078e (approval update-transition deadlock fix + actor-check; gap#2 tách P078f) |
| `docs/discoveries/P078e.md` | Discovery Report (mới) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `templates.rs` — d2a multi-path guard (all-path block-if-any) | #6 SHIPPED — nương, KHÔNG regress (Task 1 chỉ nới exemption + thêm actor-check) |
| `templates.rs` — 3 startup render-fn + SubagentStart marker (`:315-317` touch `.sos-state/{architect,worker}-active`) | d1/d2b SHIPPED — KHÔNG đổi output; **verify marker path khớp actor-check** (anchor #3) |
| `scripts/orchestrator-guard.sh:596` | Actor-marker precedent — đọc để khớp semantics, KHÔNG sửa |
| `crates/sos-install/**`, `crates/sos-cli/**`, `scripts/install-hooks.sh` | **P078f** (arm-hooks) — KHÔNG chạm |
| `crates/sos-core/**`, `crates/sos-adapter-claude/**` | Untouched |

---

## Luật chơi (Constraints)

1. **Exemption tối thiểu + actor-gated + marker-path-khớp + coupled:** exempt CHỈ `.sos-state/ticket-state.env` đơn độc (create+update) **VÀ CHỈ khi worker/architect marker absent** (main-thread). Actor-check phải đọc ĐÚNG marker filename+dir SubagentStart touch (d2b) — sai path → no-op → security regression. An toàn nhờ actor-check (chính) + d2a all-path #6 (lớp 2, bundle → BLOCK). KHÔNG mở exemption path khác, KHÔNG nới all-path guard.
2. **Actor-check bất biến chống self-approve:** worker/architect marker SET → KHÔNG exempt → BLOCK. Exemption tạo self-approve path MỚI khi marker SET → ESCALATE, KHÔNG land (anchor #4).
3. **Codex caveat honest:** KHÔNG khai actor-check = full protection trên Codex (in-subagent hook không fire, #4/d2b). Ghi best-effort main-thread + human-review-at-commit backstop. Nhất quán honest-MISSING story.
4. **Additive + Oracle real:** adapter→core, no core→adapter import, KHÔNG đổi public signature. Task 2 mock-payload REAL b3 fixture qua `run_guard` + marker fixture thật; negative-test bắt buộc (revert actor-check → self-approve ALLOW = FAIL). KHÔNG string-contains thô.
5. **Dựa round-2 findings THẬT:** bug không khớp spec → DISCOVERY + báo, KHÔNG "sửa mò". Lane Normal declared; Worker re-check lane khi EXECUTE — actor-check balloon → khai Guarded + escalate.

---

## Nghiệm thu

### Automated
- [x] `cargo check` clean
- [x] `cargo test -p sos-adapter-codex` pass (update ALLOW main-thread + create no-regress + **worker/architect self-approve BLOCK** + bundle BLOCK + pre-approval BLOCK) — 53/53
- [x] **Oracle:** marker-absent + state V1/empty + update-only ticket-state.env → **ALLOW** (deadlock gỡ); **worker-active SET + ticket-state.env-alone → BLOCK** (actor-check); bundle ticket-state.env + code → **BLOCK** (all-path); pre-approval code-alone → BLOCK
- [x] **Negative-test:** revert actor-check → worker self-approve BLOCK→ALLOW (FAIL); revert update-exemption → update ALLOW→BLOCK (FAIL). Ghi Discovery
- [x] Flake gate: `cargo test -p sos-adapter-codex` ×20 → 0-flaky
- [x] Dep-direction guard xanh (adapter→core)
- [x] Additive proof: d2a guards + 3 startup render + marker touch + core/adapter-claude KHÔNG đổi output

### Manual Testing
- [ ] (nếu có Codex 0.145.0) round-2 repro: bootstrap V1 → orchestrator approve (main-thread) → EXECUTE trọn KHÔNG cần manual state edit — **N/A this session (no live Codex instance available); mock-payload oracle covers it, live repro deferred to round-3 owner-run**
- [ ] (nếu có Claude) spawn worker → thử ghi ticket-state.env approved_version đơn độc → BLOCK (actor-check fire in-subagent trên Claude) — **N/A this session; same defer**

### Regression
- [x] d2a multi-path guard (#6) + bootstrap create (#5) behavior KHÔNG đổi
- [x] 3 startup-file (d1) + SubagentStart marker (d2b) render output KHÔNG đổi
- [x] Guard hợp lệ cũ (single valid ticket patch main-thread, product-edit sau approval) vẫn ALLOW — không over-block

### Docs Gate
- [x] `adapters/codex/CAPABILITY.md` — approval-gate actor-check + **Codex caveat**
- [x] `SECURITY.md` — control-plane write actor-gated; worker self-approve BLOCKED (Claude) / best-effort (Codex). **Trust-gate:** SECURITY.md not in `SURFACE_GLOBS` baseline (only unicode-scanned) — no rebaseline needed; content is plain ASCII (INV-TRUST-02 clean)
- [x] `CHANGELOG.md` — entry P078e
- [x] `docs/discoveries/P078e.md`

### Discovery Report
- [x] Write to `docs/discoveries/P078e.md`
  - Anchor #1–5 — CORRECT / WRONG (file:line thật cho approval update-branch + actor-marker path + run_guard marker-fixture harness)
  - **Actor-check note (bắt buộc):** exemption an toàn nhờ actor-check (marker absent = main-thread) + d2a all-path #6; path-set-only (V1) sẽ mở worker self-approve LẶP LẠI
  - **Codex caveat (bắt buộc):** in-subagent enforcement MISSING (#4/d2b) → actor-check best-effort Codex; human-review-at-commit; full trên Claude. KHÔNG giả full-protection Codex
  - **Marker path verify:** actor-check đọc đúng `.sos-state/{worker,architect}-active` khớp SubagentStart touch (anchor #3)
  - Negative-test kết quả (revert actor-check → worker self-approve ALLOW; revert exemption → update BLOCK)
  - **Split note:** gap#2 arm-hooks → P078f (install-hooks.sh security-arming non-trivial)
  - trust-gate SECURITY.md rebaseline — có chạy không, tại sao
  - Docs updated (CAPABILITY/SECURITY hoặc "N/A" explicit)
  - Tier escalations (None expected — exemption mở self-approve khi marker SET → escalate)
- [x] Append 1-line index vào `docs/DISCOVERIES.md`
