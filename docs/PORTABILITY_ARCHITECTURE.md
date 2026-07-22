# SOS Kit Portability Architecture

> P074 target decision, 2026-07-20. Current reality nằm ở `docs/RUNTIME_BOUNDARY_INVENTORY.md`.

## Product decision

SOS Kit là **một product trong một monorepo**, phát hành theo một version và có một user-facing binary: `sos`.

```text
one repository → one release/version → one `sos` entrypoint
                                      ├─ portable core
                                      ├─ Claude Code adapter
                                      ├─ Codex adapter
                                      ├─ Git hooks/gates
                                      └─ managed sister tools
```

Một repo không đồng nghĩa một module. Boundary được giữ bằng Rust crates và dependency rules; người dùng không phải hiểu hoặc cài từng crate/tool.

## Target workspace

Tên crate là target shape để P077 chốt, không phải cam kết public API trong P074:

```text
sos-kit/
├── Cargo.toml                    # workspace + version chung
├── crates/
│   ├── sos-cli/                  # binary duy nhất: `sos`
│   ├── sos-core/                 # state, policy, roles, workflow, config schema
│   ├── sos-install/              # plan/apply/sync/rollback/manifest
│   ├── sos-adapter-claude/       # Claude renderer + detector + lifecycle binding
│   ├── sos-adapter-codex/        # Codex renderer + detector + permission/config mapping
│   └── sos-hooks/                # universal Git hook dispatcher
├── core/                         # canonical Markdown/templates/recipes, runtime-neutral
├── adapters/
│   ├── claude/                   # adapter-owned templates/assets
│   └── codex/
└── tool-manifest.toml            # managed external tool versions/assets/checksums
```

`bootstrap/sos-rs` là nguồn khởi đầu được nâng vào workspace này ở P077, không extract sang repo khác. `CLAUDE.md` hiện nói repo không chứa runtime source; P077 phải sửa contract đó cùng lúc workspace trở thành canonical.

**P077b status (crate boundary carved):** 5 crate skeleton dựng tại `bootstrap/sos-rs/crates/` với đúng dependency direction (adapter→core, core zero adapter dep, CLI composition root — xem `bootstrap/sos-rs/README.md` "Module layout"). Deviation so target — đúng 3 điểm đã biết trước, KHÔNG phát sinh thêm:
1. Workspace root ở `bootstrap/sos-rs/` (target: repo-root) — relocate = P077e.
2. `sos-adapter-codex` chưa tạo — P078, ngoài scope P077.
3. `sos-install`/`sos-adapter-claude`/`sos-hooks` là skeleton rỗng — logic P077d.

**P077d1 status (adapter contract + manifest schema carved):** trait `Adapter` (5 method — `detect/plan/render/verify/uninstall`, khớp dòng 63-69 dưới) render tại `sos-core/src/adapter.rs` cùng type runtime-neutral (`Capabilities`/`Plan`/`Artifact`/`Findings`/`RemovalPlan`); `ManagedManifest` (6 field, khớp `core/ASSETS.md:57-64`) render tại `sos-core/src/manifest.rs`, serde TOML round-trip tested. `sos-adapter-claude` implement trait với stub bodies (zero fs mutation, zero install logic — đó là P077d2). Dependency-direction giữ nguyên (adapter→core one-way, guard test vẫn xanh).

**P077d3 status (tool-manifest live, OA-07 RESOLVED trong Rust path):** `tool-manifest.toml` (kit root, committed — KHÁC `.sos-manifest.toml` d2 generated) pin 10 sister tool (`name`/`version`/`required` + `[tool.asset]`/`[tool.checksum]` per 3 target-triple), 6 required (`doctor`/`claude-hooks`/`docs-gate`/`ship`/`advisory-inbox`/`inv-gate`, khớp `install.sh:40`) / 4 optional (`guard`/`vps`/`doc-rotate`/`advisory-cron`, khớp `install.sh:48`). Version fill từ source `Cargo.toml` mỗi tool (KHÔNG `releases/latest` — chính là bug OA-07), checksum = honest `TODO-sha256-<tool>-P081` placeholder (chưa có prebuilt asset hash cho pin shape này — E2, real fill = P081). `sos_install::tools` (NEW module) — `check_tools()` chạy `<tool> --version` (E1: format `"<name> <x.y.z>"` đồng nhất trên cả 9 tool live-test, không tiền tố `v`), hand-rolled dotted-tuple compare (KHÔNG có crate `semver` trong workspace — anchor #8), `Verdict` (Ok/Newer/Drift/Missing/Unparseable), `gate_required()` fail-closed core dùng CHUNG bởi 2 surface: **`sos tools status`** (table + exit 1 nếu required drift/missing, optional = warn-only exit 0) và **step 5 `resolve_tools()`** (nay ĐỌC manifest thật, KHÔNG còn no-op — fail trước khi mutate fs, nên "rollback" trivial vì chưa ghi gì). **KHÔNG có `sos doctor` subcommand** trong sos-cli (mọi `doctor` reference là shell-out `Command::new("doctor")` gọi binary ngoài) → fail-clear surface qua 2 đường trên, không tạo subcommand xung đột. d3 GIỚI HẠN: verify-only, KHÔNG tải/atomic-upgrade/auto-fetch — đó là P081. Live smoke trên dev machine reproduce đúng OA-07 evidence (doctor 0.1.1 vs pinned 0.1.3, inv-gate MISSING, exit 1). `install.sh`/`bin/sos.sh` legacy `releases/latest` giữ nguyên tới P077e cutover.

**P077d2 status (install engine live):** `sos-install::engine` implement transaction step 1-4+6-7 (step 5 tool-resolve NAY ĐÃ FILL bởi P077d3, xem status line trên — trước đó là stub seam `// SEAM P077d3`, `resolve_tools()` trả về empty), driven THUẦN qua `Adapter` trait — engine zero host token. **Narrow d1 amendment** (Worker CHALLENGE Turn 1 O1.1, Architect ACCEPT Alt A): `ManagedOperation` (`sos-core/src/adapter.rs`) thêm field `content: String` — `plan()` trước đó chỉ trả path+description, không đủ cho engine biết ghi bytes gì; Adapter 5-method + ManagedManifest 6-field KHÔNG đổi. Non-clobber discrimination = 4-way (`Create`/`NoOp`/`Update`/`Conflict`): target absent → CREATE; target hiện có + hash == desired → NoOp (idempotence, zero mutation); hash khác desired nhưng khớp `content_hash` đã ghi trong `.sos-manifest.toml` → UPDATE cho phép; khác + không khớp/không record → CONFLICT, stage `.sos-install-incoming/<path>` (mirror `.sos-adopt-incoming`), original giữ nguyên. Rollback = snapshot-before-mutate (CREATE→delete on fail, UPDATE→restore prior bytes), `.sos-manifest.toml` không commit khi fail, exit non-zero. Manifest artifact: `.sos-manifest.toml` tại project root, `[[managed]]` array-of-tables. Oracle = 5 correctness fixtures (`crates/sos-install/tests/install.rs`, `MockAdapter`) — KHÔNG parity-vs-Bash (lệnh mới). `sos install --runtime <auto|claude|codex>[--dry-run]` wired trong `sos-cli`; `claude` dùng `ClaudeAdapter` (vẫn stub d1 → plan tối thiểu/rỗng, render thật vẫn defer); `codex` → lỗi rõ "not yet available (P078)". `bin/sos.sh`/`install.sh` zero-touch.

## Ownership và dependency direction

### Portable core

Sở hữu role semantics, workflow/state machine, policy, phiếu, recipes, canonical skill body, universal Git gates và state/config schemas. Core không được chứa:

- runtime event/tool/model name;
- `CLAUDE_*`, Claude/Codex manifest path;
- permission serialization;
- lệnh spawn một agent host cụ thể.

### Runtime adapters

Mỗi adapter implement cùng contract tối thiểu:

```text
detect() → capabilities
plan(core, project) → managed file/tool/hook operations
render(asset, capabilities) → runtime-native artifact
verify(project) → findings
uninstall(manifest) → safe removal plan
```

- Claude adapter sở hữu `CLAUDE.md`, `.claude/**`, Claude agent frontmatter, lifecycle events, tool/model/env mapping.
- Codex adapter sở hữu `AGENTS.md`, `.codex/**`, Codex skills/agents/config/MCP/permission mapping.
- Adapter được phụ thuộc core contract; core không import adapter. CLI là composition root chọn một hoặc nhiều adapter.

### Generated artifacts

Install/sync ghi manifest gồm owner, source version, target path và content hash. Generated file không là source of truth. Luật apply:

- mặc định additive/non-clobber;
- file chưa đổi kể từ lần cài có thể update;
- file người dùng đã sửa phải báo conflict hoặc ghi incoming copy;
- `--dry-run` luôn cho thấy plan;
- mutation có backup/rollback record;
- uninstall chỉ xóa artifact vẫn khớp managed hash.

## Một lệnh cài toàn bộ

Public UX cuối cùng:

```bash
# bootstrap không cần Rust toolchain
curl -fsSL <official-installer> | sh

# hoặc npm/pnpm convenience wrapper tải cùng prebuilt binary
pnpm dlx sos-kit install --runtime codex
```

Bootstrap chỉ tải binary `sos` đúng platform và verify checksum. Sau đó:

```bash
sos install --runtime auto        # detect Claude/Codex; cài common + adapter tương ứng
sos install --runtime claude      # common + Claude
sos install --runtime codex       # common + Codex
sos install --runtime claude,codex
```

`sos install` lập một transaction plan:

1. Detect project, OS/arch và runtime capabilities.
2. Cài/update common core assets.
3. Render runtime adapter assets.
4. Cài Git hook stubs gọi `sos hooks run pre-commit|pre-push`.
5. Resolve tool manifest, tải đúng version/checksum của required và optional tools.
6. Ghi managed manifest/state.
7. Chạy `sos doctor`; lỗi required component làm install fail rõ ràng và rollback.

## Hooks, gates và sister tools

Tất cả được expose qua một namespace ổn định:

```text
sos doctor
sos gate inv
sos gate docs
sos docs rotate
sos ship
sos hooks install
sos hooks run pre-commit
sos hooks run pre-push
sos tools status|install|upgrade
```

Ban đầu `doctor`, `inv-gate`, `docs-gate`, `doc-rotate`, `ship` và các tool hiện hữu có thể tiếp tục là binary/repo riêng. `sos` khóa version/checksum trong `tool-manifest.toml`, tải và dispatch chúng. Như vậy one-command UX không bị block bởi một cuộc nhập source lớn. Chỉ nhập một tool thành workspace crate khi có lý do kỹ thuật/release rõ ràng; đó không phải điều kiện portability.

Git hook trong repo đích là stub nhỏ và ổn định:

```sh
exec sos hooks run pre-commit "$@"
```

Policy và phase logic sống trong Rust/tool manifest, nên `sos upgrade` cập nhật hành vi mà không rải lại shell logic tùy tiện.

## Migration order và gates

| Phiếu | Deliverable | Gate để đi tiếp |
|---|---|---|
| P075 | Portable core (`SOS.md`, roles/policy/workflow/assets neutral) | Xóa mọi adapter path vẫn đọc hiểu và validate được core; zero runtime token trong canonical core allowlist |
| P076 | Claude adapter từ cùng core | Golden Claude behavior trước/sau tương đương: agents, skills, hooks, permissions, new/adopt/sync |
| P077 | Rust workspace + adapter/install contract + manifest/dry-run/non-clobber/rollback/sync/doctor | Rust CLI là canonical; Bash oracle parity xanh; repo contract đổi sang runtime monorepo |
| P078 | Codex native adapter | Fresh install sinh đúng Codex-native files và doctor xác nhận; không copy doctrine fork |
| P079 | Dogfood Codex trên chính sos-kit | Một phiếu thật đi đủ DRAFT→CHALLENGE→APPROVAL→EXECUTE→DISCOVERY→MERGE |
| P080 | Dual-runtime brownfield dogfood | Claude + Codex cùng repo, fresh/brownfield, repeated install/sync, conflict/rollback, platform matrix xanh |
| P081 | Distribution | Prebuilt/checksum/release provenance xanh; installer và pnpm wrapper gọi cùng binary; one-command acceptance |

Không khởi công packaging P081 trước hai dogfood gates P079/P080. Cursor, OpenCode và Antigravity là future adapters; không nằm trong acceptance hiện tại.

## Non-goals của vòng này

- Không tạo doctrine copy riêng cho Claude và Codex.
- Không ép mọi sister tool nhập source vào monorepo trước khi installer dùng được.
- Không giữ Bash và Rust là hai canonical implementations sau parity.
- Không hứa runtime chưa có môi trường dogfood.
