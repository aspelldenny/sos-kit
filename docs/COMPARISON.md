# Comparison — SOS Kit vs gstack

> This document compares SOS Kit to gstack, the closest-adjacent framework by Garry Tan (YC CEO, early 2026). Not a competitive takedown — an honest map of where each fits.

## Quick comparison

| | SOS Kit | gstack |
|---|---|---|
| Author | aspelldenny | Garry Tan (YC CEO) |
| Language | Rust (CLI) + markdown (skills) | Bun/TypeScript |
| Binary size | 4 tools, ~12MB total | 58MB |
| Approach | CLI tools + role-separated skills | 31 Markdown skills |
| Role model | 3-role (Chủ nhà / Kiến trúc sư / Thợ) | Flat skill set |
| Ticketing | Phiếu + worktree per ticket | None |
| Browser automation | No (not needed) | Playwright daemon |
| Planning skills | `/insight` + `/route` + `/decide` + `/plan` | 6 planning skills |
| Design skills | No (intentional — delegates to Stitch / Figma) | 4 design skills |
| Ship automation | ✅ Full (test → PR → deploy → canary) | ✅ Full (test → PR) |
| Deploy | ✅ SSH / Render / Actions / cargo | ❌ Manual |
| Health monitoring | ✅ Canary + Uptime (`guard` + `vps`) | ✅ Canary |
| Cross-project learnings | ✅ JSONL (`ship learn`) | ✅ JSONL |
| Docs enforcement | ✅ docs-gate (Rust, pre-commit) | ❌ Manual |
| MCP server | ✅ 4 tools (ship, docs-gate, guard, vps) | ❌ No |
| Pre-commit hooks | ✅ | ❌ No |

## Where each fits

**SOS Kit** focuses on the **tail of the pipeline** — from "code is ready" to "it's live and healthy in production." It deliberately doesn't try to replace your planning methodology; it assumes you already have one (Shape Up, vibe, whatever). The 3-role separation (Chủ nhà / Kiến trúc sư / Thợ) and phiếu workflow are the unique bits.

**gstack** is broader — 31 skills spanning ideation to retrospective. More opinionated about *how* you think, plan, and design. If you don't have a methodology yet and want a full turnkey system with design + planning tools, gstack covers more ground.

## When to pick which

### Pick SOS Kit when:
- You already have a planning methodology and just want ship automation + role separation
- You value fast Rust tooling (< 5ms startup, small binaries)
- You care about documentation enforcement at commit time
- You want to separate "architect" from "worker" explicitly (Claude Web vs Claude Code)
- You need production ops tools (`guard` pre-deploy checks, `vps` for Docker logs/restart/status)

### Pick gstack when:
- You want one opinionated system covering ideation → design → planning → shipping
- You want 31 pre-built skills and are happy with TypeScript
- You like browser automation for verifying visual work
- You don't want to design your own planning process

### Use both?
They're not mutually exclusive. gstack's planning/design skills complement SOS Kit's ship/role-separation infrastructure. If one ever gets in the way of the other — trust the one closest to the friction.

## Honest limitations of SOS Kit

- **Vietnamese-flavored** — 3-role names (Chủ nhà / Kiến trúc sư / Thợ) and "phiếu" are Vietnamese. Conceptually translate well, but some users may prefer English-native terms (Owner / Architect / Worker).
- **Not a planning tool** — no ideation skill, no user-research skill. Assumes Chủ nhà already has clarity about what to build.
- **Small community** — gstack has 54K+ GitHub stars and active iteration from YC; SOS Kit is a single-maintainer kit.
- **Not designed for teams** — every feature assumes one human. Multi-user auth / shared dashboards / Slack integration are intentionally out of scope.

## Closing thought

SOS Kit and gstack converge on similar constraints ("one person shipping production software") and arrive at similar structural principles (separate roles, gates between steps, cross-project learnings, shipping automation) from different angles. The convergence validates the underlying problem, not any one answer.

Use whichever one reduces your friction the most. Both are MIT-licensed. Both are small enough to read end-to-end in an afternoon.
