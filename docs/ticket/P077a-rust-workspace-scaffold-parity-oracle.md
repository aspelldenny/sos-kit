# PHIẾU P077a: Rust workspace scaffold + freeze Bash golden oracle + parity harness skeleton

> Sub-phiếu ĐẦU của P077 decomposition (`docs/plans/P077-decomposition.md`). Additive-only; Bash `bin/sos.sh` GIỮ canonical.

---

> **Loại:** Feature (infra)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — build system + oracle foundation cho mọi sub-phiếu P077 sau; sai thì LAN xuống P077b–e)
> **Lane:** Guarded
> **Ảnh hưởng:** `bootstrap/sos-rs/Cargo.toml`, `bootstrap/sos-rs/tests/**` (mới), `bootstrap/sos-rs/README.md`, golden fixtures (mới)
> **Dependency:** P076 merged (adapters/claude landed). None khác.

---

## Context

### Vấn đề hiện tại

P077 (Rust workspace + adapter/install framework) quá lớn cho một delivery unit → đã chia (`docs/plans/P077-decomposition.md`). Sub-phiếu đầu này giải OA-06: Rust `sos-rs` hiện **0 test**, chưa parity với Bash, thiếu `new/adopt/map/sync`. Không có golden oracle thì mọi Rust dev sau (P077b–e) drift mù. Phải **freeze Bash làm oracle TRƯỚC** khi Rust thành canonical.

### Giải pháp

Additive, 3 mảnh, KHÔNG đổi hành vi user-facing (users vẫn dùng `bin/sos.sh`):

1. **Workspace scaffold (transitional)** — biến `bootstrap/sos-rs/Cargo.toml` từ `[package]` đơn thành `[workspace]` root chứa member hiện tại. **Root đặt tại `bootstrap/sos-rs/` (KHÔNG repo-root)** để tránh flip repo contract sớm — relocate lên repo-root để P077e cutover. Chưa carve crate (đó là P077b).
2. **Freeze Bash golden oracle** — capture stdout/exit của các subcommand parity-critical (`new`, `adopt`, `map`, `sync`) từ `bin/sos.sh` vào fixtures committed. Đây là canonical reference OA-06.
3. **Parity-harness skeleton** — test crate/module chạy được, diff Rust-output vs Bash-golden. Rust chưa impl các command này → harness **report "not yet parity" dạng informational, KHÔNG hard-fail** (hard-fail là P077c).

### Scope

- CHỈ: workspace scaffold + golden fixtures + harness skeleton + README ownership fix.
- KHÔNG cutover (P077e). KHÔNG đổi `CLAUDE.md` repo contract (P077e). KHÔNG carve crate sos-core/adapter (P077b). KHÔNG impl `new/adopt/sync/map` logic đầy đủ trong Rust (P077c). KHÔNG đụng `bin/sos.sh` behavior (nó là oracle — freeze, không sửa).

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `bootstrap/sos-rs/Cargo.toml` là `[package]` đơn (name=`sos`, bin path `src/main.rs`), CHƯA có `[workspace]` | Đã Read | ✅ `[verified]` (Cargo.toml dòng 1-9) |
| 2 | Target crate names = sos-cli/sos-core/sos-install/sos-adapter-claude/sos-adapter-codex/sos-hooks | `docs/PORTABILITY_ARCHITECTURE.md` dòng 24-38 | ✅ `[verified]` (doc read) |
| 3 | `bootstrap/sos-rs/src/` có command init/blueprint/contract/apply/recipe/launch/status; THIẾU new/adopt/sync/map | `grep -rE "\"(new|adopt|sync|map|init|blueprint|contract|apply|recipe|launch|status)\"" bootstrap/sos-rs/src/` (Worker enumerate clap subcommands) | ✅ confirmed — no top-level new/adopt/sync/map Cmd variant |
| 4 | `bootstrap/sos-rs` hiện chạy 0 test | `cd bootstrap/sos-rs && cargo test 2>&1 \| grep "test result"` → expect `0 passed` | ✅ confirmed — `0 passed; 0 failed` pre-Task-3 |
| 5 | `bin/sos.sh` có đủ 11 subcommand (Bash oracle surface) gồm new/adopt/map/sync | `grep -nE "^\s*(new|adopt|map|sync|init|blueprint|contract|apply|recipe|launch|status)\)" bin/sos.sh` (Worker confirm dispatch cases) | ✅ confirmed — `bin/sos.sh:1276-1286` all 11 cases |
| 6 | `adapters/claude/{README,MAPPING}.md` tồn tại (landed P076) | `ls adapters/claude/README.md adapters/claude/MAPPING.md` | ✅ confirmed |
| 7 | `bin/sos.sh` chạy được deterministic (không cần Claude runtime) cho `map`/`sync` để capture golden — nếu subcommand delegate Claude skill (in "Open Claude Code") thì output đó CHÍNH LÀ golden (freeze as-is, không unfold) | `bash bin/sos.sh map --help` / chạy trong fixture repo (Worker) | ✅ confirmed deterministic (2-run byte-identical after normalization); PLUS found `sync` depends on sos-kit's own git history — HEAD-pin documented in `tests/README.md` |

**⚠️ Anchor 3-7 là `[needs Worker verify]`** — Architect docs-only, không grep src/ hay chạy Bash/cargo. Worker verify ở EXECUTE.

---

## Debate Log

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Worker accepted V1 — no blocking objections.** Anchor verification:
- #3 ✅ (Rust src has init/blueprint/contract/apply/recipe/launch/status; no top-level new/adopt/sync/map)
- #4 ✅ (`cargo test` → 0 passed, 0 failed pre-Task-3)
- #5 ✅ (`bin/sos.sh:1276-1286` dispatches all 11 subcommands incl. new/adopt/map/sync)
- #6 ✅ (`adapters/claude/{README,MAPPING}.md` exist, P076 landed)
- #7 ⚠️→✅ deterministic-capture feasible for all 4 commands with normalization (absolute paths + dates), PLUS one non-cosmetic finding: `sos sync`'s take-newer/flag classification depends on sos-kit's OWN git history (`_blob_in_history`, `bin/sos.sh:992-999`), not just fixture-repo state — must pin sos-kit HEAD sha alongside the `sync.golden` fixture (documented in Task 2/tests/README.md at EXECUTE, not blocking V1).
- `cargo build` green (1.83s, no shared-cache config found, not blocking).
- Harness design confirmed coherent: Rust binary has no new/adopt/map/sync subcommands → invoking them yields clap "unrecognized subcommand" (non-zero exit), which the harness diffs against golden and reports informationally without asserting.

Advisory (non-blocking, self-closeable at EXECUTE, no Architect respond needed): Task 2's "pin fixture-repo state" instruction should be read to also include pinning sos-kit's own commit sha for the `sync` fixture specifically — Worker will add this to `tests/README.md` reproduction notes.

Ready for Chủ nhà approval gate.

**Status:** ✅ APPROVED V1 (Chủ nhà routed CHALLENGE APPROVED → EXECUTE, sprint delegated)

---

## Nhiệm vụ

### Task 1: Chuyển `bootstrap/sos-rs/Cargo.toml` thành workspace root (transitional)

**File:** `bootstrap/sos-rs/Cargo.toml`

**Tìm:** khối `[package]` hiện tại (name=`sos`, version=`0.1.0`) + `[[bin]]` + `[dependencies]`.

**Thay bằng / Thêm:** thêm `[workspace]` table giữ member hiện tại là crate `sos` in place. Giữ `[package]`/`[[bin]]`/`[dependencies]` để crate `sos` vẫn build như cũ. Cấu trúc tối thiểu:
```toml
[workspace]
members = ["."]
resolver = "2"

# [package], [[bin]], [dependencies] giữ nguyên bên dưới
```

**Lưu ý:**
- Giữ crate `sos` build được y hệt trước (additive). `cargo build` trong `bootstrap/sos-rs/` phải vẫn ra binary `sos`.
- KHÔNG đặt workspace ở repo-root (`/Cargo.toml`) — điều đó flip repo contract "Not a runtime binary source", để P077e.
- KHÔNG carve `sos-core`/`sos-cli`/adapter crate ở đây — chỉ workspace shell (P077b carve). Nếu resolver=2 gây warning với edition 2021, giữ nguyên; Worker verify `cargo build` clean.

### Task 2: Freeze Bash golden oracle cho subcommand parity-critical

**File:** `bootstrap/sos-rs/tests/golden/` (thư mục mới) — mỗi subcommand một fixture: `new.golden`, `adopt.golden`, `map.golden`, `sync.golden`.

**Tìm:** N/A (tạo mới).

**Thay bằng / Thêm:** capture output canonical của `bin/sos.sh <cmd>` (stdout + exit code) vào từng `.golden`. Kèm một script/README ghi rõ **cách tái tạo** fixture (command + fixture-repo state) để P077c re-verify.

**Lưu ý:**
- **`bin/sos.sh` là ORACLE — freeze, KHÔNG sửa** (OA-06, OA-10 điểm giữ oracle).
- Nếu subcommand delegate Claude skill (in "Open Claude Code..."), output ĐÓ chính là golden — freeze nguyên văn, KHÔNG cố unfold logic LLM (Anchor 7).
- Capture phải **deterministic**: pin fixture-repo state, strip/normalize timestamp/absolute path (ghi rõ normalization rule trong tests README). Nếu một subcommand không capture được deterministically non-interactive → Worker DISCOVERY + capture phần dry/`--help` được, note limitation.
- Ưu tiên `new/adopt/map/sync` (OA-06 parity list). `init/blueprint/contract/apply/recipe/launch/status` để sau nếu thời gian — không bắt buộc ở P077a.

### Task 3: Parity-harness skeleton (informational, no hard-fail)

**File:** `bootstrap/sos-rs/tests/parity.rs` (integration test mới) hoặc module tương đương.

**Tìm:** N/A (tạo mới).

**Thay bằng / Thêm:** harness đọc mỗi `tests/golden/*.golden`, chạy Rust `sos <cmd>` tương ứng, diff output. Ở P077a: **report diff (println/eprintln "P077a: <cmd> not yet parity — Rust command unimplemented/differs")** và **PASS test** (informational). Cấu trúc để P077c chỉ cần flip một flag/const `HARD_FAIL = false → true`.

**Lưu ý:**
- KHÔNG hard-fail: Rust chưa impl `new/adopt/map/sync` (Anchor 3) → nếu assert equal sẽ đỏ ngay, phá gate additive. Test phải xanh nhưng in rõ trạng thái non-parity.
- Đặt điểm chuyển hard-fail thành **một chỗ duy nhất** (const/env) để P077c mở khóa cơ học, không phải viết lại harness.
- Normalize output (timestamp/path) đồng bộ với rule capture ở Task 2 — cùng một normalizer, tránh false-diff.

### Task 4: Fix ownership contract mâu thuẫn trong `bootstrap/sos-rs/README.md`

**File:** `bootstrap/sos-rs/README.md`

**Tìm:** dòng nói "deprecate bash and move to its own repo `github.com/aspelldenny/sos`" (README dòng ~27) + "When Rust port reaches feature parity... move to its own repo".

**Thay bằng / Thêm:** cập nhật theo target hiện tại (RUNTIME_BOUNDARY finding #5 + PORTABILITY_ARCHITECTURE dòng 41): Rust workspace được **lift vào chính `sos-kit`**, KHÔNG extract sang repo khác. Ghi rõ trạng thái: workspace scaffold landed (P077a), Bash vẫn canonical tới cutover P077e.

**Lưu ý:** chỉ sửa câu ownership/extraction mâu thuẫn + status. KHÔNG viết lại toàn README. KHÔNG tuyên bố Rust đã canonical (chưa — P077e).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `bootstrap/sos-rs/Cargo.toml` | Task 1: thêm `[workspace]` shell, giữ crate `sos` build in place |
| `bootstrap/sos-rs/tests/golden/*.golden` (mới) | Task 2: golden fixtures từ `bin/sos.sh` |
| `bootstrap/sos-rs/tests/README.md` (mới) | Task 2: cách tái tạo fixture + normalization rule |
| `bootstrap/sos-rs/tests/parity.rs` (mới) | Task 3: harness skeleton informational |
| `bootstrap/sos-rs/README.md` | Task 4: fix ownership contract (finding #5) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `bin/sos.sh` | ORACLE — behavior KHÔNG đổi; 11 subcommand vẫn chạy như trước (Anchor 5) |
| `CLAUDE.md` | "Not a runtime binary source" — KHÔNG đụng ở P077a (flip ở P077e) |
| `bootstrap/sos-rs/src/**` | KHÔNG impl new/adopt/sync/map logic (P077c); build vẫn xanh sau Task 1 |
| `adapters/claude/**` | Landed P076 — không sửa |

---

## Luật chơi (Constraints)

1. **Additive-only.** Sau phiếu này, user chạy `bin/sos.sh` KHÔNG thấy khác biệt. Rust dựng song song.
2. **Bash `bin/sos.sh` là oracle — freeze, không sửa.** Bất kỳ nhu cầu sửa Bash = DISCOVERY, không tự sửa.
3. **Harness KHÔNG hard-fail** ở P077a. Rust chưa parity là expected. Test phải xanh + in trạng thái non-parity.
4. **Không repo-root Cargo.toml**, không flip repo contract, không carve adapter/core crate — các thứ đó thuộc P077b/P077e.
5. **Golden capture phải deterministic + tái tạo được** — normalization rule ghi rõ, cùng normalizer cho capture và harness.
6. Nếu subcommand không capture được deterministically → DISCOVERY, không bịa fixture.

---

## Nghiệm thu

### Automated
- [x] `cd bootstrap/sos-rs && cargo build` — xanh, ra binary `sos` như trước
- [x] `cd bootstrap/sos-rs && cargo test` — xanh; harness chạy + in "not yet parity" cho các subcommand chưa impl (KHÔNG đỏ)
- [x] `bash -n bin/sos.sh` — vẫn PASS (oracle chưa bị đụng)

### Manual Testing
- [x] `git diff bin/sos.sh` rỗng (oracle untouched)
- [x] `bootstrap/sos-rs/tests/golden/` có fixture cho `new/adopt/map/sync` (hoặc DISCOVERY note nếu subcommand nào không deterministic-capture được)
- [x] Chạy lại script tái tạo fixture → output khớp golden đã commit (reproducibility)

### Regression
- [x] `bin/sos.sh new`/`adopt`/`map`/`sync` chạy y hệt trước phiếu (additive verify)
- [x] Binary `sos` từ `cargo build` vẫn dispatch các command cũ (init/blueprint/... ) như trước

### Docs Gate
- [x] `CHANGELOG.md` — entry P077a
- [x] `bootstrap/sos-rs/README.md` — ownership contract fixed (Task 4, finding #5)
- [x] `bootstrap/sos-rs/tests/README.md` — fixture reproduction + normalization documented
- [x] `CLAUDE.md` "Not a runtime binary source" — **KHÔNG đổi ở P077a**; xác nhận N/A trong Discovery (flip thuộc P077e). Lưu ý: thêm `[workspace]` tại `bootstrap/sos-rs/` (không repo-root) giữ contract root nguyên vẹn — nếu Worker thấy contract đã bị vi phạm bởi scaffold, DISCOVERY để founder quyết đặt flip sớm.

### Discovery Report
- [x] Write to `docs/discoveries/P077a.md`
  - Anchor 3-7 verify results (Bash oracle surface, Rust command gap, adapters exist, deterministic-capture khả thi?)
  - Subcommand nào KHÔNG capture được deterministically (nếu có) + lý do
  - Docs updated: bootstrap README + tests README (list)
  - Repo-contract touch: xác nhận "N/A — scaffold rooted at bootstrap/, repo-root contract untouched" HOẶC escalate nếu Worker đánh giá khác
  - Tier escalations (None nếu không)
- [x] Append 1-line index vào `docs/DISCOVERIES.md`
