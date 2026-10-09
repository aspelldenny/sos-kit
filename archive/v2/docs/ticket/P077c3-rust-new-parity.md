# P077c3 — Rust `sos new` → parity (greenfield bootstrap, bug-for-bug Bash)

**Loại:** Impl + parity-harness (Rust port)
**Ưu tiên:** P1 (sprint P077c, sub-phiếu 3/5 — sau c1 map + c2 sync SHIPPED)
**Ảnh hưởng:** `bootstrap/sos-rs/` (Rust `sos new` command + parity harness). `bin/sos.sh` KHÔNG đổi.
**Dependency:** P077c1 (per-command `PARITY_ENFORCED` set + two-fixture pattern + `run_rust_with_kit` env-override), P077c2 (synthetic self-contained fake-kit fixture pattern + determinism proof). Cả 2 đã SHIPPED.
**Tầng:** 1 (móng — impl + parity oracle; sai thì LAN xuống c4 adopt vốn tái dùng cùng harness).
**Lane:** Guarded
**Lane token (trần):** `LANE:GUARDED cmd=new tier=1 oracle=parity-hardfail`

---

## Context

### Vấn đề
`sos new` (Bash `bin/sos.sh:355-604`) là **command mutate-filesystem nặng nhất** trong bộ: nó COPY nguyên spine kit assets vào target + GENERATE ~9 skeleton file authored + defaults + verify-setup + git init. `new.golden` hiện tại (P077a, ~1.8k) chỉ freeze **stdout report** → mù với file-creation (đúng false-green class OA-02 mà c1 đã vá cho `map`, c2 cho `sync`). `new` hiện informational trong harness; c3 phải flip nó vào `PARITY_ENFORCED` với oracle **thật sự thấy work-product**.

### Thách thức thiết kế cốt lõi — chống false-green NHƯNG chống kit-content-coupling
`sos new` copy nguyên kit assets → nếu freeze full-content-hash cả cây thì: (a) fixture khổng lồ, (b) **drift mỗi khi BẤT KỲ file kit đổi** (kit-content-coupling — tệ hơn `sync`). Nếu chỉ freeze stdout thì mù với file creation. **Giải: tách 3 lớp fixture + chạy trên synthetic fake-kit** (kế thừa c2), KHÔNG bao giờ hash content của copied kit assets (chúng identical-by-construction cho cả Rust lẫn Bash vì cùng đọc `$SOS_KIT_DIR`).

### Giải pháp — 3-layer fixture, chạy trên SYNTHETIC fake-kit
| Fixture | Freeze gì | Chống được gì | KHÔNG couple với |
|---|---|---|---|
| `new.golden` (**re-froze**) | stdout report `[1/4]..[4/4]` + git block, normalized | report drift | (re-froze dưới fake-kit + doctor-absent) |
| `new.tree.golden` (**NEW**) | **path-shape manifest**: relpath mọi file/dir tạo dưới target, sorted, **KHÔNG content**, EXCLUDE `.git/` | Rust tạo ĐÚNG BỘ file như Bash | kit-content (chỉ path; và path-set cố định do fake-kit cố định) |
| `new.gen.golden` (**NEW**) | **content-hash manifest** `<relpath> <sha256>` (sorted) CHỈ cho file GENERATED-authored, normalized trước hash | nội dung authored (work-product thật của `new`) sai | copied kit assets (KHÔNG hash chúng) |

**Vì sao synthetic fake-kit (c2 pattern), KHÔNG real `$SOS_KIT_DIR`:** nếu `new.tree.golden` liệt path từ real kit → thêm/xoá BẤT KỲ file kit nào → tree drift → chính cái coupling task này cấm. Fake-kit = fileset CỐ ĐỊNH tối thiểu (đủ để `sos_new` chạy không lỗi) → tree + gen ổn định bất kể real kit lớn lên. Determinism đã được c2 chứng minh cho pattern này. Cơ chế env-override `run_rust_with_kit` (set `SOS_KIT_DIR`) c2 đã dựng — tái dùng.

**Copied vs Generated (phân loại — quyết định file nào vào `new.gen.golden`):**
- **COPIED verbatim** (KHÔNG hash content — identical by construction): `.claude/agents` (deref `-RL`), `.claude/commands`, `.claude/settings.json`, `.claude/settings.local.json`, `.claude/skills/*` (5 living, skip `attic`), `scripts/`, `phieu/`, `templates/`, `hooks/pre-commit`, `hooks/pre-push`, `.gitignore`. → chỉ vào `new.tree.golden` (path-shape).
- **GENERATED-authored** (heredoc do `new` tự viết → hash content vào `new.gen.golden`): `.mcp.json` (bin/sos.sh:439-469), `docs/security/INVARIANTS.md` (cp template **+ appended** block :479-480 → authored phần đuôi), `docs/AGENT_MAP.yaml` (stub heredoc :483 / `sos_emit_map_stub` :263-277), `docs/BACKLOG.md` (cp template :485 — **copied**, → tree-only), `CLAUDE.md` (:487-502, có `$name`+`$stack`), `docs/ARCHITECTURE.md` (:514-530, `$name`), `CHANGELOG.md` (:532, có date), stack manifest (`pyproject.toml`/`Cargo.toml`+`src/main.rs`/`package.json` :504-511, `$name`), `.docs-gate.toml` (:537-564 static), `.sos-stack.toml` (`sos_init_security` :138-247, có `detected_at` ts).
  - ⚠️ `INVARIANTS.md` = copied-template + authored-append → hash TOÀN FILE (append là phần authored; template phần đầu identical-by-construction nhưng hash cả file vẫn parity-đúng vì cả 2 copy cùng template). BACKLOG.md = thuần copied-template → tree-only, KHÔNG gen-hash.

### Scope (additive, parity bug-for-bug)
1. Impl Rust `sos new`: `crates/sos-cli/src/commands/new.rs` + clap `New { dir, stack }` variant + dispatch trong `main.rs`. Copy kit assets (deref symlink cho agents/skills), generate skeletons, defaults (`.docs-gate.toml` + shell-out/replicate `sos init security`), verify-setup dispatch, git init + `symbolic-ref main` + arm hooks. **Bug-for-bug Bash** (kể cả flag `--pilot` declared-nhưng-unused, `--force`, guard non-empty target).
2. Harness: `PARITY_ENFORCED` từ `&["map","sync"]` → `&["map","sync","new"]`. Test `parity_new_enforced` multi-assert: stdout (`new.golden`) + tree-shape (`new.tree.golden`) + gen-content (`new.gen.golden`). **Hard-fail bất kỳ assert nào lệch.**
3. `capture.sh` mở rộng additive: build synthetic fake-kit + freeze 3 fixture.

### KHÔNG trong scope
- Sửa `bin/sos.sh` (Bash canonical, bất di).
- OA-02 fix (P077c5). `sos init security` gọi trong `new` [3/4] giữ **bug-for-bug** (KHÔNG upgrade).
- Test verify-setup **CONNECTED** path (cần `doctor` pinned) — xem Constraint 5: fixture ép **doctor-absent** → chỉ parity nhánh skip-line. Đúng scope: `new`'s [4/4] logic = "shell-out + echo verdict", dispatch identical 2 bên.
- adopt / init-security parity = P077c4.

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Marker / Result |
|---|---|---|---|
| 1 | `sos_new` 4-bước + git block ở `bin/sos.sh:355-604`; [1/4] copy :404-474, [2/4] gen :476-533, [3/4] defaults :535-566, [4/4] validator :568-583, git :585-599 | Read range | `[verified]` — Architect đọc trực tiếp |
| 2 | Copied-vs-generated split (bảng trên) đúng cho **mọi** file `new` chạm | `grep -n 'cp \|cat >\|printf.*>' bin/sos.sh` trong 355-604, đối chiếu bảng | `[needs Worker verify]` — Worker liệt kê đủ từng heredoc + cp, xác nhận không sót file authored |
| 3 | verify-setup gọi `"$doctor_bin" verify-setup --repo "$target"` `bin/sos.sh:574`; `doctor_bin="${DOCTOR_BIN:-doctor}"` :570; nhánh skip khi `command -v` fail VÀ `! -x` :571 | Read :568-581 | `[verified]` |
| 4 | Ép **doctor-absent deterministic** bằng `DOCTOR_BIN=/nonexistent/doctor` → `command -v` fail + `-x` fail → in `⏭ doctor not found — skip verify-setup` (:580), env-independent (bất kể `doctor` có trên PATH) | Set env, chạy 1 lần, quan sát nhánh | `[needs Worker verify]` — Worker confirm nhánh skip fire + Rust `new.rs` replicate cùng dòng |
| 5 | git init KHÔNG commit (:587-599) → không có commit-sha nondeterminism; `.git/` tạo nhưng phải EXCLUDE khỏi `new.tree.golden`; `install-hooks.sh` stdout (piped sed) deterministic | Chạy fake-kit `new` 2 lần, diff tree (không `.git/`) | `[needs Worker verify]` — Worker xác nhận `.git/` exclude + install-hooks output ổn định |
| 6 | Synthetic fake-kit tối thiểu phải chứa ĐỦ mọi path `sos_new` đọc từ `$K`: `.claude/agents` + `.claude/commands` + `.claude/settings.json`, `agents/orchestrator.md`, `templates/{claude-settings.local.json,INVARIANTS-template.md,BACKLOG_template.md}`, `skills/*/` (≥1 living + `attic/` để test skip), `scripts/` (gồm `install-hooks.sh`, `parsers/*`), `phieu/`, `hooks/{pre-commit,pre-push}`, `.gitignore` | Enumerate mọi `"$K/..."` read trong :385-599, dựng fake-kit khớp | `[needs Worker verify]` — **ESCAPE HATCH gắn ở đây (xem dưới)** |
| 7 | `capture.sh` + `parity.rs` cơ chế c1/c2 (`run_rust_with_kit` set `SOS_KIT_DIR`, sed-normalize target→`<TARGET>`/kit→`<SOS_KIT_DIR>`/date→placeholder, `PARITY_ENFORCED` set, `TempFixture` helper) tái dùng được cho `new` | Read `crates/sos-cli/tests/{capture.sh,parity.rs}` + README | `[needs Worker verify]` — Architect không đọc `tests/` (envelope); tin citation c1/c2 discoveries |
| 8 | `new.golden` hiện có (P077a real-kit stdout) → **re-froze** dưới fake-kit + doctor-absent (KHÔNG additive — justify như c2 re-froze `sync.golden`) | `git diff` sau capture; ghi rationale | `[needs Worker verify]` |
| 9 | `grep -rl "# TODO"` (:583) list order — filesystem order? cần sort để deterministic? Rust phải match Bash order bug-for-bug | Chạy 2 lần, so order; check Bash có sort không (KHÔNG — raw `grep -rl`) | `[needs Worker verify]` — nếu order flaky, freeze cần normalize/sort HOẶC Rust replicate cùng raw order |
| 10 | `.sos-stack.toml` `detected_at="$ts"` (:145,:139) + CHANGELOG date (:532) + INVARIANTS append tĩnh → normalize ts/date → placeholder TRƯỚC khi hash vào `new.gen.golden` | Đối chiếu sed rules c1/c2 (đã có date rule) | `[needs Worker verify]` |

### ⚠️ ESCAPE HATCH (Worker DỪNG + escalate qua RELAY nếu):
- **(A) Fake-kit bất khả thi/brittle** — nếu dựng minimal fake-kit thoả mãn MỌI `$K` read của `sos_new` quá nặng hoặc dễ vỡ (anchor #6): DỪNG, escalate. Fallback đề xuất để Architect chốt: chạy tree/stdout trên **real `$SOS_KIT_DIR`** (chấp nhận kit-path-coupling ở tree — kém hơn nhưng gen vẫn hash-only-authored), HOẶC tỉa fake-kit xuống subset + normalize path-set. **KHÔNG tự quyết** — đây là design-tradeoff Tầng 1.
- **(B) verify-setup output nondeterministic dù ép doctor-absent** — nếu `DOCTOR_BIN=/nonexistent` KHÔNG ổn định hoá được nhánh (anchor #4) hoặc môi trường capture buộc doctor present: DỪNG, escalate (có thể phải normalize/strip cả block `[4/4]` khỏi `new.golden`).
- **(C) Copied-vs-generated không phân định được** — nếu 1 file vừa copied vừa authored kiểu không hash sạch được (ngoài `INVARIANTS.md` đã xử ở bảng): DỪNG, escalate.

---

## Debate Log

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Worker accepted V1 — no challenges.** Anchor verification: #1 ✅, #2 ✅, #3 ✅, #4 ✅ (live-probed: `DOCTOR_BIN=/nonexistent/doctor` deterministically fires `⏭ doctor not found` skip-line, 2 runs identical), #5 ✅ (live-probed: no commit made, `.git/` present must be excluded from tree golden, `install-hooks.sh` stdout stable 2 runs), #6 ✅ (live-probed: minimal synthetic fake-kit — plain dirs, no git history needed, simpler than `sync`'s c2 fixture — `sos_new` ran clean exit 0 twice, stdout + tree byte-identical), #7 trusted per c1/c2 discovery citation (also self-read `parity.rs`/`capture.sh`, confirmed `TempFixture`/`run_rust_with_kit`/normalize pattern generic and reusable), #8 ✅ confirmed live — committed `new.golden` currently reflects the CONNECTED-path (real doctor present at last capture time, full `[WIRED] J1..J6` block) → **mandatory re-froze under fake-kit + `DOCTOR_BIN=/nonexistent/doctor`** per Constraint 5, no design ambiguity (mechanically deterministic flip, confirmed via live probe).

Escape hatches A/B/C: **none triggered.** Fake-kit feasible and cheap (simpler than sync's, no git history needed). verify-setup doctor-absent branch deterministic. Copied-vs-generated split is clean (no ambiguous file beyond the already-resolved `INVARIANTS.md` case).

Ready for Chủ nhà approval gate.

---

## Nhiệm vụ

### 1. Impl Rust `sos new` — `crates/sos-cli/src/commands/new.rs` (NEW file)
- **Tạo:** `new.rs` port bug-for-bug `sos_new` (`bin/sos.sh:355-604`).
- **Lưu ý:**
  - Arg parse: `<dir>` positional + `--stack python|rust|ts` (bắt buộc, reject unknown/empty như :379-383) + `--pilot` (declared, **unused** — parse nhưng bỏ qua, bug-for-bug) + `--force`.
  - Guard: `SOS_KIT_DIR` phải có `.claude/agents` (:386); non-empty target không `--force` → refuse (:392-397).
  - [1/4] copy: deref symlink (`-RL` tương đương) cho `.claude/agents`, `.claude/commands`, mỗi living skill (skip `attic` :420); cp `scripts`/`phieu`/`templates`/`hooks/{pre-commit,pre-push}`/`.gitignore`; write `.mcp.json` heredoc (:439-469 nguyên văn); strip `.DS_Store`/`.pyc`/`__pycache__` (:471-473); echo dòng summary :474.
  - [2/4] gen: INVARIANTS (cp template + append :479-480), AGENT_MAP stub (:483), BACKLOG (cp template :485), CLAUDE.md (:487-502), stack manifest theo `$stack` (:504-511, rust thêm `src/main.rs` stub), ARCHITECTURE.md (:514-530), CHANGELOG (:532); echo summary :533.
  - [3/4] defaults: `.docs-gate.toml` (:537-564), gọi `sos init security` in-target (:565 — replicate `sos_init_security` heredoc bug-for-bug, KHÔNG OA-02).
  - [4/4] validator: `doctor_bin = DOCTOR_BIN|doctor`; nhánh skip khi absent → in dòng :580 (fixture ép nhánh này); grep `# TODO` list (:583).
  - [+] git: `git init -q` + `git symbolic-ref HEAD refs/heads/main` (KHÔNG `init -b`, git cũ) + `bash scripts/install-hooks.sh` (:592); KHÔNG auto-commit.
  - Git read/shell-out: dùng `std::process::Command::new("git")` như c2 (KHÔNG `git2` crate).

### 2. clap enum + dispatch — `crates/sos-cli/src/main.rs`
- **Tìm:** clap enum (hiện `Init/Blueprint/Contract/Apply/Recipe/Launch/Status/Map/Sync` per c1/c2). `[needs Worker verify]` exact variant list.
- **Thêm:** `New { dir: String, stack: String, pilot: bool, force: bool }` + match arm → `commands::new::run(...)`.

### 3. Harness — `crates/sos-cli/tests/parity.rs`
- **Tìm:** `const PARITY_ENFORCED: &[&str] = &["map", "sync"];`
- **Thay bằng:** `&["map", "sync", "new"];`
- **Thêm:** `#[test] fn parity_new_enforced()` — dựng fake-kit + target-dir (via `TempFixture`), chạy Rust `sos new <tgt> --stack <X>` với `SOS_KIT_DIR=<fake-kit>` + `DOCTOR_BIN=/nonexistent/doctor` (`run_rust_with_kit`), assert 3 thứ hard-fail: stdout==`new.golden`, tree-manifest==`new.tree.golden`, gen-hash-manifest==`new.gen.golden`. `[needs Worker verify]` exact helper names từ c1/c2.
- **Lưu ý:** `parity_skeleton_informational` tự skip cmd trong set (c1 đã dựng) → `new` rời informational sang enforced.

### 4. `capture.sh` — build fake-kit + freeze 3 fixture (additive)
- **Thêm** nhánh cho `new`: (a) construct synthetic fake-kit (anchor #6 fileset), (b) chạy Bash `sos_new` với `SOS_KIT_DIR=<fake-kit> DOCTOR_BIN=/nonexistent/doctor`, (c) freeze:
  - `new.golden` = stdout normalized (re-froze).
  - `new.tree.golden` = `find <tgt> -not -path '*/.git/*' | sed 's|^<tgt>/||' | sort` (path-shape, no content).
  - `new.gen.golden` = với mỗi GENERATED-authored relpath (bảng): normalize (date/ts→placeholder) → `sha256` → `<relpath> <sha256>`, sorted.
- **Lưu ý:** normalize sed tái dùng rule c1/c2 (target/kit/date) + thêm ts-rule cho `.sos-stack.toml` nếu date-rule chưa phủ. `[needs Worker verify]`.

### 5. Nghiệm thu ghi Discovery + docs (xem Nghiệm thu).

---

## Files cần sửa
- `bootstrap/sos-rs/crates/sos-cli/src/commands/new.rs` (NEW)
- `bootstrap/sos-rs/crates/sos-cli/src/main.rs` (clap + dispatch)
- `bootstrap/sos-rs/crates/sos-cli/tests/parity.rs` (`PARITY_ENFORCED` += `new`, new test)
- `bootstrap/sos-rs/crates/sos-cli/tests/golden/capture.sh` (fake-kit + 3 freeze)
- `bootstrap/sos-rs/crates/sos-cli/tests/golden/new.golden` (re-froze)
- `bootstrap/sos-rs/crates/sos-cli/tests/golden/new.tree.golden` (NEW)
- `bootstrap/sos-rs/crates/sos-cli/tests/golden/new.gen.golden` (NEW)
- Docs (DOCS-GATE): `bootstrap/sos-rs/README.md`, `bootstrap/sos-rs/crates/sos-cli/tests/README.md`, `CHANGELOG.md`, `docs/plans/P077c-decomposition.md` (mark c3 SHIPPED)

## Files KHÔNG sửa
- `bin/sos.sh` — **canonical, bất di.** `git diff bin/sos.sh` phải rỗng.
- `map.rs`, `sync.rs`, `map.golden`, `map.agent_map.golden`, `sync.golden`, `sync.tree.golden` — không regress.
- `adopt.golden` — c4 xử.

---

## Luật chơi (Constraints)
1. **Bash canonical, additive.** `git diff bin/sos.sh` rỗng. Rust chứng minh BẰNG (bug-for-bug), KHÔNG "sửa tốt hơn" (đó là c5).
2. **KHÔNG hash content của copied kit assets.** Chỉ path (tree) cho chúng. Content-hash CHỈ cho GENERATED-authored (gen). Đây là điểm chống kit-content-coupling — vi phạm = fixture drift mỗi kit-change.
3. **Synthetic fake-kit, KHÔNG real `$SOS_KIT_DIR`** cho tree/gen/stdout (trừ khi escape-hatch A kích hoạt → Architect chốt fallback). Fake-kit fileset CỐ ĐỊNH → determinism.
4. **Hard-fail cả 3 assert.** stdout + tree + gen; bất kỳ lệch = FAIL (không informational).
5. **verify-setup ép doctor-absent** (`DOCTOR_BIN=/nonexistent/doctor`) trong capture VÀ test → cả Bash+Rust đi nhánh skip-line deterministic. CONNECTED-path KHÔNG parity-test (out-of-scope, cần doctor pinned).
6. **`.git/` EXCLUDE** khỏi `new.tree.golden`. git KHÔNG commit → không sha nondeterminism.
7. **`sos init security` trong [3/4] giữ bug-for-bug** — KHÔNG OA-02 (P077c5).
8. **Escape hatch (A/B/C) = DỪNG + RELAY escalate**, KHÔNG tự quyết design-tradeoff Tầng 1.
9. **Determinism proof bắt buộc**: 2 lần `capture.sh` độc lập → 3 fixture byte-identical (như c2).

---

## Nghiệm thu

### Automated
- `cd bootstrap/sos-rs && cargo build --workspace` → clean.
- `cargo test -p sos-cli --test parity` → `parity_map_enforced`, `parity_sync_enforced`, `parity_new_enforced`, `parity_skeleton_informational` ĐỀU pass; adopt còn informational.
- `cargo test --workspace` → all green (dep_direction giữ pass).

### Parity proof (Discovery bắt buộc)
- Chạy Bash `sos_new` VÀ Rust `sos new` trên **cùng fake-kit + cùng target**, capture stdout + tree + gen-hash, diff cả 3 → `STDOUT IDENTICAL` / `TREE IDENTICAL` / `GEN IDENTICAL`.
- Ghi Rust deviations (nếu có) — mọi deviation phải là **intentional bug-for-bug match** (vd raw `grep -rl` order, `--pilot` unused).

### Negative test (Discovery bắt buộc — chứng minh oracle bắt regression)
- **Tree**: sabotage `new.rs` bỏ 1 `cp` (vd skip copy `phieu/`) → stdout vẫn OK nhưng `parity_new_enforced` FAIL trên **tree-manifest** assert. Revert (`mv` + `touch` fix stale-mtime per c1 gotcha).
- **Gen**: sabotage 1 authored heredoc (vd đổi 1 token trong CLAUDE.md skeleton) → FAIL trên **gen-content** assert (tree unaffected). Revert.
- Xác nhận từng fail fire đúng assert kỳ vọng (proof 3 lớp không thừa).

### Regression
- `git diff bin/sos.sh` rỗng.
- Re-chạy `capture.sh` 2 lần → `new.golden`/`new.tree.golden`/`new.gen.golden` byte-identical giữa 2 lần (determinism); `map.*`/`sync.*` golden không đổi.
- `bash scripts/trust-gate.sh` → exit 0 (`bootstrap/sos-rs/**` ngoài `.sos-trust-baseline` — no rebaseline; confirm).

### Docs Gate (Tầng 1)
- `bootstrap/sos-rs/README.md` — cập nhật parity-status row `new` → "Parity (hard-fail, stdout + tree-shape + gen-content)".
- `crates/sos-cli/tests/README.md` — thêm mô tả 3-layer fixture cho `new` (tree-shape vs gen-content split rationale, synthetic fake-kit, doctor-absent lever); update `PARITY_ENFORCED = &["map","sync","new"]` + Layout.
- `CHANGELOG.md` — entry P077c3.
- `docs/plans/P077c-decomposition.md` — status line c3 SHIPPED + feed-forward cho c4 (adopt tái dùng 3-layer + fake-kit; adopt gọi map bug-for-bug từ c1).
- Discovery ghi rõ: "Tầng 1 docs updated: <list>".

### Discovery Report
- Anchor #1-10 kết quả (verified/corrected).
- Escape-hatch A/B/C có kích hoạt không; nếu có → RELAY + Architect quyết đã ghi.
- Copied-vs-generated final list (Worker-confirmed).
- Fake-kit fileset thực tế đã dựng.
- Tier escalation (nếu 2→1) — dự kiến None (đã Tầng 1).
