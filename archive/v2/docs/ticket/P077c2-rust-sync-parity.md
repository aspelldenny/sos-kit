# PHIẾU P077c2: Rust `sos sync` → parity (bug-for-bug) + file-side-effect oracle

---

> **Loại:** Feature
> **Ưu tiên:** P1
> **Tầng:** 1 (impl + parity oracle + git-history provenance semantics = móng; sai thì LAN sang KIT-LAG cure của mọi adopted repo, và file-copy sai = KHÔNG-đảo)
> **Lane:** Guarded
> **Ảnh hưởng:** `bootstrap/sos-rs/crates/sos-cli/src/**` (Sync command mới), `bootstrap/sos-rs/crates/sos-cli/tests/parity.rs` + `tests/golden/capture.sh` + `tests/golden/sync*.golden`
> **Dependency:** P077c1 (per-command `PARITY_ENFORCED` set + two-fixture pattern trong harness) — SHIPPED, branch `P077c-rust-parity-impl`

---

## Context

### Vấn đề hiện tại

P077a đóng băng `sync.golden` (439b) làm parity contract cho `sos sync`, nhưng harness để `sync` ở mức **informational** (không hard-fail). c2 impl Rust `sos sync` tới parity bug-for-bug với Bash `sos_sync` (`bin/sos.sh:973-1053`) và flip `sync` sang PARITY_ENFORCED.

**Hai bẫy thiết kế phải giải (không được lặp lỗi c1):**

1. **False-green kiểu c1 (work-product là FILE, không phải stdout).** `sos_sync` thực chất **ghi file side-effects**: `cp` spine file vào target (ADDED / UPDATED) và `cp` file customized vào `.sos-sync-incoming/<destrel>` (FLAGGED) — xem `_sync_one` `bin/sos.sh:1000-1014`. `sync.golden` hiện chỉ freeze **stdout report** của kịch bản "0 thay đổi / 58 ALREADY-CURRENT". Stdout-only + 0-change = **mù hoàn toàn với logic copy/flag** (đúng false-green class OA-02 mà c1 đã vá). Một Rust `sos sync` in đúng dòng "ADDED 0 / UPDATED 0 / FLAGGED 0" mà KHÔNG copy file nào vẫn "pass" oracle hiện tại. Phải có file-output fixture exercise CẢ 3 case thật.

2. **Non-determinism qua sos-kit git history.** `sos_sync` phân loại UPDATED-vs-FLAGGED bằng `_blob_in_history` (`bin/sos.sh:992-999`): dest blob (`git hash-object`) khớp **bất kỳ historical blob nào của canonical path trong `$SOS_KIT_DIR` git history** → "stale-unmodified" → take-newer; khớp NONE → "customized" → flag. Golden `sync.golden` gốc chụp trên **real sos-kit tại một HEAD nào đó** → chỉ reproducible nếu pin HEAD sha, và mỗi commit vào sos-kit lại làm drift. `tests/README.md` đã có note này (từ c1/P077a).

### Giải pháp

**Synthetic self-contained fixture (chốt determinism + 3-case + no-real-HEAD-dependency).** Thay vì phụ thuộc real sos-kit HEAD, fixture dựng một **mini fake-kit git repo** làm `$SOS_KIT_DIR` với history được kiểm soát (blob v1 → blob v2 của spine file), trỏ `SOS_KIT_DIR` env vào đó, chạy Bash `sos_sync` để capture golden, rồi chạy Rust `sos sync` để diff. Điều này **loại bỏ hẳn phụ thuộc vào real-sos-kit HEAD** → golden = frozen bytes, reproducible mãi mãi, và exercise được cả 4 outcome trong một scenario deterministic:

| Outcome | Cách dựng trong fixture |
|---|---|
| **ADDED** | spine file có trong fake-kit HEAD, **absent** ở target → `_sync_one` copy vào target |
| **UPDATED** | target chứa **blob v1** (nằm trong fake-kit history) nhưng fake-kit HEAD là blob v2 → `_blob_in_history` khớp → take-newer overwrite |
| **FLAGGED** | target chứa content **tùy ý** (không nằm trong bất kỳ history blob nào) → copy vào `.sos-sync-incoming/<destrel>` |
| **ALREADY-CURRENT** | target chứa blob == fake-kit HEAD (`cmp -s` khớp) → không đụng |

**Two-fixture oracle (tái dùng cơ chế c1):** capture.sh freeze CẢ HAI, harness hard-fail cả hai:
1. `sync.golden` (**re-froze** từ real-kit-0-change sang synthetic-3-case) = **stdout report** (ADDED N / UPDATED N / FLAGGED N / ALREADY-CURRENT N).
2. `sync.tree.golden` (**MỚI**) = **manifest file-side-effect deterministic**: với mỗi path bị mutate, một dòng `<verb> <relpath> <sha256-content>` (verb ∈ ADDED/UPDATED/INCOMING), sorted. Freeze CẢ placement (file copy đúng chỗ, `.sos-sync-incoming/` đúng cấu trúc) LẪN content (sha256 nội dung). Đây là artifact bắt regression copy/flag mà stdout-only bỏ lọt.

**Rust impl phải đọc cùng git-history semantics:** replicate `_blob_in_history` = với canonical relpath, dest blob khớp bất kỳ blob nào qua `git rev-list --all -- <rel>` của `$SOS_KIT_DIR`. Bug-for-bug — cùng rev-list-all walk, cùng `git hash-object`. Worker verify Rust shell-out git vs git2 (anchor #10).

### Scope
- CHỈ sửa: `bootstrap/sos-rs/crates/sos-cli/src/**` (thêm Sync command + clap enum variant + dispatch), `tests/parity.rs` (thêm `parity_sync_enforced`, thêm `"sync"` vào `PARITY_ENFORCED`), `tests/golden/capture.sh` (synthetic fake-kit fixture + freeze 2 golden), `tests/golden/sync.golden` (re-froze), `tests/golden/sync.tree.golden` (mới).
- KHÔNG sửa: `bin/sos.sh` (Bash canonical — bug-for-bug, giữ nguyên logic kể cả quirk). KHÔNG đụng golden của map/new/adopt. KHÔNG fix gì (OA-02 không chạm sync).

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `sync.golden` (439b) hiện = stdout report real-kit kịch bản 0-change (ADDED/UPDATED/FLAGGED 0, ALREADY-CURRENT 58), KHÔNG chứa file-side-effect data | `cat tests/golden/sync.golden` | `[needs Worker verify]` (guard chặn Architect đọc golden) |
| 2 | `_blob_in_history` `bin/sos.sh:992-999` = walk `git -C $K rev-list --all -- <rel>`, so `git hash-object <dest>` với mỗi `<c>:<rel>` blob, return 0 nếu khớp bất kỳ | đã đọc trực tiếp | `[verified]` bin/sos.sh:992-999 |
| 3 | File side-effects `_sync_one` `bin/sos.sh:1000-1014`: ABSENT→`cp src dest` (ADDED); `cmp -s` match→ALREADY-CURRENT; blob-in-history→`cp src dest` (UPDATED); else→`cp src $incoming/$destrel` (FLAGGED). Identity files không nằm trong walk | đã đọc trực tiếp | `[verified]` bin/sos.sh:1000-1014 |
| 4 | Stdout report format `bin/sos.sh:1046-1052`: 4 dòng count + block list mỗi loại (`+`/`^`/`~` prefix) + "ALREADY-CURRENT: N" + dòng identity-files | đã đọc trực tiếp | `[verified]` bin/sos.sh:1046-1052 |
| 5 | Spine-walk set `bin/sos.sh:1027-1044`: roots `scripts phieu templates` (find -type f, loại `__pycache__`/`.pyc`/`.DS_Store`); rồi `hooks/pre-commit hooks/pre-push docs/ORCHESTRATION.md .claude/settings.json`; rồi `agents/*.md`→`.claude/agents/` (skip README.md); `.claude/commands/*`; `skills/**/SKILL.md`→`.claude/skills/` (skip attic) | đã đọc trực tiếp | `[verified]` bin/sos.sh:1027-1044 |
| 6 | `sos_sync` guards `bin/sos.sh:985-987`: `$K/.git` phải tồn tại (dùng history làm oracle), `$K/.claude/agents` phải tồn tại (nhận diện sos-kit). `$K = $SOS_KIT_DIR` → **overridable qua env** cho synthetic fixture | đã đọc trực tiếp | `[verified]` bin/sos.sh:985-987 |
| 7 | Harness c1: `const PARITY_ENFORCED: &[&str] = &["map"]`; `parity_skeleton_informational` skip command trong set; two-fixture pattern (capture.sh freeze file + dedicated `#[test]` assert stdout+file) | `grep -n "PARITY_ENFORCED\|two-fixture" tests/parity.rs tests/README.md` | `[needs Worker verify]` (guard chặn tests/) — precedent ghi ở `docs/discoveries/P077c1.md:75-79` |
| 8 | `crates/sos-cli` clap enum CHƯA có `Sync` variant (chỉ Init/Blueprint/Contract/Apply/Recipe/Launch/Status + Map từ c1) | `grep -n "enum Command\|Sync\|Map" crates/sos-cli/src/main.rs` | `[needs Worker verify]` — c1 discovery `docs/discoveries/P077c1.md:5` liệt kê enum không có Sync |
| 9 | **[ESCAPE HATCH]** Synthetic fake-kit fixture khả thi deterministic: `git init` mini-kit + 2 commit (blob v1→v2) + minimal spine (đủ qua guard `.claude/agents` + walk) + `SOS_KIT_DIR=<fake>` override, Bash+Rust cho cùng byte output | Worker dựng thử trong `parity.rs` build + chạy Bash `sos_sync` 2 lần → diff phải empty | `[needs Worker verify]` — **nếu flaky/non-det → DỪNG escalate (xem Constraint 6)** |
| 10 | Rust đọc git-history: workspace có git2 crate hay Rust shell-out `Command::new("git")`? Bug-for-bug cần cùng `rev-list --all` + `hash-object` semantics | `grep -rn "git2\|Command::new(\"git\"\|rev-list" crates/ Cargo.toml` | `[needs Worker verify]` |

**❌/⚠️ handling:** Anchor #1 re-froze — nếu Worker xác nhận golden hiện là 0-change real-kit thì việc re-froze sang synthetic-3-case là **có chủ đích** (ghi rõ trong Discovery: golden scenario thay đổi, KHÔNG chỉ additive như c1). Anchor #9 = escape-hatch gate, xem Constraint 6.

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Worker accepted V1 — no challenges.**

**Anchor verification (recap từ Task 0):**
- #1 ✅ `sync.golden` (439b) confirmed stdout-only, 0-change scenario (`ADDED/UPDATED/FLAGGED: 0`, `ALREADY-CURRENT: 58`), zero file-side-effect data.
- #2 ✅ `_blob_in_history` `bin/sos.sh:992-999` matches phiếu description exactly.
- #3 ✅ `_sync_one` `bin/sos.sh:1000-1014` matches; live-tested all 4 outcomes (ADDED/UPDATED/FLAGGED/ALREADY-CURRENT) against a real synthetic fake-kit — all behaved as phiếu's table predicts.
- #4 ✅ Stdout report format `bin/sos.sh:1046-1052` confirmed via live run.
- #5 ✅ Spine-walk set `bin/sos.sh:1027-1044` confirmed exact.
- #6 ✅ Guards `bin/sos.sh:985-987` confirmed; `SOS_KIT_DIR` override live-tested working against synthetic fake-kit.
- #7 ✅ `PARITY_ENFORCED = &["map"]` only (`parity.rs:28`); two-fixture pattern documented `tests/README.md:51-101`.
- #8 ✅ No `Sync` variant in clap enum yet (`main.rs`: Init/Blueprint/Contract/Apply/Recipe/Launch/Status/Map only).
- #9 ✅ **Escape hatch NOT triggered — synthetic determinism holds.** Live-built synthetic fake-kit (`git init` + 2 commits, blob v1→v2) + 3 target dirs covering all 4 outcomes; ran real `sos_sync` via `SOS_KIT_DIR` override 3× → normalized stdout byte-identical all 3 runs. `git init`-in-test not flaky (3/3 clean). Blob-sha confirmed content-only deterministic.
  - **Finding (new, not a blocker):** Bash's `find` for sync (`bin/sos.sh:1030-1031,1043-1044`) is **UNSORTED** — live-probe confirms it returns filesystem creation-order, NOT alphabetical (differs from c1's `map.rs`/Bash `scan_files`, which both explicitly `sort`). Rust's sync walker must **NOT** call `.sort()` on the report list (unlike `map.rs`'s `hits.sort()` precedent) — must preserve raw `read_dir` enumeration to match Bash's find-order for the stdout report (`sync.golden`). `sync.tree.golden` (file manifest) is designed SORTED so it's order-independent/safe regardless. Same-machine repeat-run order is deterministic (verified 3×); cross-platform (macOS dev vs future Linux CI) order-match is unverified but not urgent — no CI currently wires `cargo test` for `bootstrap/sos-rs` (checked `.github/workflows/*.yml`, no hits). Task 2's lưu ý + Constraint 6 already anticipated this exact risk — Worker will verify live find-order match at EXECUTE (Nghiệm thu step) and escalate per Constraint 6 if it doesn't reconcile, rather than silently sorting.
- #10 ✅ No `git2` dependency in any `Cargo.toml` — Rust must shell out `Command::new("git")`.

**Objections:** None — accept V1. The one real technical risk found (unsorted-find order-parity) is already correctly anticipated in Task 2's lưu ý and Constraint 6's escape hatch.

Ready for Chủ nhà approval gate.

### Final consensus
- Phiếu version: V1 (accepted, no revision needed)
- Approved by Chủ nhà: 2026-07-22 (delegated — CHALLENGE APPROVE V1 → EXECUTE mode, sprint delegated tự duyệt)

---

## Nhiệm vụ

### Task 1: Impl Rust `sos sync` — clap enum + dispatch

**File:** `bootstrap/sos-rs/crates/sos-cli/src/main.rs` (+ command module theo cấu trúc c1's `map.rs`, `[needs Worker verify]` đường dẫn module `src/commands/sync.rs` hay flat — theo pattern c1)

**Tìm:** clap `Command` enum (variant list Init/Blueprint/Contract/Apply/Recipe/Launch/Status/Map) + match dispatch.

**Thêm:** variant `Sync { target: PathBuf }` (+ dispatch gọi hàm sync). CLI shape khớp Bash: `sos sync <adopted-repo-dir>` (positional target, các `--*` flag bị nuốt như Bash `bin/sos.sh:982`).

**Lưu ý:** Bug-for-bug. Đọc `$SOS_KIT_DIR` env cho kit dir (KHÔNG hardcode). Guard giống Bash (`bin/sos.sh:984-987`): target tồn tại + là dir; `$K/.git` tồn tại; `$K/.claude/agents` tồn tại — cùng error string + exit code.

### Task 2: Port provenance + sync logic bug-for-bug

**File:** module sync Rust (Task 1).

**Tìm:** N/A (impl mới).

**Thêm:** replicate `_blob_in_history` (`bin/sos.sh:992-999`) + `_sync_one` (`:1000-1014`) + spine-walk (`:1027-1044`) + report (`:1046-1052`):
- Provenance: dest blob (`git hash-object` equiv) khớp bất kỳ blob qua `git -C $K rev-list --all -- <rel>` → UPDATED; else FLAGGED. Dùng git2 hoặc shell git theo anchor #10.
- File ops: ADDED = `cp src dest` (mkdir -p parent); UPDATED = overwrite; FLAGGED = copy vào `.sos-sync-incoming/<destrel>`; ALREADY-CURRENT = byte-cmp match, no-op.
- Walk set + skip rules (`__pycache__`/`.pyc`/`.DS_Store`, agents README.md, skills attic) **bit-exact** theo anchor #5.
- Dirty-warn (`bin/sos.sh:1021-1025`) + report block `+`/`^`/`~` prefix + count lines — byte-exact với `sync.golden`.

**Lưu ý:** Sorted-order của walk phải khớp Bash `find` order (Bash không `sort` walk — thứ tự `find` filesystem-dependent). **Worker verify:** nếu `find` order non-deterministic giữa run → có thể cần normalize/sort trong CẢ Bash-capture path lẫn Rust (nhưng KHÔNG đổi `bin/sos.sh`). Nếu order lệch → escalate (Constraint 6).

### Task 3: Synthetic fake-kit fixture trong capture.sh + freeze 2 golden

**File:** `bootstrap/sos-rs/crates/sos-cli/tests/golden/capture.sh`

**Tìm:** branch capture cho `sync` (hiện freeze stdout-only vào `sync.golden`).

**Thay bằng:** build synthetic scenario (xem bảng Giải pháp): `git init` mini-kit với minimal spine + 2-commit blob history cho ≥1 spine file; dựng target repo với 4 file trạng thái ADDED-absent / UPDATED-stale-v1 / FLAGGED-custom / CURRENT-v2; `SOS_KIT_DIR=<fake-kit>` chạy Bash `sos_sync`; normalize (target-abs → `<TARGET>`, kit-abs → `<SOS_KIT_DIR>`) rồi freeze:
1. stdout → `sync.golden` (re-froze).
2. post-sync target-tree manifest → `sync.tree.golden` (mỗi mutate path: `<verb> <relpath> <sha256>`, sorted).

**Lưu ý:** re-froze `sync.golden` = **scenario change có chủ đích** (không phải additive). Ghi rõ Discovery. Content của `.sos-sync-incoming/<flagged>` phải == kit source (verify qua sha256 trong manifest).

### Task 4: Flip `sync` → PARITY_ENFORCED, two-assert hard-fail

**File:** `bootstrap/sos-rs/crates/sos-cli/tests/parity.rs`

**Tìm:** `const PARITY_ENFORCED: &[&str] = &["map"];` + `parity_skeleton_informational`.

**Thay bằng:** thêm `"sync"` vào set; thêm `#[test] fn parity_sync_enforced()` — dựng cùng synthetic fixture, chạy Rust `sos sync`, assert stdout vs `sync.golden` AND tree-manifest vs `sync.tree.golden`, hard-fail cả hai. `parity_skeleton_informational` giữ `new`/`adopt` informational.

**Lưu ý:** Negative test (kiểu c1 Negative-test A): tạm sabotage 1 file-op (vd bỏ FLAGGED copy) → confirm fail fire trên **tree-manifest assert** (không phải stdout — nếu report count vẫn tính đúng mà file không copy). Chứng minh oracle bắt được false-green. Revert + re-green.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-cli/src/main.rs` (+ module sync) | Task 1-2: Sync command + provenance/file-op logic bug-for-bug |
| `crates/sos-cli/tests/golden/capture.sh` | Task 3: synthetic fake-kit fixture + freeze 2 golden |
| `crates/sos-cli/tests/golden/sync.golden` | Task 3: re-froze real-kit-0-change → synthetic-3-case stdout |
| `crates/sos-cli/tests/golden/sync.tree.golden` | Task 3: MỚI — file-side-effect manifest |
| `crates/sos-cli/tests/parity.rs` | Task 4: `sync`→PARITY_ENFORCED + `parity_sync_enforced` two-assert |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `bin/sos.sh` | Canonical — `git diff bin/sos.sh` PHẢI empty. Bug-for-bug, KHÔNG fix quirk |
| `tests/golden/{map,new,adopt}.golden` + `map.agent_map.golden` | c1/P077a fixtures — KHÔNG đụng |

---

## Luật chơi (Constraints)

1. **Bash `bin/sos.sh` bất khả xâm phạm** — `git diff bin/sos.sh` empty. Rust chứng minh bằng nhau, không sửa Bash.
2. **Bug-for-bug** — mọi quirk Bash (find-order, dirty-warn wording, identity-file exclusion) port y nguyên. KHÔNG "cải thiện". OA-02 không chạm sync.
3. **Two-fixture hard-fail** — `sync` phải fail nếu stdout HOẶC tree-manifest lệch. Stdout-only = c1 false-green đã bị cấm.
4. **File-side-effect PHẢI được verify** — golden 0-change stdout-only KHÔNG đủ nghiệm thu. Fixture PHẢI exercise thật ADDED+UPDATED+FLAGGED và freeze kết quả file-tree.
5. **Determinism qua synthetic kit** — golden KHÔNG được phụ thuộc real sos-kit HEAD. Fake-kit history frozen trong capture.sh. Nếu Worker vẫn phải pin real HEAD → escalate (đây là regression so với thiết kế).
6. **[ESCAPE HATCH]** Nếu synthetic-fake-kit fixture (anchor #9) chứng minh **non-deterministic hoặc quá phức tạp** (git-init-in-test flaky, blob-history không reproducible, find-order lệch Bash↔Rust, SOS_KIT_DIR override vỡ guard) → Worker **DỪNG, KHÔNG force**, escalate Chủ nhà với 2 option: (a) **split c2** → c2a stdout parity (real-HEAD-pin, file-op để lại note) + c2b file-fixture riêng; (b) fallback real-HEAD-pin 0-change golden + unit-test file-op tách rời (⚠️ tái nhập false-green risk c1 → PHẢI Chủ-nhà-approve, KHÔNG silent). KHÔNG tự ý chọn (b).

---

## Nghiệm thu

### Automated
- [ ] `cd bootstrap/sos-rs && cargo build --workspace` clean
- [ ] `cargo test -p sos-cli --test parity` — `parity_sync_enforced` PASS (stdout + tree-manifest), `parity_map_enforced` vẫn PASS, `parity_skeleton_informational` giữ new/adopt informational
- [ ] `cargo test --workspace` green

### Manual Testing
- [ ] Chạy Bash `sos_sync` + Rust `sos sync` trên cùng synthetic fixture, diff stdout → IDENTICAL
- [ ] Diff post-sync target file-tree (ADDED file content, UPDATED overwrite, `.sos-sync-incoming/<flagged>` content) Bash↔Rust → IDENTICAL
- [ ] Negative test: sabotage 1 file-op → `parity_sync_enforced` FAIL trên tree-manifest assert; revert → green

### Regression
- [ ] `git diff bin/sos.sh` empty
- [ ] map/new/adopt goldens byte-identical (re-run capture.sh → chỉ `sync.golden` re-froze + `sync.tree.golden` mới; new/adopt/map unchanged)
- [ ] `bash scripts/trust-gate.sh` exit 0 (bootstrap/sos-rs ngoài baseline — no rebaseline)

### Docs Gate
- [ ] `bootstrap/sos-rs/README.md` — command parity status table: `sync` → ✅ parity (enforced)
- [ ] `crates/sos-cli/tests/README.md` — mở rộng: HEAD-pin note (giờ dùng synthetic fake-kit, không phụ thuộc real HEAD) + 3-case fixture repro + `sync.tree.golden` two-fixture entry
- [ ] `docs/plans/P077c-decomposition.md` — mark c2 done + cập nhật feed-forward (c3/c4: sync đã dùng synthetic-kit determinism pattern nếu chúng cũng cần history)
- [ ] `CHANGELOG.md` — P077c2 entry

### Discovery Report
- [ ] Write `docs/discoveries/P077c2.md`
  - Anchors #1-10 CORRECT/WRONG (file:line)
  - Re-froze `sync.golden` scenario change (0-change → synthetic-3-case) — ghi rõ vì sao (không chỉ additive)
  - Escape-hatch: synthetic-kit khả thi hay phải fallback? (anchor #9)
  - Rust git-read cách nào (git2 vs shell) — anchor #10
  - Bug-for-bug deviations (find-order xử lý ra sao)
  - Docs updated list / Tier escalations (None nếu không)
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`
