# Setup Guide — SOS Kit

## Quick Start (5 minutes)

> **Fastest path — no Rust toolchain needed:** `curl -fsSL https://raw.githubusercontent.com/aspelldenny/sos-kit/main/install.sh | sh` downloads prebuilt binaries for all 10 tools (`ship`/`docs-gate`/`guard`/`vps`/`doctor`/`claude-hooks`/`advisory-inbox`/`inv-gate`/`doc-rotate`/`advisory-cron`) + the kit's own `sos` binary, clones the kit, and puts `sos` on PATH. See [`../INSTALL.md`](../INSTALL.md) for the full walkthrough. **The steps below are the `cargo install --path` dev path** — use them only if you're hacking on the Rust tool sources themselves; skip straight to step 2 otherwise.

### 1. Install Rust tools (dev path — building from source)

```bash
# Ship CLI — release pipeline
git clone https://github.com/aspelldenny/ship.git ~/tools/ship
cd ~/tools/ship && cargo install --path .

# docs-gate — pre-commit docs enforcement
git clone https://github.com/aspelldenny/docs-gate.git ~/tools/docs-gate
cd ~/tools/docs-gate && cargo install --path .

# guard — pre-deploy infrastructure gate
git clone https://github.com/aspelldenny/guard.git ~/tools/guard
cd ~/tools/guard && cargo install --path .

# vps — production ops (logs, status, restart)
git clone https://github.com/aspelldenny/vps.git ~/tools/vps
cd ~/tools/vps && cargo install --path .
vps init                 # generate ~/.vps.toml with your SSH + project paths
```

Verify:
```bash
ship --version
docs-gate --version
guard --version
vps --version
```

### 2. Install Claude Code skills (5 living — each with a declared mechanical caller)

```bash
# From this repo
# Chủ nhà layer
cp -r skills/init    ~/.claude/skills/init
# Kiến trúc sư layer
cp -r skills/forge   ~/.claude/skills/forge
# Thợ layer
cp -r skills/apply   ~/.claude/skills/apply
cp -r skills/retro   ~/.claude/skills/retro
```

8 more (`insight route decide plan verify review qa ship`) are parked in `skills/attic/` — no mechanical caller (see caller law in `CLAUDE.md`); not installed.

`/idea` is project-local (lives in `.claude/skills/idea/`) — copied per-project alongside the v2 subagent envelope, not installed globally.

See [`LAYERS.md`](./LAYERS.md) for which skill belongs to which layer.

### 3. Install phiếu shell function (ticket workflow)

```bash
# Source the phiếu shell function
echo "source ~/path/to/sos-kit/phieu/phieu.sh" >> ~/.zshrc
source ~/.zshrc

# Onboard each project you want the workflow on
phieu-init ~/my-project       # creates .phieu-counter, ~/my-project-wt/, updates .gitignore
```

Also copy the ticket template into each project:
```bash
mkdir -p ~/my-project/docs/ticket
cp ~/path/to/sos-kit/phieu/TICKET_TEMPLATE.md ~/my-project/docs/ticket/TICKET_TEMPLATE.md
```

See [`../phieu/README.md`](../phieu/README.md) for daily commands.

### 4. Setup each project you want SOS Kit to run on

Run these once per project. After this, `phieu`, `/plan`, `/verify`, `/ship`, `guard`, `vps` all work on that project.

```bash
cd ~/my-project

# 4a. Phiếu workflow — register project with counter + worktree dir
phieu-init .                # creates .phieu-counter, ~/my-project-wt/, updates .gitignore

# 4b. Copy ticket template into project
mkdir -p docs/ticket
cp ~/path/to/sos-kit/phieu/TICKET_TEMPLATE.md docs/ticket/TICKET_TEMPLATE.md

# 4c. Initialize Discoveries log (worker feedback to architect)
cat > docs/DISCOVERIES.md <<'EOF'
# Discoveries Log

> Worker → Architect feedback loop. Each entry records what the phiếu assumed vs. what the code actually was, plus edge cases found during implementation. Architect reads this BEFORE writing the next phiếu.
>
> Newest entries on top. See sos-kit `phieu/DISCOVERY_PROTOCOL.md` for format.

---

(no entries yet)
EOF

# 4d. Copy vision doc skeletons (Chủ nhà fills these iteratively)
mkdir -p docs
cp ~/path/to/sos-kit/phieu/VISION_TEMPLATES/PROJECT_template.md docs/PROJECT.md
cp ~/path/to/sos-kit/phieu/VISION_TEMPLATES/SOUL_template.md docs/SOUL.md
# CHARACTER.md only if the product has an AI character / named voice.
# If your project has a named character or multiple character/voice files,
# rename to docs/CHARACTER_<NAME>.md (e.g. docs/CHARACTER_CHI_HA.md).
# Architect globs docs/CHARACTER*.md and reads every match — naming is flexible.
cp ~/path/to/sos-kit/phieu/VISION_TEMPLATES/CHARACTER_template.md docs/CHARACTER.md

# 4e. Auto-generate ship + docs-gate configs
ship init                   # detects stack, generates .ship.toml
docs-gate init              # generates .docs-gate.toml

# 4f. One-time: global vps config (only needs to run once per machine,
#     not per project — skip if already done)
vps init                    # generates ~/.vps.toml

# 4g. Edit .ship.toml with your canary URL + deploy target
#     (see "Per-Stack Setup" below for stack-specific configs)

# 4h. Initialize security pipeline metadata (P040+)
# Detects stack (Node/Python/Rust/Go) via manifest files, writes .sos-stack.toml.
# Required before /advisory-scan (P041) or /security-review (P042) — those subagents
# read .sos-stack.toml to know which parser + which ecosystem to query.
sos init security
```

After these steps, your project is ready. Chủ nhà fills `docs/PROJECT.md` and `docs/SOUL.md` as vision firms up, then Architect in Claude Web can start writing phiếu.

**Env vars (bootstrap):** `sos new` runs `doctor verify-setup` as its post-bootstrap gate. It calls `doctor` on PATH by default; set **`DOCTOR_BIN=/path/to/doctor`** to point at a custom/local build (e.g. before `cargo install --path ~/doctor`). `SOS_KIT_DIR` (default: the sos-kit checkout) tells `sos new` where to copy the golden spine from.

**Rust cutover (P077e, workspace relocated to repo-root P077f):** the 6 heavy `sos` subcommands — `new`/`adopt`/`sync`/`map`/`install`/`tools` — now dispatch (`exec`) to the canonical Rust `sos` binary built from the repo-root workspace (`Cargo.toml` + `crates/`); `bin/sos.sh` fails LOUD (no silent Bash fallback) if it can't resolve one. For a sos-kit dev checkout: `cargo build --bin sos` from the repo root (or set `SOS_RUST_BIN=/path/to/sos`). **Prebuilt-binary bootstrap for end users (P081 Stage 1, route A, shipped):** `install.sh` fetches a release `sos-<triple>` binary + `.sha256` companion from `.github/workflows/release.yml`'s tag-`v*` GitHub Release, verifies the checksum (fail-CLOSED — mismatch or missing aborts the install), and drops it at a `sos-bin` sidecar next to the generated `sos` wrapper (never at the wrapper's own path). The wrapper exports `SOS_RUST_BIN` with a `:=` default pointing at that sidecar — a user's own `export SOS_RUST_BIN=/path/to/sos` still takes precedence, since `:=` never overrides an already-set value. The 7 guidance subcommands (`init`/`blueprint`/`contract`/`apply`/`recipe`/`launch`/`status`) are unaffected — still plain Bash.

### 5. Install git hooks

The tracked `hooks/` dir holds `pre-commit` + `pre-push`. Activate by pointing git
at that dir — no copy into `.git/hooks/`, so the tracked hook IS the running hook
(edits are live immediately, no stale untracked copy):

```bash
bash scripts/install-hooks.sh      # sets: git config core.hooksPath hooks
```

`core.hooksPath` is local git state (not committed) → re-run after a fresh clone.
`sos new` runs this automatically on spawn.

**Pre-commit chain ([1/8]…[8/8]):**
The chain includes two agent-agnostic git-level gates (P049–P052 harvest) plus a content-integrity gate (P073):

**`[6/8]` no-code-on-default** (`scripts/no-code-on-default.sh`):
Blocks product code (`.ts`/`.rs`/`.py`/`.go`/`.swift`/etc.) committed directly on the
default branch — forcing a feature branch for code changes. Docs-only (`*.md`) commits
on the default branch remain allowed (kit maintenance, README fixes, doctrine edits).

- **Downstream repos** get this gate live after `sos new` copies `scripts/` + `hooks/`.
- **sos-kit itself** self-opts-out via the committed `.sos-state/sos-kit-self` marker
  (this repo commits maintenance scripts/docs directly to `main`).
- **Override** (intentional code-on-default, one-off): `touch .sos-state/allow-code-on-default`
  before commit, `rm` after. Do NOT use `--no-verify`.
- **Pattern derivation**: the gate reads `.sos-stack.toml` `type` to derive file-extension
  patterns. If absent, falls back to the full extension-union and **blocks** (greenfield
  commits on main are the primary failure target — ket P020 live failure).
- **fail-CLOSED if `scripts/no-code-on-default.sh` is missing** (P080x — ports P078i backstop
  semantics into the dev `[8/8]` hook): the hook's own else-branch now prints a loud `❌` and
  bumps `FAIL_COUNT` (commit blocked), instead of the old `⏭ skip` (commit silently allowed).
  A missing security-invariant guard must never mean "no check happened."

**`[3/8]` sub-check `3f` — lane-check-contract** (`scripts/lane-check-contract.sh`) — P082:
Runs `doctor lane-check --ticket phieu/TICKET_TEMPLATE.md` whenever `phieu/TICKET_TEMPLATE.md` is staged, guarding against the template silently losing its `**Lane:**` field (OA-01 regression). Exit-code mapping: `doctor` exit 2 (missing/unparseable field) → **FAIL LOUD**; exit 1 (over Normal budget) → WARN, does not block; exit 0 → OK. `doctor` absent from PATH → degraded **warn-skip** (`exit 0`), fresh-env friendly — this is NOT a security gate, so it does not fail-closed. Does not add a new phase — stays a sub-check inside `[3/8]`.

**`[7/8]` block-env-commit** (`scripts/block-env-commit.sh`) — P052:
Blocks staging any `.env*` file so a secret-bearing env file cannot enter git history
(irreversible). Matches on **basename** across the full staged path — so `config/.env.docker`
is caught, not just root-level `.env`. `.env.example` (the template) is the only allowlisted
exception. Agent-agnostic git-level backstop to the Claude-only PreToolUse `block-env-edit.sh`
(P046); both layers share the same regex `^\.env($|\.)` so they cannot drift.

- **`.envrc` (direnv) is deliberately NOT covered** — it is usually committed on purpose
  (it points at `.env` to load secrets); blocking it = false-positive. See P052 Debate Log [O1.1].
- **sos-kit does NOT self-opt-out** — unlike the no-code gate, sos-kit also must never commit
  a `.env*`; the gate runs live in the kit (near-no-op by absence of any tracked `.env*`).
- **Override** (rare, intentional, you accept the irreversible leak): `touch .sos-state/allow-env-commit`
  before commit, `rm` after. Do NOT use `--no-verify`.
- **fail-CLOSED if `scripts/block-env-commit.sh` is missing** (P080x — same fix as `[6/8]`
  above, ports P078i backstop semantics into the dev `[8/8]` hook): missing guard → loud `❌`
  + `FAIL_COUNT` bump (commit blocked), not the old `⏭ skip` (commit silently allowed). This
  closes the round-1 P080 dogfood gap D1 (live-verified: deleting the guard used to let a real
  `.env` commit through with exit 0).

**`[8/8]` trust-gate** (`scripts/trust-gate.sh`) — P073:
Provides content-integrity for auto-exec surfaces (hooks, scripts, `.mcp.json`, `.claude/settings.json`, `install.sh`, `phieu/phieu.sh`, `templates/setup-dev.sh`). Two checks in one phase:

1. **Baseline-diff**: compares sha256 of each tracked auto-exec surface against `.sos-trust-baseline`. Any changed/added/removed surface BLOCKS the commit with a clear message naming the offending file(s).
2. **Hidden-unicode scan**: scans instruction/doc files for U+FEFF BOM, zero-width, bidi, and tag-range codepoints (prompt injection vector). Any hit BLOCKS the commit with `file:line`.

**Rebaseline workflow** (when a legitimate auto-exec surface change is reviewed and accepted):

```bash
# 1. Edit the surface (e.g. hooks/pre-commit, scripts/*.sh, install.sh)
# 2. Stage all changes
git add <changed-surface-file>
# IMPORTANT: new auto-exec files must be `git add`ed BEFORE rebaseline —
# `git ls-files` only sees tracked files; untracked = silently missed.
# 3. Regenerate the baseline
scripts/trust-gate.sh rebaseline
# 4. Stage the updated baseline
git add .sos-trust-baseline
# 5. Commit — trust-gate passes
git commit -m "feat(...): ..."
```

The `.sos-trust-baseline` diff in the PR is the human-readable audit trail of which auto-exec surfaces changed.

- **sos-kit self-tracks**: `trust-gate.sh` itself is an auto-exec surface tracked in its own baseline (a malicious edit to the gate is exactly what we want to catch).
- **`settings.local.json` excluded**: globally gitignored (per-machine), not in baseline. See `SECURITY.md` for rationale.

### 5a. Bootstrap `docs-gate` config

The pre-commit hook invokes `docs-gate` to verify documentation hygiene. On a fresh repo, generate the config:

```bash
docs-gate init
```

Or copy the reference template and tune it:

```bash
cp ~/path/to/sos-kit/templates/.docs-gate.toml .docs-gate.toml
# then tune docs_dir / changelog / [architecture] for your repo
```

If `.docs-gate.toml` is absent, the hook prints a yellow warning and skips the docs-gate check — other checks (type-check, BACKLOG, Discovery) still run. No hard fail on missing config.

### 5b. Windows checkout — EOL + symlinks (P088)

Windows' git defaults (`core.autocrlf=true`, `core.symlinks=false`) fight two things this kit relies on: byte-stable text checkouts and per-file symlinks. Both are fixed structurally in `.gitattributes` / documented here — you don't need to touch `core.autocrlf` yourself, but `core.symlinks` needs one manual step.

**EOL (handled automatically):** `.gitattributes` force-LFs `*.sh`/`*.bash`/`*.py`/`hooks/*`/`bin/*` (P059) plus `*.golden`/`*.toml`/`*.json`/`*.md`/`*.yaml` (P088) — every checkout (macOS/Linux/Windows) gets LF for these families regardless of `core.autocrlf`. This also keeps `.claude/settings.json`/`.mcp.json` sha256 stable across platforms for the trust-gate baseline (P087).

- **Existing checkout, pulling this fix for the first time:** run `git add --renormalize .` then commit, OR simply re-clone. Renormalize only updates the git index — if a file still shows CRLF on disk afterward, force a re-materialize: delete it and `git checkout -- <path>` (or `git checkout -- .` for everything).
- **Fresh clone:** nothing to do — checkout is already LF for the tracked families above.

**Symlinks (needs one manual step):** `.claude/agents/`, `.claude/commands/`, and `.claude/skills/<name>` are tracked as real git symlinks (mirrors the `.claude/agents/ -> agents/` convention — see `agents/README.md`). Without Windows support enabled, git materializes each as a tiny **text file** containing the literal link-target path (e.g. `../../skills/apply`, ~17-18 bytes) instead of the real content — Claude Code skills/agents silently fail to load.

Fix (one-time per machine):
1. Enable **Windows Developer Mode** (Settings → Privacy & Security → For developers) — lets git create symlinks without elevated/Administrator privileges.
2. `git config --global core.symlinks true`
3. Re-clone the repo (or delete + `git checkout -- .claude/` on an existing checkout).

No Developer Mode available? Use WSL, or manually copy the real file content over each stub (`agents/<name>.md` → `.claude/agents/<name>.md`, `skills/<name>/` → `.claude/skills/<name>/`) — you'll need to repeat this after every `sos sync`/`sos adopt` that touches those paths, since it's not tracked as a real symlink on your machine.

**This applies to BOTH checkouts, not just yours:** if you (or anyone) uses a Windows checkout of sos-kit itself as the `SOS_KIT_DIR`/`--kit` source for `sos new`/`sos adopt`, and that checkout still has the text-stub symlinks, the scaffolder will copy the *stub text* into every new project instead of the real skill/agent content — with no error. `sos new` and `sos adopt` both print a warning + the fix above automatically if they detect stub files in the kit source or the scaffolded target (mechanical net, no separate command to run).

### 6. Add canary to GitHub Actions

Copy the snippet from `integrations/github-actions/canary.yml` into your deploy workflow.

## Per-Stack Setup

### Next.js
```bash
ship init                        # detects Next.js, sets pnpm test --run
docs-gate init                   # detects docs structure
```

Edit `.ship.toml`:
```toml
[canary]
url = "https://your-app.com"

[deploy]
provider = "ssh"
ssh = "deploy@your-server.com:22"
command = "cd /opt/app && git pull && docker compose build && docker compose up -d"
```

### Flask
```bash
ship init                        # detects Flask, sets pytest
```

Edit `.ship.toml`:
```toml
[canary]
url = "https://your-app.onrender.com"

[deploy]
provider = "render"              # auto-deploys on push
```

### Rust
```bash
ship init                        # detects Rust, sets cargo test
```

Edit `.ship.toml`:
```toml
[docs_gate]
blocking = true                  # strict for Rust projects

[deploy]
provider = "cargo"               # cargo publish
```

## Optional: Uptime Monitor (Telegram Bot)

If you have a Telegram bot running 24/7, add the uptime monitor from `integrations/jarvis/uptime_monitor.py`. It pings your production URL every 10 minutes and alerts you on Telegram if it goes down.

## Security pipeline (P040 + P041 + P042)

Once your project has shipped its first version, optionally enable the security pipeline:

1. **Install PyYAML** (one-time, required by pnpm-lock parser):
   ```bash
   python3 -c 'import yaml' || pip3 install pyyaml
   ```
   (Trinh sát subagent also runs this check at Bước 0 — but pre-installing keeps the first scan smooth.)

2. **Detect stack** (one-time per project):
   ```bash
   sos init security
   ```
   Writes `.sos-stack.toml` documenting which package manifest + lock file your project uses. See P040 ship notes.

3. **Run advisory scan** (manual or via cron):
   In Claude Code session: `/advisory-scan`
   This spawns the Trinh sát subagent (read-only-output, scoped Bash for parser invocation) which queries GitHub Advisory Database + vendor pages, matches advisories against your resolved dep versions, and appends results to `docs/security/advisory-inbox.md`.

4. **Review inbox** (Chủ nhà):
   Open `docs/security/advisory-inbox.md`. For each row, either:
   - Mark status `dismissed` (false positive or unaffected code path), or
   - Create a follow-on phiếu via `phieu <slug>` to patch.

Currently implemented parsers: pnpm v9 + npm v3 (P041). Other ecosystems (pip, cargo, go) have stubs only — implementation deferred to follow-on phiếu.

5. **Pre-merge security boundary check** (manual or on each PR):
   In Claude Code session: `/security-review <PR-number>` (or `/security-review <branch>` / `<range>` / no-arg = current branch vs main).
   This spawns the Giám sát subagent (read-only-output, scoped Bash for `git diff` + `grep` only) which checks the diff against 5 generic INV (env var template / external service timeout / cross-user binding / webhook signature / dep major changelog audit) and posts an ADVISORY comment to the PR. KHÔNG block merge — Chủ nhà reads comment and decides.

   Extend the INV catalog with project-specific INV-6+ by copying `templates/INVARIANTS-template.md` to your project (typically `docs/security/INVARIANTS.md`) and filling the "User-added INV" section.

## Verify Setup

```bash
# In your project directory:
ship check               # preflight + test results
ship canary              # health check of production URL
docs-gate                # docs compliance pass/fail
docs-gate --verbose      # show all check details
guard --dry-run          # pre-deploy checks (no SSH)
vps status               # production container status (needs ~/.vps.toml)
```
