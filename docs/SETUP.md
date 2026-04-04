# Setup Guide — SOS Kit

## Quick Start (5 minutes)

### 1. Install Rust tools

```bash
# Ship CLI
git clone https://github.com/aspelldenny/ship.git ~/tools/ship
cd ~/tools/ship && cargo install --path .

# docs-gate
git clone https://github.com/aspelldenny/docs-gate.git ~/tools/docs-gate
cd ~/tools/docs-gate && cargo install --path .
```

Verify:
```bash
ship --version    # ship 0.1.0
docs-gate --version  # docs-gate 0.1.0
```

### 2. Install Claude Code skills

```bash
# From this repo
cp -r skills/ship ~/.claude/skills/ship
cp -r skills/review ~/.claude/skills/review
cp -r skills/qa ~/.claude/skills/qa
cp -r skills/retro ~/.claude/skills/retro
```

### 3. Setup your project

```bash
cd my-project
ship init           # generates .ship.toml
docs-gate init      # generates .docs-gate.toml
```

### 4. Install pre-commit hook

```bash
mkdir -p .githooks
cp ~/path/to/sos-kit/hooks/pre-commit .githooks/pre-commit
chmod +x .githooks/pre-commit
git config core.hooksPath .githooks
```

### 5. Add canary to GitHub Actions

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
ship check          # should show preflight + test results
ship canary         # should show health check
docs-gate           # should show pass/fail
docs-gate --verbose # show all check details
```
