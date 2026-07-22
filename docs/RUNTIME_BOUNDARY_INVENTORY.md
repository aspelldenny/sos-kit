# Runtime Boundary Inventory

> P074 snapshot: tracked tree tại `HEAD`, khảo sát ngày 2026-07-20. Đây là current reality, không phải target layout.

## Cách đọc

- `CORE`: semantics dùng được mà không cần biết agent runtime nào đang chạy.
- `CLAUDE`: source/wiring chỉ Claude Code hiểu.
- `MIXED`: portable semantics và Claude serialization đang nằm chung một surface.
- `GENERATED`: registration/state/manifest được installer hoặc adapter sinh/đồng bộ; không là doctrine source.

Các row áp dụng từ trên xuống; row cụ thể thắng glob rộng hơn. Evidence chỉ lấy từ `git ls-files` và `git grep` trên tracked snapshot, không tính `.backup/`, build artifact hay file local ignored.

## Inventory

| Path/pattern | Class | Current responsibility | Coupling evidence | Target owner | Migration |
|---|---|---|---|---|---|
| `.claude/settings.json` | `CLAUDE` | Bind lifecycle hook vào Claude events và tool matchers | `SessionStart`, `PreToolUse`, `UserPromptSubmit`, Claude tool names | `adapter-claude` template | P076 |
| `.claude/commands/**` | `CLAUDE` | Claude slash-command entrypoints | Files nằm dưới Claude manifest namespace | `adapter-claude` | P076 |
| `.claude/agents/**`, `.claude/skills/**` | `GENERATED` | Tracked symlink đăng ký canonical agents/skills cho Claude | Git mode `120000`; targets trỏ về `agents/**`, `skills/**` | install manifest của `adapter-claude` | P076/P077 |
| `templates/claude-settings.local.json` | `CLAUDE` | Permission template theo Claude tool syntax | Marker Bash permissions và Claude settings schema | `adapter-claude` template | P076 |
| `CLAUDE.md` | `MIXED` | Repo doctrine + Claude contributor entrypoint | Gọi Claude persona, skills, `.claude/**`, tool/model rules; đồng thời chứa repo contract chung | Portable phần chung vào `SOS.md`; file này chỉ còn Claude overlay | P075/P076 |
| `agents/*.md` | `MIXED` | Role semantics và handbook thực thi | Frontmatter `tools`, `model: opus/sonnet`, `background`; body dùng Claude Task/AskUserQuestion/marker semantics | Canonical role spec ở core; renderer/overlay ở adapters | P075/P076/P078 |
| `agents/README.md` | `MIXED` | Hướng dẫn đăng ký agents | Mô tả `.claude/agents`, Claude settings/background | Core agent catalog + Claude install guide | P075/P076 |
| `skills/**/SKILL.md`, `skills/attic/**` | `MIXED` | Workflow skills và lịch sử skill | Semantics phần lớn portable nhưng caller/frontmatter, `/skill` invocation và tool capability mang Claude shape | Canonical skill bodies ở core; runtime renderer ở adapter | P075/P076/P078 |
| `scripts/architect-guard.sh`, `scripts/orchestrator-guard.sh`, `scripts/block-env-edit.sh`, `scripts/idea-smell.sh`, `scripts/session-start-banner.sh`, `scripts/block-unsafe-merge.sh` | `MIXED` | Policy/guard chung được serialize thành Claude lifecycle hooks | `CLAUDE_PROJECT_DIR`, Claude stdin payload/tool names, `claude-hooks` | Policy trong core; Claude binding trong adapter; universal Git phần riêng | P075-P077 |
| `hooks/pre-commit`, `hooks/pre-push`, `scripts/block-env-commit.sh`, `scripts/no-code-on-default.sh`, `scripts/security-gate.sh`, `scripts/trust-gate.sh`, `scripts/check-*`, `scripts/parsers/**`, `scripts/install-hooks.sh` | `CORE` | Universal Git/security gates | Chạy ở Git boundary; vài comment/history nhắc Claude nhưng trigger không phụ thuộc Claude runtime | `sos-core`/tool crates; hooks chỉ gọi `sos hooks run ...` ở target | P075/P077 |
| `.mcp.json` | `MIXED` | Đăng ký sister tools làm MCP servers | MCP là portable concept nhưng shape/config hiện được cài cùng Claude kit và tool list hard-coded | Adapter-neutral tool manifest + runtime-specific MCP renderer | P077/P078 |
| `.sos-state/sos-kit-self` | `GENERATED` | Local lifecycle marker/state | State do SOS tạo, không phải doctrine | Installer state manifest | P077 |
| `.sos-trust-baseline` | `GENERATED` | Hash snapshot auto-exec surfaces | Được `trust-gate rebaseline` sinh; chứa cả `.claude` paths | Portable trust manifest với adapter-owned entries | P077 |
| `.docs-gate.toml`, `templates/.docs-gate.toml`, `templates/.sos-stack.toml.example`, `templates/INVARIANTS-template.md`, `templates/BACKLOG_template.md`, `templates/advisory-inbox.md` | `CORE` | Config/schema/template độc lập runtime | Không cần Claude event/tool/model | `sos-core` templates | P075/P077 |
| `configs/**` | `CORE` | Stack/tool configuration examples | Stack detection và ship config không phụ thuộc agent host | `sos-core` config catalog | P075 |
| `phieu/**` | `CORE` | Ticket lifecycle, audit, discovery, relay và vision templates | Các runtime-token hit mô tả lịch sử/current operator; contract chính là role/state semantics | `sos-core` workflow assets | P075 |
| `recipes/**` | `CORE` | Reusable implementation patterns | Recipe semantics không cần Claude; slash-command mention là caller hiện tại | `sos-core` recipe catalog | P075 |
| `docs/WORKFLOW_V2.1.md`, `docs/WORKFLOW_V2.2.md`, `docs/LAYERS.md`, `docs/HANDOFF.md`, `docs/ORCHESTRATION.md`, `docs/PHILOSOPHY.md` | `MIXED` | Canonical doctrine hiện tại | Role/state doctrine trộn với Claude Agent tool, model, hooks và permission mechanics | Runtime-neutral doctrine; runtime notes chuyển sang adapter docs | P075/P076 |
| `docs/GENESIS.md`, `docs/COMPARISON.md`, `docs/SETUP.md`, `docs/BOOTSTRAP_AUTOMATION_DRAFT.md`, `docs/MECHANIZATION_AUDIT_2026-05-30.md`, `docs/GAP_AUDIT_tarot_to_soskit.md`, `docs/TAROT_ADOPTION_HANDOFF.md`, `docs/security/**` | `MIXED` | Product/setup/audit/security documentation | Runtime hits vừa là normative current instructions vừa là historical evidence | Product docs chung + adapter-specific setup sections | P075-P078 |
| `docs/archive/**`, `docs/discoveries/**`, `docs/retro/**`, `docs/ticket/**`, `docs/DISCOVERIES.md`, `CHANGELOG.md` | `CORE` | Immutable audit/history trail | Runtime-token hits là bằng chứng lịch sử, không phải runtime dependency; không rewrite lịch sử | Core audit archive | Giữ nguyên; future entries dùng neutral vocabulary khi phù hợp |
| `README.md`, `INSTALL.md`, `SECURITY.md` | `MIXED` | User entry/install/threat model | Hiện dẫn Claude-first install và Claude auto-exec surfaces | Product docs chung với runtime selector | P075-P078/P081 |
| `bin/sos.sh` | `MIXED` | Bash MVP cho new/adopt/sync/state và skill delegation | In thẳng “Open Claude Code”, gọi `/init`, `/apply`, `/forge`, sinh Claude wiring | Golden behavior oracle; logic sang Rust core + adapters | P075-P077 |
| `crates/**` (repo-root, relocated from `bootstrap/sos-rs/` P077f) | `MIXED` | Rust CLI skeleton và state machine | CLI portable nhưng LLM phases vẫn delegate Claude skills; README định extraction cũ | Rust workspace chính thức trong `sos-kit` | P077 |
| `install.sh`, `templates/setup-dev.sh` | `MIXED` | One-command bootstrap và dev install | Tải `claude-hooks`, clone kit, cài launcher; tool list hard-coded | Thin bootstrap tải `sos`; `sos install` xử lý manifest/runtime | P077/P081 |
| `integrations/**` | `CORE` | CI canary và uptime example | Không cần Claude host; runtime mentions nếu có là integration history | Core integration catalog | P075 |
| `.gitattributes`, `.gitignore`, `LICENSE` | `CORE` | Repository mechanics/legal | Không runtime coupling | Monorepo root | Giữ nguyên/P077 |

## Runtime-token reconciliation

Mọi path trả về bởi:

```bash
git grep -Il -E 'Claude|CLAUDE_|\.claude|sonnet|opus|AskUserQuestion|PreToolUse|SessionStart|UserPromptSubmit' -- .
```

được cover bởi một row phía trên. Hit trong `archive/discoveries/retro/ticket/CHANGELOG` được giữ như historical evidence, không được xem là dependency sống. Hit trong Git hooks/security scripts được phân loại theo trigger thực: comment hoặc compatibility env fallback không tự biến một universal Git gate thành Claude adapter. Không có tracked path ngoài bảng: các glob root (`docs/**`, `scripts/**`, `templates/**`, v.v.) cùng ba root mechanics cover toàn bộ output của `git ls-files`.

## Boundary findings

1. Canonical role files là điểm trộn lớn nhất: cùng một Markdown đang giữ role doctrine lẫn Claude frontmatter.
2. Claude coupling không chỉ nằm trong `.claude/`; nó lan vào Bash CLI, docs, skills và lifecycle scripts qua env/tool/event names.
3. Universal Git gates đã là portable asset tốt nhất. Target chỉ cần đổi hook stub sang gọi một entrypoint ổn định của `sos`.
4. `install.sh` đã chứng minh nhu cầu one-command, nhưng manifest hiện nằm trong shell và cài nhiều binary trước khi có product CLI quản lý chúng.
5. Rust ownership decision cũ trong `crates/README.md` (relocated from `bootstrap/sos-rs/README.md` P077f) mâu thuẫn target mới. P077 sẽ cập nhật repo contract và giữ Rust workspace tại đây.
