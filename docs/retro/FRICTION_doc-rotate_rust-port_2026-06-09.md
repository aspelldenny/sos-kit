# WORKFLOW FRICTION LOG — Rust port pilot (→ hand off to sos-kit)

> **Mục đích:** Ghi mọi defect / drift / friction của workflow v2.2 gặp trong lúc chạy Rust port
> (RP01→hết dự án). Sếp vứt sang `~/sos-kit` xử lý vì đang áp workflow sos-kit.
> **Format mỗi mục:** `[ID] severity — triệu chứng → root cause → đề xuất fix`.
> Sev: 🔴 block/correctness · 🟡 friction/ergonomics · 🟢 worked-as-intended (ghi để giữ).

---

## 📬 HANDOFF COVER — triage cho sos-kit (delivered 2026-06-09)

> **Nguồn:** Rust port doc-rotate (RP01-RP07) + Phase 3 dogfood-feedback (DR01-DR06, adopt vào tarot P343). Khác batch với `WORKFLOW_V2.3_RETRO_doc-rotate.md` (đó là pilot Python P001-P007).
> **Cách đọc:** 13 finding dưới. Phân 2 nhóm theo CHỦ THỂ xử:

### A. sos-kit DOCTRINE/TEMPLATE phải sửa ở SOURCE (repo khác sẽ thừa kế nếu không)
| ID | Sev | sos-kit action |
|---|---|---|
| **F09** | 🔴 | ⛔ **`scripts/install-hooks.sh` sos-kit SHIP có bug hijack `core.hooksPath` + nuốt hook security của adopter.** doc-rotate đã fix bản local (DR01: thêm `install-cap-check.sh` additive + guard), NHƯNG **template gốc ở sos-kit chưa fix** → mọi repo spawn từ sos-kit thừa kế bug. Port fix DR01 về sos-kit. |
| **F01** | 🔴 | Orchestrator handbook (`agents/orchestrator.md`) + `docs/ORCHESTRATION.md` + `agents/*.md` reference trong CLAUDE.md/hook nhưng KHÔNG tồn tại trong repo spawn. Chốt 1 nguồn (commit file hoặc trỏ registry). |
| **F02** | 🔴 | Scaffold recipe thiếu "toolchain mới → gitignore build dir" sensor (`target/`/`node_modules/`/`dist`). + DOCS-GATE row. |
| **F04** | 🔴 | Doctrine doc claim implementation-behavior phải tag `[intent]` vs `[verified-impl]` (Architect docs-only ăn stale doc). Đồng thời = bằng chứng v2.2 §2 oracle-3-field HOLD (Worker route Architect đúng). |
| **F06** | 🟡 | Phiếu template "golden snapshot fields" phải spec unit (char vs byte) cho cross-language parity. |
| **F07** | 🟡 | Hard-rule "no fixture tự chế" disambiguate: cấm fixture FILE vs cho synthetic in-code instance (parity probe). |
| **F13** | 🟡 | Scaffold recipe: release-process sync `Cargo.toml version` với CHANGELOG (placeholder `0.0.0` trôi im 6 phiếu). |

### B. doc-rotate-LOCAL đã fix (ghi để sos-kit biết pattern, KHÔNG cần sos-kit action)
| ID | Sev | Trạng thái |
|---|---|---|
| **F08** | 🟡 | Hook "Phase 5"→[8/8] drift — fixed RP07b docs sweep. |
| **F10** | 🟡 | AGENT_MAP stale Python paths — ✅ CLOSED DR06. |
| **F11** | 🟡 | Classifier token-brittle → fixed DR02 (keep-marker). Product, doc-rotate-specific. |
| **F12** | 🟡 | Cross-ref hard-block → fixed DR03 (soft default). Product, doc-rotate-specific. |
| **F03** | 🟡 | `phieu/` tracking convention mơ hồ — workaround local, NHƯNG convention cần sos-kit chốt dứt (tracked vs local-only). |
| **F05** | 🟢 | `no-code-on-default` hook worked-as-intended — giữ. |

**Ưu tiên sos-kit:** F09 🔴 (template bug lan repo khác) > F01/F02/F04 🔴 > F06/F07/F13 🟡 > F03 (convention chốt).

---

## F01 🔴 — Orchestrator handbook + spec file KHÔNG TỒN TẠI (doc-vs-reality drift)

- **Triệu chứng:** Session-start hook + `CLAUDE.md` reference `agents/orchestrator.md (~85 lines)` và `docs/ORCHESTRATION.md (spec đầy đủ)`. Cả hai **không tồn tại** trong repo. `agents/` dir cũng không có (CLAUDE.md "Hard envelope rules" trỏ `agents/architect.md`, `agents/worker.md`, `agents/boundary-check.md`, `agents/advisory-watch.md` — đều vắng mặt).
- **Root cause:** Agent envelope thật được register như subagent type toàn cục (architect/worker/boundary-check/advisory-watch tồn tại như spawnable type), nhưng CLAUDE.md + hook vẫn trỏ đường dẫn file cục bộ cũ. Doc trỏ artifact đã di chuyển/không sync.
- **Đề xuất:** sos-kit chốt 1 nguồn — hoặc commit `agents/*.md` + `docs/ORCHESTRATION.md` vào repo, hoặc sửa CLAUDE.md/hook trỏ tới registry toàn cục. Hiện orchestrator chạy "mù handbook" (em phải tự suy state machine từ hook text).

## F02 🔴 — `target/` không gitignore khi scaffold toolchain mới (DOD gap RP01)

- **Triệu chứng:** RP01 "Cargo scaffold" tạo 167MB `target/` artifacts, KHÔNG có rule `.gitignore`. Nổi `?? target/` trong git status; lần `git add -A` nào sau sẽ nuốt trọn vào repo. Sếp bắt tay (em chưa tự thấy trong RP01).
- **Root cause:** Phiếu RP01 scope "Cargo.toml + clap skeleton + test" nhưng **DOD thiếu gitignore**. DOCS GATE trigger table có row `.mcp.json`/`Cargo.toml` nhưng KHÔNG có row "ngôn ngữ/toolchain mới → gitignore build dir". Scaffold-a-new-stack là sub-mech chưa có sensor.
- **Đề xuất:** sos-kit thêm vào scaffold recipe / DOCS-GATE: "new build toolchain (Cargo/node_modules/__pycache__/dist) → MANDATORY gitignore build-artifact dir, verify `git check-ignore`". Worker RP01 verify-cò nên có dòng "git status sạch artifact".

## F03 🟡 — `phieu/` tracking convention mơ hồ (tracked artifact hay local-only?)

- **Triệu chứng:** `git mv phieu/active/RP01.md phieu/done/` FAIL "not under version control" → phiếu RP01 chưa từng `git add`. Nhưng `phieu/done/` lại hiện `?? untracked` (KHÔNG bị ignore như `.sos-state/`). `.gitignore` không có rule `phieu/`. CLAUDE.md workflow nói "move active→done after merge" (hàm ý phiếu quan trọng) nhưng chúng không vào git.
- **Root cause:** Convention nửa vời — phiếu vừa "matter cho workflow" vừa "không persist trong git". Move-to-done thành thao tác local vô hình với history.
- **Đề xuất:** sos-kit quyết dứt: (a) phiếu là local working artifact → thêm `phieu/` vào `.gitignore` cho sạch, HOẶC (b) phiếu là audit trail → track + commit khi move done. Hiện trạng nhập nhằng dễ mất phiếu.

## F04 🔴 — Doctrine doc (`ARCHITECTURE.md`) mô tả INTENT ≠ implementation → Architect docs-only ăn phải giả định sai

- **Triệu chứng:** `docs/ARCHITECTURE.md` ghi offset "byte-exact". Thực tế `parse.py` lưu **char offset** (Python `str.index` = char-based). Architect (envelope docs-only, KHÔNG đọc code) propagate "byte-exact" vào phiếu RP02 V1. Nếu Worker không verify, Rust port sẽ byte-slice → panic/corrupt MỌI entry sau ký tự tiếng Việt non-ASCII đầu tiên.
- **Root cause:** Đây là **lõi pilot Q3** — "AGENT_MAP/docs có thay được tấm lưới rustc không?". Câu trả lời đang lộ: **docs-only Architect dễ tổn thương trước doc aspirational/stale**, và compiler Rust KHÔNG vớt char-vs-byte (cả hai compile sạch). Chỉ Worker-grep-data-thật + golden test (PARTIAL oracle) bắt được. Workflow ĐÚNG đã chặn được (Worker CHALLENGE O1.2 → Architect RESPOND Turn 2) — nhưng chỉ vì hard-rule "data thật + oracle 3-field" ép.
- **Đề xuất:** sos-kit (1) thêm guard "doctrine doc claim về implementation behavior phải tag `[intent]` vs `[verified-impl]`"; (2) ghi nhận đây là bằng chứng v2.2 §2 (oracle 3-field self-stop) HOLD trên PARTIAL oracle — Worker không self-close O1.2, route Architect đúng. **Pilot win, ghi vào retro.**

## F05 🟢 — `no-code-on-default` hook chặn commit trên main (worked-as-intended)

- **Triệu chứng:** Worker RP01 lỡ commit code trên `main`, pre-commit hook BLOCK, buộc sang branch `feat/RP01-cargo-scaffold`. Worker fix đúng, không bypass.
- **Đánh giá:** Enforcement CÓ RĂNG — đúng triết lý "bệnh gốc là enforcement không phải công cụ" (CLAUDE.md hard rule #4). Giữ. Ghi để counter-balance: không phải mọi friction là defect.

---

## F06 🟡 — Golden snapshot `body_len` semantic mơ hồ: char count (Python) ≠ byte count (Rust)

- **Triệu chứng:** RP02 snapshot generation dùng Python `len(e.body)` = char count (1635). Rust golden test compare `e.body.len()` = byte count (1772). Golden FAIL lần 1. Root: phiếu spec `body_len` không chỉ rõ unit (char hay byte).
- **Root cause:** Python `len(str)` = char; Rust `String::len()` = byte. Khi cross-language parity test, "body length" cần chỉ rõ unit. Phiếu §Golden "body_len" = ambiguous prose — Worker phải suy ra từ context.
- **Fix:** regenerate snapshot với `len(body.encode('utf-8'))` (byte count). sha256 (byte-exact) là assert mạnh hơn `body_len`; `body_len` giờ là sanity signal thứ yếu.
- **Đề xuất:** phiếu template "golden snapshot fields" nên có note: "body_len = byte count of UTF-8 body (for cross-language parity: Rust `String::len()` = bytes, Python `len(str)` = chars — explicitly spec byte unit)".

## F07 🟡 — Hard rule #1 wording mơ hồ: "fixture tự chế" = file `.md` hay synthetic test instance?

- **Triệu chứng:** RP03 phiếu Constraint #9 buộc synthetic DOCTRINE test (vì REAL fixture 71/71 OPERATIONAL → golden không chạy R1/R4 DOCTRINE branch = parity giả). Orchestrator NGHI va CLAUDE.md hard rule #1 ("KHÔNG fixture tự chế cho classify. Data THẬT only") → phải thêm 1 nhánh CHALLENGE riêng để reconcile, tốn 1 vòng soi.
- **Root cause:** Hard rule #1 dùng từ "fixture tự chế" không phân biệt: (a) **fixture FILE** `.md` hand-author (cái rule cấm — chống classify-correctness fixture bịa), vs (b) **synthetic Entry INSTANCE** trong test code cho port-parity probe (Python SINH golden, không phải người phán). Worker verify `test_classify.py:212,230,247` đã dùng (b) sẵn → rule #1 thực tế cho phép (b). Conflict là ảo, do wording.
- **Đề xuất:** sos-kit disambiguate hard-rule kiểu này: tách rõ "no synthetic fixture FILES" vs "synthetic in-code instances OK khi oracle SINH expected (parity probe), CẤM khi người hand-assert correctness". Rule prose hiện ép orchestrator tốn chu kỳ nghi ngờ cho cái đáng ra hiển nhiên.

## F08 🟡 — Hook phase-count drift: CLAUDE.md says "Phase 5" but hook is [8/8]

- **Triệu chứng (RP07 Architect + Worker phát hiện):** `CLAUDE.md` §Hook chain ghi `"Pre-commit Phase 5 — Cap enforcement (scripts/cap-check.py)"`. `docs/ARCHITECTURE.md` §Enforcement ghi `"5 phases"`. Thực tế `hooks/pre-commit` là 8-phase (`[1/8]..[8/8]`). Drift tồn tại từ khi hook upgrade P006 (RP05/RP06 period) nhưng CLAUDE.md/ARCHITECTURE chưa bao giờ cập nhật phase count. `tests/test_hook_cap.py:177` `test_hook_phase_5_renumber` assert `/5]` headers — test đã stale TRƯỚC RP07.
- **Root cause:** DOCS GATE trigger table (CLAUDE.md) có row "`hooks/pre-commit` SECTION add/remove → CLAUDE.md 'Hook chain' + `scripts/install-hooks.sh`". Phiếu P006 add [8/8] phase nhưng CLAUDE.md update missed the phase count. Stale doc drifted silently 3 phiếu (RP05→RP06→RP07a).
- **Impact:** Architect reads CLAUDE.md stack/hook section docs-only. "5 phases" stale → Architect có thể spec wrong phase number trong phiếu tương lai. Worker CHALLENGE bắt được (Anchor #7 re-verify), nhưng tốn 1 verify round.
- **Fix:** RP07b docs sweep sẽ fix CLAUDE.md/ARCHITECTURE.md "Phase 5"→"[8/8]" + "5 phases"→"8 phases". `test_hook_phase_5_renumber` + `test_hook_calls_cap_check` removed at RP07b retire.
- **Đề xuất sos-kit:** DOCS GATE "hook phase count → CLAUDE.md" trigger cần include phase NUMBER, không chỉ section add/remove. Hoặc add `grep -c "blue \"\[.*\]"` count check vào hook-integrity test.

## F09 🔴 — `install-hooks.sh` hijacks `core.hooksPath` and silences adopter repo's security hook + README stale

- **Triệu chứng:** `README.md:88` ghi "copies `hooks/pre-commit` → `.git/hooks/`" nhưng script thật chạy `git config core.hooksPath hooks` + rename `.git/hooks/{pre-commit,pre-push}` → `.pre-hookspath.bak`. Repo adopt (tarot) có `core.hooksPath` + hook P275 security gate riêng → chạy `install-hooks.sh` âm thầm redirect hook resolution sang doc-rotate's `hooks/` (giả định full sos-kit 8-check suite KHÔNG tồn tại ở adopter) → mất gác security của họ. Tarot workaround: KHÔNG chạy script, append tay — nhưng README vẫn mislead rằng `install-hooks.sh` là đúng path.
- **Root cause:** 1 installer phục vụ 2 audience khác nhau (doc-rotate dev có full suite vs adopter chỉ cần cap-check) nhưng chỉ có 1 path = hooksPath-hijack. README mô tả sai (copy) che mất hành vi thật. Không có additive path cho adopter.
- **Fix (DR01):** (a) README tách 2 audience + mô tả đúng cơ chế hooksPath; (b) thêm `scripts/install-cap-check.sh` additive (chỉ inject cap-check block, KHÔNG đụng hooksPath, idempotent sentinel); (c) `install-hooks.sh` guard detect hook/hooksPath sẵn có → warn+confirm (TTY) / abort-exit-1 (non-TTY) trước khi hijack. Happy-path doc-rotate clean repo: KHÔNG trigger guard.
- **sos-kit takeaway:** "installer ship hook" pattern cần tách "tool-dev full-suite install" vs "adopter additive inject" ngay từ đầu — 1 installer 2 audience = class lỗi nuốt-hook-security. `core.hooksPath` doctrine đúng cho tool-repo, sai khi export sang consumer-repo without guard.

## F10 🟡 ✅ CLOSED DR06 — AGENT_MAP has stale Python paths post-RP07b Rust port

- **Triệu chứng (DR05):** `docs/AGENT_MAP.yaml` surfaces (`classify`, `rotate_plan`, `cli_surface`) reference Python file paths (`src/doc_rotate/core/classify.py`, `src/doc_rotate/cli/`) and test paths (`tests/test_classify.py`, etc.). Python retired at RP07b 2026-06-09. Correct Rust paths: `src/{parse,classify,plan,entry}.rs` (core), `src/main.rs` + `src/apply.rs` (cli), `tests/golden_*.rs` (tests). DR05 adds `serve_mcp` with correct Rust paths, but old surfaces remain stale.
- **Root cause:** RP07b docs sweep updated `ARCHITECTURE.md` + `CLAUDE.md` + `README.md` but missed `AGENT_MAP.yaml`. DOCS GATE trigger table has no row for "retire language / rename module → update AGENT_MAP". AGENT_MAP is a separate doc category (validate-map target) not in the standard DOCS GATE table.
- **Impact:** Architect reads AGENT_MAP (docs-only envelope). Stale Python paths → if Architect uses AGENT_MAP `edit:` list to spec code changes, will reference non-existent Python files. Worker CHALLENGE would catch it (Anchor path verify), but adds 1 friction round per phiếu.
- **Fixed DR06 (2026-06-09):** All 3 stale surfaces updated to Rust paths. Added `config` + `cap_check` surfaces (were missing). Staleness comment removed. All 16 AGENT_MAP paths `ls`-verified. CHANGELOG [0.21.1] entry added.
- **Đề xuất sos-kit:** DOCS GATE trigger table add: "language port / module rename → AGENT_MAP `edit:` paths update required". Or add AGENT_MAP path-existence check to `validate-map` MCP tool.

## F11 🟡 — Classifier token-exact match forces adopter to rename content to match tool (F-2 dogfood)

- **Triệu chứng (DR02):** R1 doctrine detection matches token word-boundary `\bSub-mech\b`. Entry
  doctrine title "Knowledge durability doctrine / Sub-mechanism D" (tarot P287) misses (word-boundary
  fails: character after `Sub-mech` is `a`, not a boundary) → classifies Operational → entry was
  eligible for archive. Adopter had to RENAME entry title from "Sub-mechanism D" to "Sub-mech D"
  to match tool.
- **Root cause:** heuristic token-match forces content to be written using exact tool tokens.
  Tool bends content to tool (vision violation: tool should serve content, not vice versa).
  Token-loosening option (a) (widen R1 to match `Sub-mechanism` substring) = looser R1 →
  increases false-DOCTRINE (unrecoverable, the bệnh doc-rotate exists to cure).
- **Fix (DR02):** R0 explicit keep-marker `<!-- doc-rotate: keep -->` — author affirmatively pins
  entry as doctrine. DOES NOT loosen R1 token. Evolves beyond Python parity (Python had no R0).
  Unmarked entries continue with same R1-R5 classification logic — zero regression.
- **sos-kit takeaway:** heuristic-classifier tools should provide an explicit-override escape hatch
  (keep-marker) from the start. Token-only detection that forces content-bending is a class of
  adopter friction. Marker should be in the initial design, not added reactively.

## F12 🟡 — Hard-block cross-ref đánh nhau doctrine "soft-link broken OK after rotate" (F-3 dogfood)

- **Triệu chứng (DR03):** `apply` refuse (exit 2) khi entry doctrine-keep (tarot P311) bare-ref entry
  operational-archive (P287). Adopter phải convert bare ref `P281/P285/P286` → soft-link `[[Pxxx]]`
  mới qua được apply.
- **Root cause:** repo tiêu thụ có doctrine "Cross-ref DOCTRINE → DISCOVERIES = soft link, broken OK
  after rotate" — nhưng tool hard-block khi thấy cross-ref → đánh nhau với mô hình "broken OK". Tool
  bắt content bẻ theo tool (bare ref → `[[...]]`) — vision violation (giống F-2/DR02, giống F-003
  false-positive claude-hooks).
- **Fix (DR03):** cross-ref block = SOFT default (warn + proceed, blocked giữ trong log via
  `build_reduced_log` keep ∪ blocked đã sẵn — path dead pre-DR03, LIVE từ DR03) + `--strict-refs`
  opt-in cho ai cần gác chặt. Evolve beyond parity (Python refuse-on-blocked). Path blocked=0
  (REAL fixture) byte-exact unchanged. 3 new synthetic golden tests (soft/strict/conservation).
- **sos-kit takeaway:** hard-block trên heuristic-detected cross-ref ép adopter bẻ content = class
  friction. Default SOFT (warn) + opt-in strict là pattern an toàn hơn cho tool destructive: tôn trọng
  doctrine repo tiêu thụ, để user quyết gác chặt khi cần. Blocked-defensive code path (giữ blocked
  trong log) nên LIVE từ đầu, không để dead sau refuse.

## F13 🟡 — Cargo.toml `version = "0.0.0"` drift im lặng qua RP01→DR03 (scaffold placeholder)

- **Triệu chứng (DR04):** Adding `-V`/`--version` flag exposed that `Cargo.toml version = "0.0.0"` (scaffold placeholder from RP01). If only the flag were added without syncing `version`, `doc-rotate --version` would print `doc-rotate 0.0.0` — useless for the F-4 use-case ("adopter pin version lúc adopt"). CHANGELOG had reached `[0.20.0]` while Cargo.toml stayed `0.0.0` — ~20 minor versions of silent drift across RP02-DR03. Architect caught this as a defect-within-F-4 BEFORE it shipped.
- **Root cause:** RP01 scaffold set `version = "0.0.0"` as placeholder. No phiếu had a task to sync `Cargo.toml version` with CHANGELOG version bump. DOCS GATE trigger table has no row "CHANGELOG version bump → Cargo.toml `[package] version` sync". The gap is invisible until something READS `CARGO_PKG_VERSION`.
- **Fix (DR04):** Cargo.toml `version` set to `"0.21.0"` (DR04 minor bump) + CHANGELOG `[0.21.0]` entry — 3-way sync: Cargo.toml == CHANGELOG == `--version` output.
- **Đề xuất sos-kit:** (1) Scaffold recipe for Rust/Python projects: set `version` to a meaningful initial value (e.g., `"0.1.0"`), not `"0.0.0"` placeholder. (2) DOCS GATE: add trigger row "release version bump (CHANGELOG) → Cargo.toml `[package] version` sync required". Or add `grep '^version' Cargo.toml` vs `grep -m1 '^\#\# \[' CHANGELOG.md` version-match check to pre-commit.

## Pilot question scoreboard (FINAL — RP07b COMPLETE)

- **Q1 (oracle PARTIAL — worker tự dừng?):** ✅ HOLD. RP02 O1.2 char/byte — Worker KHÔNG self-close, route Architect (F04). v2.2 §2 confirmed.
- **Q2 (M1 nổ ở đâu?):** ✅ char-offset vs byte-offset trên fixture tiếng Việt UTF-8 (RP02/F04). Format-drift thật, không bịa.
- **Q3 (AGENT_MAP thay lưới rustc?):** ⚠️ Partly — docs-only Architect ăn stale doc aspirational (F04). Workflow chặn lại được nhờ Worker-grep + golden test (oracle pair). Lưới = Worker+oracle, KHÔNG chỉ docs.
- **Q4 (hook cap enforce thật?):** ✅ YES — P006 Python wired; RP07a Rust port + golden tests assert exit code + stdout; RP07b hook Rust-only (Python fallback removed). §4-Q4 CLOSED.
