# PHIẾU P077d2: Install engine — transaction plan / dry-run / non-clobber / rollback / apply

> **Loại:** Feature
> **Ưu tiên:** P1
> **Tầng:** 1 (install engine mutate filesystem + rollback + là contract surface cho `uninstall` d1 và tool-resolve seam d3 — AUTO Tầng 1 dù additive)
> **Lane:** Guarded
> **Ảnh hưởng:** `bootstrap/sos-rs/crates/sos-install/**` (engine logic), `crates/sos-cli` (wire `sos install` command), engine correctness fixtures, **+ narrow additive `crates/sos-core/src/adapter.rs` (V2: thêm field `content` vào `ManagedOperation` placeholder — xem Debate Log O1.1)**. `bin/sos.sh` + `install.sh` ZERO-touch.
> **Dependency:** P077d1 (Adapter trait + ManagedManifest schema — SHIPPED 2026-07-22)

---

## Context

### Vấn đề hiện tại

P077d1 đã carve **foundation** vào `sos-core`: `Adapter` trait (5 method) + `ManagedManifest` schema (6 field), `sos-adapter-claude` implement trait với **stub bodies** (zero fs mutation). `sos-install`/`sos-hooks` vẫn skeleton rỗng ("logic lands in P077d" — `bootstrap/sos-rs/README.md:20-24,44`).

Chưa có install ENGINE: chưa lệnh nào lập transaction plan, chưa apply asset vào project, chưa non-clobber/rollback/dry-run. `PORTABILITY_ARCHITECTURE.md:109-117` mô tả `sos install` như một transaction 7-step; `core/POLICY.md:78-86` "Safe mutation" đặt luật (additive/non-clobber, backup+rollback, missing-gate-fail-visible). d2 impl các luật đó thành engine chạy được, **oracle = correctness fixtures** (KHÔNG parity-vs-Bash — `sos install` là command MỚI, không có Bash counterpart; xem `docs/plans/P077d-decomposition.md:9`).

### Giải pháp

Impl install engine trong `sos-install`, **drive hoàn toàn qua `Adapter` trait của d1** (engine không biết gì về `.claude/**` hay runtime cụ thể — nó chỉ execute cái `Plan` mà một adapter trả về). Wire lệnh `sos install --runtime <r>` NEW trong `sos-cli`, chạy **SONG SONG** `install.sh` (KHÔNG flip default distribution — `install.sh` + `bin/sos.sh` KHÔNG đổi 1 byte).

**Engine-vs-adapter separation (CHỐT — orchestrator proposal ACCEPTED):**
- Engine correctness được oracle-prove bằng một **MockAdapter deterministic** định nghĩa TRONG test harness (emit một `Plan` gồm mix managed-file operations). Đây là điểm mấu chốt của abstraction d1: engine đúng **độc lập với adapter nào drive nó** → test cô lập transaction logic, KHÔNG kéo Claude render vào.
- `ClaudeAdapter.plan()/render()` (emit `.claude/**` operations THẬT) **GIỮ STUB — deferred, KHÔNG thuộc d2.** Đây là **seam** documented. `sos install --runtime claude` được wire (construct `ClaudeAdapter`, feed engine) và chạy end-to-end an toàn, nhưng vì plan còn stub nó render tối thiểu/rỗng — đủ smoke chứng minh command wired, KHÔNG phình d2 thành cả Claude render surface.
- **Lý do:** giữ d2 = ENGINE ONLY (đơn vị nặng nhất theo decomposition). Real Claude asset rendering là surface riêng, lands ở phiếu sau (P076 parity wiring / P078). Nếu ép render thật vào d2, engine fixture bị couple với Claude asset content → oracle nhiễu, lane budget vỡ.

**V2 amendment — `ManagedOperation` carry `content` (Debate Log O1.1 ACCEPT / Alt A):** Worker CHALLENGE phát hiện `ManagedOperation` (d1 placeholder, `crates/sos-core/src/adapter.rs:42-46`) chỉ có `{description, target_path}` — KHÔNG field bytes → engine KHÔNG có đường trait-pure lấy content để write/hash/record vào manifest (block additive/non-clobber/rollback). Fix = **narrow additive**: thêm field `content` vào `ManagedOperation` (placeholder struct engine consume), **GIỮ NGUYÊN** Adapter 5-method trait + ManagedManifest 6-field. Đây là first-consumer (engine) lộ ra placeholder thiếu; host-neutral (chỉ bytes + path, không host token) → không phá "agnostic".

**Step 5 tool-resolve = seam cho d3:** transaction 7-step của `PORTABILITY_ARCHITECTURE.md:109-117`, engine impl step 1-4 + 6-7; **step 5 (resolve tool manifest / download tools) = stub no-op** trả về empty, đánh dấu rõ chỗ d3 (OA-07) fill. d2 KHÔNG assert gì về step 5.

### Scope
- CHỈ sửa: `crates/sos-install/**` (engine), `crates/sos-cli/**` (wire `sos install` command + subcommand dispatch), engine correctness fixtures (new test file), **`crates/sos-core/src/adapter.rs` NARROW (V2: chỉ thêm field `content` vào `ManagedOperation` placeholder — KHÔNG đụng Adapter trait 5-method, KHÔNG đụng ManagedManifest)**, docs (xem Docs Gate).
- KHÔNG sửa: `bin/sos.sh`, `install.sh` (additive proof — `git diff` empty), `crates/sos-core/src/{manifest,state}.rs` + Adapter trait signature (d1 landed — consume, đừng đổi trait/schema), `crates/sos-adapter-claude/**` render bodies (giữ stub — real render KHÔNG thuộc d2), mọi golden của `parity.rs`.

---

## Task 0 — Verification Anchors

> Architect docs-only (no Bash/Grep/Read-src). Mọi anchor code-level = `[needs Worker verify]`; anchor doc-level tôi đã Read = `[verified]`. Worker grep-verify TRƯỚC khi EXECUTE.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `Adapter` trait (5 method: detect/plan/render/verify/uninstall) live tại `crates/sos-core/src/adapter.rs`, dùng được từ `sos-install`. **V2:** `ManagedOperation` placeholder (`adapter.rs:42-46`) hiện `{description, target_path}` — d2 thêm field `content` (KHÔNG đụng trait 5-method). | `grep -n "trait Adapter\|struct ManagedOperation" crates/sos-core/src/adapter.rs` + `grep "fn plan\|fn render\|fn uninstall"` | ⏳ `[needs Worker verify]` — per `docs/discoveries/P077d1.md:19` + Worker CHALLENGE Turn 1 (`adapter.rs:42-46` confirmed) |
| 2 | `ManagedManifest` struct 6 field (`owner`/`source_version`/`source_identity`/`target_path`/`content_hash`/`rollback_ref: Option<String>`) + serde TOML round-trip tại `crates/sos-core/src/manifest.rs` — **GIỮ NGUYÊN, d2 KHÔNG đổi** | `grep -n "struct ManagedManifest" crates/sos-core/src/manifest.rs` + đọc field list | ⏳ `[needs Worker verify]` — per `docs/discoveries/P077d1.md:20` |
| 3 | `crates/sos-install/` là skeleton rỗng (engine logic lands here); `sos-install/Cargo.toml` deps `sos-core` (thấy trait/manifest) | `ls crates/sos-install/src/` + `grep sos-core crates/sos-install/Cargo.toml` | ⏳ `[needs Worker verify]` — per `bootstrap/sos-rs/README.md:20-24,44` |
| 4 | `sos-cli` là composition root, deps `sos-install` + `sos-adapter-claude` (wire được `sos install` construct adapter + gọi engine) | `grep "sos-install\|sos-adapter-claude" crates/sos-cli/Cargo.toml` | ⏳ `[needs Worker verify]` — per `bootstrap/sos-rs/README.md:19-24` |
| 5 | Transaction 7-step (detect / common-assets / render-adapter / hook-stubs / **tool-resolve=step5** / write-manifest / doctor+rollback) đúng `PORTABILITY_ARCHITECTURE.md:109-117` | Read `docs/PORTABILITY_ARCHITECTURE.md:109-117` | ✅ `[verified]` — Architect Read L109-117 |
| 6 | Safe-mutation luật (additive/non-clobber, never silently overwrite user-customized, backup+rollback record, missing-gate-fail-visible) tại `core/POLICY.md:78-86` + generated-artifact apply rules `PORTABILITY_ARCHITECTURE.md:79-86` | Read cả 2 range | ✅ `[verified]` — Architect Read cả 2 |
| 7 | Non-clobber incoming-copy precedent: `adopt` ghi `.sos-adopt-incoming/<path>` (`adopt.rs:216-248`) + preservation-assert (seeded file sha256 unchanged). **V2 clarify (Worker CHALLENGE):** `.sos-adopt-incoming` là **presence-only** staging (KHÔNG hash) → d2 hash-discrimination (UPDATE-if-hash-matches-recorded) là **logic MỚI**, KHÔNG reuse từ adopt. Chỉ mirror pattern *ghi incoming copy*, KHÔNG mirror discrimination. | `grep -rn "sos-adopt-incoming" crates/sos-cli/` + đọc `adopt.rs:216-248` | ⏳ `[needs Worker verify]` — per `bootstrap/sos-rs/README.md:82`, `docs/discoveries/P077c4.md` + Worker CHALLENGE Turn 1 |
| 8 | Collision-safe `TempFixture` (pid+nanos+**AtomicU64 counter**) tại `crates/sos-cli/tests/parity.rs`; `cargo test --workspace` ×20 parallel = 0 flaky là chuẩn chung post-c6 | `grep -n "TEMP_FIXTURE_COUNTER\|AtomicU64" crates/sos-cli/tests/parity.rs` | ⏳ `[needs Worker verify]` — per `docs/discoveries/P077c6.md:54-66`. **Nếu d2 fixtures ở crate khác (`sos-install`), TempFixture phải factor ra shared test-util HOẶC replicate pattern AtomicU64-keyed — Worker self-decide (Tầng 2), nhưng collision-safe là BẮT BUỘC.** |

**⚠️ Anchor #3/#8 là seam-risk:** nếu engine KHÔNG drive được thuần qua trait (cần host knowledge rò vào `sos-install`), hoặc rollback KHÔNG phải snapshot-restore đơn giản (op không đảo được) → **STOP, DISCOVERY_REPORT + escalate** (xem Escape hatch trong Luật chơi). Đừng force engine bẩn. **V2:** O1.1 (ManagedOperation thiếu content) đã resolve bằng narrow additive — KHÔNG còn là escape-hatch trigger.

### Pre-phiếu snapshot (Worker auto first-step)

Chuẩn per template — snapshot `.backup/${PHIEU_ID}/` trước edit.

---

## Debate Log

> Schema: 1 turn = Worker Challenge + Architect Response. Cap = 3 turns. Append-only.

**Phiếu version:** V2 (Turn 1 resolved — O1.1 ACCEPT / Alt A)

### Turn 1 — Worker Challenge

**Anchor verification (recap Task 0):**
- Anchor #1: ⚠️ `Adapter` trait live `adapter.rs`, nhưng `ManagedOperation` placeholder (`adapter.rs:42-46`) chỉ `{description: String, target_path: String}` — thiếu content.
- Anchor #7: ⚠️ `.sos-adopt-incoming` (`adopt.rs:216-248`) là presence-only staging, KHÔNG hash → hash-discrimination d2 là logic mới.

**Objections (Tầng 1 only):**
- **[O1.1] CRITICAL:** `ManagedOperation` (`crates/sos-core/src/adapter.rs:42-46`, d1 placeholder) chỉ có `{description, target_path}` — KHÔNG field content/bytes, không link Asset identity. `render()` trả `Artifact{target_path, content}` riêng nhưng `Plan`/`ManagedOperation` KHÔNG nói engine render asset NÀO cho mỗi op → engine KHÔNG có cách trait-pure lấy bytes để write/hash/record vào `.sos-manifest.toml`. Block Task 1 (additive/non-clobber/rollback đều cần content). Nhưng `adapter.rs` đang ở Files-KHÔNG-sửa. Rollback snapshot-restore ĐỦ (escape-hatch b KHÔNG fire); non-clobber hash `ManagedManifest.content_hash` ĐỦ (MockAdapter pre-seed manifest simulate customized).

**Status:** ✅ RESOLVED — see Architect Response

### Turn 1 — Architect Response (phiếu V2)

- **[O1.1] → ACCEPT (Alt A).** Thêm field `content: String` vào `ManagedOperation` trong `crates/sos-core/src/adapter.rs` — **narrow additive**: Adapter 5-method trait + ManagedManifest 6-field GIỮ NGUYÊN, chỉ placeholder struct `ManagedOperation` thêm 1 field.
  - **Doctrine:** `ManagedOperation` là placeholder d1 tạo *cho engine consume*; d2 là first-consumer lộ ra nó thiếu content. `PORTABILITY_ARCHITECTURE.md` "plan → managed file operations" ngụ ý operation phải carry đủ để engine apply. Field `content` host-neutral (bytes + path, không host token) → không phá "agnostic". Alt B (description-as-key convention) **REJECT** — reintroduce host-ish indirection vào engine agnostic. `content: String` (op luôn có bytes trong d2 scope; nếu Worker gặp op không-content như DELETE thì `Option<String>` — Tầng-2 self-decide, note Discovery).
  - **Action:** adapter.rs bỏ khỏi Files-KHÔNG-sửa, thêm vào Files-cần-sửa (narrow row); Scope + Ảnh hưởng + Luật #6 amend; Docs-Gate note `PORTABILITY_ARCHITECTURE.md` + `docs/discoveries/P077d1.md` (note d2 amend, KHÔNG rewrite discovery); dep_direction confirm green (`content: String` plain type, no host token).
- **Rollback approach → DEFEND.** Snapshot-restore internal engine (copy pre-bytes trước overwrite; delete-on-rollback cho CREATE; restore snapshot cho UPDATE; no-op CONFLICT) đúng như phiếu, độc lập trait. Escape-hatch b KHÔNG fire — confirm Worker.
- **Anchor #7 → ACCEPT clarify.** Cập nhật ghi rõ `.sos-adopt-incoming` presence-only; d2 hash-discrimination = logic mới, chỉ mirror pattern ghi-incoming-copy.

**Status:** ✅ RESPONDED — phiếu bumped to V2. Ready for Worker CHALLENGE re-verify (consensus) → Chủ nhà approval.

### Turn 2 — Worker EXECUTE (V2 accepted, amendment applied)

Bước 0 re-verify: `content: String` on `ManagedOperation` is **sufficient** — engine computes `sha256(op.content)`, resolves against on-disk state + manifest record, writes exactly `op.content`'s bytes. No asset-identity indirection needed (content is self-contained per operation) — no further gap surfaced, no re-escalation.

Shipped per V2 (see Nhiệm vụ below + `docs/discoveries/P077d2.md` for full report): install engine (`crates/sos-install/src/engine.rs`), `MockAdapter` + 5 correctness fixtures (`crates/sos-install/tests/install.rs`), `sos install --runtime <r> [--dry-run]` wired in `sos-cli`. One Tầng-2 self-decide found mid-EXECUTE (not an escalation): the phiếu's 3-way non-clobber split (`Create`/`Update`/`Conflict`) broke idempotence — a target already at desired content still classified `Update`, rewriting identical bytes and bumping `rollback_ref` on every re-run. Added a 4th `Decision::NoOp` variant (on-disk hash == desired hash → zero mutation, zero manifest touch) — internal engine logic only, no trait/schema/CLI-surface change.

Nghiệm thu, all green: `cargo build --workspace` clean; `cargo test --workspace` green (8 pre-existing `parity.rs` + 2 manifest + 1 adapter-trait-shape + dep_direction guard + 6 new install fixtures); `cargo clippy --workspace` — only the pre-existing `sync.rs:102` warning; **5 install fixtures PASS** (additive/non-clobber×2/rollback/idempotence/dry-run), hard-fail, rollback triggered via a genuine io error (no fail-injection hook); **`cargo test --workspace` × 20 TRUE-parallel (`xargs -P 20`) — 0 flaky**; `git diff bin/sos.sh install.sh` empty; `dep_direction.rs` guard green; smoke `sos install --runtime claude --dry-run` exit 0, empty plan (ClaudeAdapter still d1-stub), zero mutation; trust-gate exit 0.

**Status:** ✅ SHIPPED.

### Final consensus
- Phiếu version: V2 · Total turns: 2 · Approved by Chủ nhà: delegated 2026-07-22 (sprint self-approve — amendment was Worker's own O1.1 recommendation, Architect ACCEPT Alt A)

---

## Nhiệm vụ

### Task 0.5 (V2 NEW): Narrow additive — `ManagedOperation.content`

**File:** `crates/sos-core/src/adapter.rs` (`ManagedOperation` struct, `[needs Worker verify]` L42-46 per Worker CHALLENGE).

**Thêm:** field `content: String` vào `ManagedOperation`. GIỮ NGUYÊN `{description, target_path}` sẵn có. **KHÔNG** đụng `Adapter` trait (5-method) hay `ManagedManifest` (6-field).

**Lưu ý:** Đây là narrow additive để engine lấy được bytes trait-pure (O1.1). Plain `String`, KHÔNG host token → dep_direction guard vẫn xanh. Nếu Worker gặp op không-content (e.g. DELETE) trong MockAdapter → cân nhắc `content: Option<String>` (Tầng-2 self-decide, note Discovery). MockAdapter (Task 4 fixture) construct `ManagedOperation` với `content` bytes deterministic.

### Task 1: Install engine — transaction executor trong `sos-install`

**File:** `crates/sos-install/src/` (module mới, tên Worker chốt — e.g. `engine.rs`; `[needs Worker verify]` crate hiện có module nào)

**Thêm:** Một engine execute một transaction plan bám `PORTABILITY_ARCHITECTURE.md:109-117` (step 1-4 + 6-7; step 5 = seam d3):

1. **Input = một `Plan`** lấy từ `adapter.plan(core, project)` (d1 trait), mỗi `ManagedOperation` carry `content` (Task 0.5). Engine KHÔNG tự phát sinh operations — nó chỉ EXECUTE plan của adapter (đây là separation: engine agnostic với runtime).
2. **Resolve exact targets + record rollback point** TRƯỚC mutation (POLICY:80). Với mỗi managed-file op: tính target path + content-hash mong muốn (hash của `op.content`).
3. **Non-clobber discrimination** (POLICY:82-83, PORTABILITY:81-83) — cho mỗi target đã tồn tại:
   - target absent → **CREATE** (additive).
   - target present + hash khớp `content_hash` đã ghi trong `.sos-manifest.toml` (unmodified since last install) → **UPDATE cho phép**.
   - target present + hash KHÁC recorded (user-customized) HOẶC không có manifest record → **CONFLICT**: KHÔNG overwrite; ghi incoming copy ra `.sos-install-incoming/<relpath>` (mirror pattern *ghi incoming copy* của `.sos-adopt-incoming` — anchor #7; **discrimination logic là MỚI, KHÔNG reuse presence-only staging của adopt**) + list vào report.
4. **Rollback record:** mỗi op mutate ghi backup (overwritten file content / created-path marker) vào rollback log. Nếu BẤT KỲ op fail giữa chừng → engine **restore về pre-transaction state** (xóa file đã create, restore file đã overwrite), manifest KHÔNG commit, exit non-zero LOUD (POLICY:80,84-85; PORTABILITY:117 "lỗi required làm install fail rõ ràng và rollback"). Snapshot-restore internal, độc lập trait.
5. **Commit manifest:** thành công → ghi `.sos-manifest.toml` (step 6) — collection các `ManagedManifest` entry (d1 schema, `content_hash` = hash của `op.content`). Xem Task 3.
6. **Step 5 seam (d3):** một hàm `resolve_tools()` **stub no-op** trả về empty + comment `// SEAM P077d3 (OA-07): tool-manifest resolve — no-op until d3`. Engine gọi nó ở đúng vị trí step-5 nhưng KHÔNG phụ thuộc kết quả. KHÔNG assert.
7. **`--dry-run`:** compute + in transaction plan (would-CREATE / would-UPDATE / would-CONFLICT per target) → **ZERO filesystem mutation** (PORTABILITY:84, POLICY: additive-first). Không ghi target, không ghi `.sos-manifest.toml`, không ghi `.sos-install-incoming`.

**Lưu ý:** Engine drive THUẦN qua `Adapter` trait — KHÔNG import host token (`.claude`, `CLAUDE_*`, Codex) vào `sos-install` (giữ nguyên tinh thần dep-direction; `sos-install` có thể thấy `sos-core` nhưng đừng nhét runtime-cụ-thể). Idempotence là hệ quả của rule 3 (re-run: mọi target hash khớp recorded → toàn no-op, manifest unchanged).

### Task 2: Wire `sos install --runtime <r>` trong `sos-cli`

**File:** `crates/sos-cli/src/` (subcommand dispatch — `[needs Worker verify]` file: `main.rs`/`commands/mod.rs`)

**Thêm:** Subcommand `install` NEW, flags `--runtime <auto|claude|codex|claude,codex>` + `--dry-run`. Dispatch:
- Parse `--runtime` → construct adapter(s). `claude` → `ClaudeAdapter` (d1). `auto` → detect (dùng `adapter.detect()`); nếu detect chưa impl đủ, `auto` fallback `claude` cho d2 smoke (`[needs Worker verify]` detect state). `codex` → KHÔNG có adapter (P078) → error rõ "codex adapter not yet available".
- Feed adapter vào engine (Task 1), chạy transaction.
- `--dry-run` → engine dry-run path.

**Lưu ý:** `sos install --runtime claude` chạy end-to-end nhưng vì `ClaudeAdapter.plan()` còn STUB (d1) → plan tối thiểu/rỗng, render KHÔNG tạo `.claude/**` thật. Đây ĐÚNG scope d2 (engine, không phải Claude render). Smoke đủ: `sos install --runtime claude --dry-run` in ra plan (dù minimal) + ZERO mutation, exit 0. **KHÔNG** impl real Claude asset rendering ở đây.

### Task 3: `.sos-manifest.toml` on-disk artifact

**File:** engine write path (Task 1 step 5) + serde type.

**CHỐT tên artifact:** `.sos-manifest.toml` tại **project root** (concrete-hóa PORTABILITY step 6 "Ghi managed manifest/state"). Format: array-of-tables, mỗi `[[managed]]` = một `ManagedManifest` entry (d1 schema, 6 field). Chọn TOML nhất quán d1 (`ManagedManifest` đã TOML round-trip — `discoveries/P077d1.md:20`) + `sos-core` đã dep `toml` cho `state.toml`.

**Lưu ý (`[needs Worker verify]` — Tầng-2 co-location decide):** nếu `crates/sos-core/src/state.rs` đã ghi một `.sos-state/` dir, Worker có thể co-locate manifest dưới đó thay vì root — **NHƯNG tên gốc `sos-manifest` GIỮ**, và `uninstall` (d1) + d3 phải đọc được cùng path. Vì đây là contract surface (uninstall/d3 consumer), việc concrete-hóa path/name → **DOCS-GATE update `PORTABILITY_ARCHITECTURE.md`** (xem Docs Gate). Nếu đổi từ root sang `.sos-state/` → note trong Discovery.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-core/src/adapter.rs` | **V2 NARROW (Task 0.5):** thêm field `content: String` vào `ManagedOperation` placeholder. GIỮ Adapter 5-method trait + ManagedManifest 6-field. |
| `crates/sos-install/src/**` | Task 1: install engine (transaction / non-clobber / rollback / dry-run / step-5 seam) |
| `crates/sos-cli/src/**` | Task 2: wire `sos install --runtime <r>` subcommand (additive dispatch) |
| `crates/sos-install/` (Cargo.toml nếu cần dep) | serde/toml/sha nếu engine cần (dùng workspace dep sẵn có — tránh crate mới) |
| engine correctness fixture (new test file, e.g. `crates/sos-install/tests/install.rs`) | 5 fixture: additive / non-clobber / rollback / idempotence / dry-run + MockAdapter (construct `ManagedOperation` với `content`) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `bin/sos.sh`, `install.sh` | `git diff` EMPTY — additive proof, distribution default KHÔNG flip |
| `crates/sos-core/src/adapter.rs` — **Adapter trait 5-method** | Trait signature (detect/plan/render/verify/uninstall) KHÔNG đổi. CHỈ `ManagedOperation` struct thêm field `content` (Task 0.5) — không đụng trait. |
| `crates/sos-core/src/{manifest,state}.rs` | Consume d1 schema — KHÔNG đổi `ManagedManifest` 6-field hay state format |
| `crates/sos-adapter-claude/src/lib.rs` | Render bodies GIỮ stub — d2 KHÔNG impl real `.claude/**` render |
| `crates/sos-cli/tests/parity.rs` + `tests/golden/**` | Mọi golden byte-identical — d2 không đụng parity oracle |

---

## Luật chơi (Constraints)

1. **Additive tuyệt đối** — `git diff bin/sos.sh install.sh` phải empty. KHÔNG flip distribution default (không có deliverable nào đổi `install.sh` → **no escalate** per decomposition L55). Nếu Worker thấy BUỘC phải đổi `install.sh` default để d2 chạy → STOP, escalate Chủ nhà (founder eyeball trigger).
2. **Engine driven thuần qua `Adapter` trait** — `sos-install` KHÔNG chứa host token runtime-cụ-thể. Engine đúng độc-lập-adapter (đó là lý do oracle dùng MockAdapter).
3. **Safe-mutation POLICY:78-86 là luật cứng** — additive/non-clobber default, never silently overwrite user-customized, backup+rollback proportional, failed mutation → restore pre-state + fail LOUD. Dry-run = ZERO mutation.
4. **Fixtures dùng collision-safe `TempFixture`** (AtomicU64-keyed, anchor #8) + deterministic dưới parallel. `cargo test --workspace` ×20 parallel = **0 flaky** (chuẩn chung post-c6). KHÔNG dùng pid+nanos-only.
5. **Step 5 (tool-resolve) = stub no-op seam** — đánh dấu `// SEAM P077d3`. KHÔNG impl tool download/pin ở đây (đó là d3/OA-07).
6. **`crates/sos-core` — NARROW additive ONLY (V2).** Được phép: thêm field `content` vào `ManagedOperation` placeholder (Task 0.5). **KHÔNG đụng:** Adapter trait 5-method signature, `ManagedManifest` 6-field, state format. KHÔNG đụng `parity.rs` golden. Bất kỳ đổi sos-core NGOÀI narrow `ManagedOperation.content` → STOP, escalate.
7. **ESCAPE HATCH:** nếu (a) engine-vs-adapter separation KHÔNG sạch (engine cần host knowledge → trait abstraction thủng — **NGOÀI** narrow `ManagedOperation.content` đã ACCEPT ở V2), hoặc (b) rollback semantics phức tạp hơn snapshot-restore (op không đảo ngược được an toàn, cần 2-phase / staging dir) → **KHÔNG force**. Viết `docs/discoveries/P077d2.md` mô tả gap + đề xuất (e.g. tách rollback thành sub-phiếu, hoặc staging-then-atomic-swap), escalate Chủ nhà qua orchestrator. Ship-bẩn engine mutate fs = rủi ro data-loss > trễ.

---

## Nghiệm thu

### Automated
- [ ] `cargo build --workspace` clean.
- [ ] `cargo clippy --workspace` — no NEW warning (pre-existing `sync.rs:102` OK).
- [ ] **Install correctness fixtures — `[oracle: install correctness fixtures — additive/non-clobber/rollback/idempotence/dry-run, hard-fail]`** — 5 fixture PASS:
  - **additive** (greenfield trống): engine execute MockAdapter plan (≥3 managed file, mỗi op có `content`) → mọi file tạo đúng target path + content-hash; `.sos-manifest.toml` ghi 1 entry/file (rollback_ref=None); exit 0.
  - **non-clobber**: (i) target user-customized (hash ≠ recorded / no record) → **KHÔNG overwrite**, original sha256 unchanged, incoming copy ở `.sos-install-incoming/<path>` byte-match `op.content`, conflict listed; (ii) target hash == recorded (unmodified) + kit content mới → **UPDATE cho phép** (không treat conflict) — chứng minh hash-discrimination (logic mới, không phải adopt presence-only).
  - **rollback**: MockAdapter plan fail ở op k → filesystem restore **byte-identical** pre-transaction snapshot (file op<k đã tạo bị xóa, file overwritten restore), `.sos-manifest.toml` KHÔNG commit; exit non-zero + error rõ.
  - **idempotence**: run install ×2 → run 2 mọi target hash khớp recorded → **no-op**, `.sos-manifest.toml` byte-unchanged, không file nào rewrite; exit 0.
  - **dry-run** (`--dry-run` greenfield): in plan (would-CREATE/UPDATE/CONFLICT) non-empty → **ZERO mutation** (dir snapshot before==after, no target, no manifest, no incoming); exit 0.
- [ ] **`cargo test --workspace` ×20 parallel = 0 flaky** (chuẩn chung post-c6; fixtures collision-safe TempFixture).
- [ ] Smoke: `sos install --runtime claude --dry-run` exit 0, in plan, ZERO mutation (ClaudeAdapter stub → plan minimal OK).

### Manual Testing
- [ ] `sos install --runtime codex` → error rõ "codex adapter not yet available" (P078), không panic.
- [ ] `sos install` (không `--dry-run`) trên temp greenfield → tạo asset + `.sos-manifest.toml`, re-run = no-op.

### Regression
- [ ] `git diff bin/sos.sh install.sh` — **EMPTY**.
- [ ] `crates/sos-cli/tests/parity.rs` 8/8 + goldens byte-identical (d2 không đụng).
- [ ] **`dep_direction.rs` guard green** — confirm `ManagedOperation.content: String` là plain type (bytes), KHÔNG host token → `sos-core` vẫn host-neutral, guard xanh (V2 narrow additive không phá dep-direction).
- [ ] `bash scripts/trust-gate.sh` exit 0 (`bootstrap/sos-rs/**` ngoài `.sos-trust-baseline` scope — precedent P077c5/d1).

### Docs Gate (Tầng 1)
- [ ] `CHANGELOG.md` — entry `[P077d2]` dưới v2.3 forge.
- [ ] `bootstrap/sos-rs/README.md` — cập nhật status `sos-install` (skeleton → install engine LIVE; deviation-list item 3); ghi `sos install` command tồn tại (song song `install.sh`), ClaudeAdapter render vẫn stub (d2 = engine only).
- [ ] `docs/PORTABILITY_ARCHITECTURE.md` — concrete-hóa: manifest artifact = `.sos-manifest.toml` (path/name), step-5 = d3 seam, "P077d2 status" line (mirror P077d1 pattern). **V2:** note `ManagedOperation` giờ carry `content` (nếu doc mô tả ManagedOperation shape / "plan → managed file operations").
- [ ] `docs/discoveries/P077d1.md` — **note (KHÔNG rewrite)** d2 amend: `ManagedOperation` thêm field `content` (first-consumer engine lộ placeholder thiếu bytes; narrow additive, trait 5-method + manifest 6-field giữ nguyên).
- [ ] `bootstrap/sos-rs/crates/sos-cli/tests/README.md` — thêm section "install correctness oracle" (5 fixture + MockAdapter, khác parity-oracle: no Bash counterpart).
- [ ] `docs/plans/P077d-decomposition.md` — status line: P077d2 SHIPPED.

### Discovery Report
- [ ] `docs/discoveries/P077d2.md`:
  - Anchors #1-8 CORRECT/WRONG (file:line).
  - **`ManagedOperation.content` narrow additive: final type (`String` vs `Option<String>`) + lý do** (op không-content xuất hiện?).
  - Engine-vs-adapter separation: có sạch không (trait + `ManagedOperation.content` đủ drive engine?) — evidence.
  - Rollback semantics: snapshot-restore đủ hay cần staging? (escape-hatch có fire không).
  - Manifest artifact final path (root `.sos-manifest.toml` hay co-locate `.sos-state/`) + lý do.
  - MockAdapter design + 5 fixture assert thật đã chạy (paste ×20 parallel result).
  - dep_direction guard result (confirm green sau khi thêm `content`).
  - Docs updated (list) / Tier escalations (None nếu không).
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`.
