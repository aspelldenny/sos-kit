---
name: apply
description: |
  Apply one SOS Kit recipe (an implementation pattern taken from shipped code, such as payment/payos-vn, auth/nextauth-google-credentials, infra/pii-encryption) to the current project: check its inputs against the real code, adapt and implement its steps, run its verification anchors, and report evidence.
  Use when the user or Quản đốc says "apply recipe X", "áp recipe X", "dùng recipe X", or a brief names a recipe.
---

# apply — use a recipe in this project

A recipe is one Markdown file in SOS Kit's `recipes/<category>/<name>.md`: purpose, inputs, outputs, steps, verification anchors, discovery hooks (known ways it goes wrong), env vars, sources. It was extracted from a shipped project; your job is to fit it to *this* project, not to paste it.

You act as Thợ under `harness-lite/CONTRACT.md` and `harness-lite/roles/worker.md` when the project has them.

## Find the recipe

1. A local sos-kit checkout: `$SOS_KIT_DIR/recipes/` (default `~/sos-kit/recipes/`).
2. Otherwise `https://github.com/aspelldenny/sos-kit/tree/main/recipes` (read-only).

If the name does not match, list the available recipes and ask which one. Do not improvise a recipe that does not exist.

## Steps

1. **Read the whole recipe**, including discovery hooks and the "Last verified" date. An old date or a fast-moving dependency means checking the current official docs for breaking changes before you start.
2. **Check inputs against reality.** For each input: is the prerequisite actually in the code (search for it), is the runtime version compatible, does the project already have a different solution for the same job? Env vars: check names in `.env.example` only; never open `.env` (it holds live secrets). Ask Chủ nhà to confirm a real value exists when you need one.
3. **Report mismatches before building.** If the project differs from the recipe's assumptions (other ORM, other auth, existing payment table), say what differs and propose the adaptation. Continue with independent parts; do not silently re-architect.
4. **Implement within the recipe's outputs.** Read each file before editing it. No refactors or extras "while you are there". Put new env var names with placeholder values in `.env.example`.
5. **Run every verification anchor** from the recipe, plus the project's own relevant tests. Write tests with independently computed expectations for money, time and security logic. A failing anchor means the recipe is not applied; fix the cause and rerun.
6. **Commit** through the normal git gates (one recipe per commit or PR). Never bypass them.

## Hand back

- Recipe and its "Last verified" date; what you adapted and why.
- Files changed; commands run with their results (anchors and tests).
- Discovery: which hooks applied, anything new the recipe did not anticipate, and whether the recipe itself needs an update. Do not edit the recipe here; recipe changes go to sos-kit as a separate, verified change.
- Remaining limits (an anchor you could not run, a sandbox-only payment test, a missing real credential).
