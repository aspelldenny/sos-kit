# skills/attic/ — parked skills (NOT shipped by sos new/adopt/sync)

> Parked 2026-06-11 after the full-13 dogfood (`docs/retro/SKILLS_DOGFOOD_2026-06-11.md`).
> Law that put them here (Sếp-ratified): **"skill đẹp mà không dùng = không tác dụng"** —
> a skill enters the kit only with a declared MECHANICAL caller (`caller:` frontmatter:
> hook / cron / CLI / gate / agent-handbook contract). No caller = attic.
> Evidence: tarot had all 13 registered for months — 0 real invocations.

| Skill | Why parked | Revive condition |
|---|---|---|
| `plan` | STALE (still says "Kiến trúc sư lives in Claude Web Project" — v1 doctrine, dropped). Real content lives in `agents/architect.md` (Task 0 / TICKET inline ×13) | Never as-is. If revived: rewrite from architect.md, not from this file |
| `verify` | Absorbed — `agents/worker.md` inlines Task 0 ×14, Thợ runs it natively in EXECUTE. Also name-collides with Claude Code's built-in `verify` skill | A standalone caller appears (e.g. CI step needing Task-0-only pass) + rename |
| `decide` | Absorbed — Quản đốc does options+impact+recommend+AskUserQuestion inline (verified live 2026-06-11: private-repos decision matched this spec step-for-step) | Orchestrator inline behavior degrades (regression evidence) |
| `route` | 0 callers + lane taxonomy stale: code/marketing/design/strategy/skip has NO ops/infra lane — first real inbound tested (jarvis PAT ops) didn't fit | Fix lanes + a caller (e.g. inbox-triage cron) |
| `insight` | Targets PROJECT/SOUL/CHARACTER — only product repos have those; 0 usage even in tarot | Product-vision work resumes AND a caller is wired |
| `qa` | References gstack-era "V8 pipeline"; role covered by Thợ EXECUTE testing + Giám sát | Rewrite against current doctrine + caller |
| `review` | Role taken by Giám sát `/security-review` (alive BECAUSE the merge gate demands its sentinel) | Generic non-security review gets its own gate |
| `ship` | The `ship` BINARY is alive (Tier 1, `ship_canary` used in tarot). This skill is a thin prose wrapper around the CLI | Binary grows a flow that genuinely needs LLM orchestration |

Parked ≠ deleted: git history + this folder keep them. Downstream repos that already
copied the 13 keep their copies harmlessly (`sos sync` never deletes).
