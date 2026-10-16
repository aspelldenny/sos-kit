---
name: depth-check
description: |
  Check whether a project, plan or agent role is deep enough: does the work start from the rawest source, use the field's real instruments, and get judged by something outside the model; are there hard targets, and does it aim at the real outcome (altitude) with the right neighbouring fields (range). Read-only; ends with a judge table and at most five ordered fixes.
  Use when the user or Quản đốc says "depth check", "kiểm độ sâu", "đủ sâu chưa", "thiếu độ sâu hay chiều rộng", "theo Felix", before a Hub brief or a BLUEPRINT, when designing a new role or persona, and before dogfood or ship.
---

# depth-check — is the expertise real, or a costume?

Lens: Felix Rieseberg, "How to be more ambitious: a tutorial for humans with models" (felixrieseberg.com, read 2026-10-15). Four kinds of ambition: **Speed** (same work, faster), **Altitude** (aim at the outcome, not the task), **Depth** (work like the best in the field, until experts could learn from it), **Range** (bring in other fields). His warnings: a persona changes vocabulary, not method; a model must not grade its own work.

Working definition of an expert, used below. A real expert has four layers:

1. **Knowledge**: the concepts and principles of the field. Books and the Hub supply this.
2. **Instruments and procedure**: what they open, measure and run, in what order.
3. **Calibrated judgment**: many repetitions with fast, clear feedback, so intuition matches reality. No book supplies this; only a feedback loop does.
4. **Known limits**: they know when they are guessing.

An agent built from books plus "you are X" has layer 1 only. Depth means building layers 2 and 3: instruments in code, and an outside judge that answers often.

## Scope

Input: a repo, a plan or brief, or one role/persona file. Read-only: do not edit the project. On a large repo, give each area to a read-only subagent with the questions below, then spot-check every claim you will repeat (open the file, run the grep). Say which claims you checked yourself.

## Steps

1. **Outcome and promise (Altitude).** Write one line for the product's promise as the project states it, and one line for the real outcome above it (a paying user who keeps using it, a reader who reaches chapter 3, positive expectancy after costs). If the work optimises something below that outcome, say so.

2. **Roles.** List every agent role and persona. Put each in one of three types:
   - **voice**: the role *is* the product's voice (a character, a narrator). Persona text is legitimate; method still belongs in steps and gates, not in the biography.
   - **job**: a division of labour (writer, editor, builder, analyst). Fine when the file is mostly procedure, inputs and outputs. Estimate the share of persona/biography against concrete steps.
   - **simulated audience or judge**: a model pretending to be a user, reader, viewer or expert critic. Useful for finding faults; never allowed to conclude "pass", "good" or a score.
   Flag any role whose quality claim rests on "Bạn LÀ… / You ARE… / You are a [expert]" rather than on what it runs and checks.

3. **Judge table.** For each output that matters, fill one row. Leave nothing implied.

   | Output | Rawest source available | Source actually used | Instrument (in code?) | Who judges | Independent of the maker? | Loop closed? |
   |---|---|---|---|---|---|---|

   - *Rawest source*: the thing itself (card images, the official PDF, tick data, real transcripts, platform exports), not a summary of it.
   - *Instrument*: what a top professional would run (backtest, statistical test, tap/time count, loudness meter, schema check). Mark "on the shelf" when the method exists only in books or docs.
   - *Independent*: a judge is not independent if it is the same model family, or if its knowledge came from **the same source** as the work (fixtures, persona rules and the reviewer's answers all written by one research pass agree with each other by construction).
   - *Loop closed*: a threshold or prediction was written **before** the result, and someone compared it with the actual result and recorded pass/fail and the lesson.
   - *Unknown model family*: when independence depends on which model produced a verdict (often a research pass or Hub output that does not record it), write "chưa rõ", not "independent" or "not independent". Recommend that the producer stamp model and family on each verdict it writes.
   - *Outcome data outside the repo*: real-user numbers often live in App Store Connect, RevenueCat, Sentry, analytics or the production DB. Before concluding the real judge is silent, name where that data would live, say "không thấy trong repo", and ask the owner for an export. Absent from the repo is not absent.

4. **Correlated sources.** Trace where the rules, answers and judges' knowledge come from. If everything descends from one research pass or one model, propose one independent check: a different model family deriving answers blind from the raw source, a second dataset, or a real person.

5. **Predictions and targets.** Find written thresholds and predictions; check whether each was graded against the actuals, and list the ungraded ones with where they were written and when they fall due. The default fix is a trigger, not memory: put a "due predictions" list into something that already runs or is already read (the daily report, the session-start status, the weekly review), not a new tool. Note whether a numeric, "unreasonable but reachable" target exists for the product's promise (for example: every table action in at most 2 taps). A promise with no number cannot be gated.

6. **Feedback speed.** Compare how often the judge answers with how often decisions are made. A loop that needs months for a sample (live trades, slow organic views) needs a faster proxy: replay history, count operations, run the deterministic part offline.

7. **Range.** Replace the product noun with a verb ("a sheet app" → "keeping a resource ledger while the table waits"). Name at most three fields that do that verb for a living and one practice each could lend. Keep only what could change a decision.

8. **Deepen the top gap.** For the one or two biggest gaps, answer: what would the best professional do beyond what was asked; what is the rawest material they would start from; which methods and tools would they use; what besides the model or the owner could confirm the result. Turn the answers into a concrete next step.

## Red flags seen in real projects

- An "expert" persona prompt that decides quality, while the good behaviour actually comes from code gates added after the model failed.
- A simulated reader or user panel that passes work the owner then fails (it decodes dense prose for free; it is more patient and cleverer than people).
- Fixtures, persona rules and rule answers that all come from one source, so "matches" proves nothing.
- Success thresholds written in every brief, results reported later, and no one comparing the two.
- Book knowledge entering as citations in a prompt, never as a computed check.
- The field's instruments (backtest, overfitting tests, task-time models) present in the library, absent from the code.
- A judge that answers far more slowly than decisions change, so the sample never grows.
- A simulated user that wanders freely and never exercises the core promise (never pressed Undo).
- "Image-first" or "data-first" doctrine while the model never receives the image or the data.

## What is fine

- Voice roles; job roles that are mostly procedure.
- Simulated audiences used to find faults before real people or the owner read the work.
- The owner as the one real reader or user. Calibrate any simulated panel against the owner's recorded verdicts: how many of the owner's faults it caught, how many it raised that the owner rejected. Keep the parts that track the owner. Look for verdicts already on record first (an owner column in a verdict table, owner bug reports, cold-read notes) and count them before proposing a new calibration round; note when the owner may have read the panel's verdict before giving theirs.
- The owner as a real *novice* user when they do not know the domain; their confusion is real data.

## Report

Write to the user in their language. Do not score out of 10. Separate what you found, what you did not find, and what you did not check.

- **Domain facts from model memory** (what a symbol on a card means, what a regulation's form says, a rule of the game) are labelled "trí nhớ model, chưa kiểm" and sent through the project's own evidence path (for example the Hub's cite and verify) before anyone acts on them.
- **Common gaps go short.** When the top gap is one seen in most projects (no real-user numbers, every judge in one model family, ungraded thresholds), state it in a line or two with its evidence and spend the report on what is specific to this project.
- **Deliver the whole report as plain text in the final message**, before any closing question, button box or wrap-up call, so it is not hidden behind one. Then offer to save it as `docs/audits/<YYYY-MM-DD>-depth-check.md` in the project (one file per run; no other new files).

```
## Kết luận
<one or two lines: which axis is short (Depth / Altitude / Range / Speed) and where exactly>

## Đã đúng
<what already works, with file:line, so it is not removed by mistake>

## Bảng người chấm
<the table from step 3>

## Chỗ thiếu
<each gap: evidence (file:line or measured number) → why it matters at the real outcome>

## Nên làm, theo thứ tự
<at most five steps, cheapest high-value first; each names its instrument or judge>

## Chưa kiểm
<what you could not read or run, and claims taken from subagents without a spot-check>
```

Changes to the project go through that project's normal roles and gates. Changes to this skill go to sos-kit through a PR, citing the project and failure that prompted them.
