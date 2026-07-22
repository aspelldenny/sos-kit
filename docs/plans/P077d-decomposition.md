# P077d decomposition — Install framework + adapter contract + managed manifest + tool-manifest (OA-07)

> **Status:** P077d1 SHIPPED (2026-07-22) — `Adapter` trait (5 method) + `ManagedManifest` schema (6 field) carved in `sos-core`, `sos-adapter-claude` implements trait (stub bodies, zero fs mutation). d2 (install engine) / d3 (tool-manifest OA-07) chưa mở phiếu.
> **Parent:** P077d row trong `docs/plans/P077-decomposition.md` — "Install framework: transaction plan / dry-run / non-clobber / rollback record / sync / managed manifest + doctor. `tool-manifest.toml` pin version+asset+checksum (OA-07). Adapter contract detect/plan/render/verify/uninstall. `sos tools status` version-drift. Gate: install fixtures xanh; doctor fail-clear; manifest pin verified. **`sos install` mới chạy SONG SONG `install.sh` (chưa thay default).**"
> **Depends:** P077c CLOSED (c1–c5 shipped — `map/sync/new/adopt` parity/correctness enforced, OA-02 fixed). `sos-install`/`sos-adapter-claude`/`sos-hooks` vẫn là skeleton rỗng (`bootstrap/sos-rs/README.md` "logic lands in P077d").

## Điểm ĐẶC THÙ (khác P077c parity)

Không như P077c (mọi command có Bash counterpart để parity), P077d **KHÔNG có Bash `sos install` để parity** — `install.sh` là bootstrap KHÁC (175 dòng: tải binary + clone kit, `releases/latest` unpinned, checksum verify có sẵn P071/P073). `sos install` là command **MỚI** → oracle = **correctness fixtures** (additive/non-clobber/rollback/idempotence/dry-run), KHÔNG parity-vs-Bash. Đây là điểm phân biệt cốt lõi so P077c.

## Tại sao phải chia (lean TÁCH)

P077d gộp 3 domain rủi-ro-độc-lập với **3 oracle khác nhau**:

1. **Adapter contract trait + managed-manifest schema** — abstraction NỀN (trait `detect/plan/render/verify/uninstall` + manifest owner/version/hash/rollback). KHÔNG install engine, KHÔNG mutate filesystem. Oracle = compile + trait-shape/schema round-trip unit test. Foundation mà **Claude declarative (P076)** + **Codex (P078)** đều dựa vào — đáng carve + review RIÊNG như P077b carve crate boundary trước khi impl.
2. **Install engine** — plan/dry-run/non-clobber/rollback/apply trên kit+adapter assets, `sos install` additive alongside `install.sh`. Đơn vị **NẶNG NHẤT** (mutate fs, cần fixture harness như `adopt` ở c4). Oracle = install correctness fixtures.
3. **tool-manifest.toml (OA-07)** — pin external sister-tool version+asset+checksum + `sos tools status` drift-check + doctor fail-clear. Concern **TRỰC GIAO**: version-drift của binary NGOÀI (doctor/inv-gate/ship), KHÔNG phải install kit-asset VÀO project. Oracle = manifest-pin-verify + status-drift, KHÁC hẳn non-clobber của #2.

Gộp cả 3 = khối un-CHALLENGE-able (engine fixture một mình đã bằng `adopt.golden` 4.7k class), phá lane budget, và trộn foundation abstraction (irreversible-ish contract surface) với engine + external-tool concern. Chia để mỗi sub có **một oracle nhất quán** + gate riêng.

## Sub-phiếu (3) — thứ tự cứng

| ID | Deliverable | Oracle | Lane | Additive? | Founder? | Dep |
|---|---|---|---|---|---|---|
| **P077d1** | **Adapter contract trait** (`detect/plan/render/verify/uninstall`, 5 method, `PORTABILITY_ARCHITECTURE.md:63-69`) + **managed-manifest schema** (owner/source-version/source-identity/target-path/content-hash/rollback-ref, `core/ASSETS.md:57-64`) — runtime-neutral types. `sos-adapter-claude` implement trait (stub bodies OK). KHÔNG install engine, KHÔNG fs mutation, KHÔNG `sos install` command. Dependency-direction (adapter→core) GIỮ. | Compile + trait-shape/schema serde round-trip unit tests + dep-direction guard vẫn xanh | Guarded | ✅ | ❌ (reversible — contract carve) | P077c |
| **P077d2** | **Install engine**: transaction plan / `--dry-run` / non-clobber / rollback record / apply trên kit+adapter assets (`PORTABILITY_ARCHITECTURE.md:107-115` 7-step, TRỪ step 5 tool-resolve = d3). `sos install` NEW, chạy SONG SONG `install.sh` (install.sh + bin/sos.sh KHÔNG đổi). Consume d1's manifest schema + adapter trait. | **Install correctness fixtures** (additive / non-clobber / rollback / idempotence / dry-run), hard-fail. KHÔNG parity-vs-Bash. | Guarded | ✅ (alongside install.sh) | ❌ (additive) — ⚠️ optional eyeball CHỈ nếu đổi install.sh default | P077d1 |
| **P077d3** | **tool-manifest.toml (OA-07)**: pin version+platform-asset+checksum cho external tools (doctor/inv-gate/ship/docs-gate...) + `sos tools status` (expected vs installed drift) + `sos doctor` fail-clear khi required tool thiếu/version cũ. Fill d2's step-5 tool-resolve seam. | Manifest-pin-verify fixture + status-drift fixture + doctor fail-clear (exit rõ khi missing required) | Guarded | ✅ | ❌ (additive) | P077d2 |

## Thứ tự + lý do

`d1 → d2 → d3`, tuyến tính, KHÔNG parallel:

1. **d1 trước** — trait + schema là abstraction NỀN. Impl engine (d2) mà chưa có contract = refactor chồng. Carve foundation trước (như P077b carve boundary trước c1–c4 impl). d1 landed alone cho **P078 Codex** dựa vào một trait đã review + ổn định, không chờ cả engine.
2. **d2** — install engine build trên d1's trait + manifest schema. Nặng nhất, cần fixture harness. Để step-5 (tool-resolve) làm **seam stub** cho d3 fill.
3. **d3 CUỐI** — OA-07 external-tool pinning trực giao, layer lên sau khi core install (d2) chạy. Oracle version-drift riêng, không phá install-correctness gate của d2.

## Adapter-contract shape (CHỐT)

Đúng `PORTABILITY_ARCHITECTURE.md:63-69` — trait 5 method, runtime-neutral:

```text
detect() → capabilities
plan(core, project) → managed file/tool/hook operations
render(asset, capabilities) → runtime-native artifact
verify(project) → findings
uninstall(manifest) → safe removal plan
```

- Trait + associated types (Capabilities, ManagedOperation/Plan, Artifact, Findings, RemovalPlan, Manifest) = **runtime-neutral core contract**. Đặt nơi cả `sos-install` LẪN `sos-adapter-claude` thấy được mà core KHÔNG import adapter — ứng viên là `sos-core` (adapter→core direction: `sos-adapter-claude` deps chỉ `sos-core` per P077b, nên trait phải ở core để adapter implement). Manifest schema = GENERATED-artifact state (`core/ASSETS.md` "Generated assets") = core config/state schema. **Crate placement chính xác = `[needs Worker verify]` ở d1 EXECUTE** (Architect docs-only không đọc Cargo.toml deps thật).
- `sos-install` engine (d2) consume core trait + core manifest type; `sos-adapter-claude` implement trait. Core self-contained (không dep install/adapter).

## Founder-decision points

- **Mặc định KHÔNG escalate** — d1/d2/d3 đều additive (`install.sh` + `bin/sos.sh` KHÔNG đổi). APPROVAL_GATE thường.
- **ESCALATE trigger (d2 only, KHÔNG mặc định):** CHỈ nếu một deliverable BUỘC đổi `install.sh` default behavior (user-facing distribution flip) → orchestrator hỏi Sếp (founder eyeball). Trong decomposition này KHÔNG có deliverable nào đổi install.sh default → **no escalate**.

## Ngoài scope P077d (đừng kéo vào)

- Cutover canonical + flip repo contract (`CLAUDE.md` "Not a runtime binary source") = **P077e**.
- Codex adapter impl = **P078** (d1 chỉ định NGHĨA trait Codex sẽ implement, KHÔNG tạo `sos-adapter-codex`).
- Distribution / prebuilt / npm wrapper = **P081**.
- OA-01/03/04/05/08/09 — upgrade order riêng, không block P077d.
