# Security Policy — SOS Kit (v3)

The v2 policy (sister-tool binaries, Codex adapter guards, rendered backstop hooks) is archived at `archive/v2/docs/SECURITY-v2.md`.

## What runs automatically

### In this repository (for kit contributors)

| Surface | Trigger | What it does |
|---|---|---|
| `hooks/pre-commit` | `git commit` | `cargo check`, the trust gate, then `sos gate all` from the freshly built binary |
| `hooks/pre-push` | `git push` | `sos gate local-secrets` |
| `.claude/settings.json`, `.codex/hooks.json` | Agent session in this folder | Session status line; `.env` edit guard (via `adapters/`) |
| `scripts/*`, `adapters/*/*` | Called by the hooks above | Agent-neutral checks and payload translators |
| `install.sh` | `curl … \| sh` | Downloads the `sos` binary for one pinned release tag, verifies its `.sha256`, puts it in `~/.local/bin` |
| `scripts/npm-postinstall.sh`, `bin/sos-npm` | `npm install -g sos-kit` | Fetch `install.sh` from a pinned tag, verify it against `scripts/install-sh.sha256`, run it |
| `templates/app/hooks/*` | Installed into apps by `sos install` | See below |

**Not auto-exec, but loaded by agents as instructions:** `harness-lite/*.md`, `templates/app/*.md`, `skills/`, `recipes/`, `docs/` (scanned for hidden Unicode by the trust gate). `archive/` is history and is not loaded.

### In a repository where you run `sos install`

`sos install` writes only files embedded in the binary (no network access):

| Written | Owner | Runs when |
|---|---|---|
| `harness-lite/hooks/pre-commit`, `pre-push` (+ `core.hooksPath`) | Kit | Every commit / push: `sos gate all`, `sos gate local-secrets` |
| `harness-lite/scripts/*`, `harness-lite/adapters/*` | Kit | Called by agent hooks; `scripts/advise` calls the `claude` or `codex` CLI after repeated test failures (disable: `SOS_ADVISOR=off`) |
| `.claude/settings.json`, `.codex/hooks.json` | Project (created only if missing) | Agent sessions |

It never overwrites an existing project file, never changes an existing `core.hooksPath` without `--force-hooks`, and records the SHA-256 of every kit file in `harness-lite/UPSTREAM.json`. `sos update` replaces only kit files whose hash still matches that record; an edited file is kept and the new version is written beside it as `.sos-new`.

## Invariants

**INV-TRUST-01 — Auto-exec content integrity.** Every tracked auto-exec surface in this repo has its SHA-256 in `.sos-trust-baseline`; `scripts/trust-gate.sh` fails the commit on any difference. A reviewed change is accepted only by `scripts/trust-gate.sh rebaseline`, so the diff is visible in the PR.

**INV-TRUST-02 — No hidden Unicode in instruction files.** The trust gate rejects BOM, zero-width and bidi control characters in files agents load as instructions (the "rules file backdoor" vector).

**INV-TRUST-03 — Gates do not fetch.** Git hooks and `sos gate` read local files and git only. The one networked component is the optional advisor, which sends the question, the latest test output and `git diff` (excluding `evidence/`) to the model CLI you already use.

**INV-TRUST-04 — Fail closed.** A gate that cannot run (missing `sos`, `gitleaks`, unreadable file, unparseable patch, unknown `.sos.toml` key) blocks instead of passing.

**INV-TRUST-05 — Verified downloads.** `install.sh` installs one binary for one pinned tag and aborts on a missing or mismatching checksum. npm pins both the tag and the hash of `install.sh`.

## Boundaries (what this does not protect)

- Agent hooks see tool calls, not shell writes; Codex runs no hooks inside custom-role subagents (`openai/codex#21753`). The git gates are the boundary for both.
- `--no-verify` skips git hooks. Server-side protection (branch rules, secret scanning, push protection) is the GitHub layer and is enabled on this repository.
- A compromised maintainer account can publish a malicious release. Review `.sos-trust-baseline` diffs and pin a tag you have read.

## Rebaseline workflow

```bash
git add <changed-surface>              # new files must be added first
scripts/trust-gate.sh rebaseline
git add .sos-trust-baseline
git commit
```

## Reporting a vulnerability

Open a [GitHub Security Advisory](https://github.com/aspelldenny/sos-kit/security/advisories/new) (private until fixed), or email the maintainer (see GitHub profile). Please do not open a public issue before a fix is available.
