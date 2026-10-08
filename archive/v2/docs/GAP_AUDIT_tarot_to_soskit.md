# Gap Audit — tarot (flagship) → sos-kit (kit) — 2026-05-29

> Read-only audit (agent-run, this session). sos-kit (the distribution **kit**) lagged behind tarot (the living **flagship** product where the workflow actually evolved, tickets P281→P306). This inventories what the kit is **MISSING / STALE** vs tarot — **GENERIC workflow doctrine only** (product-specifics: chị Hạ prompts, PayOS, Next.js/Prisma, tarot cards — deliberately EXCLUDED as correctly-tarot-only).

## ⚠️ Sync direction (Sếp 2026-05-29) — READ FIRST
- **tarot = FROZEN.** Đang chạy ổn định → **KHÔNG áp ngược (sos-kit→tarot) bây giờ.** Back-port lên tarot CHỈ sau khi kit-workflow ngon lại.
- **Áp fixes vào: kit → DỰ ÁN MỚI / media-rating** (proving-ground; media "hơi nát" = chỗ test trước).
- **Tarot sau cùng** — khi kit + media chạy tốt mới quay sang tarot.

## Dominant disease (3/3 top gaps)
Kit **ships the GATE but drops the DOCTRINE that fires it** → Sub-mech A "trigger gap" re-created **INSIDE the kit**. Hook present; the handbook/banner line that triggers it absent. This is what made today's media collapse: the adopted orchestrator had the hook (`block-unsafe-merge.sh`) but not the trigger-doctrine (Rule 9 / spawn-discipline).

## Gap inventory (prioritized; GENERIC only)
| # | Gap | sos-kit state | why it matters | effort |
|---|---|---|---|---|
| 1 | **Advisory auto-spawn (Rule 10 + banner staleness)** | MISSING — banner only shows BACKLOG, never checks `.advisory-scan-state` mtime; no Rule 10 | Trinh sát + `/advisory-scan` + inbox ship but NEVER fire unprompted = Sub-mech A in the kit. Scanner dead-on-arrival. | Small (~35-line banner port + Rule 10) |
| 2 | **Pre-merge security-gate doctrine (Rule 9)** | STALE — hook wired (`settings.json:38`) but `orchestrator.md` has NO "invoke `/security-review` before merge" section; ORCHESTRATION rules stop at #9 (Skills-only) | Hook bypass (branch-only `gh pr merge`) = 0 behavioral backstop. Exact P297 incident hole. **Closest to today's collapse.** | Small-med (orchestrator §section + Rule 9) |
| 3 | **Architect Bước 0 — Layer-1 capability check** | MISSING — only Worker Layer-2 backported | "ship≠chạy" 2-layer defense half-installed; architect can spec impossible integrations (OSV POST/405 class), caught late at EXECUTE | Medium (generic Bước 0 matrix) |
| 4 | **INV-108 git-auth / credential surface (judgment)** | MISSING from the 5 generic INV (mechanical INV-010 present) | Sub-mech F judgment net for PR diffs touching `.git/`, ssh config, `git remote set-url`, GHA secrets. Generic. | Small (+1 INV to rubric+template) |
| 5 | **INV-105 concurrency → atomic write (judgment)** | MISSING from 5 generic INV | Generic data-integrity (any balance/counter/ledger write); stack-agnostic wording | Small |
| 6 | **AI-bias doctrine (golden question + completeness-bias)** | MISSING as agent-facing rule (only a passing mention in WORKFLOW_V2.2) | Cross-AI completeness bias is generic; no scope-discipline guidance in a fresh kit. `[judgment]→guidance` (NOT a gate). | Medium · BACKLOG P044 (DEFERRED) |
| 7 | **6 sub-mech catalog location** | INTENTIONALLY transformed → WORKFLOW_V2.2 §7 (hooks/`doctor`) + worker.md. NOT a blind port. | Decision, not port: confirm §7 is canonical home OR add durable cross-agent pointer | Medium (decision-first) · P044 |
| 8 | **architect-guard Write/Edit block** | STALE/WEAKER — blocks Read/Glob only; Write allows any `*.md` (tarot restricts to `docs/ticket/P*-*.md`) | Architect can write CLAUDE.md/BACKLOG/guides — envelope leak. Low blast, real regression. | Small (~30-line port) |
| 9 | **`check-port-bind.py` (INV-001 mechanical)** | ABSENT — commented stub references non-existent script | Host-bind `0.0.0.0` scan, generic to Docker-Compose repos. Arguably correctly compose-specific. | Small or skip |

## Top 3 (would break a freshly-adopted repo)
1. **#1 Advisory auto-spawn** — scanner dead-on-arrival.
2. **#2 Pre-merge security-gate doctrine** — closest to today's collapse (P297 hole).
3. **#3 Architect Layer-1** — ship≠run defense half-installed.

## Where sos-kit is AHEAD — do NOT sync DOWN from tarot
- **v2.2 doctrine (sos-kit-only, newer):** lane-budget pre-CHALLENGE gate, sensor-arm watchlist (N2–M6), boundary-check rubric-injection / INV-LOCAL-*, Oracle awareness, AGENT_MAP consult, INV-4 **replay-protection** (tarot INV-106 only verifies signature). **Tarot is BEHIND here** → these are candidates to back-port UP to tarot *later* (after kit solid).
- **§7 prose→mechanism transform is deliberate** (v2.2 "one disease, one cheapest mechanism") — do NOT port tarot's prose sub-mech catalog blindly.
- **advisory-watch multi-stack** (`.sos-stack.toml` + parser-per-ecosystem) > tarot's pnpm-only — don't regress.
- **`check-runtime-secrets.py` at full P306 parity** (`.git/config` + `.mcp.json` + `.claude/settings.local.json`). No gap. Sentinel marker `SECURITY_REVIEW_START` vs `security-review-start` is **NOT a bug** — each repo internally self-consistent (sos-kit fixed its own in `03f0579`).

## Note on the Tầng "≤200 LOC"
That orchestrator.md drift is ONE symptom of gap #2's class (kit orchestrator stale vs tarot). Fix Tầng via the **consequence-definition single-source** (LAYERS.md is source; orchestrator.md routes, doesn't redefine) — NOT by adding a keyword/LOC list. See `docs/retro/WORKFLOW_V2.3_RETRO_doc-rotate.md` Vòng 13/14.
