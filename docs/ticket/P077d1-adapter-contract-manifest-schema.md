# PHIẾU P077d1: Adapter contract trait + managed-manifest schema (foundation)

> **Loại:** Feature
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — adapter contract + generated-artifact schema; sai thì LAN sang install engine d2, Claude adapter P076-declarative, Codex P078. Contract surface → AUTO Tầng 1)
> **Lane:** Guarded
> **Ảnh hưởng:** `bootstrap/sos-rs/crates/sos-core/**`, `bootstrap/sos-rs/crates/sos-adapter-claude/**` (trait impl stub)
> **Dependency:** P077c CLOSED (c1–c5 shipped). Đầu chuỗi P077d — xem `docs/plans/P077d-decomposition.md`.

---

## Context

### Vấn đề hiện tại

`sos-install`, `sos-adapter-claude`, `sos-hooks` là skeleton RỖNG (`bootstrap/sos-rs/README.md:22-34`: "logic lands in P077d"). Chưa có:
- **Adapter contract trait** — mỗi runtime adapter phải implement cùng contract tối thiểu (`docs/PORTABILITY_ARCHITECTURE.md:63-69`): `detect / plan / render / verify / uninstall`. Hiện `sos-adapter-claude` chỉ là stub không ràng buộc bởi trait nào.
- **Managed-manifest schema** — install/sync ghi manifest cho mỗi generated artifact (`core/ASSETS.md:57-64`): owner / source-version / source-identity / target-path / content-hash / rollback-ref. Chưa có struct nào biểu diễn.

Không có 2 abstraction này thì d2 (install engine) không có contract để build lên, và P078 (Codex) không có trait để implement — sẽ refactor chồng.

### Giải pháp

Carve **foundation abstraction** RIÊNG, trước khi impl engine (như P077b carve crate boundary trước c1–c4):

1. Định nghĩa Rust trait `Adapter` (tên `[needs Worker verify]`) 5 method đúng `PORTABILITY_ARCHITECTURE.md:63-69`, cùng associated/param types runtime-neutral (Capabilities, ManagedOperation/Plan, Artifact, Findings, RemovalPlan).
2. Định nghĩa `ManagedManifest` schema struct (6 field `core/ASSETS.md:57-64`) + serde round-trip.
3. `sos-adapter-claude` implement trait với **stub bodies** (`todo!()`/minimal — logic thật ở d2). Mục đích: chứng minh trait implementable + giữ dependency-direction (adapter→core).

**KHÔNG** làm ở d1: install engine, filesystem mutation, `sos install` command, tool-manifest (d3), render nội dung thật. Pure contract + schema carve.

### Scope
- CHỈ sửa: `sos-core` (thêm trait + manifest schema module), `sos-adapter-claude` (impl trait stub), Cargo.toml nếu cần thêm serde dep, tests.
- KHÔNG sửa: `bin/sos.sh`, `install.sh`, `sos-install` engine logic (d2), `sos-hooks`, bất kỳ file P077c parity fixture.

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `sos-install`/`sos-adapter-claude`/`sos-hooks` là skeleton rỗng, logic để P077d | `grep -rn "P077d\|skeleton\|todo" bootstrap/sos-rs/crates/sos-adapter-claude/src/` | ⏳ TO VERIFY `[needs Worker verify]` |
| 2 | 5-method contract = `detect/plan/render/verify/uninstall` | `docs/PORTABILITY_ARCHITECTURE.md:63-69` | ✅ `[verified]` — Architect đọc, dòng 63-69 |
| 3 | Managed-manifest 6 field = owner / source-version / source-identity / target-path / content-hash / rollback-ref | `core/ASSETS.md:57-64` | ✅ `[verified]` — Architect đọc, dòng 57-64 |
| 4 | Dep-direction: `sos-adapter-claude` deps CHỈ `sos-core`; core deps zero adapter | `grep -A20 dependencies bootstrap/sos-rs/crates/sos-adapter-claude/Cargo.toml` + `crates/sos-core/tests/dep_direction.rs` | ⏳ TO VERIFY `[needs Worker verify]` — README:27-29 claims guard test exists `[unverified]` |
| 5 | Trait NÊN sống ở `sos-core` (để `sos-adapter-claude` — deps chỉ core — implement được) | Xác nhận adapter chỉ deps core → trait phải ở core | ⏳ TO VERIFY `[needs Worker verify]` — nếu adapter cũng deps `sos-install`, trait có thể ở install; Worker đọc Cargo.toml chốt |
| 6 | Dep-direction guard test file tồn tại | `ls bootstrap/sos-rs/crates/sos-core/tests/dep_direction.rs` | ⏳ TO VERIFY `[needs Worker verify]` — README:29 refs nó |
| 7 | `install.sh` (bootstrap 175 dòng) KHÔNG cần đổi cho d1 | `wc -l install.sh` + confirm d1 zero-touch | ✅ `[verified]` — d1 pure core/adapter carve, install.sh out of scope |
| 8 | serde/serde-derive có sẵn trong workspace deps (cho manifest round-trip) | `grep -rn "serde" bootstrap/sos-rs/Cargo.toml bootstrap/sos-rs/crates/sos-core/Cargo.toml` | ⏳ TO VERIFY `[needs Worker verify]` — nếu thiếu, thêm; c1–c5 map/adopt ghi TOML/YAML nên khả năng đã có `[unverified]` |

**⚠️ Anchors 1,4,5,6,8 = `[needs Worker verify]`** — Architect docs-only không đọc được crate source/Cargo.toml. Worker grep xác nhận TRƯỚC khi impl; nếu anchor 5 sai (adapter deps install) → chọn crate placement khác + ghi Discovery.

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Schema: 1 turn = Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Anchor verification (recap from Task 0):**
- Anchor 1 (skeletons empty): ✅ `sos-adapter-claude/src/lib.rs` = 6-line comment stub proving dep-graph only; `sos-core/src/` has only `lib.rs`+`state.rs` (no adapter/manifest module). `sos-install/src/lib.rs`, `sos-hooks/src/lib.rs` likewise skeleton.
- Anchor 4 (dep-direction): ✅ `crates/sos-adapter-claude/Cargo.toml` deps = `sos-core` ONLY. `crates/sos-core/Cargo.toml` deps = anyhow/serde/toml/chrono, ZERO sos-* crate.
- Anchor 5 (trait placement = sos-core): ✅ confirmed by anchor 4 — adapter deps only core, so trait must live in core for adapter to implement it.
- Anchor 6 (dep_direction guard exists): ✅ `crates/sos-core/tests/dep_direction.rs` exists, 2-layer (compiler graph + forbidden-token scan of `sos-core/src/**` for `sos_adapter|sos_install|sos_hooks|sos_cli`). Adding a trait module inside `sos-core/src/` does not touch adapter tokens — guard stays green as long as new module doesn't import adapter crates (it won't, per constraint).
- Anchor 8 (serde available): ✅ root `Cargo.toml:10` workspace serde = `{ version = "1", features = ["derive"] }`; `sos-core/Cargo.toml:10` already has `serde = { workspace = true }` — derive feature inherited, no new dep needed.

**Crate placement + dep-direction (CRITICAL):** Confirmed correct — trait belongs in `sos-core`. As long as Task 1's placeholder types (`Capabilities`/`Plan`/`Artifact`/`Findings`/`RemovalPlan`/`ManagedManifest`) are plain host-neutral structs/enums (no `CLAUDE_*` token, no Claude/Codex path literal), the dep_direction guard's forbidden-token scan (`sos_adapter|sos_install|sos_hooks|sos_cli`) is unaffected by trait addition — it only scans for crate-name import tokens, not semantic content, so the guard mechanically cannot regress from Task 1/2 alone. Task 3 (`sos-adapter-claude` implementing the trait) is the correct/expected direction (adapter imports `sos_core::...`) — this is exactly what the guard permits.

**Manifest schema:** 6 fields as spec'd (owner/source-version/source-identity/target-path/content-hash/rollback-ref) match `core/ASSETS.md:57-64` verbatim ("owning integration or portable component; source product version; source identity; target path; installed content hash; previous-state or rollback reference when mutation occurred"). Sufficient for d2 (rollback-ref + content-hash cover non-clobber/rollback) and d3 (source-version covers tool-version pin, though d3's tool-manifest.toml is a separate manifest concept — external tool pins, not generated-asset manifest — no overlap risk). Serde format not specified by phiếu; `sos-core` already depends on `toml` crate (state.toml precedent) — Worker EXECUTE should default to TOML unless a reason emerges for JSON; this is Tầng-2 (param-plumbing-class decision per phiếu Lưu ý), not an objection.

**Trait generic (Claude+Codex):** Phiếu's own spec text keeps method signatures as commented placeholders (`/* project */`, `/* asset */`) explicitly deferring exact shape to Worker EXECUTE — this is correctly conservative. No existing code exists yet to check for Claude-only baking (trait doesn't exist). The constraint list (Luật chơi #2 "core zero runtime token") is the correct mechanism to prevent Claude-only leakage; nothing in Task 0 evidence contradicts feasibility — `PORTABILITY_ARCHITECTURE.md:63-69` states the same 5-method contract for "mỗi adapter" generically (Claude adapter section is separate, describing ownership not trait shape).

**Objections:** none — accept V1.

**Status:** ✅ WORKER ACCEPTED — ready for Chủ nhà approval gate.

### Final consensus
- Phiếu version: V<N>
- Approved by Chủ nhà: [date]

---

## Nhiệm vụ

### Task 1: Định nghĩa adapter contract trait trong `sos-core`

**File:** `bootstrap/sos-rs/crates/sos-core/src/**` — module mới (tên `adapter.rs` hoặc `contract.rs`, `[needs Worker verify]` naming convention crate hiện dùng).

**Thêm:** Rust trait 5 method đúng `PORTABILITY_ARCHITECTURE.md:63-69`:

```rust
// Tên type/trait cụ thể [needs Worker verify] — giữ đúng SHAPE 5 method:
pub trait Adapter {
    fn detect(&self /* , project */) -> Capabilities;
    fn plan(&self, /* core, project */) -> Plan;          // managed file/tool/hook operations
    fn render(&self, /* asset, */ caps: &Capabilities) -> Artifact; // runtime-native artifact
    fn verify(&self /* , project */) -> Findings;
    fn uninstall(&self, manifest: &ManagedManifest) -> RemovalPlan; // safe removal plan
}
```

**Lưu ý:**
- Associated/param types (`Capabilities`, `Plan`/`ManagedOperation`, `Artifact`, `Findings`, `RemovalPlan`) = runtime-neutral **placeholder** types (struct/enum tối thiểu — đủ để trait compile + implementable). KHÔNG nhồi Claude/Codex token vào core (core zero runtime token, `core/POLICY.md` "Portable core" §54-57). Bodies thật = d2.
- Exact signatures (owned vs borrow, project param type) = design detail Worker chốt lúc EXECUTE — giữ đúng **5 method + đúng ý nghĩa return** là hard constraint; param plumbing là Tầng-2 Worker self-decide.
- Trait ĐẶT ở `sos-core` VÌ `sos-adapter-claude` deps chỉ `sos-core` (anchor 4-5). Nếu Worker verify thấy adapter cũng deps `sos-install` → có thể đặt trait ở `sos-install`; ghi Discovery lý do.

### Task 2: Định nghĩa `ManagedManifest` schema + serde round-trip

**File:** `bootstrap/sos-rs/crates/sos-core/src/**` (cùng module hoặc `manifest.rs`, `[needs Worker verify]`).

**Thêm:** struct 6 field đúng `core/ASSETS.md:57-64` — mỗi managed artifact record:

```rust
// Field names cụ thể [needs Worker verify] — giữ đúng 6 SEMANTIC field:
pub struct ManagedManifest {
    owner: /* owning integration or portable component */,
    source_version: /* source product version */,
    source_identity: /* source identity */,
    target_path: /* target path */,
    content_hash: /* installed content hash */,
    rollback_ref: Option</* previous-state / rollback reference when mutation occurred */>,
}
```

**Lưu ý:**
- `#[derive(Serialize, Deserialize)]` (serde — anchor 8; thêm dep nếu thiếu).
- `rollback_ref` là `Option` — chỉ set khi mutation xảy ra (`core/ASSETS.md:64` "when mutation occurred").
- Schema là data-only, KHÔNG có install logic ở d1 (apply/rollback = d2).

### Task 3: `sos-adapter-claude` implement trait với stub bodies

**File:** `bootstrap/sos-rs/crates/sos-adapter-claude/src/**`

**Thêm:** `impl Adapter for ClaudeAdapter` (tên struct `[needs Worker verify]`) với 5 method body = `todo!("d2")` hoặc minimal-return.

**Lưu ý:**
- Mục đích DUY NHẤT: chứng minh trait implementable + giữ dependency-direction (adapter deps core, impl core-defined trait). KHÔNG render Claude asset thật (d2).
- Dep-direction guard (`crates/sos-core/tests/dep_direction.rs`, anchor 6) PHẢI vẫn xanh — core không được import adapter.

### Task 4: Unit test — trait-shape + schema round-trip

**File:** `bootstrap/sos-rs/crates/sos-core/tests/**` hoặc `#[cfg(test)]` inline (`[needs Worker verify]` convention).

**Thêm:**
- Test serde round-trip `ManagedManifest` (serialize → deserialize → equal).
- Test `ClaudeAdapter` satisfies `Adapter` trait bound (compile-level: `fn _assert(a: impl Adapter)` hoặc trait-object construct).

**Lưu ý:** Oracle của d1 = **compile + 2 test + dep-direction guard xanh**. KHÔNG có install fixture (đó là d2).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-core/src/**` (module mới) | Task 1: trait `Adapter` 5 method + placeholder types |
| `crates/sos-core/src/**` (module mới) | Task 2: `ManagedManifest` struct + serde derive |
| `crates/sos-core/src/lib.rs` | export module mới `[needs Worker verify]` |
| `crates/sos-adapter-claude/src/**` | Task 3: impl trait stub |
| `crates/sos-core/Cargo.toml` hoặc workspace | serde dep nếu thiếu (anchor 8) |
| `crates/sos-core/tests/**` | Task 4: round-trip + trait-bound test |
| `bootstrap/sos-rs/README.md` | Module-layout: đánh dấu adapter-contract + manifest schema DEFINED (d1), engine vẫn d2 |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `bin/sos.sh` | Zero-touch (canonical Bash không đổi P077a–d) |
| `install.sh` | Zero-touch (anchor 7) |
| `crates/sos-install/src/**` | Engine vẫn skeleton — d1 KHÔNG impl engine |
| `crates/sos-cli/tests/parity.rs` + `tests/golden/**` | P077c parity/correctness fixtures không đổi |
| `crates/sos-core/tests/dep_direction.rs` | Guard vẫn XANH sau khi thêm trait (core zero adapter dep) |

---

## Luật chơi (Constraints)

1. **Additive** — `bin/sos.sh` + `install.sh` KHÔNG đổi. `sos install` command CHƯA tồn tại sau d1 (đó là d2). d1 chỉ thêm types + trait + stub impl.
2. **Core zero runtime token** (`core/POLICY.md` §54-57) — trait + types runtime-neutral; KHÔNG `CLAUDE_*` / Claude/Codex path / permission serialization trong `sos-core`. Claude-specific = `sos-adapter-claude`.
3. **Dependency-direction bất di** — adapter→core one-way. Dep-direction guard test phải xanh.
4. **5 method + 6 field là hard SHAPE** — tên/signature/param-plumbing là Tầng-2 Worker self-decide, nhưng số method (5) + ý nghĩa return + số manifest field (6) + semantic mỗi field KHÔNG được đổi (đó là contract surface P076/P078 dựa vào).
5. **KHÔNG install logic** — apply/rollback/mutation/render-thật = d2. d1 chỉ định nghĩa hình dạng.
6. **Lane Guarded** — no-cap, nhưng scope kỷ luật: KHÔNG kéo engine/tool-manifest vào.

---

## Nghiệm thu

### Automated
- [ ] `cd bootstrap/sos-rs && cargo build --workspace` xanh
- [ ] `cargo test --workspace` xanh (bao gồm round-trip + trait-bound test mới + dep_direction guard vẫn pass)
- [ ] `cargo clippy --workspace` không warning mới (nếu repo enforce)

### Manual Testing
- [ ] `ManagedManifest` serde round-trip: serialize→deserialize→bằng nhau (test đỏ nếu bỏ 1 field → chứng minh 6 field enforced)
- [ ] `ClaudeAdapter` compile-satisfy `Adapter` trait bound

### Regression
- [ ] P077c parity/correctness fixtures (`parity.rs` 8 test) vẫn xanh — d1 không chạm
- [ ] Dep-direction guard xanh — core không import adapter sau khi thêm trait

### Docs Gate (Tầng 1)
- [ ] `bootstrap/sos-rs/README.md` — module layout: adapter contract + manifest schema DEFINED (d1), engine/install logic vẫn "lands in d2"
- [ ] `docs/plans/P077d-decomposition.md` — đánh dấu P077d1 SHIPPED (status line như P077c pattern)
- [ ] `docs/PORTABILITY_ARCHITECTURE.md` — nếu contract cụ-thể-hoá đáng ghi: thêm "P077d1 status" line (như P077b status §43-46) ghi trait/manifest đã render, crate placement thực tế. Nếu Worker thấy không đổi semantic doc → ghi "N/A" + lý do.
- [ ] `core/ASSETS.md` — dòng 32-33 refs "physical render P077" cho agents/skills; nếu d1 render generated-artifact manifest schema đáng note → cập nhật, else N/A explicit.
- [ ] `CHANGELOG.md` — entry P077d1

### Discovery Report
- [ ] Write `docs/discoveries/P077d1.md`
  - Anchors 1,4,5,6,8 — CORRECT/WRONG (file:line). ĐẶC BIỆT anchor 5: trait đặt `sos-core` hay `sos-install`? (crate placement thực tế + lý do)
  - Trait/manifest exact names chốt (Worker's naming choices)
  - serde dep: có sẵn hay phải thêm?
  - Docs updated (list) hoặc "None"
  - Tier escalations ("None" nếu không)
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`
