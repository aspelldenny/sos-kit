# PHIẾU P077c1: Rust `sos map` → parity + per-command harness hard-fail flip

> Sub-phiếu ĐẦU của P077c decomposition (`docs/plans/P077c-decomposition.md`). Additive-only; Bash `bin/sos.sh` GIỮ canonical + KHÔNG đổi. OA-02 fix KHÔNG ở đây (P077c5 — oracle mâu thuẫn với parity, xem decomposition).

---

> **Loại:** Feature (infra)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — parity impl + harness flip mechanism dùng lại cho c2/c3/c4; sai thì LAN xuống toàn P077c)
> **Lane:** Guarded
> **Ảnh hưởng:** `bootstrap/sos-rs/crates/sos-cli/src/main.rs`, `bootstrap/sos-rs/crates/sos-cli/src/commands/map.rs` (mới), `bootstrap/sos-rs/crates/sos-cli/tests/parity.rs`, `bootstrap/sos-rs/crates/sos-cli/tests/golden/capture.sh` (additive: freeze file-output golden), `.../tests/golden/map.agent_map.golden` (mới)
> **Dependency:** P077a (golden oracle + informational harness) + P077b (crate boundary) — cả hai đã landed.

---

## Context

### Vấn đề hiện tại

P077a freeze Bash golden (`map/new/adopt/sync`) + dựng parity-harness **informational** (`HARD_FAIL = false`, chỉ in "not yet parity"). P077b carve crate boundary, dời harness sang `crates/sos-cli/tests/`. Rust hiện **chưa có** command `new/adopt/map/sync` (P077b Discovery #3: `crates/sos-cli/src/commands/` chỉ có `apply/blueprint/contract/init/launch/recipe/status`).

**⚠️ V2 — lỗ hổng parity oracle (Worker CHALLENGE Turn 1 verified):** `bin/sos.sh` map (`bin/sos.sh:279-352`) ghi surfaces THẬT vào **file** `$target/docs/AGENT_MAP.yaml`, chỉ echo **1 dòng** stdout confirmation (`✓ sos map: scanned <TARGET> → ... (draft_needs_review ...)`, 123b — KHÔNG chứa surface data). `map.golden` = đúng 1 dòng stdout đó. `capture.sh` freeze **stdout only**, không freeze file. Hệ quả: parity oracle của map **MÙ với scan correctness** — Rust map chỉ cần in đúng 1 dòng + ghi *một* file bất kỳ là "parity pass", KHÔNG verify nội dung `AGENT_MAP.yaml` (pattern-list, sort, OA-02 bug reproduction). Đó chính là **false-green class OA-02** (mỉa mai: oracle chống-drift lại tự drift). Vì c4 (adopt) gọi map bug-for-bug từ c1, scan impl chưa-verify sẽ LAN xuống c4.

P077c1 giải command **nhỏ nhất trước** (`map`) để: (a) impl Rust map tới parity bug-for-bug với `map.golden` **VÀ** với file-output golden mới; (b) biến harness từ "in thông báo" → **thực sự chạy Rust map, normalize, diff CẢ stdout LẪN file-content**; (c) refactor `HARD_FAIL` 1-const → **per-command set** để `map` hard-fail còn `new/adopt/sync` giữ informational. Cơ chế per-command flip này c2/c3/c4 dùng lại.

### Giải pháp

Additive, KHÔNG đổi `bin/sos.sh`:

1. **Impl Rust `map`** — thêm clap subcommand `map` + `commands/map.rs`, port logic scan-surface từ `bin/sos.sh` map **bug-for-bug** (KHÔNG OA-02 — giữ nguyên scanner pattern-list hiện tại của Bash, giữ nguyên việc KHÔNG map `src/*.rs`, giữ nguyên sort order tại nguồn). Reproduce **cả hai** work-product: (i) ghi `<target>/docs/AGENT_MAP.yaml` nội dung surface, (ii) echo đúng 1 dòng stdout confirmation.
2. **Harness real comparison — hai fixture** — `parity.rs` hiện in "not yet parity" mà KHÔNG chạy Rust. Đổi: dựng lại cùng fixture repo mà `capture.sh` dùng cho map, chạy Rust `map` trên đó, normalize theo đúng rule của `capture.sh`, diff **stdout** với `map.golden` **VÀ** diff nội dung `docs/AGENT_MAP.yaml` với `map.agent_map.golden` (fixture mới). Hard-fail nếu **một trong hai** khác.
3. **Freeze file-output golden** — mở rộng `capture.sh` (additive branch, KHÔNG đổi normalize rule hiện có) để `sos_map` fixture ngoài freeze stdout còn `cat` + normalize `<target>/docs/AGENT_MAP.yaml` → `map.agent_map.golden`. Đây là work-product THẬT của map, phải là reference chính.
4. **Per-command hard-fail flip** — `HARD_FAIL` (single bool P077a để lại) → set/list các command đã-parity (vd `PARITY_ENFORCED = ["map"]`). Command trong set → assert equal cả stdout+file. Command ngoài set (`new/adopt/sync`) → giữ informational print như cũ.

### Scope

- CHỈ: `commands/map.rs` (mới) + clap wiring trong `main.rs` + `parity.rs` (real comparison stdout+file + per-command flip) + `capture.sh` additive freeze file-output + `map.agent_map.golden` (mới).
- KHÔNG: OA-02 (P077c5) · `new/adopt/sync` impl (c2/c3/c4) · install framework (P077d) · cutover (P077e) · đổi `bin/sos.sh` (oracle, freeze) · đổi normalize rule / `map.golden` nội dung (nó là reference; nếu Rust không match, sửa Rust KHÔNG sửa golden — trừ khi golden sai, thì DISCOVERY + escalate).

---

## Task 0 — Verification Anchors

> Architect docs-only (no Bash/grep/cargo, no read src/ hay tests/). Anchor code-level = `[needs Worker verify]` — Worker grep/cat/run ở EXECUTE.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Rust `crates/sos-cli/src/commands/` KHÔNG có `map.rs`; clap trong `main.rs` KHÔNG có `map` variant | `ls bootstrap/sos-rs/crates/sos-cli/src/commands/` + grep `map` trong `main.rs` clap enum | `[unverified]` — per P077b Discovery #3 recap ("no new/adopt/map/sync present") |
| 2 | Golden `crates/sos-cli/tests/golden/map.golden` = **1 dòng stdout confirmation** (`✓ sos map: scanned <TARGET> → <TARGET>/docs/AGENT_MAP.yaml (draft_needs_review ...)`, 123b) — **KHÔNG chứa surface data**. Surface THẬT nằm ở file `<target>/docs/AGENT_MAP.yaml` mà map ghi ra, `capture.sh` hiện **chưa freeze**. | `cat bootstrap/sos-rs/crates/sos-cli/tests/golden/map.golden` | `[verified per Worker CHALLENGE Turn 1]` — Worker confirmed 123b, 1-dòng stdout, file write tại `bin/sos.sh:279-352` |
| 3 | `bin/sos.sh` map logic (`bin/sos.sh:279-352`): scan surface, ghi vào `<target>/docs/AGENT_MAP.yaml`; scanner pattern-list = `routes/handlers/views/controllers/api`, `models/entities/schema`, `services/lib`, `migrations`, `templates/components/static` + vài config — **KHÔNG có generic `src/*.rs`** (chính là bug OA-02, port Y NGUYÊN ở c1); output sorted tại nguồn | Worker đọc block map dispatch `bin/sos.sh:279-352` | `[needs Worker verify]` — pattern-list từ audit OA-02 §Root cause; Worker cat block để lấy pattern + sort chính xác |
| 4 | `capture.sh` normalize map với rule: abs-path→`<TARGET>`/`<SOS_KIT_DIR>`, date→`<DATE>` (map đã sorted sẵn nên path-norm là rule chính áp cho **cả** stdout lẫn file-content AGENT_MAP.yaml) | `cat bootstrap/sos-rs/crates/sos-cli/tests/golden/capture.sh` (Worker đọc `sos_map` branch + normalize fns) | `[needs Worker verify]` — per P077a Discovery §Normalization |
| 5 | `parity.rs` hiện có `HARD_FAIL` const (single bool = false) là điểm flip duy nhất; test `parity_skeleton_informational` in per-command "not yet parity" **mà VẪN chạy binary thật + diff stdout** (KHÔNG phải "print without run"). Thiếu: fixture construction (routes/+models/ files) + normalize step + **file-content diff** | `cat bootstrap/sos-rs/crates/sos-cli/tests/parity.rs` (Worker đọc struct hiện tại) | `[verified per Worker CHALLENGE Turn 1]` — Worker confirmed `parity.rs` đã run binary + diff stdout; V1 anchor #5 ("KHÔNG chạy Rust binary") SAI, đã sửa |
| 6 | Fixture repo mà `capture.sh` dùng cho map là throwaway repo dựng trong script (isolated `bash -c` subshell) với `routes/` + `models/` files — harness cần **tái dựng cùng fixture** để chạy Rust map so sánh cả stdout+file | `cat capture.sh` (Worker xem cách dựng fixture cho `sos_map`) | `[needs Worker verify]` — per P077a Discovery §capture.sh + Worker CHALLENGE (routes/+models/) |
| 7 | AGENT_MAP.yaml file-output (surface data thật) KHÔNG chứa `src/*.rs` surface (bug OA-02 hiện diện trong scan) → Rust map bug-for-bug cũng KHÔNG emit `src/*.rs` → `map.agent_map.golden` match | Worker so nội dung `docs/AGENT_MAP.yaml` (do Bash map sinh trên fixture) với anchor #3 | `[needs Worker verify]` — **NẾU file CÓ src surface, anchor #3 sai → DISCOVERY + hỏi Architect** |

**⚠️ Anchors #3,4,6,7 = `[needs Worker verify]`** — Bash map logic + capture.sh normalize + fixture setup + file-content, Worker cat/grep/run ở EXECUTE trước khi impl. Anchors #2,#5 = `[verified per Worker CHALLENGE Turn 1]`.

**⚠️ Collision escape hatch:** nếu Worker phát hiện `AGENT_MAP.yaml` fixture output đã chứa `src/*.rs` HOẶC kit-managed assets mà scanner sẽ đụng (tức parity bug-for-bug và OA-02 sẽ va nhau ngay ở c1) → DỪNG, ghi DISCOVERY, escalate Architect: có thể phải kéo OA-02 vào sớm hoặc đổi fixture. (Kỳ vọng: fixture tối giản → không đụng, nhưng phải verify.)

---

## Debate Log

**Phiếu version:** V2 (Architect respond Turn 1 — file-output parity gap ACCEPTED)

### Turn 1 — Worker Challenge

**Anchor verification (recap Task 0):**
- Anchor #2: ⚠️ CORRECTION — `map.golden` = **1 dòng stdout** (123b) confirmation, KHÔNG chứa surface data. `bin/sos.sh:279-352` ghi surface THẬT vào **file** `<target>/docs/AGENT_MAP.yaml`; `capture.sh` freeze stdout only, không freeze file. V1 anchor #2 ("nội dung = surface map đã normalize") gây hiểu lầm.
- Anchor #5: ⚠️ CORRECTION — `parity.rs` ĐÃ chạy binary thật + diff stdout (không phải "print without run"). Thiếu: fixture construction + normalize + file-content diff. V1 anchor #5 sai factually.

**Objections (Tầng 1 only):**
- [O1.1] Task 2 (V1 dòng ~106) viết "emit đúng format `map.golden`... surface names, path list, `status:` field" — SAI factually: golden chỉ là 1-dòng stdout, KHÔNG có mấy field đó. Cần sửa mô tả target đúng (1-dòng stdout confirmation + file write `AGENT_MAP.yaml` verify riêng).
- [O1.2] Nghiệm thu chỉ check golden byte-match (stdout) + eyeball file — KHÔNG có automated fixture freeze/check nội dung `AGENT_MAP.yaml`. Vì parity oracle mù với work-product THẬT (file), "parity bug-for-bug" cho map **không được verify ở phần quan trọng** (pattern-list, sort, OA-02 reproduction). c4 (adopt gọi map) kế thừa scan impl unverified.

**2 alternatives Worker đề xuất:**
- **A (Worker recommend):** thêm fixture thứ 2 — freeze nội dung file `AGENT_MAP.yaml` thật từ `capture.sh` map fixture (`map.agent_map.golden`) + diff cả 2 trong harness, hard-fail cả hai.
- **B:** ship stdout-only, document gap loud trong Discovery + scope c4 re-verify map file output.

**Status:** ✅ RESPONDED — phiếu bumped to V2

### Turn 1 — Architect Response (phiếu V2)

- **[O1.1] → ACCEPT.** Facts đúng: `map.golden` = 1-dòng stdout, không có surface fields. Sửa anchor #2 (nay ghi rõ 1-dòng + file-write ở `bin/sos.sh:279-352`), sửa anchor #5 (parity.rs đã run binary + diff stdout). Task 2 viết lại: impl map ghi `<target>/docs/AGENT_MAP.yaml` (surface data) + echo đúng 1 dòng stdout confirmation; bỏ mô tả sai "surface names/path list/status field trong golden".
- **[O1.2] → ACCEPT (Alt A).** Parity oracle mù với work-product THẬT = false-green class OA-02 (oracle chống-drift tự drift). Ship stdout-only (Alt B) để lại đúng lỗ mà toàn P077c chống. Chọn Alt A: thêm fixture `map.agent_map.golden` (freeze nội dung `AGENT_MAP.yaml` do Bash map sinh, normalize qua `capture.sh` additive branch), harness diff **cả stdout LẪN file-content**, hard-fail nếu một trong hai lệch. `capture.sh` mở rộng là **additive** (thêm 1 freeze artifact, KHÔNG đổi normalize rule / golden hiện có) — Constraint #1 (`bin/sos.sh` freeze) không đụng; nới Constraint để cho phép additive freeze branch trong `capture.sh`.
- **Containment (KHÔNG phình scope c1):** fix c1 đúng phạm vi (map + file fixture). Gap tổng quát — golden của c3(new)/c4(adopt) freeze stdout-only trong khi work-product của chúng LÀ file (new gen files, adopt onboarding writes) → thiếu tương tự. KHÔNG kéo c3/c4 vào c1; thay vào **FLAG feed-forward** trong `docs/plans/P077c-decomposition.md` để c3/c4 thêm file-output fixture tương ứng.

**Status:** ✅ RESPONDED — phiếu V2. Recommend orchestrator: Worker CHALLENGE lại verify consensus (anchors #3,4,6,7 vẫn cần grep) HOẶC proceed approval gate nếu Worker đồng thuận file-fixture design.

### Final consensus
- Phiếu version: V<N>
- Approved by Chủ nhà: [date]

---

## Nhiệm vụ

### Task 1: Thêm Rust `map` subcommand (clap wiring)

**File:** `bootstrap/sos-rs/crates/sos-cli/src/main.rs`

**Tìm:** clap subcommand enum (nơi khai `Apply`/`Blueprint`/`Contract`/`Init`/`Launch`/`Recipe`/`Status` variants) `[needs Worker verify]` — Worker grep enum để thấy tên + style chính xác.

**Thêm:** một `Map` variant + dispatch arm gọi `commands::map::run(...)`. Match đúng style các variant hiện có (args, doc-comment). Nếu Bash `map` nhận args (vd target dir / `--help`), mirror `[needs Worker verify]` theo `bin/sos.sh` map dispatch (anchor #3).

**Lưu ý:** giữ `mod map;` trong `commands/mod.rs`. KHÔNG đụng các variant khác.

### Task 2: Impl `commands/map.rs` — port Bash map bug-for-bug (file + stdout)

**File:** `bootstrap/sos-rs/crates/sos-cli/src/commands/map.rs` (mới)

**Tìm:** N/A (file mới).

**Thêm:** hàm `run(...)` reproduce `bin/sos.sh` map (`bin/sos.sh:279-352`) — **hai work-product**:
- **(i) File write** `<target>/docs/AGENT_MAP.yaml`: scan repo theo **đúng pattern-list Bash hiện tại** (anchor #3) — `routes/handlers/views/controllers/api`, `models/entities/schema`, `services/lib`, `migrations`, `templates/components/static` + config files. **KHÔNG thêm `src/*.rs`** (đó là OA-02/c5 — c1 bug-for-bug). Sort surface tại nguồn (Bash sort, anchor #3) → output deterministic. Ghi YAML surface content byte-identical (sau normalize) với `map.agent_map.golden`. Dùng `sos_core::state` nếu Bash map dùng để ghi (Worker verify path + writer).
- **(ii) Stdout** đúng **1 dòng** confirmation `✓ sos map: scanned <TARGET> → <TARGET>/docs/AGENT_MAP.yaml (draft_needs_review ...)` (anchor #2) — byte-identical (sau normalize) với `map.golden`.

**Lưu ý:** mục tiêu = **cả hai** khớp golden sau normalize: stdout==`map.golden` VÀ file-content==`map.agent_map.golden`. Bất kỳ khác biệt = sửa Rust, KHÔNG sửa golden. Nếu golden có gì Rust không thể reproduce deterministic → DISCOVERY.

### Task 3: Freeze file-output golden (capture.sh additive)

**File:** `bootstrap/sos-rs/crates/sos-cli/tests/golden/capture.sh`

**Tìm:** `sos_map` freeze branch (nơi chạy Bash `sos map` trên fixture + normalize stdout → `map.golden`) `[needs Worker verify]` anchor #4,#6.

**Thêm (additive, KHÔNG đổi normalize rule / branch khác):** sau khi Bash map chạy, `cat` file `<target>/docs/AGENT_MAP.yaml` mà map vừa ghi, normalize qua **đúng cùng** helper (abs-path→`<TARGET>`, date→`<DATE>`) → ghi `tests/golden/map.agent_map.golden`. Chỉ thêm cho `sos_map`; `new/adopt/sync` branch KHÔNG đụng ở c1 (feed-forward c3/c4).

**Lưu ý:** đây là freeze artifact mới, additive — KHÔNG sửa `map.golden` cũ, KHÔNG đổi normalize fn. Nếu re-run `capture.sh` đổi `map.golden` byte nào → DISCOVERY (nghĩa là branch cũ bị chạm nhầm).

### Task 4: Harness — real comparison stdout+file cho `map` + per-command hard-fail flip

**File:** `bootstrap/sos-rs/crates/sos-cli/tests/parity.rs`

**Tìm:** `HARD_FAIL` const (single bool, `= false`) + test hiện chạy binary + diff stdout, in "not yet parity" cho 4 command `[needs Worker verify]` anchor #5.

**Thay bằng / Thêm:**
- Đổi `HARD_FAIL` bool → cấu trúc **per-command** (vd `const PARITY_ENFORCED: &[&str] = &["map"];`).
- Cho `map` (trong set): dựng lại fixture như `capture.sh` (anchor #6 — routes/+models/), chạy Rust `map` (qua `assert_cmd`/`Command::cargo_bin` hoặc gọi `commands::map::run` trực tiếp — Worker chọn cơ chế rẻ nhất), normalize theo rule capture.sh (anchor #4), rồi **hai assert**:
  - `assert_eq!` normalized **stdout** vs `map.golden`;
  - `assert_eq!` normalized nội dung `<target>/docs/AGENT_MAP.yaml` (file Rust vừa ghi) vs `map.agent_map.golden`.
  - Diff **một trong hai** → **hard-fail** (message rõ fixture nào lệch).
- Cho `new/adopt/sync` (ngoài set): giữ nguyên print "not yet parity" informational (KHÔNG hard-fail).

**Lưu ý:** per-command set là cơ chế c2/c3/c4 dùng lại. Normalize phải KHỚP capture.sh 100% (tái dùng helper nếu Rust-side có, hoặc port đúng rule) — lệch normalize = false-diff. File-content assert là điểm bắt scan-correctness mà stdout-only bỏ lọt.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `bootstrap/sos-rs/crates/sos-cli/src/main.rs` | Task 1: thêm `Map` clap variant + dispatch |
| `bootstrap/sos-rs/crates/sos-cli/src/commands/mod.rs` | Task 1: `mod map;` |
| `bootstrap/sos-rs/crates/sos-cli/src/commands/map.rs` | Task 2: impl map bug-for-bug — ghi AGENT_MAP.yaml + echo 1-dòng stdout (mới) |
| `bootstrap/sos-rs/crates/sos-cli/tests/golden/capture.sh` | Task 3: additive branch freeze `map.agent_map.golden` (chỉ `sos_map`, KHÔNG đổi normalize/branch khác) |
| `bootstrap/sos-rs/crates/sos-cli/tests/golden/map.agent_map.golden` | Task 3: fixture mới — nội dung AGENT_MAP.yaml đã normalize (do capture.sh sinh) |
| `bootstrap/sos-rs/crates/sos-cli/tests/parity.rs` | Task 4: real comparison stdout+file + per-command flip |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `bin/sos.sh` | Oracle — freeze. `git diff bin/sos.sh` phải empty sau phiếu |
| `bootstrap/sos-rs/crates/sos-cli/tests/golden/map.golden` | Reference stdout — Rust match theo nó, KHÔNG sửa (trừ khi golden chứng minh sai → DISCOVERY) |
| `.../tests/golden/{new,adopt,sync}.golden` | Không đụng — c2/c3/c4 |

> **Lưu ý normalize:** `capture.sh` được sửa **additive-only** (thêm freeze `map.agent_map.golden` cho `sos_map`). Normalize fn + branch `new/adopt/sync` + `map.golden` cũ KHÔNG đổi. Re-run `capture.sh` phải để `map.golden`/`new`/`adopt`/`sync` golden byte-identical như trước.

---

## Luật chơi (Constraints)

1. **`bin/sos.sh` bất khả xâm phạm** — `git diff bin/sos.sh` empty; `bash scripts/trust-gate.sh` exit 0 (bootstrap/sos-rs/** không trong `.sos-trust-baseline`).
2. **Bug-for-bug, KHÔNG OA-02** — Rust map giữ nguyên bug scanner (không map `src/*.rs`, không exclude managed assets, không 3-verdict) cho **cả** file-output lẫn stdout. OA-02 là P077c5.
3. **Không sửa golden để match Rust** — chiều đúng là Rust→golden (cả `map.golden` lẫn `map.agent_map.golden`). Golden sai = DISCOVERY + escalate.
4. **`capture.sh` chỉ additive** — thêm freeze `map.agent_map.golden` cho `sos_map`; KHÔNG đổi normalize rule, KHÔNG chạm branch/golden khác. Re-run capture.sh → golden cũ byte-identical.
5. **Per-command flip additive** — `new/adopt/sync` PHẢI vẫn informational sau phiếu (chạy `cargo test -p sos-cli --test parity -- --nocapture`, confirm 3 command kia vẫn in "not yet parity", chỉ `map` enforced).
6. **Normalize khớp capture.sh 100%** — tái dùng/port đúng rule cho cả stdout lẫn file-content, không tự chế rule mới.

---

## Nghiệm thu

### Automated
- [ ] `cd bootstrap/sos-rs && cargo build --workspace` clean
- [ ] `cargo test -p sos-cli --test parity` — `map` parity test PASS: **cả** stdout==`map.golden` **VÀ** file-content==`map.agent_map.golden` (hai assert xanh)
- [ ] `cargo test --workspace` — toàn bộ xanh (dep-direction guard c2 P077b vẫn PASS)
- [ ] `cargo test -p sos-cli --test parity -- --nocapture` — confirm `new/adopt/sync` vẫn in "not yet parity" informational

### Manual Testing
- [ ] Chạy Rust `sos map` trên fixture của capture.sh → normalize tay → so `map.golden` (stdout) + so `docs/AGENT_MAP.yaml` với `map.agent_map.golden` (file), cả hai byte-identical
- [ ] Negative test A (file): sửa tạm 1 pattern trong `map.rs` scanner (vd bỏ 1 surface dir) → `cargo test parity` FAIL loud tại **file-content assert** (chứng minh oracle KHÔNG còn mù với scan-correctness) → revert
- [ ] Negative test B (stdout): sửa tạm dòng stdout confirmation → FAIL tại stdout assert → revert
- [ ] Re-run `capture.sh` → `git diff` cho thấy CHỈ `map.agent_map.golden` mới; `map.golden`/`new`/`adopt`/`sync` golden KHÔNG đổi (Constraint #4)

### Regression
- [ ] `bin/sos.sh map` chạy fixture y như trước (code path Bash không đụng)
- [ ] `new/adopt/sync` informational KHÔNG bị flip nhầm sang hard-fail

### Docs Gate
- [ ] `CHANGELOG.md` — entry P077c1
- [ ] `bootstrap/sos-rs/README.md` — command-status: `map` → "parity (hard-fail, stdout+file)", `new/adopt/sync` → "informational (pending c2/c3/c4)"
- [ ] `docs/plans/P077c-decomposition.md` — đánh dấu P077c1 done + **feed-forward note** (c3/c4 cần file-output fixture, đã thêm ở V2)
- [ ] `crates/sos-cli/tests/README.md` — cập nhật HARD_FAIL→per-command mechanism + map now enforced (stdout+file two-fixture pattern)

### Discovery Report
- [ ] `docs/discoveries/P077c1.md`
  - Anchors #3,4,6,7 — Bash-map-logic (`bin/sos.sh:279-352`) / normalize / fixture / file-content thực tế (cite `file:line`)
  - Anchor #7 collision check — AGENT_MAP.yaml có/không `src/*.rs` (quyết định OA-02 có va sớm không)
  - map.rs deviation từ Bash (nếu có) + lý do
  - Per-command flip mechanism + **two-fixture (stdout+file) pattern** cho c2/c3/c4 dùng lại — mô tả interface
  - Docs updated / Tier escalations (None nếu không)
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`
