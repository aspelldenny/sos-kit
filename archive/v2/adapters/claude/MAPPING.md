# Claude artifact → core source ID mapping

> P076 declarative boundary. Every row: physical path stays put; only the mapping is new. Physical move column tracks the future migration ticket, not present state.
> Paths verified against `docs/golden/P076-claude-baseline.md` and live repo state at P076 EXECUTE time (see `docs/discoveries/P076.md` for the verification run).

| Artifact (physical, verified) | Class | Core source ID | Physical move |
|---|---|---|---|
| `.claude/settings.json` | ADAPTER_OWNED | lifecycle binding of policy intent → `core/POLICY.md` (+ guard scripts) | P077 |
| `.claude/commands/advisory-scan.md`, `.claude/commands/security-review.md` | ADAPTER_OWNED | command entry → `core/WORKFLOW.md` (security-review flow) | P077 |
| `templates/claude-settings.local.json` | ADAPTER_OWNED | permission template → `core/POLICY.md` (authority/scope) | P077 |
| `.mcp.json` (MCP server `doctor`) | ADAPTER_OWNED | optional protocol-server registration → `core/POLICY.md` (mechanical gate authority: lane-check, validate-map, rotate-check, runtime-scan) | P077 |
| `.claude/agents/advisory-watch.md`, `.claude/agents/architect.md`, `.claude/agents/boundary-check.md`, `.claude/agents/worker.md` (4 symlinks) | GENERATED | registration of `agents/*.md` | P077 install manifest |
| `.claude/skills/apply`, `.claude/skills/forge`, `.claude/skills/idea`, `.claude/skills/init`, `.claude/skills/retro` (5 symlinks) | GENERATED | registration of `skills/*/SKILL.md` | P077 install manifest |
| `agents/architect.md` | TRANSITIONAL_MIXED | `core/ROLES.md#architect` | P077 render, P078 |
| `agents/worker.md` | TRANSITIONAL_MIXED | `core/ROLES.md#worker` | P077 render |
| `agents/orchestrator.md` | TRANSITIONAL_MIXED | `core/ROLES.md#orchestrator` | P077 render |
| `agents/advisory-watch.md` | TRANSITIONAL_MIXED | `core/ROLES.md#advisory_watch` | P077 render |
| `agents/boundary-check.md` | TRANSITIONAL_MIXED | `core/ROLES.md#boundary_check` | P077 render |
| `agents/README.md` | TRANSITIONAL_MIXED | agent index → `core/ROLES.md` (all role IDs) | P077 render |
| `skills/idea/SKILL.md` | TRANSITIONAL_MIXED | portable body (Chủ nhà intake workflow) → core skill semantics; `caller:` invocation binding = adapter part | P077 render, P078 |
| `skills/retro/SKILL.md` | TRANSITIONAL_MIXED | portable body (weekly velocity/hotspot workflow) → core skill semantics; `caller:` invocation binding = adapter part | P077 render, P078 |
| `skills/init/SKILL.md` | TRANSITIONAL_MIXED | portable body (0→1 vision capture workflow) → core skill semantics; `caller:` invocation binding = adapter part | P077 render, P078 |
| `skills/apply/SKILL.md` | TRANSITIONAL_MIXED | portable body (recipe apply workflow) → core skill semantics; `caller:` invocation binding = adapter part | P077 render, P078 |
| `skills/forge/SKILL.md` | TRANSITIONAL_MIXED | portable body (recipe authoring workflow) → core skill semantics; `caller:` invocation binding = adapter part | P077 render, P078 |
| Lifecycle guard scripts (`scripts/architect-guard.sh`, `scripts/orchestrator-guard.sh`, `scripts/block-env-edit.sh`, `scripts/block-unsafe-merge.sh`, `scripts/idea-smell.sh`, `scripts/session-start-banner.sh`) | TRANSITIONAL_MIXED | policy intent → `core/POLICY.md`; host event payload/binding = adapter part | P076/P077 |

## Coverage vs golden baseline (`docs/golden/P076-claude-baseline.md`)

| Golden section | Covered by row(s) above |
|---|---|
| 1. `.claude` tree + git modes | ✅ settings.json, commands, agents symlinks, skills symlinks |
| 2. Symlink topology | ✅ agents/skills symlink rows |
| 3. Role capability matrix (agents frontmatter) | ✅ 5 `agents/*.md` rows |
| 4. Skills registration (skills frontmatter) | ✅ 5 `skills/*/SKILL.md` rows |
| 5. Slash commands | ✅ `.claude/commands/**` row |
| 6. Hook wiring | ✅ `.claude/settings.json` row + lifecycle guard scripts row |
| 7. `sos` CLI surface (`bin/sos.sh`) | **Out of scope** — `bin/sos.sh` migration owner is P077 per `core/ASSETS.md` line 35 (not a Claude-specific artifact) |
| 8. MCP servers | ✅ `.mcp.json` row (added per Worker CHALLENGE completeness note) |
| 9. `doctor` connectivity | **Out of scope** — verified by `doctor` binary itself, not a static Claude artifact to map |

Every row above has a non-empty core source ID (`core/ASSETS.md` line 51 requirement). No row references `adapters/` from `core/` — dependency direction stays one-way (adapter → core).
