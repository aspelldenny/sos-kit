# P077 decomposition — Rust workspace + adapter/install framework

> **Status:** PROPOSAL — Architect draft, chưa mở phiếu (trừ P077a đã draft ở `docs/ticket/P077a-rust-workspace-scaffold-parity-oracle.md`).
> **Parent:** P077 row trong `docs/PORTABILITY_ARCHITECTURE.md` — "Rust workspace + adapter/install contract + manifest/dry-run/non-clobber/rollback/sync/doctor. Gate: Rust CLI là canonical; Bash oracle parity xanh; repo contract đổi sang runtime monorepo."
> **Dependency:** P076 merged (Claude adapter parity, `adapters/claude/{README,MAPPING}.md`).

## Tại sao phải chia

P077 gộp 4 việc độc lập-về-rủi-ro: (1) dựng workspace, (2) carve crate boundary, (3) đạt parity Rust↔Bash, (4) install framework + manifest, (5) cutover canonical + đổi repo contract. Gộp = một delivery unit khổng lồ, không có gate trung gian, và trộn cả bước **irreversible** (bỏ Bash oracle, flip repo contract) với các bước **additive/reversible**. Nguyên tắc OA-06: **freeze Bash làm golden oracle TRƯỚC**, Rust dựng song song, chỉ thành canonical khi mọi parity test xanh. Chia để mỗi bước có oracle + gate riêng và bước irreversible được cô lập ở cuối cho founder confirm.

## Bất biến xuyên suốt decomposition

- **Bash `bin/sos.sh` GIỮ canonical** từ P077a đến hết P077d. Rust dựng SONG SONG. Chỉ P077e cutover.
- P077a–P077d là **additive**: user-facing behavior (dùng `bin/sos.sh`) KHÔNG đổi. Chỉ P077e đổi entrypoint + repo contract.
- Repo-contract change (`CLAUDE.md` "Not a runtime binary source") đặt ở **P077e** (cutover), KHÔNG ở sub-phiếu đầu — vì contract chỉ đúng khi Rust workspace thật sự là canonical.
- Không giữ hai canonical engines sau parity (PORTABILITY_ARCHITECTURE non-goals + OA-06).
- **Không sub-phiếu nào cần Codex** — Codex adapter là P078, ngoài P077.

## Sub-phiếu đề xuất (5)

| ID | Deliverable | Gate mở khóa (→ đi tiếp) | Oracle | Additive? | Founder? |
|---|---|---|---|---|---|
| **P077a** | Rust workspace scaffold (transitional, root tại `bootstrap/sos-rs/`) + freeze Bash golden oracle (capture `new/adopt/map/sync` output) + parity-harness skeleton (chạy + báo "not yet parity", KHÔNG hard-fail) | `cargo build`+`cargo test` xanh; golden fixtures committed; harness chạy được và report diff dạng informational | Bash golden fixtures + `cargo build/test` | ✅ | ❌ (reversible) |
| **P077b** | Crate boundary carve: `sos-core` / `sos-cli` / `sos-install` / `sos-adapter-claude` / `sos-hooks` skeleton + dependency-direction rule (core KHÔNG import adapter; adapter→core only). Lift sos-rs source vào `crates/sos-cli`+`sos-core`. Relocate workspace root nếu quyết định (bootstrap→repo-root vẫn để P077e nếu chạm repo contract) | Workspace compiles; import-direction check xanh (adapter→core one-way); harness vẫn chạy | `cargo build` + import-direction lint | ✅ | ❌ (reversible) |
| **P077c** | Impl các Rust command còn thiếu tới parity: `new/adopt/map/sync` + `init security`. Parity-harness chuyển sang **hard-fail on diff**. Cover OA-06 list: dry-run plan, non-clobber conflict, sync provenance, hook collision, map generation, rollback, idempotence. Sửa OA-02 (scanner stack-aware, survey-before-install) trong Rust impl | Parity suite xanh cho MỌI subcommand vs Bash golden. Bash vẫn canonical (Rust chứng minh bằng nhau, chưa switch) | Parity harness (Rust output == Bash golden), hard-fail | ✅ | ❌ (Bash vẫn authoritative) |
| **P077d** | Install framework: transaction plan / dry-run / non-clobber / rollback record / sync / managed manifest + `doctor`. `tool-manifest.toml` pin version+asset+checksum (OA-07). Adapter contract `detect/plan/render/verify/uninstall`. `sos tools status` version-drift check | Install fixtures xanh (additive/non-clobber/rollback/idempotence); doctor fail-clear khi thiếu required; manifest pin verified. `sos install` mới chạy SONG SONG `install.sh` (chưa thay default) | Install fixtures + manifest verify | ✅ (nếu additive alongside `install.sh`) | ⚠️ optional (founder eyeball nếu đổi `install.sh` default = user-facing) |
| **P077e** | **CUTOVER**: Rust `sos` thành canonical entrypoint. Bash `bin/sos.sh` → thin launcher hoặc xóa. Đổi repo contract `CLAUDE.md` "Not a runtime binary source" → runtime monorepo. Relocate workspace lên repo-root Cargo.toml nếu chưa. Update `bootstrap/sos-rs/README.md` ownership (finding #5) nếu còn tồn tại | **P077 top-level gate**: Rust CLI canonical; full parity suite xanh; fixture matrix (OA-10) xanh; repo contract updated + docs-gate xanh | Full parity suite + fixture matrix | ❌ irreversible-ish (bỏ Bash oracle + flip contract) | ✅ **FOUNDER CONFIRM BẮT BUỘC** |

## Thứ tự + lý do

`P077a → P077b → P077c → P077d → P077e`, tuyến tính, KHÔNG parallel:

1. **P077a trước** — không có golden oracle thì mọi Rust dev sau đó drift mù (OA-06). Scaffold + freeze oracle là móng.
2. **P077b** — carve crate boundary trước khi impl để dependency direction (core⊥adapter) được enforce từ đầu, tránh refactor chồng ở P077c.
3. **P077c** — impl tới parity trên nền boundary đã sạch; harness hard-fail biến parity thành gate cơ học.
4. **P077d** — install framework build trên các command đã parity; manifest pin (OA-07) trước distribution.
5. **P077e cuối** — chỉ cutover khi 4 bước trên xanh. Đây là bước duy nhất irreversible → cô lập cho founder.

## Founder-decision points

- **P077e = hard founder confirm** (bắt buộc): bỏ Bash oracle + flip repo contract = irreversible-ish. Không auto-approve.
- **P077d = optional founder eyeball**: nếu deliverable đổi `install.sh` default behavior (user-facing distribution). Nếu chỉ thêm `sos install` song song → additive, không cần.
- P077a/b/c = additive + reversible → chạy theo APPROVAL_GATE thường, không cần founder đặc biệt.

## Ngoài scope P077 (đừng kéo vào)

- Codex adapter (P078), Codex dogfood (P079), dual-runtime brownfield (P080), packaging/npm wrapper (P081).
- OA-01 (Lane↔Tầng taxonomy), OA-03 (ship dirty-tree), OA-04 (docs-sync config) — các finding này có upgrade order riêng (audit §"Upgrade order"), không block P077 chuỗi.
