# PHIẾU P080: Dual-runtime brownfield dogfood (Claude + Codex cùng repo)

---

> **Loại:** Feature (dogfood / verification protocol — no product-code edit unless FAIL)
> **Ưu tiên:** P1
> **Tầng:** 1 — chạm security backstop (Git hooks) + cross-runtime state contract (`.sos-state/ticket-state.env`) + install non-clobber. Sai thì LAN sang cả 2 runtime → AUTO Tầng 1.
> **Lane:** Guarded — dogfood matrix nhiều anchor/checklist (> 5 anchor), no-cap. Debate flow đầy đủ (Tầng 1).
> **Ảnh hưởng:** `docs/adapters/P080-FINDINGS-2026-07-23.md` (mới); no code edit trừ khi FAIL → mở P080x gap ticket.
> **Dependency:** P079 (DONE 2026-07-23, round-5 ALL PASS @56a243a). Blocks P081 distribution.

---

## Context

### Vấn đề hiện tại
P074→P079 verify từng runtime ĐƠN LẺ: Claude parity (P076), Codex adapter (P078), Codex self-dogfood 5-round (P079). Chưa verify **DUAL-runtime**: khi CẢ Claude Code lẫn Codex cùng dùng MỘT repo. 2 rủi ro chưa đóng: (1) **2 cơ chế render KHÁC NHAU chạy chung 1 repo** — Claude asset render qua `sos new`/`sos adopt` (copy_tree, dev `[8/8]` hook + full `scripts/` tree) còn Codex render qua `sos install --runtime codex` (engine-embedded backstop hook); 2 đường chưa từng chạy chung; (2) brownfield — repo thật đã có pre-commit / `hooksPath` / `AGENTS.md` / `CLAUDE.md` riêng → adapter phải non-clobber + backup, backstop KHÔNG phá hook có sẵn. P081 (distribution) GATED sau P080 xanh.

### Giải pháp
Chạy test matrix 5 nhóm (A fresh-dual / B brownfield / C cross-runtime-state / D regression-P079 / E cross-platform) theo format checklist P079 rounds. Mỗi item PASS/FAIL đo được. FAIL → mở `P080x` gap ticket (giống arc P078g/h/i/j của P079). Phân vai: Thợ chạy local được (Claude side + fixture `/tmp`) vs `[Sếp+Codex]` bắt buộc `codex exec` thật.

### Scope
- CHỈ tạo `docs/adapters/P080-FINDINGS-2026-07-23.md` + (nếu FAIL) phiếu gap `P080x`.
- KHÔNG sửa `crates/**/src` trong phiếu này (fix đi qua P080x riêng, có Debate). KHÔNG đụng adapter render logic ad-hoc.

---

## Task 0 — Verification Anchors

> CHALLENGE round-1 đã verify anchor #1/#4/#5/#7 bằng code thật (file:line ghi bên dưới). Anchor còn `[needs Worker verify]` = Worker xác nhận runtime tại EXECUTE.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `sos install --runtime <claude\|codex>` là entry install | `grep -n "runtime" crates/sos-cli/src/commands/install.rs crates/sos-cli/src/main.rs` | ✅ `[verified]` — entry `--runtime` tại `crates/sos-cli/src/commands/install.rs:45-56`, dispatch `main.rs:91` |
| 2 | `render_embedded_hooks()` + `arm_git_hooks()` (refuse-when-set) sống ở `crates/sos-install/src/engine.rs` | `grep -n "render_embedded_hooks\|arm_git_hooks\|hooks_present_and_executable" crates/sos-install/src/engine.rs` | ⏳ `[unverified]` — per BACKLOG P078g/i, path đúng; Worker xác nhận signature |
| 3 | Non-clobber abort message = `core.hooksPath already set to '<x>'; refusing to clobber` | `grep -rn "refusing to clobber" crates/sos-install/src/` | ⏳ `[unverified]` — quote lấy từ P079 round-5 findings A5 |
| 4 | Dual-install non-clobber KHÔNG chặn nhầm runtime thứ 2: guard chỉ fire khi `existing_hp != "hooks"`; cả 2 runtime đều arm `core.hooksPath=hooks` → idempotent | `grep -n "existing_hp\|!= \"hooks\"" crates/sos-install/src/engine.rs` | ✅ `[verified]` — guard tại `crates/sos-install/src/engine.rs:355-356`, chỉ abort khi hooksPath ≠ `hooks`. Install lần 2 KHÔNG bị chặn. **Task 1-A2 = confirm thực nghiệm, KHÔNG phải unknown.** |
| 5 | `ClaudeAdapter::plan()` là STUB — `sos install --runtime claude` KHÔNG render `.claude/*`; Claude asset render qua `sos new`/`sos adopt` (copy_tree) | `grep -n "Plan::default\|fn plan" crates/sos-adapter-claude/src/lib.rs`; `grep -n "copy_tree" crates/sos-cli/src/commands/new.rs` | ✅ `[verified]` — `ClaudeAdapter::plan()` = `Plan::default()` (0 asset) tại `crates/sos-adapter-claude/src/lib.rs:23-26` (known gap P078b1/c/g). Claude render thật = copy_tree `new.rs:312`. **→ Dual thật = `sos new`/`sos adopt` (Claude) + `sos install --runtime codex` cùng repo.** |
| 6 | State file `.sos-state/ticket-state.env` (fields `version`/`approved_version`) là contract dùng chung 2 runtime | `core/STATE.md` §Lifecycle-state-artifact (đã đọc) + `grep -rn "ticket-state.env" crates/` | ✅ docs `core/STATE.md:29-37` `[verified docs]`; code path `[needs Worker verify]` |
| 7 | Codex-side guard `scripts/codex/*.sh` chỉ render vào TARGET repo (không có tại root sos-kit) | Verify TRONG fixture đã `sos install --runtime codex`: `ls <fixture>/scripts/codex/`; source render `crates/sos-adapter-codex/src/templates.rs:115-119` | ✅ `[verified]` — target_path render `templates.rs:115-119`. **Worker chạy grep guard TRONG fixture, KHÔNG tại root sos-kit.** |

**⚠️ Overlap thật cần test (anchor #5):** `sos new`/`sos adopt` render dev `[8/8]` hook + full `scripts/` tree; `sos install --runtime codex` render engine-embedded minimal backstop hook. 2 cơ chế render hook khác nhau trên CÙNG repo, cùng arm `core.hooksPath=hooks` — đây là điểm chưa từng chạy chung. Worker verify thứ tự cả 2 chiều (Task 1).

### Pre-phiếu snapshot
Theo TICKET_TEMPLATE (Worker auto first-step trong worktree). Phiếu này chủ yếu ghi file findings + fixture `/tmp` — snapshot vẫn chạy cho an toàn.

---

## Debate Log

**Phiếu version:** V2 (CHALLENGE round-1 responded)

### Turn 1 — Worker Challenge
Verdict: **APPROVE-với-sửa-nhỏ.** Verified anchor #1/#4/#5/#7 bằng code thật; 3 sửa mức verify-command/expectation + 3 case matrix bổ sung.

### Turn 1 — Architect Response (phiếu V2)
CHALLENGE round-1 → 5 fixes applied (anchor #1/#4/#5/#7 + matrix A-reversed/A5-uninstall/sync-dual):
- [O1.#1] ACCEPT → anchor #1 verify-command sửa sang `crates/sos-cli/src/commands/install.rs` + `main.rs` (entry `install.rs:45-56`, `main.rs:91`).
- [O1.#4] ACCEPT → anchor #4 hạ khỏi "unknown trung tâm"; guard `engine.rs:355-356` chỉ fire khi `!= "hooks"` → Task 1-A2 = confirm thực nghiệm.
- [O1.#5] ACCEPT (quan trọng nhất) → `ClaudeAdapter::plan()` STUB (`lib.rs:23-26`); dual thật = `sos new`/`sos adopt` + `sos install --runtime codex`. Sửa Task 1/2 + ghi chú A1 tránh nhầm stub thành regression.
- [O1.#7] ACCEPT → anchor #7 verify TRONG fixture (`scripts/codex/*.sh` render vào target, `templates.rs:115-119`), không tại root.
- [O1.matrix] ACCEPT → thêm A-reversed (codex trước), A5-uninstall/rollback dual (`RemovalPlan`/`uninstall()`), sync-dual smoke.

**Status:** ✅ RESPONDED — phiếu bumped to V2

### Final consensus
- Phiếu version: V<N>
- Approved by Chủ nhà: [date]

---

## Nhiệm vụ

> Đây là **test protocol**, không phải code-change. Mỗi Task = 1 nhóm test. Thợ chạy nhóm `[Thợ-local]` bằng Bash/git trong fixture `/tmp`; nhóm `[Sếp+Codex]` cần `codex exec` thật → Quản đốc ping Sếp, KHÔNG tự chạy được.
>
> **⚠️ QUAN TRỌNG (anchor #5):** `sos install --runtime claude` là STUB (render 0 asset — known gap, KHÔNG phải regression mới). Kịch bản dual THẬT = Claude side qua **`sos new`/`sos adopt`** (copy_tree) + Codex side qua **`sos install --runtime codex`**. Đừng dùng `sos install --runtime claude` làm đường render Claude.

### Task 1 — [Thợ-local] Nhóm A: fresh dual-install (2 cơ chế render chung 1 repo)
**Fixture:** repo git riêng trong `/tmp` (KHÔNG clone sos-kit — tránh `.sos-state/sos-kit-self`).
- A1. **Claude side qua `sos new`/`sos adopt`** (KHÔNG `install --runtime claude` — đó là stub, ghi rõ trong findings để không nhầm thành regression): render `.claude/*` + dev `[8/8]` hook + full `scripts/` tree; `git config --local core.hooksPath` = `hooks`; hook + guard executable.
- A2. **(anchor #4, confirm thực nghiệm)** Cùng repo, chạy tiếp `sos install --runtime codex` → EXPECT: KHÔNG "refusing to clobber" (existing hooksPath = `hooks` = kit-owned, guard `engine.rs:355-356` không fire). Render 18 file Codex (`AGENTS.md`/`.codex/*`) KHÔNG đè `.claude/*`. Ghi exit + file-set. Nếu bị chặn nhầm → FAIL → P080x.
- A2-rev. **(matrix bổ sung — thứ tự ngược)** Fixture mới: `sos install --runtime codex` TRƯỚC, rồi `sos new`/`sos adopt` (Claude) SAU. EXPECT: hành vi đối xứng A2 — Codex embedded backstop hook có sẵn, Claude copy_tree dev hook đè/co-exist thế nào? Ghi rõ hook nào thắng ở `hooks/pre-commit` + có double-arm/conflict không.
- A3. Sau dual-install (cả 2 thứ tự): `.claude/` lẫn `.codex/`+`AGENTS.md` tồn tại; `hooks/pre-commit` armed 1 lần. Giao 2 file-set = chỉ `hooks/`+`.sos-state/`.
- A4. Regression backstop: commit thật `.env` → BỊ CHẶN (exit≠0); commit product code trên default → BỊ CHẶN (backstop chung 2 runtime).
- A5. **(matrix bổ sung — uninstall/rollback dual)** Trên repo dual, gỡ 1 runtime (`RemovalPlan`/`uninstall()` — engine có, chưa test dual) → EXPECT: runtime kia + `hooks/`+`.sos-state/` shared còn nguyên, chỉ file riêng của runtime bị gỡ. `[needs Worker verify]` signature `uninstall()` tại `crates/sos-install/src/engine.rs`.

### Task 2 — [Thợ-local] Nhóm B: brownfield non-clobber + backup
**Fixture:** repo git có sẵn: `core.hooksPath=custom-hooks` + `custom-hooks/pre-commit` riêng + `AGENTS.md` + `CLAUDE.md` nội dung sẵn.
- B1. `sos install --runtime codex` (non-TTY) → ABORT không đè `core.hooksPath` (giữ `custom-hooks`), theo P079 round-5 A5 (guard `engine.rs:355-356` fire vì `custom-hooks != "hooks"`). Exit + message.
- B2. `AGENTS.md` / `CLAUDE.md` có sẵn → non-clobber HOẶC backup `.bak` đúng (KHÔNG mất nội dung user). Worker verify hành vi thật + ghi (render skip vs backup-then-write).
- B3. Hook có sẵn của user KHÔNG bị backstop phá: `custom-hooks/pre-commit` còn nguyên executable + nội dung.
- B4. Cùng repo Claude side qua `sos new`/`sos adopt` (KHÔNG stub `install --runtime claude`) → hành vi non-clobber/backup đối xứng B1-B3 (`adopt` phải reconcile `CLAUDE.md`/hooksPath sẵn có).

### Task 3 — [Sếp+Codex] Nhóm C: cross-runtime state
> **BẮT BUỘC `codex exec` thật.** Quản đốc ping Sếp. Claude-side đọc/ghi state Worker mô phỏng được; Codex-side actor-check phải chạy real (hoặc manual-marker theo caveat).
- C1. Runtime này ghi `.sos-state/ticket-state.env` (`version=V2, approved_version=V2`), runtime kia ĐỌC + tôn trọng gate: Claude ghi approval → Codex `codex exec` EXECUTE thấy `version==approved_version` → ALLOW.
- C2. Actor-check cross: `.sos-state/worker-active` present + thử advance state qua Codex → BỊ CHẶN (manual-marker repro; real-subagent marker KHÔNG fire = caveat `openai/codex#21753`).
- C3. Ngược lại: Codex ghi state, Claude-side guard (`scripts/architect-guard.sh`/main-thread) đọc + tôn trọng. Verify approval gate 2 chiều nhất quán.

### Task 4 — [Thợ-local] Nhóm D: regression P079 round-5 + sync-dual smoke
Chạy lại các test key round-5 trên fixture dual-install (đảm bảo dual không phá single):
- D1. Backstop fail-CLOSED: xoá `scripts/block-env-commit.sh` → commit `.env` vẫn BỊ CHẶN (không "missing → allowed").
- D2. Backstop minimal (Codex-install path không phải dev [8/8]): grep hook backstop đã render qua `sos install --runtime codex` KHÔNG ref `docs-gate`/`trust-gate`/`type-check`/`install-hooks.sh`. (Lưu ý: nếu thứ tự A2-rev để dev `[8/8]` hook thắng thì phân biệt rõ hook nào đang active.)
- D3. Path canonicalize (P078j): manual-marker advance qua `/tmp/...` path → BỊ CHẶN; main-thread approval qua `/tmp/...` → ALLOW (macOS symlink `/tmp`→`/private/tmp`).
- D4. **(matrix bổ sung — sync/map dual smoke)** `sos sync` + `sos map` trên repo dual → EXPECT: no crash, không phá file-set runtime nào. Smoke-level. `[needs Worker verify]` hành vi `sync`/`map` trên repo có cả 2 runtime nếu không chắc.

### Task 5 — [Thợ-local, macOS] + [Sếp, nếu có Linux env] Nhóm E: cross-platform note
- E1. macOS (chính): tất cả A-D pass.
- E2. Linux (nếu Sếp có env): lặp Task 1 (fresh dual-install) + Task 4 D1/D2 — ghi khác biệt line-ending / `sha256sum` vs `shasum` / hooksPath. KHÔNG block P080 nếu thiếu Linux env — ghi "Linux DEFERRED" như P071 Task 6.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `docs/adapters/P080-FINDINGS-2026-07-23.md` | MỚI — verdict + PASS/FAIL từng item A1-E2 + output verbatim (format P079 round-5 findings) |
| `docs/adapters/P080-CHECKLIST-2026-07-23.md` | (optional) checklist tick cho Sếp+Codex trước khi chạy `codex exec` (format round-5 checklist) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `crates/sos-install/src/engine.rs` | `render_embedded_hooks`/`arm_git_hooks`/`uninstall` chạy đúng ở dual-install; guard `:355-356` idempotent với `hooks`; KHÔNG edit trong P080 |
| `crates/sos-cli/src/commands/{install,new}.rs` | `--runtime` dispatch (`install.rs:45-56`) + `copy_tree` (`new.rs:312`) — hành vi, không edit |
| `crates/sos-adapter-claude/src/lib.rs` | `plan()` STUB (`:23-26`) — xác nhận là known gap, KHÔNG "sửa" trong P080 |
| `crates/sos-adapter-codex/src/{lib,templates}.rs` | guard + state logic cross-runtime; render target `templates.rs:115-119`; edit đi qua P080x nếu FAIL |
| `core/STATE.md` | state contract `.sos-state/ticket-state.env` — nếu dual bộc lộ gap serialization → note, không edit ad-hoc (§Deferred-scope) |

---

## Luật chơi (Constraints)

1. **KHÔNG sửa `crates/**/src` trong P080.** FAIL bất kỳ item → mở phiếu gap `P080x-<slug>` (Tầng 1, có Debate) — giống P078g/h/i/j đóng gap của P079. P080 chỉ ghi findings + route.
2. **Claude render = `sos new`/`sos adopt`, KHÔNG `install --runtime claude`** (stub, anchor #5). Ghi rõ trong findings để reviewer không nhầm stub-0-asset thành regression mới.
3. **`[Sếp+Codex]` (Task 3, E2-Linux) KHÔNG tự chạy được** — Quản đốc ping Sếp cho `codex exec` thật. Codex real-subagent marker caveat (`openai/codex#21753`) giữ nguyên — Task 3 dùng manual-marker khi cần, git backstop + human-review = net cuối.
4. **Fixture repo riêng trong `/tmp`**, KHÔNG clone/dùng sos-kit checkout (tránh `.sos-state/sos-kit-self` nhiễu). Guard grep (anchor #7) chạy TRONG fixture, KHÔNG tại root sos-kit.
5. **Structural-oracle-gap guard (bài học P078f/g/i):** fixture KHÔNG được seed sẵn thứ mà install/scaffold phải tự render (hooks/guard scripts). Pristine-no-seed + dependency-closure assert.

---

## Nghiệm thu

### Automated (Thợ-local)
- [x] `cargo build --release` clean (cached, 0.11s); dual render 2 chiều: (`sos new` Claude → `install --runtime codex`) VÀ (`install --runtime codex` → `sos adopt` Claude) cùng repo → exit + hook-active ghi rõ (A2 PASS + A2-rev PASS, non-symmetric hook-winner documented).
- [x] Nhóm A (gồm A2-rev PASS, A5 N/A-not-FAIL) + B (B1-B4 PASS) + D (D1 **FAIL**, D2 PASS, D3 PASS-proxy, D4 PASS) chạy trong fixture scratchpad, mọi item có output verbatim trong `docs/adapters/P080-FINDINGS-2026-07-23.md`.

### Manual Testing ([Sếp+Codex])
- [ ] Task 3 (cross-runtime state) qua `codex exec` thật — C1/C2/C3 PENDING, chưa chạy round này.
- [ ] E2 Linux — DEFERRED, không có Linux env round này (per phiếu, không block P080 round-1 report).

### Regression
- [x] Nhóm D: P079 round-5 key tests **KHÔNG hoàn toàn xanh** trên fixture dual-install — D1 phát hiện regression thật (dev `[8/8]` hook fail-open khi thiếu script, P078i fix chỉ áp cho Codex backstop hook). D2-D4 xanh.

### PASS/FAIL rule
- [x] **FAIL: D1 (HIGH)** = mở `P080x-hook-fail-open-parity` gap ticket (Tầng 1, Debate) — port fail-closed pattern (Codex backstop hook) vào dev `[8/8]` hook's `[6/8]`/`[7/8]` phases. A5 = N/A không tính FAIL (honest stub, no CLI). **P081 GIỮ gated** tới khi P080x đóng + re-run round-2 xanh + Task 3/E2 xong.

### Docs Gate
- [x] `CHANGELOG.md` — entry P080 (verdict FAIL + P080x gap ticket cần mở).
- [x] `docs/BACKLOG.md` — tick `[P080]` với verdict, cập nhật resume pointer (P081 vẫn gated).
- [ ] `adapters/codex/CAPABILITY.md` — N/A round này (gap là ở dev hook Claude-side, không phải Codex capability caveat mới; sẽ note khi P080x đóng nếu cần).

### Discovery Report
- [x] `docs/discoveries/P080.md` — anchor #1-7 CORRECT + 1 anchor mới (uninstall stub) ghi rõ, dual-install non-clobber kết luận, hook-thắng ở A2-rev (order-dependent), uninstall-dual N/A + sync-dual PASS, gap ticket P080x-hook-fail-open-parity mở, cross-platform note (E1 partial vì D1 FAIL, E2 DEFERRED).
- [x] Append 1-line index vào `docs/DISCOVERIES.md`.
