# Setup Guide — SOS Kit

## Quick Start (5 minutes)

### 1. Install Rust tools

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

### 2. Install Claude Code skills (all 3 layers)

```bash
# From this repo
# Chủ nhà layer
cp -r skills/init    ~/.claude/skills/init
cp -r skills/insight ~/.claude/skills/insight
cp -r skills/route   ~/.claude/skills/route
cp -r skills/decide  ~/.claude/skills/decide
# Kiến trúc sư layer
cp -r skills/plan    ~/.claude/skills/plan
cp -r skills/forge   ~/.claude/skills/forge
# Thợ layer
cp -r skills/verify  ~/.claude/skills/verify
cp -r skills/apply   ~/.claude/skills/apply
cp -r skills/review  ~/.claude/skills/review
cp -r skills/qa      ~/.claude/skills/qa
cp -r skills/ship    ~/.claude/skills/ship
cp -r skills/retro   ~/.claude/skills/retro
```

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

### 5. Install pre-commit hook

```bash
mkdir -p .githooks
cp ~/path/to/sos-kit/hooks/pre-commit .githooks/pre-commit
chmod +x .githooks/pre-commit
git config core.hooksPath .githooks
```

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
