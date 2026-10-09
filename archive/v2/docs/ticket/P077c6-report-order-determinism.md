# PHIẾU P077c6: parity fixture content-nondeterminism under parallel execution — root-cause + fix

> **ID format:** P077c6 (follow-up c-series, sau P077c CLOSED tại c5).
> **Filename:** `docs/ticket/P077c6-report-order-determinism.md` (path GIỮ — V1→V2 pivot, tránh phiếu-ID churn; title/scope đã đổi).
> **Branch:** `fix/P077c6-report-order-determinism`

---

> **Loại:** Bugfix + Diagnose (parity oracle flaky — content-nondeterminism dưới parallel `cargo test`; NOT order)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — parity/correctness oracle soundness = surface contract; flaky test LAN sang mọi CI run + mọi phiếu dùng test result làm tín hiệu)
> **Lane:** Guarded
> **Ảnh hưởng:** `bootstrap/sos-rs/crates/sos-cli/tests/parity.rs`, test fixture harness (`build_new_fixture`/`build_*_fixture`), CÓ THỂ `src/commands/adopt.rs`/`new.rs`/`sync.rs` (chỉ nếu diagnosis chỉ ra production race)
> **Dependency:** P077c2 (Rust sync parity + `sync.golden` + `parity_sync_enforced`), P077c5 (adopt CORRECTNESS oracle + `adopt.golden`) — both SHIPPED

---

## Context

### ⚠️ V1→V2 PIVOT (đọc trước)

**V1 premise SAI HOÀN TOÀN, đã RÚT LẠI.** V1 chẩn "flaky = report-list unsorted (find-order non-deterministic)" → fix = sort report lists + đính chính finding c2 "SAI". Worker CHALLENGE + orchestrator điều tra đã bác:

- Worker chạy `parity_*` 28 lần → 0 fail order-flaky → nghi premise "20-25% order-flaky".
- Orchestrator re-reproduce + **capture full assertion diff (left vs right)**: fail 4/12 (~10-25% flaky THẬT), NHƯNG **`same set of lines? False`** → khác về **CONTENT, KHÔNG phải order**.
- Failing adopt output chứa: `⏭ skip (symlink escapes kit tree): templates/INVARIANTS-template.md`, `hooks NOT armed (no .git or scripts/install-hooks.sh missing)`, `no stack manifest detected`. THIẾU: `core.hooksPath → hooks/`, `phieu/README.md`, `hooks/pre-commit`, `.sos-stack.toml`. → adopt vận hành trên **view kit/target SAI/THIẾU dưới parallel execution** = concurrency race THẬT về **nội dung fixture**, không phải thứ tự dòng.

**Kết luận:** sort không chữa content-nondeterminism. sync/adopt unsorted-list KHÔNG phải nguyên nhân. c2 finding về order (same-platform deterministic) ĐÚNG — KHÔNG đính chính "c2 SAI". Bug là chuyện khác: fixture/exec race.

Evidence đầy đủ: `scratchpad/c6-evidence-latest.txt` (full left-vs-right diff).

### Vấn đề thật (V2)

`cargo test --workspace` FLAKY ~10-25% dưới parallel load: `parity_adopt_enforced` (+ có thể `new`/`sync`) fail vì adopt/command **thấy kit-tree hoặc target-tree thiếu file / symlink escape / stack manifest / .git / install-hooks.sh** một cách không ổn định dưới parallel. Fail = **content mismatch** (missing/extra lines về armed-hooks, stack-detect, symlink-skip), KHÔNG phải line-order.

Đây là **concurrency race trong fixture build hoặc command exec**, KHÔNG phải froze-non-deterministic-order. Chưa biết chính xác cơ chế → **Task 1 = DIAGNOSE trước, fix sau.**

### Giải pháp (V2)

**Diagnose root-cause → fix theo finding.** KHÔNG chốt sẵn cơ chế. Worker EXECUTE với code-tools + instrumentation, đo tới khi pin chính xác vì sao parallel run đôi khi thấy kit/target thiếu/sai, RỒI fix (fixture harness HOẶC production code, tuỳ diagnosis chỉ ra).

- **KHÔNG sort** (trừ khi diagnosis chứng minh order thật sự góp phần — evidence hiện tại nói KHÔNG).
- Fix nhỏ nhất chữa đúng bệnh: nếu fixture-only race → sửa test harness; nếu adopt/new/sync production có global-state/cwd/shared-path race → sửa production.

### Scope
- **Diagnose:** parallel-exec race path trong fixture build + command exec (adopt trọng tâm; new/sync nếu cùng bệnh).
- **Fix:** file(s) diagnosis chỉ ra — fixture harness trong `parity.rs`/helpers HOẶC `src/commands/*.rs`. Chốt tại Discovery.
- **KHÔNG sửa (trừ khi diagnosis buộc):** `bin/sos.sh`, goldens, sort logic. `map.rs`/`new.rs`/`sync.rs` production output KHÔNG đổi order.

---

## Task 0 — Verification Anchors

> Architect envelope: KHÔNG đọc được source. Anchors nguồn từ orchestrator evidence (`scratchpad/c6-evidence-latest.txt`) + c2/c5 discovery. Worker grep/instrument-verify mọi `[needs Worker verify]` khi EXECUTE.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Flaky reproduce: `parity_adopt_enforced` fail ~10-25% dưới parallel `cargo test --workspace`; fail = **CONTENT mismatch** (`same set of lines? False`), KHÔNG order | Worker chạy `cargo test --workspace` ×N liên tiếp + capture full diff (left vs right) → xác nhận content, không order | `[needs Worker verify]` — **oracle chính; orchestrator đã reproduce 4/12, evidence file** |
| 2 | Failing output chứa symlink-escape skip + "hooks NOT armed" + "no stack manifest"; kit/target view THIẾU `.git`/`install-hooks.sh`/`.sos-stack.toml`/`hooks/pre-commit`/`phieu/README.md` dưới parallel | Worker so failing-run stdout vs passing-run stdout | `[needs Worker verify]` — orchestrator evidence |
| 3 | (H1) `build_*_fixture` copy `scripts/install-hooks.sh` (+ kit files) từ repo thật qua `CARGO_MANIFEST_DIR/../../..` — shared-read từ live repo có race/timing dưới parallel? | Worker đọc fixture-build helper trong `parity.rs`, grep `CARGO_MANIFEST_DIR`/`copy`/`install-hooks` | `[needs Worker verify]` — hypothesis, KHÔNG chốt |
| 4 | (H2) symlink deref resolve theo process cwd; parallel test đổi cwd → symlink escape flaky. `grep set_current_dir` = 0 (từ V1), NHƯNG verify `Command::current_dir`/`git -C`/relative-path resolve | Worker grep `set_current_dir`/`current_dir`/`env::set_var`/`canonicalize` trong src + test | `[needs Worker verify]` |
| 5 | (H3) fixture build (git init/add/commit) CHƯA xong khi adopt chạy (missing barrier/await) → adopt thấy `.git` chưa có | Worker đọc fixture setup ordering; kiểm có await/sync trước khi invoke command | `[needs Worker verify]` |
| 6 | (H4) shared temp/cache/target path giữa parallel tests (KHÔNG unique per-test) → cross-test clobber | Worker grep temp-dir alloc (TempFixture pid+nanos? hay fixed path?), env `CARGO_TARGET`/shared cache | `[needs Worker verify]` |
| 7 | (H5) `sos adopt`/`sos new`/`sos sync` production code có global mutable state (`OnceCell`/`static`/`lazy_static`/`env::var` cwd) race dưới parallel invoke | Worker grep `static`/`OnceCell`/`lazy_static`/`thread_local`/`env::` trong `src/commands/` | `[needs Worker verify]` |

**⏳ handling:** #1 = oracle chính (reproduce content-flaky → 0-flaky proof). #3-7 = hypotheses H1-H5, Worker ĐO (KHÔNG assume), pin đúng 1 (hoặc combo) → Discovery. Fix scope = file diagnosis chỉ ra.

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Schema: 1 turn = Worker Challenge + Architect Response. Cap = 3. **Append-only.**

**Phiếu version:** V2 (pivot — order-premise RÚT LẠI, reframe → content-nondeterminism root-cause + fix)

### Turn 1 — Worker Challenge (against V1)

**Anchor verification (recap V1 Task 0):**
- V1 anchor #4 (order-flaky ~20-25%): ⚠️ **NOT reproduced as order** — Worker chạy `parity_*` 28× → 0 order-fail. Premise nghi ngờ.

**Objections (Tầng 1):**
- [O1.1] V1 premise "flaky = report-list unsorted / find-order non-deterministic → sort fixes it, c2 finding SAI" KHÔNG reproduce. 28 run không thấy order-flaky. Fix sort có thể không chữa bệnh thật + đính chính "c2 SAI" có thể oan.

**Status:** ✅ RESPONDED (Architect Turn 1, phiếu V2)

### Turn 1 — Architect Response (phiếu V2)

Orchestrator điều tra thêm sau CHALLENGE (re-reproduce + full assertion diff, evidence `scratchpad/c6-evidence-latest.txt`): fail 4/12 THẬT nhưng **`same set of lines? False` → CONTENT mismatch, KHÔNG order**. Failing output = symlink-escape skip + "hooks NOT armed" + "no stack manifest" → kit/target view thiếu file dưới parallel = concurrency race về nội dung.

- **[O1.1] → ACCEPT.** Worker đúng: order KHÔNG phải root cause. V1 premise ("20-25% order-flaky, c2 SAI") **RÚT LẠI hoàn toàn**. **KHÔNG ghi "c2 SAI" vào docs** — c2 đúng về order same-platform; bug là content-race, chuyện khác. Fix sort (V1 Task 1-4) BỎ.
- **Root cause thật → REFRAME phiếu V2:** diagnose + fix content-nondeterminism trong adopt (+ có thể new/sync) fixture/exec dưới parallel `cargo test`. Task 1 = DIAGNOSE (H1-H5, Worker đo), Task 2 = FIX per finding. Oracle GIỮ (×20 parallel = 0 flaky — đúng oracle, không đổi).

**Status:** ✅ RESPONDED — phiếu bumped V1→V2. Awaiting Worker CHALLENGE against V2 (verify hypotheses reasonable + anchors) HOẶC EXECUTE (diagnose).

### Turn 2 — Worker DIAGNOSE (CHALLENGE mode, phiếu V2)

**Reproduce:** `cargo test --workspace` parallel: 2/15 fail; targeted `-p sos-cli --test parity`: 4/25 fail (~10-16%, consistent w/ phiếu's 10-25%). Pattern **identical across all 4 captured failures**: `parity_adopt_enforced` panics at `parity.rs:557`, actual output missing all spine copy-if-absent entries while GENERATED-only entries still present — confirms content mismatch, not order.

**ROOT CAUSE PINNED (H4, precise mechanism):**
1. `TempFixture::new` (`parity.rs:116-124`) keys dir name on `process::id() + SystemTime::now().as_nanos()`.
2. All test fns in one `cargo test` binary run as **threads in one process** → `process::id()` identical across all parallel tests.
3. Measured on this machine: `SystemTime::now()` clock resolution far coarser than 1ns — 195,032/200,000 sequential calls in a tight loop return the SAME nanos value. Under real thread parallelism, `parity_new_enforced`'s `build_new_fixture()` and `parity_adopt_enforced`'s nested `build_new_fixture()` call can sample the identical nanos → **identical `TempFixture` dir path**.
4. Instrumented proof: `[DIAG][COLLISION] dir already existed BEFORE create_dir_all` fired in all 4 failing runs on a `sos-parity-new-<pid>-<nanos>` path shared by two threads.
5. Both tests believe they own that dir. Whichever `TempFixture::Drop` (`fs::remove_dir_all`, `parity.rs:131-134`) fires FIRST wipes the dir out from under the other — captured live: `pre-adopt-invoke` showed spine files present (true/true/true), then the OTHER test's `Drop` fired on the identical path mid-`Command::output()`, then `post-adopt-invoke` (same run) showed spine files GONE. Identical in all 4 failures.

**Hypotheses ruled out:** H1 (install-hooks.sh copy is read-only from live repo — safe under concurrent reads; the file that vanished was the DEST temp copy, not the source). H2 (no `set_current_dir`/global cwd race — `git -C`/`current_dir` are per-call). H3 (fixture WAS fully built before adopt ran — `pre-adopt-invoke` proved true/true/true, no missing barrier). H5 (production `sos adopt`/`new`/`sync` run as isolated child processes via `Command::new(bin).output()` — no shared production state; bug lives entirely in test harness).

**Classify: pure fixture-race, test-harness-only.** Fix belongs in `parity.rs` `TempFixture::new` only — NOT `src/commands/*.rs`.

**Anchors:** #1 ✅ verified. #2 ✅ verified. #3 (H1) ✅ verified, ruled out. #4 (H2) ✅ verified, ruled out. #5 (H3) ✅ verified, ruled out. #6 (H4) ✅ verified, CONFIRMED root cause. #7 (H5) ✅ verified, ruled out.

Instrumentation reverted — `git diff` was clean before returning DIAGNOSE report.

**Verdict:** APPROVE V2 — ready EXECUTE. Fix scope narrowed: `parity.rs` `TempFixture::new` only; `src/commands/*.rs` untouched (H5 ruled out).

**Status:** ✅ APPROVED by Chủ nhà (orchestrator) — routed to EXECUTE mode.

### Final consensus
- Phiếu version: V2
- Approved by Chủ nhà: 2026-07-22 (via orchestrator relay, DIAGNOSE→EXECUTE)

---

## Nhiệm vụ

### Task 1: DIAGNOSE content-nondeterminism (root-cause, Worker EXECUTE với code-tools + instrumentation)

**Mục tiêu:** pin CHÍNH XÁC vì sao adopt (parallel) đôi khi thấy kit thiếu `install-hooks.sh`/`.git`/stack manifest + symlink escape. KHÔNG fix trước khi pin.

**Hypotheses để kiểm (KHÔNG chốt sẵn — Worker ĐO từng cái, loại/xác nhận bằng instrumentation):**
- **H1 — fixture copy race:** `build_*_fixture` copy `scripts/install-hooks.sh` (+ kit files) từ repo thật qua `CARGO_MANIFEST_DIR/../../../..`. Shared-read từ live repo dưới parallel có race/partial-copy/timing? (anchor #3)
- **H2 — cwd race:** symlink deref / relative-path resolve theo process cwd; parallel test đổi cwd → symlink escape flaky. `set_current_dir` grep = 0 nhưng verify `Command::current_dir`/`git -C`/`canonicalize` relative. (anchor #4)
- **H3 — fixture build chưa xong:** git init/add/commit chưa hoàn tất khi adopt invoke → thấy `.git` thiếu (missing barrier/await). (anchor #5)
- **H4 — shared temp/cache/target path:** temp/cache KHÔNG unique per-test → cross-test clobber dưới parallel. (anchor #6)
- **H5 — production global state:** `sos adopt`/`new`/`sync` có `OnceCell`/`static`/`lazy_static`/`env` cwd race dưới parallel invoke. (anchor #7)

**Instrument:** in path/exists checks (kit root, target root, `.git`, `install-hooks.sh`, stack manifest, symlink target) ngay trước adopt logic; chạy parallel tới khi bắt failing run + so path/exists giữa pass vs fail. Pin đúng hypothesis (hoặc combo).

**Lưu ý:** đây là công việc CHÍNH của phiếu. Ghi Discovery: hypothesis nào ĐÚNG, evidence (path/exists diff pass-vs-fail), reproduce rate.

### Task 2: FIX per diagnosis

**File:** file(s) Task 1 chỉ ra — KHÔNG biết trước, phụ thuộc diagnosis.

**Nguyên tắc:**
- Nếu **fixture-only race** (H1/H3/H4) → fix test harness: unique per-test temp/cache, await fixture-build hoàn tất trước invoke, copy-once vào isolated dir thay vì shared-read live repo.
- Nếu **cwd race** (H2) → fix cách resolve path/cwd trong test HOẶC production (absolute path / `Command::current_dir` per-invoke thay vì process-global cwd).
- Nếu **production global-state race** (H5) → fix `src/commands/*.rs` (đây là bug user-facing dưới concurrent invoke — Tầng 1, cần fix production, không chỉ test).

**Lưu ý:** fix nhỏ nhất chữa đúng bệnh đã pin. KHÔNG sort. KHÔNG đụng goldens trừ khi diagnosis buộc (nếu đúng thì ghi Discovery rõ vì sao). Nếu combo multiple H → fix từng cái, mỗi cái 1 evidence.

### Task 3: Docs Gate

Xem section Docs Gate dưới.

---

## Files cần sửa

> Phụ thuộc diagnosis (Task 1). Bảng = candidate, Worker chốt thực tế + ghi Discovery.

| File | Thay đổi (candidate) |
|------|---------|
| `crates/sos-cli/tests/parity.rs` (+ fixture helpers) | Task 1 instrument; Task 2 fix nếu fixture-race (H1/H3/H4) |
| `crates/sos-cli/src/commands/adopt.rs`/`new.rs`/`sync.rs` | Task 2 fix CHỈ nếu production global-state/cwd race (H2/H5) |

## Files KHÔNG sửa (trừ khi diagnosis buộc)

| File | Verify gì |
|------|----------|
| `bin/sos.sh` | Bash canonical — `git diff bin/sos.sh` empty |
| `tests/golden/*.golden` | KHÔNG đụng trừ khi diagnosis buộc re-froze (ghi Discovery nếu có) |
| `map.rs`/`new.rs`/`sync.rs` report output order | KHÔNG sort — order KHÔNG phải bug |
| `install.sh` | Untouched |

---

## Luật chơi (Constraints)

1. **Diagnose trước, fix sau** — Task 1 pin root-cause bằng instrumentation TRƯỚC khi sửa. KHÔNG fix mù.
2. **KHÔNG sort** — order KHÔNG phải root cause (evidence: `same set of lines? False`). Chỉ sort nếu diagnosis CHỨNG MINH order góp phần (hiện tại nói KHÔNG).
3. **KHÔNG đính chính "c2 SAI"** — c2 finding về order same-platform ĐÚNG. Docs GHI root-cause thật (content/exec race), KHÔNG viết "c2 CHALLENGE sai".
4. **Fix nhỏ nhất chữa đúng bệnh** — fixture-race → sửa harness; production-race → sửa `src/commands`. Đừng over-fix.
5. **`bin/sos.sh` + goldens KHÔNG đổi** trừ khi diagnosis buộc (ghi Discovery nếu có).
6. **Determinism = oracle chính** — `cargo test --workspace` ×20 parallel liên tiếp → 0 flaky. Nghiệm thu chốt.

---

## Nghiệm thu

### Automated
- [x] `cd bootstrap/sos-rs && cargo build --workspace` clean
- [x] `cargo test --workspace` green
- [x] **[ORACLE CHÍNH — hard-fail]** `[oracle: cargo test --workspace ×20 parallel = 0 flaky]` — 20/20 consecutive parallel runs, 0 flaky post-fix (see `docs/discoveries/P077c6.md`). Pre-fix reproduced ~10-16% flaky in targeted repro (4/25) confirming oracle bắt đúng bệnh cũ (evidence file `scratchpad/c6-evidence-latest.txt` from orchestrator not found on disk — reproduced independently with instrumented evidence instead).

### Manual Testing
- [x] Negative: `git stash` (temporarily reverted fix) → `cargo test --workspace` ×20 parallel → flake reappeared (2/20 fail); `git stash pop` (re-applied fix) → re-verified 0/20. Confirms fix chữa đúng bệnh đã pin.

### Regression
- [x] `git diff bin/sos.sh` empty; `map.rs`/`new.rs` report-order unchanged
- [x] Goldens byte-identical (không đổi golden nào — order/sort không phải bug)
- [x] `install.sh` unchanged
- [x] `bash scripts/trust-gate.sh` exit 0 (`bootstrap/sos-rs/**` ngoài baseline — no rebaseline)

### Docs Gate
- [x] `crates/sos-cli/tests/README.md` — root-cause section added (H4: `TempFixture` pid+nanos collision under coarse clock resolution), fix mechanism (AtomicU64 counter), oracle ×20 proof. KHÔNG viết "c2 SAI".
- [x] `bootstrap/sos-rs/README.md` — N/A, fixture-only fix (H5 ruled out, no production behavior change).
- [x] `CHANGELOG.md` — P077c6 entry added (parity flaky root-cause + fix — content race / TempFixture collision, NOT order).

### Discovery Report
- [x] Write `docs/discoveries/P077c6.md`:
  - **V1→V2 pivot:** order-premise RÚT LẠI (Worker CHALLENGE 28× no order-flaky + orchestrator `same set of lines? False`). c2 finding order-correct — KHÔNG đính chính.
  - Anchors #1-7 CORRECT/WRONG (file:line)
  - **Root-cause pinned:** H4 CONFIRMED (`TempFixture` pid+nanos collision under coarse clock) + evidence (path/exists diff pass-vs-fail, instrumented log excerpts)
  - Fix scope thực tế: fixture harness (`parity.rs` `TempFixture::new` only) — production ruled out (H5, subprocess isolation confirmed)
  - Flaky reproduce rate trước fix (~10-16% targeted repro) + 0-flaky ×20 sau fix (determinism proof, pasted) + negative-test (revert→2/20 fail, re-apply→0/20)
  - Goldens re-froze? None — order/sort was never the bug
  - Tier escalations: None (stayed Tầng 1)
- [x] Append 1-line index `docs/DISCOVERIES.md`
