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
cp -r skills/insight ~/.claude/skills/insight
cp -r skills/route   ~/.claude/skills/route
cp -r skills/decide  ~/.claude/skills/decide
# Kiến trúc sư layer
cp -r skills/plan    ~/.claude/skills/plan
# Thợ layer
cp -r skills/verify  ~/.claude/skills/verify
cp -r skills/review  ~/.claude/skills/review
cp -r skills/qa      ~/.claude/skills/qa
cp -r skills/ship    ~/.claude/skills/ship
cp -r skills/retro   ~/.claude/skills/retro
```

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

### 4. Setup your project

```bash
cd my-project
ship init           # generates .ship.toml
docs-gate init      # generates .docs-gate.toml
```

### 5. Install pre-commit hook

```bash
mkdir -p .githooks
cp ~/path/to/sos-kit/hooks/pre-commit .githooks/pre-commit
chmod +x .githooks/pre-commit
git config core.hooksPath .githooks
```

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
