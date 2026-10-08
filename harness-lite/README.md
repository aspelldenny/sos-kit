# Harness Lite — adaptive pilot

Portable prompt guidance for a human who owns brainstorm and final acceptance, with AI responsible for the work between. This is an owner-authorized pilot, not a replacement for SOS Kit's existing workflow, a registered agent package, or a CLI profile. It has not yet demonstrated better results across models or production apps.

## Use it

Read this adoption page once. Runtime input is the [shared contract](CONTRACT.md), one role and the current task brief. Do not load every role or the research history into every worker.

| Session | Role prompt |
| --- | --- |
| Main coordinator | [Quản đốc](roles/orchestrator.md) |
| Structural decision, when needed | [Kiến trúc sư](roles/architect.md) |
| Implementation | [Thợ](roles/worker.md) |
| Independent assessment | [Người soát](roles/reviewer.md) |

In Codex or Claude Code, point the main session to the files using actual accessible paths, or attach their contents. For example:

> Use Harness Lite's CONTRACT.md and roles/orchestrator.md for this authorized pilot. Delegate implementation to a worker using the shared contract and roles/worker.md. Use the architect and independent reviewer when the contract calls for them. Carry the work through repair and verification, then return the result for my acceptance. Brief: [outcome; scope/edit boundaries; sources and UI references; acceptance criteria; commands/environment; authority/resource limits].

The coordinator passes the contract, appropriate role and bounded brief to each supported subagent. A handoff can be a tool message; no ticket file is required by this pilot. Include canonical source paths, current artifact/revision, dependencies and the expected return. A reviewer receives original criteria and artifacts first, then the builder's report for reconciliation.

Use the host's actual delegation and permission facilities. These Markdown files do not register agents or grant permissions. Where delegation is unavailable, separate sessions can receive the same prompts and exchange their returns manually; that requires a human relay and does **not** provide the fully autonomous middle. No new chat or cross-model runtime is opened automatically by this package.

## One prompt, adjustable execution

The same contract applies whether a worker uses Luna, Sonnet, Sol, Opus, Fable or another model. These names are not API identifiers or a ranking. Keep the selected runtime's configured model/effort; use its supported controls for any deliberate change. Adjust scope and support based on evidence from the task. Do not build a model-name switch into the quality contract.

## Feedback at useful boundaries

The loop is: inspect/challenge → implement and check → inspect the connected result → repair and recheck → close with evidence. These are responsibilities within existing work, not five mandatory agent calls. No separate ticket, debate transcript, fixed round count or human midpoint approval is required.

For example, a worker discovers that the brief calculates a pay period as if it were a workweek. They cite the existing business rule and propose a correction before building on that assumption. Quản đốc resolves the technical mismatch and informs affected workers; a change to the owner's policy goes back to the owner. Later, a reviewer finds export still uses the old grouping: the worker fixes export, verifies its content against an independent expected result, and Quản đốc closes the issue on the combined revision. No unrelated full-suite rerun is implied.

Select models by available capability and observed reliability, not a permanent brand ranking. Routine bounded work may use a cheaper proven worker; uncertain semantics, architecture or taste may warrant stronger reasoning or independent review. The acceptance bar stays the same. Do not buy more agents to compensate for missing context or an unusable environment.

When an actual miss or wasteful loop occurs, keep a short example in existing evidence: what happened, what should have happened, and the smallest proposed adjustment. Replay relevant examples when materially changing the harness/model, observing outcomes and unnecessary work rather than enforcing a fixed tool sequence. Add no general rule from one anecdote without checking its cause; retire a costly rule only after checking what protection is lost.

## Adopt without conflicting instructions

Before an app pilot, inspect its AGENTS/CLAUDE instructions, role wrappers, hooks, state schema and verification commands. Identify conflicts such as mandatory human phase approval, architect code-read bans, mandatory resets, historical pass flags treated as current readiness, or Stop/SubagentStop hooks that always run every suite. Reconcile the intended policy in an authorized, reviewed migration before activating Lite there; do not layer incompatible prompts or bypass an enforcing hook. Existing gates still apply until that migration; evidence reuse does not authorize skipping them. Preserve historical records and security boundaries. This folder alone changes none of those surfaces.

Put a short pointer in the project's existing entry instructions after reconciliation. Reuse its current state/evidence locations and name the authoritative sources in the brief. Copies need an explicit upstream revision and a chosen local owner; do not maintain both a copied contract and a second conflicting summary.

## Minimal tool wiring

Use project commands first; no new binary, mandatory ticket, AGENT_MAP or automatic test cache is required. The same CLI checks can serve different agent runtimes; Claude hook JSON is only a Claude adapter.

| Capability | Lite default |
| --- | --- |
| Verification | Quản đốc schedules existing targeted checks, `make check` or `make uitest` by risk/journey. `ship check` is an explicit configured check, not an E2E certificate. Do not run it in addition to an equivalent fresh check. |
| Stop events | No automatic full-suite `Stop`/`SubagentStop` command. A worker return alone does not trigger tests; critical verification remains required before claiming readiness. |
| Feature state | `claude-hooks features-guard --allow-regression` allows current pass status to turn false while retaining identities and structural checks. Record why in existing state/evidence; Git history preserves prior passes. Legacy callers without the flag keep their old behavior. |
| Documentation | Update actual interfaces, usage and consequential decisions. No mandatory fresh/staged CHANGELOG on every commit; `[docs_gate] enabled = false` in the app's `.ship.toml` and no unconditional pre-commit docs-gate. Keep documentation checks only for a named contract. |
| Protection | Retain relevant environment-edit, wording and secret checks. Run each distinct protection at its useful boundary; missing tooling is not a clean result. |
| Optional tools | `doc-rotate` when logs grow; `guard`/`vps`/canary for actual server deployments. Legacy lane/map checks and full security bundles are not Lite prerequisites. |

Adoption must update app instructions, role wrappers and Git guards together, verify the actual binary supports the configured flag, and start/reload the host session as required by that runtime. An already-running agent may still hold older instructions. Do not rewrite the kit's published tool manifest/checksums to describe an unreleased local build.

The current runner does not automatically decide test scope or reuse evidence: Quản đốc owns those judgments. Do not infer current quality from an old green log. Keep test commands and their environment explicit, and preserve the project's simulator lease.

## Example: five slices, checks driven by journeys

Suppose the work arrives as input, calculation, save, reopen and export. Quản đốc tracks the connected journeys in the existing brief, with credible failures such as a decimal changing value, a saved field disappearing or an exported total disagreeing.

1. Input and calculation: run focused validation and calculation tests with independent expected values; inspect input interaction/layout where changed. Do not launch the full app suite after each slice.
2. Save: run storage integration checks. If input → calculation → save is now meaningful and runnable, exercise that journey immediately and assert the stored data; do not wait for slice five.
3. Reopen: exercise save → close/reopen → edit using actual persistence. Check the resulting values; a rendered screen alone is insufficient.
4. Export: verify the produced file/content. Before product/candidate handoff to Chủ nhà, verify the combined revision's input → calculation → save → reopen → export journey, reusing earlier checks only where their evidence remains applicable.

This illustrative schedule uses fewer full UI-suite runs than one per slice. A risky shared-state change can require an earlier rerun; a UI-specific defect needs its targeted UI check. No duration or savings is claimed. Store command/setup, revision/environment and outcome in existing evidence; serialize simulator work using the project's current mechanism.

## Limits and pilot evaluation

All role behavior here is **guidance**, not mechanical enforcement. Existing runtime permissions and project gates remain the enforcement boundary. Review independence, UI taste and semantic correctness require judgment and actual access to artifacts; unavailable tools must remain visible limitations.

Try one bounded real change. Record defects caught/missed, false positives, verification gaps, time/cost and unnecessary human interruptions. A prompt read-through or scenario smoke check can expose instruction conflicts, but cannot establish production effectiveness or comparative model quality. Change one costly mechanism at a time, retaining evidence of what it protected.

### Observed pilot checks — 2026-10-05

- Bootstrap: initial authoring used the task brief; the worker then reread and used the produced contract and worker prompt for verification. The reviewer and smoke worker used the new prompts.
- A fresh-context reviewer found no blocking findings; checked 11 local links and the diff. This used the same runtime family, not a cross-model comparison.
- Quản đốc reported `docs-gate --all`: 46/46 checks passed. These are repository checks, not proof of behavioral quality.
- An isolated synthetic invoice smoke exposed a quantity defect despite the original tautological green test: 325 returned versus 850 independently expected.
- The smoke worker added five fixed-expectation tests: three failed before repair, then all five passed. Quản đốc independently reran all five and three direct acceptance assertions successfully.
- The smoke preserved historical S0 evidence while invalidating current readiness. It correctly withheld rollout readiness because a required printer check was deliberately unavailable; this was a synthetic requirement, not a real app blocker.
- Scope: prompt review and an isolated synthetic smoke only. No production adoption, full application E2E/UI verification, or comparative model effectiveness was established.

Revision note — 2026-10-05: subsequently added behavior-focused tests and journey-driven cadence. The observations above cover the earlier prompt revision; they do not validate this cadence on a real app.


### Local tool/app migration — 2026-10-05

Thirty and Tally now use the lean role/adaptor policy: no full-suite Stop/SubagentStop hook, no automatic per-commit docs gate, and explicit risk/journey verification on the integrated result. Product code, feature status and simulator configuration were not changed. App-local AGENTS and roles own their product-specific adaptation; these are not automatically synchronized copies of the kit.

`claude-hooks` 0.10.1 passed 114 tests and clippy; its locally built binary was installed and four direct CLI fixtures checked default regression rejection, opt-in acceptance, deletion rejection and malformed-container rejection before the app flags were activated. Tally's Git guard also checks feature structure, including boolean current status. Temporary app-hook fixtures exercised quality blocking and feature regressions without modifying app indexes. Independent review caught and repaired the Git/Claude validation mismatch and a stale current Stop-hook reference.

`ship` passed 61 tests, clippy, formatting and a release build. The installed release binary also passed isolated real `ship check` scenarios for shell quoting and timeout failure with subprocess cleanup. It remains a local build at the existing 0.1.1 version; its installed SHA-256 is `6924f4999f377d98bae5f77c84b55663b1ce99ba2e5b49e3199e1966a857376d`.

The app JSON/TOML, shell syntax and diff checks passed; Thirty's four pre-existing modified files retained their original hashes. Existing tool binaries are backed up locally before replacement; published kit pins remain unchanged. Reload existing agent sessions before relying on the new instructions.

Limits: no app build, simulator E2E or measured time/usage comparison was run for this configuration-only migration. Thirty's existing wording hook still checks working-tree content and warns/skips if its tool is missing; it is not equivalent to Tally's staged, fail-closed hook. Claude edit hooks do not cover shell writes. No claim of universal gate enforcement or autonomous test selection is made.


Feedback revision — 2026-10-05: made worker challenge, finding closure, role/model selection and loop stopping explicit. This revision is kit guidance only; the earlier Thirty/Tally migration does not automatically receive it. A separate CHALLENGE review found no actionable instruction conflicts; all 11 local Markdown links and whitespace checks passed. These are document checks only: behavioral effectiveness and cost remain to be measured on real work.
