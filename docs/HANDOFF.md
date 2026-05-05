# Handoff — How the 3 layers pass work

> A solo OS works only if the layers talk to each other in a fixed format. Freestyle handoffs are where scope creep, lost context, and "wait, what were we doing?" moments come from.

This doc specifies the **5 handoffs** that actually happen in day-to-day work. See [`LAYERS.md`](./LAYERS.md) for who does what.

**Critical context**: Kiến trúc sư (Claude Web Project) and Thợ (Claude Code) are **separate sessions**. They cannot ping each other. Chủ nhà (the human) is the only bridge. Every handoff that crosses Architect ↔ Worker goes through Chủ nhà as a manual paste.

---

## Handoff 0 — Chủ nhà → Kiến trúc sư (insight / vision briefing)

**Trigger:** Before a project even has tickets, Chủ nhà provides raw context — research findings, user interviews, competitive observations, philosophical positioning. Architect needs this to write coherent phiếu.

**Also triggers when:** vision shifts mid-project (pivot, new persona, new feature area), so Architect's mental model stays current.

**Format:** Insight is NOT a phiếu. It's raw material that Chủ nhà distills (with `/insight` skill help) into one of three vision docs:

- `PROJECT.md` — what the product is (vision, personas, monetization, architecture)
- `SOUL.md` — why it exists (philosophy, positioning, hard lines, anti-product)
- `CHARACTER.md` (or `CHARACTER_<NAME>.md` for named characters, e.g. `CHARACTER_CHI_HA.md`) — voice / persona / tone (if the product has a character like Chị Hạ)

**Chủ nhà workflow:**

```
Raw research paste (hundreds of lines)
  → /insight skill distills
  → Chủ nhà edits + commits to PROJECT.md / SOUL.md / CHARACTER*.md
  → Uploads to Claude Web Project as attached file
  → Architect reads in next session
```

**Architect's response:** on opening a new Claude Web session, read vision docs in order: `CLAUDE.md` → `PROJECT.md` → `SOUL.md` → `CHARACTER*.md` (glob — every match) → DISCOVERIES.md → request-specific guides. Then confirm "I've loaded context" before writing phiếu.

**Anti-pattern:** Chủ nhà jumps straight to "plan this" without Architect having read vision docs. Architect invents wrong framing because no context.

**Fix:** Day 1 of project = write vision docs first. Use `phieu/VISION_TEMPLATES/` as skeleton. No phiếu until at minimum PROJECT.md exists.

---

## Handoff 1 — Chủ nhà → Kiến trúc sư (routing an inbound to build)

**Trigger:** Chủ nhà receives an inbound (user feedback, bug report, feature idea, their own 3am thought). `/route` classifies it as `code`. Ready to spec.

**Format:** 5-bullet brief pasted into Claude Web Project session.

```
ROUTING:             code
SOURCE:              [email / Slack / my head / issue #N]
ONE-LINE:            [what the user actually wants]
SUCCESS LOOKS LIKE:  [user-visible outcome, not implementation]
CONSTRAINTS:         [hard no-no's: don't break X, must ship before Y]
RELATED CONTEXT:     [links to earlier phiếu, metrics, user quotes]
```

**Architect's response:** read brief + relevant guide docs + DISCOVERIES.md → if enough context, write phiếu → if not, ask ≤3 clarification questions back to Chủ nhà in multi-choice format (NOT open-ended).

**Anti-pattern:** Chủ nhà dumps 10 loose sentences without structure. Architect writes phiếu guessing at intent. Wrong feature ships.

**Fix:** Chủ nhà MUST produce the 5-bullet brief. `/route` skill helps.

---

## Handoff 2 — Kiến trúc sư → Thợ (phiếu, the ticket)

**Trigger:** Architect has written a ticket in `phieu/TICKET_TEMPLATE.md` format. Chủ nhà has reviewed and approved (explicit "go" — even a one-word "ok" / "đi" counts). File is at `docs/ticket/P<NNN>-<slug>.md` in the target project.

**Transport:** Chủ nhà copies phiếu file content from Claude Web session → pastes into the target project's `docs/ticket/P<NNN>-<slug>.md` (which was pre-created by `phieu <slug>` shell function on the Worker's machine).

**Phiếu format** — required sections from TICKET_TEMPLATE.md:
- Header (Loại, Ưu tiên, Ảnh hưởng, Dependency)
- Context (Vấn đề, Giải pháp, Scope)
- **Task 0 — Verification Anchors table** (Architect writes "⏳ TO VERIFY" — Thợ fills ✅/⚠️/❌)
- Nhiệm vụ (per-task: File, Tìm, Thay bằng, Lưu ý)
- Files cần sửa + Files KHÔNG sửa (verify only)
- Luật chơi (Constraints)
- Nghiệm thu (Automated + Manual + Regression + Docs Gate + Discovery Report)

**Thợ's first move on receiving phiếu:** run `/verify` Task 0 (grep every anchor). Do NOT code until Task 0 is all ✅. If any ⚠️ / ❌ → escalate via Handoff 3.

**Anti-pattern:** Architect writes "use function `getUserX`" without citing a doc source. Function doesn't exist. Thợ wastes an hour before discovering it.

**Fix:** Every anchor cites `thợ kiểm tra tại [file]:[function]` + source in docs. Task 0 is mandatory gate.

---

## Handoff 2.5 — Architect ↔ Worker debate (v2.1 Subagent mode only)

**Trigger:** Architect just wrote phiếu V1 in DRAFT mode. Before Worker EXECUTEs, orchestrator spawns Worker in CHALLENGE mode to verify phiếu's assumptions against real code.

**Transport:** `## Debate Log` section inside the phiếu file. Append-only — never delete prior turns. Phiếu version tracked at the top of the section (V1 → V2 → ...).

**Format (Worker → Architect, Turn N):**

```markdown
### Turn <N> — Worker Challenge (phiếu V<N>)
**Anchor verification:** [✅/⚠️/❌ summary from Task 0]
**Objections (Tầng 1 only):**
- [O<N>.1] Phiếu assumes X at file Y, code at file:line shows Z. Impact: …
- [O<N>.2] …
**Proposed alternatives:**
- A. … (Worker lean — because …)
- B. …
**Status:** ⏳ AWAITING ARCHITECT RESPONSE
```

**Format (Architect → Worker, Turn N response):**

```markdown
### Turn <N> — Architect Response (phiếu V<N+1>)
- [O<N>.1] → ACCEPT / DEFEND / REFRAME (Tầng 2) / DEFER TO CHỦ NHÀ → action taken
- [O<N>.2] → …
**Status:** ✅ RESPONDED — phiếu bumped to V<N+1>
```

**Termination conditions:**
- Worker accepts (no objections) → orchestrator runs Chủ nhà approval gate → EXECUTE
- Architect responded with no DEFER → orchestrator spawns Worker (CHALLENGE) again on V<N+1> to verify
- Architect used DEFER TO CHỦ NHÀ → orchestrator escalates via AskUserQuestion
- Turn 3 reached, still objections → force-escalate Chủ nhà

**Anti-pattern:** Worker dumps objection without citing `file:line` from real code → Architect has no evidence to judge → loop without progress.
**Fix:** Every objection MUST cite `file:line`. Orchestrator rejects evidence-free objections.

**Anti-pattern:** Chủ nhà summoned mid-debate when agents could have resolved it themselves → violates "Chủ nhà không làm courier."
**Fix:** Chủ nhà only enters at the approval gate, on Architect DEFER, or at max-turn cap. Never as a relay between agents.

**Replaces RELAY_PROTOCOL.md** for v2.1 Subagent mode. The relay role is now automated by the main session orchestrator. RELAY_PROTOCOL.md remains valid for v1 Web Project users.

See [`ORCHESTRATION.md`](./ORCHESTRATION.md) for the full state machine and an example session.

---

## Handoff 3 — Thợ → Chủ nhà → Kiến trúc sư (architectural blocker)

**Trigger:** Thợ hits a problem that is **architectural** (Tầng 1), not detail (Tầng 2). Examples:
- Task 0 finds ❌ — phiếu's file/function/constant doesn't exist
- Mid-code discovers the phiếu's approach conflicts with existing architecture
- Scope balloons beyond what's in the phiếu (new column needed, new API route, auth change)
- Phiếu solution is clean but Thợ sees a subtle issue (race condition, security gap)

**Key constraint:** Thợ CANNOT ping Architect directly. Claude Code session and Claude Web session are separate. Chủ nhà is the human courier.

**Format (Thợ → Chủ nhà):**

```
BLOCKER:                 [one-line what's stuck, architecturally]
PHIẾU:                   P<NNN>-<slug>
TẦNG:                    Tầng 1 — architectural (escalating)
CONTEXT:                 [what Thợ tried, what broke, 2-3 lines max]
OPTIONS (Thợ's lens):    (max 3, Thợ recommends one)
   A. [option with trade-off]
   B. [option with trade-off]
   C. [option / abandon]
THỢ LEAN:                [A/B/C] because [1-line reason]
NEED FROM ARCHITECT:     [specific decision needed — not "what should I do"]
```

**Chủ nhà's response:** if decision is clear (e.g. Thợ's recommendation is obviously right), Chủ nhà can decide directly. If genuinely architectural, Chủ nhà forwards to Architect in Claude Web.

**Format (Chủ nhà → Architect):** paste Thợ's report verbatim, add Chủ nhà's own context (whether this touches SOUL principles, timeline pressure, etc).

**Architect's response:** either
- Update phiếu (add Task N, adjust Nhiệm vụ, change Scope) → Chủ nhà forwards new phiếu back to Thợ
- Or say "Thợ's recommendation A is correct, proceed" → Chủ nhà forwards approval back

See `phieu/RELAY_PROTOCOL.md` for Chủ nhà's checklist.

**Anti-pattern:** Thợ silently "fixes" architectural issue. Architecture drifts from phiếu. Next phiếu conflicts because Architect's mental model is now stale.

**Fix:** Tầng 1 issues STOP Thợ. Write escalation. Wait for Architect's response via Chủ nhà.

---

## Handoff 4 — Thợ → Kiến trúc sư (Discovery Report, post-execution)

**Trigger:** Phiếu is done (or mostly done). Thợ writes Discovery Report documenting:
- Which phiếu assumptions were CORRECT
- Which phiếu assumptions were WRONG (Tầng 2 — Thợ self-adapted and shipped, OR Tầng 1 — already escalated in Handoff 3)
- Edge cases / limitations found during implementation
- Which docs Thợ updated to match code reality

**Format:** entry appended to project's `docs/DISCOVERIES.md` (newest on top, like CHANGELOG).

```markdown
## [P<NNN>-<slug>] — YYYY-MM-DD — <title>

### Assumptions in phiếu — CORRECT
- [assumption X matched code at file:line]

### Assumptions in phiếu — WRONG
- [Tầng 2] Assumption Y: phiếu said "use var name A", code clearer as B → adopted B, updated phiếu text
- [Tầng 1] Assumption Z: phiếu said "function X in lib/a.ts", actually in lib/b.ts → escalated via Handoff 3, Architect updated phiếu

### Edge cases / limitations found
- [Phiếu didn't mention Safari compat — iOS 15 broke with X, added workaround]
- [Rate limiter triggers at 30 req/min, docs said 60 — updated BACKEND_GUIDE.md]

### Docs updated to match reality
- `BACKEND_GUIDE.md` §3: corrected function signature for `foo`
- `PROMPTS.md` §7: marker name was stale, updated
```

**Transport:** Thợ commits DISCOVERIES.md entry as part of the phiếu's final commit. No manual relay needed — Architect reads DISCOVERIES.md directly in the next Claude Web session before writing the next phiếu.

**Architect's response (next session):** read DISCOVERIES.md since last phiếu. Fold corrections into next phiếu's assumptions. If a discovery invalidates a common guide doc pattern, update the guide doc (Chủ nhà approves).

**Anti-pattern:** Thợ skips Discovery Report ("nothing went wrong"). Architect has no signal of silent Tầng 2 adaptations. Next phiếu's anchors drift.

**Fix:** Discovery Report is a commit gate. Write "None" in each section if truly nothing — explicit "None" proves you checked.

---

## Mismatch classification — which Handoff to use

When Thợ finds a phiếu / code mismatch, classify BEFORE deciding response:

| Mismatch type | Example | Tier | Handoff | Thợ action |
|---|---|---|---|---|
| Local variable name | Phiếu says `items`, Thợ prefers `entries` inside a helper | Tầng 2 | Handoff 4 (post-ship) | Self-decide, log to Discovery |
| Internal helper location | Phiếu says new helper in `lib/utils.ts`, Thợ puts in `lib/helpers.ts` | Tầng 2 | Handoff 4 | Self-decide, log |
| CSS class / styling detail | Phiếu says `flex`, Thợ uses `grid` for the same visual result | Tầng 2 | Handoff 4 | Self-decide, log |
| Non-visible error message wording | Console log string | Tầng 2 | Handoff 4 | Self-decide, log |
| User-visible error text | Toast message, email body | Tầng 1 (Chủ nhà's call) | Handoff 3 (→ Chủ nhà directly) | Stop, ask Chủ nhà for wording |
| Function signature | Phiếu says `foo(a: string)`, code has `foo(a: string, b: number)` | Tầng 1 | Handoff 3 | Stop, escalate |
| Public API endpoint shape | Phiếu says POST, code needs PATCH | Tầng 1 | Handoff 3 | Stop, escalate |
| Missing file / constant | Phiếu references `FOO`, doesn't exist | Tầng 1 | Handoff 3 | Stop, escalate (Task 0) |
| Database schema change | Phiếu needs new column | Tầng 1 | Handoff 3 | Stop, escalate |
| New dependency added | Phiếu implies a library not currently installed | Tầng 1 | Handoff 3 | Stop, escalate |

Rule of thumb: **"Would another Worker need to know this to maintain the code later?"**
- YES → Tầng 1 → Handoff 3 (escalate)
- NO → Tầng 2 → Handoff 4 (log post-ship)

---

## Handoff triggers summary

| From → To | Trigger | Format | Skill |
|---|---|---|---|
| Chủ nhà → Architect | Vision docs need to be loaded | `VISION_TEMPLATES/` files committed to project | `/insight` (to distill raw → doc) |
| Chủ nhà → Architect | Inbound classified as `code` | 5-bullet brief | `/route` |
| Chủ nhà → any | Scope/trade-off judgment | Decision + rationale | `/decide` |
| Architect → Worker | Phiếu ready + Chủ nhà approved | `phieu/TICKET_TEMPLATE.md` | `/plan` |
| Architect ↔ Worker (auto-debate, v2.1) | Phiếu V1 just written, pre-execute | Debate Log section in phiếu | (orchestrator) |
| Worker → Worker (pre-code) | Task 0 anchor verification | Inline report per ticket | `/verify` |
| Worker → Chủ nhà → Architect | Tầng 1 blocker | Multi-choice escalation | Worker frames choices → Chủ nhà invokes `/decide` |
| Worker → Architect (post-code) | Discovery Report | `docs/DISCOVERIES.md` entry | (manual, end of phiếu) |

## Why this matters

Every handoff is a potential information loss point. In a 20-person team you'd have Jira + Slack + stand-ups — redundant channels smooth over loss.

Solo, there is no redundancy. If a handoff fails, the signal is gone. Formalizing the 5 handoffs above is the minimum needed to keep the three layers coherent — especially because two of the layers (Architect, Worker) live in separate sessions that cannot talk directly.

## When to break the format

Rarely. Specifically:

- **Same-day hotfix (< 30 min scope):** skip the formal phiếu, use a CHANGELOG entry as the ticket. Still write Discovery Report if you found something.
- **Pure exploration ("can we even do X?"):** no phiếu. Architect answers Chủ nhà with findings + recommendation. Phiếu only when Chủ nhà says "ok, build it."
- **Pure refactor (no behavior change):** phiếu is lightweight — scope + nghiệm thu only. Task 0 still mandatory.

Anything longer than those: use the full format. The overhead is small, the friction it prevents is large.
