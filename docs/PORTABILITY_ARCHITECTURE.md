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
