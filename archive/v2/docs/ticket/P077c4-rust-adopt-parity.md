# P077c4 — Rust `sos adopt` → parity (brownfield retrofit, non-clobber, bug-for-bug Bash)

**Loại:** Impl + parity-harness (Rust port)
**Ưu tiên:** P1 (sprint P077c, sub-phiếu 4/5 — sau c1 map + c2 sync + c3 new SHIPPED)
**Ảnh hưởng:** `bootstrap/sos-rs/` (Rust `sos adopt` command + parity harness). `bin/sos.sh` KHÔNG đổi.
**Dependency:** P077c1 (per-command `PARITY_ENFORCED` set + two-fixture pattern + `run_rust_with_kit` env-override + **`map.rs` — adopt GỌI lại map logic**), P077c2 (synthetic self-contained fake-kit + `git init`-in-fixture + find-order finding), P077c3 (3-layer fixture split: stdout + tree-shape + gen-content; `DOCTOR_BIN=/nonexistent/doctor` doctor-absent lever; `strip_timestamp` before bare-date; `LC_ALL=C sort`). Cả 3 đã SHIPPED (branch `P077c*`, chưa merge).
**Tầng:** 1 (móng — impl + parity oracle của command NẶNG NHẤT; sai thì LAN xuống c5 vốn re-froze/mở rộng chính golden của adopt).
**Lane:** Guarded
**Lane token (trần):** `LANE:GUARDED cmd=adopt tier=1 oracle=parity-hardfail`

---

## Context

### Vấn đề
`sos adopt` (Bash `bin/sos.sh:606-971`) là **command PHỨC TẠP NHẤT** trong bộ — brownfield retrofit với kỷ luật NGƯỢC với `sos new`: **RESPECT** những gì đã có trong repo đích. Ba hành vi đặc thù mà 3 command trước KHÔNG có:
- **NON-CLOBBER (điểm khác `new`):** file đích đã tồn tại → **KHÔNG bao giờ overwrite**; thay vào đó stage bản của kit vào `.sos-adopt-incoming/<path>` + report merge tay. File đích ABSENT → copy (ADDED). Doc đích tồn tại → generate SKIP.
- **adopt GỌI `sos_map`** (`:814`) để sinh `docs/AGENT_MAP.yaml` khi absent — map SAU KHI [1/4] đã copy kit assets vào target → **map chính kit assets = OA-02 bug**. c4 GIỮ bug-for-bug, KHÔNG fix (P077c5).
- **Born-wire [3/4]** (arm hooks + `sos init security`) + **validator [4/4]** thêm `validate-map` (ngoài `verify-setup` như `new`).

`adopt.golden` hiện tại (P077a, ~4.7k — fixture LỚN NHẤT) chỉ freeze **stdout report** → mù với: (a) file nào thật sự được copy, (b) file nào thật sự được stage vào `.sos-adopt-incoming/`, (c) **file đích cũ có bị đụng KHÔNG** (non-clobber invariant). Đúng false-green class OA-02 mà c1/c2/c3 đã vá. `adopt` hiện informational; c4 flip vào `PARITY_ENFORCED` với oracle **4 lớp** thấy work-product THẬT + chứng minh preservation.

### Thách thức thiết kế đặc thù adopt (giải trong phiếu)

**(1) NON-CLOBBER — cần fixture BROWNFIELD có file PRE-EXISTING.** `new` chạy trên target rỗng; `adopt` VÔ NGHĨA nếu target rỗng (`:629-631` reject empty → "use sos new"). Fixture brownfield phải cố tình dựng đủ **3 collision-case**:
| Case | Seed trong brownfield target | adopt xử | Phải verify |
|---|---|---|---|
| (a) spine ABSENT | (không seed path đó) | copy → ADDED list | tree có path mới |
| (b) spine COLLISION | seed file **cùng path với 1 kit spine item**, nội dung KHÁC | stage → `.sos-adopt-incoming/<path>`, **target KEPT nguyên** | tree có `.sos-adopt-incoming/<path>` + **preservation: file cũ byte-unchanged** + staged copy == kit source |
| (c) doc EXISTING | seed 1 Cat-C doc (vd `docs/ARCHITECTURE.md` hoặc `CHANGELOG.md`) | generate SKIP → conflicts "exists — kept" | preservation: doc cũ byte-unchanged; KHÔNG có skeleton đè |
| (d) source non-spine | seed `src/routes/api.py` + `src/models/user.py` (map surfaces, reuse c1 shape) | không đụng | preservation: unchanged; **feeds AGENT_MAP real surfaces** |

**(2) Multi-layer fixture (kế thừa c3 + 1 lớp preservation MỚI):**
| Fixture | Freeze gì | Chống được gì |
|---|---|---|
| `adopt.golden` (**re-froze**) | stdout report `[1/4]..[4/4]` + `═══ report ═══` ADDED/REVIEW lists, normalized | report drift |
| `adopt.tree.golden` (**NEW**) | path-shape manifest MỌI file/dir dưới target **BAO GỒM `.sos-adopt-incoming/**`**, sorted, EXCLUDE `.git/`, KHÔNG content | Rust copy ĐÚNG BỘ file + **stage đúng bộ collision** như Bash |
| `adopt.gen.golden` (**NEW**) | content-hash `<relpath> <sha256>` (sorted) CHỈ cho file **GENERATED-authored** (adopt tự viết khi absent), normalized trước hash | nội dung authored (gồm **AGENT_MAP.yaml scan = OA-02 bug content**) sai |
| **preservation-assert** (**NEW lớp 4, đặc thù adopt**) | in-test INVARIANT (KHÔNG golden freeze — universal property cả Bash lẫn Rust phải thoả): (i) mọi seeded pre-existing file → sha256 **BẤT BIẾN** trước/sau adopt; (ii) mỗi `.sos-adopt-incoming/<path>` staged copy **byte-match** kit source file | **non-clobber bị vi phạm** (adopt overwrite file cũ) HOẶC stage sai nội dung |

**(3) Preservation-assert cách làm (vì sao INVARIANT, KHÔNG golden):** "file cũ không đổi" là **thuộc tính phổ quát** (cả Bash lẫn Rust phải thoả), không phải giá trị-cần-đóng-băng. Freeze nó vào golden = thừa + couple với nội dung seed. Thay vào đó: test seed nội dung đã biết → hash TRƯỚC adopt → chạy adopt → hash SAU → assert bằng. Mạnh hơn golden (bắt cả trường hợp adopt ghi đè bằng nội dung giống hệt tình cờ thì… vẫn bằng — chấp nhận; điểm cốt lõi là bắt CLOBBER). Staged-copy check: `.sos-adopt-incoming/<path>` phải byte-match `$K/<path>` (kit đã copy y nguyên vào incoming). Cả 2 nhánh (i)+(ii) chạy identical cho Bash-run và Rust-run trong `parity_adopt_enforced`.

**(4) Fake-kit + `LC_ALL=C` (kế thừa c2/c3):** synthetic fake-kit fileset CỐ ĐỊNH (đủ để `sos_adopt` chạy không lỗi) → tránh kit-content-coupling. `LC_ALL=C sort` mọi sort site (c3 finding). `git init`-in-fixture cho born-wire [3/4] (c2 pattern).

### Non-clobber determinism (rủi ro cốt lõi — như c2 find-order)
`adopt_item` copy directory theo **per-file `find -L`** (`:661,:679`) + skills loop `find` (`:708,:717`) → thứ tự `added`/`conflicts` string trong stdout report = **find-enumeration order** (KHÔNG sort — Bash không sort các list này). Đây ĐÚNG rủi ro c2 đã gặp với sync: c2 thấy Rust `WalkDir` khớp Bash `find` trên macOS/APFS cho frozen fixture, tree golden (sorted) miễn nhiễm. c4 kế thừa: `adopt.tree.golden` + preservation (sorted/set-based) miễn nhiễm; **chỉ `adopt.golden` stdout list-order phụ thuộc find-order** → residual cross-platform risk (document, không block; escape-hatch B nếu vỡ same-platform).

### Giải pháp — Scope c4 (additive, parity bug-for-bug — GIỮ OA-02 bug)
1. Impl Rust `sos adopt`: `crates/sos-cli/src/commands/adopt.rs` + clap `Adopt { dir, stack }` + dispatch. Port bug-for-bug `sos_adopt` (`:606-971`): arg/guards, dirty-warn, `adopt_item` non-clobber (copy-if-absent / stage-existing→`.sos-adopt-incoming/`), skills remap loop, `.mcp.json`/`settings.local.json` create-if-absent (jq-merge branch → xem Constraint 7), [2/4] generate-if-missing, **[2b] gọi map logic tái dùng `map.rs` từ c1** (bug-for-bug OA-02), `.gitignore` append, `.phieu-counter`, born-wire (install-hooks + `sos_init_security`), validator (`verify-setup` + `validate-map`, honor `DOCTOR_BIN`).
2. Harness: `PARITY_ENFORCED` từ `&["map","sync","new"]` → `&["map","sync","new","adopt"]` (giờ **4 command**). Test `parity_adopt_enforced` **4-assert**: stdout (`adopt.golden`) + tree (`adopt.tree.golden`) + gen (`adopt.gen.golden`) + **preservation invariant**. **Hard-fail bất kỳ assert nào.**
3. `capture.sh` mở rộng additive: build synthetic fake-kit + brownfield target (3 collision-case) + freeze 3 golden.

### KHÔNG trong scope
- Sửa `bin/sos.sh` (Bash canonical, bất di).
- **OA-02 fix (P077c5).** map-within-adopt GIỮ map kit assets. `sos init security` trong [3/4] GIỮ bug-for-bug. **Feed-forward: c5 sẽ fix OA-02 trên CẢ map+adopt → re-froze/mở rộng `adopt.gen.golden` (AGENT_MAP content đổi) + `map.agent_map.golden` sang correctness set.**
- jq-merge branch của `.mcp.json`/`settings.local.json` (fixture đi nhánh create-if-absent — xem Constraint 7).
- verify-setup/validate-map **CONNECTED** path (cần doctor pinned) — fixture ép doctor-absent → chỉ parity dòng skip.
- `init security` standalone parity (đã port trong c3's `new` [3/4]; adopt gọi cùng `sos_init_security` → không tách riêng).

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Marker / Result |
|---|---|---|---|
| 1 | `sos_adopt` toàn hàm `bin/sos.sh:606-971`; arg/guards :615-635, dirty-warn :642-646, `adopt_item` :655-690, [1/4] spine :692-804, [2/4] gen-if-missing :806-883, .gitignore :888-892, .phieu-counter :897-900, [3/4] born-wire :906-930, [4/4] validator :932-955, report :957-970 | Read range | `[verified]` — Architect đọc trực tiếp |
| 2 | `adopt_item` non-clobber core: DIR → per-file `find -L` (:661,:679), file ABSENT → `cp` + `added` (:673-674 / :685-687), file EXISTING → `cp` vào `.sos-adopt-incoming/$rel` + `conflicts` (:669-671 / :682-684), **KHÔNG đụng target file cũ**. Symlink-escape guard :667 (realpath phải trong kit tree) | Read :655-690 | `[verified]` — logic non-clobber + staging path |
| 3 | Danh mục GENERATED-authored (vào `adopt.gen.golden`) vs COPIED/staged (tree-only). Xem bảng "Copied vs Generated" dưới. Mọi heredoc/`cat >`/`printf >` trong :692-900 phải phân loại đúng | `grep -n 'cp \|cat >\|printf.*>' bin/sos.sh` trong 606-971, đối chiếu bảng | `[needs Worker verify]` — Worker liệt kê đủ, xác nhận không sót file authored |
| 4 | adopt GỌI `sos_map "$target" >/dev/null` (:814) CHỈ khi `docs/AGENT_MAP.yaml` absent → sinh AGENT_MAP scan REAL surfaces của target (**giờ chứa copied kit assets → OA-02: map kit assets**). Rust tái dùng `map.rs` logic (c1). Output map = **deterministic** cho fixed fixture (scan_files `sort`+`LC_ALL=C`+`head -25`, :296) | Read :813-816 + map.rs; chạy adopt trên fixture, kiểm AGENT_MAP content ổn định 2 lần | `[needs Worker verify]` — **ESCAPE HATCH: map-within-adopt nondeterminism khác map standalone?** (tree lớn hơn do kit assets; head-25 cap có bị find-order thay vì sort không). Nếu nondeterministic → DỪNG escalate |
| 5 | [4/4] validator: `doctor_bin="${DOCTOR_BIN:-doctor}"` (:933); `verify-setup` (:936) VÀ `validate-map` (:948-952) **CẢ HAI nằm trong** block `if command -v "$doctor_bin"... else ⏭ doctor not found` (:934/:954). Doctor-absent → **CHỈ in dòng `⏭ doctor not found` (:954)**, KHÔNG có sub-line validate-map riêng → deterministic (cùng lever c3) | Read :932-955; set `DOCTOR_BIN=/nonexistent/doctor`, chạy 1 lần, quan sát chỉ 1 dòng skip | `[needs Worker verify]` — **ESCAPE HATCH: validate-map absent-path** — confirm validate-map KHÔNG in dòng khi doctor absent; nếu có nhánh in riêng khác → DỪNG escalate |
| 6 | `adopt.golden` hiện có = **CONNECTED-path artifact** (captured với doctor present: `[WIRED] J1..J6 CONNECTED` + `validate-map: paths resolve`) → **BẮT BUỘC re-froze** dưới fake-kit + `DOCTOR_BIN=/nonexistent/doctor` (như c3 re-froze `new.golden`). KHÔNG additive | Architect KHÔNG đọc được `adopt.golden` (architect-guard chặn tests/) — Worker `cat` + confirm block `[WIRED]`/`validate-map` present → re-froze | `[needs Worker verify]` — **Architect chưa xác nhận nội dung golden (envelope); tin recon fact + c3 precedent** |
| 7 | Non-clobber staging order trong stdout report = find-enumeration order (`added`/`conflicts` tích theo :661/:679/:708 find, KHÔNG sort). Rust `WalkDir`/`read_dir` phải khớp Bash `find` order same-platform (c2 finding: khớp trên macOS/APFS cho frozen fixture) | Chạy adopt 2 lần cùng fixture, diff stdout list-order; so Rust vs Bash order | `[needs Worker verify]` — **ESCAPE HATCH: non-clobber staging non-deterministic?** Nếu list-order flaky same-platform → DỪNG escalate (có thể phải sort list trước freeze + Rust sort → nhưng đó là deviation, cần Architect chốt) |
| 8 | Brownfield fake-kit target fileset (3 collision-case + source + stack manifest + `git init`): seed (b) 1 file cùng path kit spine item nội dung khác (vd `templates/INVARIANTS-template.md` custom), (c) 1 Cat-C doc (vd `CHANGELOG.md` HOẶC `docs/ARCHITECTURE.md`), (d) `src/routes/api.py`+`src/models/user.py`, 1 manifest (`pyproject.toml`) cho `sos_init_security` detect; `git init`+**commit seeds** (clean tree → KHÔNG trigger dirty-warn :642) | Enumerate mọi `"$K/..."` read của `sos_adopt` (:632-955) → fake-kit khớp; dựng brownfield seed | `[needs Worker verify]` — **ESCAPE HATCH A: fake-kit brownfield bất khả thi/brittle** → DỪNG escalate, Architect chốt fallback (real `$SOS_KIT_DIR` chấp nhận kit-path-coupling ở tree, HOẶC subset+normalize) |
| 9 | dirty-warn (:642-646) chỉ in khi `.git` + `git status --porcelain` non-empty. Commit-seeds-first → clean → **KHÔNG warn** (stdout đơn giản, deterministic). born-wire install-hooks đổi `.git/config` (hooksPath) — KHÔNG tracked → tree vẫn "clean", `.git/` EXCLUDE khỏi tree golden | Fixture commit seeds, chạy adopt, confirm không có dòng "⚠ target has uncommitted changes" | `[needs Worker verify]` — nếu Worker muốn TEST nhánh dirty thay vì clean, đó là design choice → note trong Discovery (Tầng 2 self-decide, nhưng recommend clean cho determinism) |
| 10 | Normalize: `.sos-stack.toml` `detected_at="$ts"` (:132/:144 full ISO-8601) + `.mcp.json` header không date + CHANGELOG skeleton date (:853) + AGENT_MAP `generated_by`/paths (no date) → `strip_timestamp` (ISO-8601→`<TIMESTAMP>`) TRƯỚC bare-date rule (c3 order-bug), target-abs→`<TARGET>`, kit→`<SOS_KIT_DIR>`, `LC_ALL=C sort`. Áp trước hash vào gen + trước freeze stdout | Đối chiếu normalize rules c3 (`strip_timestamp`/`strip_bare_date`) — tái dùng | `[needs Worker verify]` — Architect KHÔNG đọc `capture.sh`/`parity.rs` (envelope); tin c3 discovery citation |
| 11 | Harness cơ chế c1/c2/c3 (`run_rust_with_kit` set `SOS_KIT_DIR`+`DOCTOR_BIN`, `TempFixture`, `normalize()`, `PARITY_ENFORCED` set, `parity_skeleton_informational` auto-skip cmd trong set) tái dùng cho `adopt` | Read `tests/{capture.sh,parity.rs,README.md}` | `[needs Worker verify]` — tin citation c1/c2/c3 discoveries |

### ⚠️ ESCAPE HATCH (Worker DỪNG + escalate qua RELAY, KHÔNG tự quyết design-tradeoff Tầng 1):
- **(A) Fake-kit brownfield bất khả thi/brittle** (anchor #8) — minimal fake-kit + 3-collision brownfield thoả MỌI `$K` read + đủ collision quá nặng/dễ vỡ → DỪNG. Fallback để Architect chốt: real `$SOS_KIT_DIR` (kit-path-coupling ở tree, gen vẫn hash-only-authored), HOẶC subset+normalize path-set.
- **(B) Non-clobber staging / stdout list-order non-deterministic same-platform** (anchor #7) — nếu `added`/`conflicts` order flaky cùng máy → DỪNG (giải pháp = sort-before-freeze + Rust sort = deviation cần Architect chốt, HOẶC normalize list order).
- **(C) map-within-adopt nondeterminism khác map standalone** (anchor #4) — nếu AGENT_MAP content flaky do tree lớn hơn / head-25 cap find-order → DỪNG.
- **(D) validate-map absent-path in dòng riêng** (anchor #5) — nếu doctor-absent KHÔNG ổn định hoá được (validate-map có nhánh in độc lập verify-setup) → DỪNG.

---

## Debate Log

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker CHALLENGE

**Anchor verification (11/11, all ✅):** #1 line ranges exact vs `bin/sos.sh:606-971` (verified via `sed`/`grep` against real file). #2 `adopt_item` core (:655-690) exact — dir-branch find-L + symlink-escape guard (:667) + existing→stage (:669-671) + absent→copy (:673-674); file-branch (:681-688) identical shape. #3 Copied/Generated split confirmed against every `cp`/`cat >`/`printf >` in range, no unclassified file. #4 live-verified NOT nondeterministic — `map.rs`'s `hits.sort()` (c1) already sorts before the 25-cap, mirrors Bash; ran adopt 2× on a live fake-kit+brownfield build, AGENT_MAP.yaml byte-identical both runs (including OA-02 kit-asset pollution, reproduced exactly). #5 live-confirmed doctor-absent → single `⏭ doctor not found` line, no separate validate-map branch. #6 confirmed `adopt.golden` is a CONNECTED-path artifact (`[WIRED] J1..J6`, `validate-map: paths resolve`) — re-froze required. #7 live-confirmed same-platform: `added`/`conflicts` list-order byte-identical across 2 independent runs. #8 **HATCH A resolved feasible** — built fake-kit + brownfield live, found 2 real fixture gotchas (needs `templates/claude-settings.local.json` + a REAL executable `scripts/install-hooks.sh`, not a stub) not flagged in the phiếu text, otherwise clean exit 2×. #9 live-confirmed: commit-seeds-first → clean tree → no dirty-warn line.

**HATCH A/B/C/D:** A resolved feasible (2 fixture gotchas noted above, folded into EXECUTE). B resolved — list-order, tree, and AGENT_MAP.yaml all byte-identical across 2 live runs same-platform. C resolved — map-within-adopt deterministic (sort-before-cap in `map.rs`, no coupling to adopt's larger post-[1/4] tree beyond content, which is the intended OA-02 bug not nondeterminism). D resolved — single skip line confirmed live, no independent validate-map branch. **None escalate.**

**Preservation-assert:** live-verified coherent on the real Bash oracle — collision file (`templates/INVARIANTS-template.md`) target-side byte-unchanged after adopt; staged `.sos-adopt-incoming/templates/INVARIANTS-template.md` byte-matches kit source; `CHANGELOG.md` and `src/*.py` untouched.

**adopt.golden re-froze:** confirmed required and done (CONNECTED→doctor-absent), same class as c3's `new.golden` fix.

**Objections:** none — accept V1, no revision.

**Status:** ✅ ACCEPTED — ready for EXECUTE.

---

## Nhiệm vụ

### 1. Impl Rust `sos adopt` — `crates/sos-cli/src/commands/adopt.rs` (NEW file)
- **Tạo:** `adopt.rs` port bug-for-bug `sos_adopt` (`bin/sos.sh:606-971`).
- **Lưu ý:**
  - Arg parse: `<dir>` positional + `--stack python|rust|ts` (optional cho adopt — :615-621 chỉ set var, không validate strict như new) + reject unknown flag (:619). Guards: target tồn tại (:626-628), non-empty (:629-631 — reject empty → "use sos new"), `$K/.claude/agents` (:633-635).
  - dirty-warn (:642-646): in warn khi `.git` + porcelain non-empty. **Bug-for-bug** (fixture commit-clean → không fire).
  - **`adopt_item` non-clobber (core):** DIR → duyệt per-file (`find -L`, deref symlink, skip `__pycache__`/`.pyc`/`.DS_Store` :679); symlink-escape guard (realpath trong kit tree :667). File ABSENT → `cp` + append `added` (`+ <rel>`). File EXISTING → `cp` vào `.sos-adopt-incoming/<rel>` + append `conflicts` (`~ <rel>`), **KHÔNG chạm target file cũ**. Giữ nguyên thứ tự visit (find-order, KHÔNG sort — bug-for-bug, xem escape-hatch B).
  - [1/4] spine (:692-804): `adopt_item` cho `.claude/agents`, `.claude/commands`, `.claude/settings.json`, `agents/orchestrator.md`→`.claude/agents/orchestrator.md`, `scripts`, `phieu`, `templates`, `hooks/pre-commit`, `hooks/pre-push`, `docs/ORCHESTRATION.md`. Skills remap loop (:706-718): `find $K/skills -not attic` → `.claude/skills/<rel>` create-if-absent / stage-if-collision. `.mcp.json` (:720-780) + `.claude/settings.local.json` (:782-803): **create-if-absent nhánh** (heredoc/cp template) — fixture đi nhánh này (Constraint 7).
  - [2/4] generate-if-missing (:806-883): INVARIANTS (cp template + append :808-812), **AGENT_MAP via `sos_map` logic tái dùng map.rs từ c1 nếu absent (:813-816) — bug-for-bug OA-02**, `.docs-gate.toml` (:817-842), CHANGELOG skeleton (:847-855), ARCHITECTURE skeleton (:856-876), BACKLOG (cp template :877-881), CLAUDE.md **report-only** (:882-883, KHÔNG generate). Mọi cái GIỮ non-clobber: file tồn tại → conflicts "exists — kept".
  - `.gitignore` append-if-missing per-line (:888-892, 10 patterns); `.phieu-counter` seed 0 nếu absent (:897-900).
  - [3/4] born-wire (:906-930): `bash scripts/install-hooks.sh` nếu `.git`+script tồn tại (:907-920, honor rc); `sos_init_security` in-target (:921-930, replicate heredoc bug-for-bug — KHÔNG OA-02).
  - [4/4] validator (:932-955): `doctor_bin=DOCTOR_BIN|doctor`; nhánh present → verify-setup (:936) + validate-map (:948-952); nhánh absent → CHỈ in `⏭ doctor not found` (:954). **Fixture ép nhánh absent.**
  - report (:957-970): `═══ sos adopt report ═══` + ADDED list (`%b` :959) + REVIEW list (:960) + Next/Heads-up (:962-970).
  - Git shell-out: `std::process::Command::new("git")` (như c2), KHÔNG `git2`.

### 2. clap enum + dispatch — `crates/sos-cli/src/main.rs`
- **Tìm:** clap enum (hiện `Init/Blueprint/Contract/Apply/Recipe/Launch/Status/Map/Sync/New` per c1/c2/c3). `[needs Worker verify]` exact variant list.
- **Thêm:** `Adopt { dir: String, stack: Option<String> }` + match arm → `commands::adopt::run(...)`. Đăng ký `adopt` trong `commands/mod.rs`.

### 3. Harness — `crates/sos-cli/tests/parity.rs`
- **Tìm:** `const PARITY_ENFORCED: &[&str] = &["map", "sync", "new"];`
- **Thay bằng:** `&["map", "sync", "new", "adopt"];`
- **Thêm:** `#[test] fn parity_adopt_enforced()` — dựng fake-kit + **brownfield target** (3 collision-case, via `TempFixture`), `git init`+commit seeds, chạy Rust `sos adopt <tgt>` với `SOS_KIT_DIR=<fake-kit>` + `DOCTOR_BIN=/nonexistent/doctor` (`run_rust_with_kit`), assert **4 thứ hard-fail**:
  1. stdout == `adopt.golden` (normalized)
  2. tree-manifest (incl `.sos-adopt-incoming/**`, excl `.git/`, sorted `LC_ALL=C`) == `adopt.tree.golden`
  3. gen-hash-manifest (GENERATED-authored only, normalized→sha256, sorted) == `adopt.gen.golden`
  4. **preservation invariant** (KHÔNG golden): (i) mỗi seeded pre-existing file → sha256 trước == sau adopt; (ii) mỗi `.sos-adopt-incoming/<path>` == kit source `<path>` byte-match.
- `[needs Worker verify]` exact helper names từ c1/c2/c3.
- **Lưu ý:** `parity_skeleton_informational` tự skip cmd trong set → `adopt` rời informational; sau c4 KHÔNG còn command informational nào (4/4 enforced).

### 4. `capture.sh` — build fake-kit + brownfield + freeze 3 golden (additive)
- **Thêm** nhánh cho `adopt`: (a) construct synthetic fake-kit (anchor #8 fileset), (b) construct brownfield target (3 collision-case + source + manifest, `git init`+commit), (c) chạy Bash `sos_adopt` với `SOS_KIT_DIR=<fake-kit> DOCTOR_BIN=/nonexistent/doctor`, (d) freeze:
  - `adopt.golden` = stdout normalized (**re-froze** — justify như c2/c3).
  - `adopt.tree.golden` = `find <tgt> -not -path '*/.git/*' | sed 's|^<tgt>/||' | LC_ALL=C sort` (incl `.sos-adopt-incoming/`).
  - `adopt.gen.golden` = với mỗi GENERATED-authored relpath: normalize (`strip_timestamp`→bare-date→target/kit) → `sha256` → `<relpath> <sha256>`, `LC_ALL=C sort`.
- **Lưu ý:** normalize tái dùng rule c3 (`strip_timestamp` trước bare-date). `[needs Worker verify]` exact site.

### 5. Nghiệm thu ghi Discovery + docs (xem Nghiệm thu).

---

## Files cần sửa
- `bootstrap/sos-rs/crates/sos-cli/src/commands/adopt.rs` (NEW)
- `bootstrap/sos-rs/crates/sos-cli/src/commands/mod.rs` (đăng ký `adopt`)
- `bootstrap/sos-rs/crates/sos-cli/src/main.rs` (clap `Adopt` + dispatch)
- `bootstrap/sos-rs/crates/sos-cli/tests/parity.rs` (`PARITY_ENFORCED` += `adopt`, new test 4-assert)
- `bootstrap/sos-rs/crates/sos-cli/tests/golden/capture.sh` (fake-kit + brownfield + 3 freeze)
- `bootstrap/sos-rs/crates/sos-cli/tests/golden/adopt.golden` (**re-froze**)
- `bootstrap/sos-rs/crates/sos-cli/tests/golden/adopt.tree.golden` (NEW)
- `bootstrap/sos-rs/crates/sos-cli/tests/golden/adopt.gen.golden` (NEW)
- Docs (DOCS-GATE): `bootstrap/sos-rs/README.md`, `bootstrap/sos-rs/crates/sos-cli/tests/README.md`, `CHANGELOG.md`, `docs/plans/P077c-decomposition.md` (mark c4 SHIPPED + feed-forward c5)

## Files KHÔNG sửa
- `bin/sos.sh` — **canonical, bất di.** `git diff bin/sos.sh` phải rỗng.
- `map.rs` (tái dùng, KHÔNG sửa — adopt gọi logic), `sync.rs`, `new.rs`, và golden `map.*`/`sync.*`/`new.*` — không regress.

---

## Luật chơi (Constraints)
1. **Bash canonical, additive.** `git diff bin/sos.sh` rỗng. Rust chứng minh BẰNG (bug-for-bug), KHÔNG "sửa tốt hơn" (đó là c5).
2. **NON-CLOBBER là invariant cứng.** Preservation-assert BẮT BUỘC: pre-existing file byte-unchanged + staged `.sos-adopt-incoming/` == kit source. Vi phạm = FAIL, không thương lượng.
3. **KHÔNG hash content của copied/staged kit assets.** Chỉ path (tree). Content-hash CHỈ cho GENERATED-authored (gen). Staged incoming = copied kit → tree-only (byte-match kit qua preservation check, KHÔNG gen-hash).
4. **map-within-adopt GIỮ OA-02 bug-for-bug.** Tái dùng `map.rs` logic (c1). AGENT_MAP.yaml maps kit assets (do adopt đã copy vào target) — GIỮ NGUYÊN, KHÔNG exclude, KHÔNG stack-aware, KHÔNG 3-verdict. **c5 fix.**
5. **Hard-fail cả 4 assert** (stdout + tree + gen + preservation). Bất kỳ lệch = FAIL.
6. **verify-setup + validate-map ép doctor-absent** (`DOCTOR_BIN=/nonexistent/doctor`) → cả 2 đi nhánh skip (chỉ 1 dòng `⏭ doctor not found`). CONNECTED-path KHÔNG parity-test.
7. **`.mcp.json`/`settings.local.json` fixture đi nhánh CREATE-IF-ABSENT** (target không seed 2 file này) → authored heredoc/cp template → gen-hash sạch. **jq-merge branch OUT-of-fixture-scope** (jq-dependent + merge-output determinism risk) — note như c3 chỉ test python-stack branch. Nếu Worker muốn cover jq-merge → escalate (design, không tự thêm).
8. **`sos init security` [3/4] + map [2b] GIỮ bug-for-bug** — KHÔNG OA-02 (P077c5).
9. **`.git/` EXCLUDE** khỏi `adopt.tree.golden`. Fixture commit-seeds-first → clean tree → không dirty-warn (recommend; Worker self-decide dirty-path nhưng note).
10. **Escape hatch (A/B/C/D) = DỪNG + RELAY escalate**, KHÔNG tự quyết design-tradeoff Tầng 1.
11. **Determinism proof bắt buộc**: 2 lần `capture.sh` độc lập → 3 golden byte-identical (như c2/c3). `LC_ALL=C` mọi sort site.

---

## Nghiệm thu

### Automated
- `cd bootstrap/sos-rs && cargo build --workspace` → clean.
- `cargo test -p sos-cli --test parity` → `parity_map_enforced`, `parity_sync_enforced`, `parity_new_enforced`, `parity_adopt_enforced` ĐỀU pass; `parity_skeleton_informational` pass (giờ skip cả 4 → không còn cmd informational).
- `cargo test --workspace` → all green (dep_direction giữ pass).

### Parity proof (Discovery bắt buộc)
- Chạy Bash `sos_adopt` VÀ Rust `sos adopt` trên **cùng fake-kit + cùng brownfield target**, capture stdout + tree + gen-hash + preservation, diff cả 4 → `STDOUT IDENTICAL` / `TREE IDENTICAL` / `GEN IDENTICAL` / `PRESERVATION OK` (pre-existing unchanged both runs + staged == kit source).
- Ghi Rust deviations (nếu có) — mọi deviation phải là **intentional bug-for-bug match** (vd find-order list, map-within-adopt OA-02 content, `--pilot`/jq-branch không exercise).

### Negative test (Discovery bắt buộc — chứng minh oracle 4 lớp không thừa)
- **Tree**: sabotage `adopt.rs` bỏ 1 spine `adopt_item` (vd skip `phieu`) → `parity_adopt_enforced` FAIL trên **tree** assert (stdout `[1/4] done` line không đổi). Revert (`mv`+`touch` fix stale-mtime per c1 gotcha).
- **Gen**: sabotage 1 authored heredoc (vd đổi token trong CHANGELOG skeleton HOẶC surface-name trong map-within-adopt) → FAIL trên **gen** assert (map-name sabotage sẽ hit AGENT_MAP.yaml hash). Revert.
- **Preservation (đặc thù adopt — BẮT BUỘC)**: sabotage `adopt.rs` để 1 collision file **overwrite** target thay vì stage (mô phỏng clobber) → FAIL trên **preservation (i)** assert (seeded file hash đổi). Revert. Chứng minh preservation layer bắt đúng non-clobber regression class mà tree/gen/stdout bỏ lọt.
- Xác nhận từng fail fire đúng assert kỳ vọng (proof 4 lớp không thừa).

### Regression
- `git diff bin/sos.sh` rỗng.
- Re-chạy `capture.sh` 2 lần → `adopt.golden`/`adopt.tree.golden`/`adopt.gen.golden` byte-identical giữa 2 lần (determinism); `map.*`/`sync.*`/`new.*` golden không đổi.
- `bash scripts/trust-gate.sh` → exit 0 (`bootstrap/sos-rs/**` ngoài `.sos-trust-baseline` — no rebaseline; confirm).

### Docs Gate (Tầng 1)
- `bootstrap/sos-rs/README.md` — parity-status row `adopt` → "Parity (hard-fail, stdout + tree-shape + gen-content + preservation)"; nếu có bảng "informational vs enforced" → cập nhật 4/4 enforced.
- `crates/sos-cli/tests/README.md` — thêm mô tả **4-layer fixture cho `adopt`** (non-clobber brownfield 3-collision-case; preservation invariant = universal property KHÔNG golden; `.sos-adopt-incoming/` staged trong tree; map-within-adopt OA-02 bug-for-bug note; jq-branch out-of-scope); update `PARITY_ENFORCED = &["map","sync","new","adopt"]` + Layout.
- `CHANGELOG.md` — entry P077c4.
- `docs/plans/P077c-decomposition.md` — status line c4 SHIPPED + **feed-forward c5: OA-02 fix sẽ re-froze/mở rộng `adopt.gen.golden` (AGENT_MAP content đổi) + `map.agent_map.golden` sang correctness set** (đã có trong bảng sub-phiếu, xác nhận consistency).
- Discovery ghi rõ: "Tầng 1 docs updated: <list>".

### Discovery Report
- Anchor #1-11 kết quả (verified/corrected).
- Escape-hatch A/B/C/D có kích hoạt không; nếu có → RELAY + Architect quyết đã ghi.
- Copied-vs-Generated-vs-Staged final list (Worker-confirmed) + brownfield fixture 3-collision-case thực tế đã dựng.
- Non-clobber find-order finding (Rust WalkDir vs Bash find match same-platform? cross-platform residual risk như c2).
- map-within-adopt: AGENT_MAP content thực tế (OA-02 surfaces mapped) + xác nhận deterministic.
- Tier escalation (nếu 2→1) — dự kiến None (đã Tầng 1).

---

## Copied vs Generated vs Staged (phân loại — quyết định file nào vào gen golden)

**COPIED verbatim** (tree-only, KHÔNG gen-hash — identical-by-construction): `.claude/agents` (deref), `.claude/commands`, `.claude/settings.json`, `agents/orchestrator.md`→`.claude/agents/orchestrator.md`, `.claude/skills/*` (living, skip attic), `scripts/`, `phieu/`, `templates/`, `hooks/pre-commit`, `hooks/pre-push`, `docs/ORCHESTRATION.md`, `docs/BACKLOG.md` (từ template), `.claude/settings.local.json` (cp template khi absent).

**GENERATED-authored** (adopt tự viết heredoc/scan khi ABSENT → hash content vào `adopt.gen.golden`, normalized): `.mcp.json` (create-if-absent heredoc :722-752), `docs/security/INVARIANTS.md` (cp template + authored append :809-810 → hash whole file như c3), **`docs/AGENT_MAP.yaml` (via `sos_map` scan :814 — OA-02 bug content, deterministic cho fixed fixture)**, `.docs-gate.toml` (:818-840 static), `CHANGELOG.md` skeleton (:853, có date→normalize), `docs/ARCHITECTURE.md` skeleton (:858-874, `$_name`), `.gitignore` (append 10 lines :888-892 — hash whole file sau append), `.sos-stack.toml` (`sos_init_security`, `detected_at` ts→normalize).

**STAGED** (`.sos-adopt-incoming/<path>` — collision copies, byte-match kit source): tree-only + **preservation-assert (ii)** byte-match, KHÔNG gen-hash.

⚠️ `INVARIANTS.md` = copied-template + authored-append → hash TOÀN FILE (như c3 đã giải). `.gitignore` seed (nếu target chưa có) hoặc append-to-existing — nếu target seed `.gitignore` sẵn thì append → **non-clobber append, KHÔNG stage** (khác spine collision) → hash whole file sau append; nếu absent → tạo mới. `[needs Worker verify]` — Worker xác nhận `.gitignore` seed hay không trong brownfield, phân loại đúng.
