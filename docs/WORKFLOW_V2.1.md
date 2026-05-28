# Workflow v2.1 — Soul Signature SOS Kit

> **Status:** SPEC (chưa pilot)
> **Home:** `~/sos-kit/docs/WORKFLOW_V2.1.md` (durable doctrine — không rotate)
> **Pilot:** `advisory-inbox` (Rust binary, repo mới)
> **Replaces:** Workflow v1 (implicit doctrine rải trong Tarot CLAUDE.md + advisory-cron CLAUDE.md)

---

## §0. Provenance — báo cáo này được forge qua 4 round phản biện

| Round | Tác giả | Output |
|-------|---------|--------|
| 1 | Agent code (advisory-cron sprint) | Retrospective report P001-P010, 10 cải tiến đề xuất |
| 2 | Orchestrator Tarot (em — Quản đốc) | 5 phản biện + 3 missing (Sub-mech A/D/F tarot) |
| 3 | ChatGPT | Accept 8.5/10 + 2 chỉnh nhỏ + nâng câu invariant lên |
| 4 | Orchestrator Tarot | 3 nuance refinement (verifiable trigger / audit trail / allowlist file) |

Source rounds quote inline trong từng điểm để truy vết.

---

## §1. Mục đích

Workflow v2.1 tối ưu **token + latency mà KHÔNG giảm an toàn**, qua 3 nguyên tắc:

1. **Phân tầng rủi ro** — không phải PR nào cũng cần full security review
2. **Tool đo sự thật, LLM quyết định** — LLM không đếm test count / LOC / surface
3. **Rule mới có home rõ + trigger fire được + behavior verify được** — không có 3 thứ này = chưa ship

Slogan: **"Build hook → test fire → assert behavior."**

---

## §2. Sub-mechanism mapping (tarot precedent)

13 thứ port vào 6 sub-mechanism tarot (CLAUDE.md tarot §AI BIAS WARNINGS):

| Thứ port | Sub-mech tarot | Lý do |
|----------|----------------|-------|
| 1. Risk lanes | — (new) | Không có precedent tarot |
| 2. Objection taxonomy | B (Capability gap) | Mechanical vs shape vs security có capability check khác nhau |
| 3. Surgical Turn 2 | — (new) | Anti-bloat rule |
| 4. Tool availability preflight | B (Capability gap) | P006 context7 architect không có tool |
| 5. Trigger structure (INV-WF-001) | **A (Trigger gap)** | P281+P297 instance #1+#9 — luật có nhưng không fire |
| 6. Deterministic classifier | B (Capability gap) | LLM classify = drift, tool = deterministic |
| 7. AGENT_MAP + validator | D (Persistence lifecycle) | YAML rename drift với code rename |
| 8. Knowledge durability home | **D (Persistence lifecycle)** | Doctrine vào DISCOVERIES rotate → lost |
| 9. Runtime state preflight | **F (Runtime state gap)** | P305 .git/config + P001 GITHUB_TOKEN inherit |
| 10. Lane override audit | D (Persistence lifecycle) | Audit trail durable trong git |
| 11. Orchestrator KHÔNG amend technical | B (Capability gap) | Quản đốc không có verify tool |
| 12. Fast lane scope cứng | — (new) | Anti-skip rule |
| 13. max_attempts policy | — (new) | Operational sanity |

**4/6 sub-mech tarot** đã cover (A, B, D, F). C (Migration completeness) + E (Environment drift) chưa map — defer khi pilot Rust phơi instance.

---

## §3. The 13 — danh sách port đầy đủ

```
 1. Risk lanes 4-tier                  (round 1 raise, round 2 keep)
 2. Objection taxonomy 3-loại          (round 1 raise, round 2 keep)
 3. Surgical Turn 2                    (round 1 raise, round 2 keep)
 4. Tool availability preflight        (round 1 raise, round 2 keep)
 5. Verifiable triggers (INV-WF-001)   (round 2 — em raise, round 3 ChatGPT nâng lên invariant)
 6. Deterministic classifier           (round 2 — em raise, round 3 ChatGPT đồng ý mạnh)
 7. AGENT_MAP + validator              (round 1 raise, round 2 em phản biện, round 3 ChatGPT giữ + validator)
 8. Knowledge durability home          (round 2 — em raise, round 3 ChatGPT đồng ý + phân 4 tầng)
 9. Runtime state preflight            (round 2 — em raise, round 3 ChatGPT đồng ý + allowlist, round 4 em rename)
10. Lane override audit                (round 3 — ChatGPT raise, round 4 em refine PR body+commit)
11. Orchestrator KHÔNG amend technical (round 2 — em phản biện, round 3 ChatGPT đồng ý)
12. Fast lane scope cứng               (round 2 — em phản biện, round 3 ChatGPT đồng ý)
13. max_attempts policy                (round 1 raise, round 2 em recommend 5, round 3 ChatGPT đồng ý)
```

---

## §4. Risk lanes 4-tier

Mỗi PR đi đúng 1 trong 4 lane. Lane do **deterministic classifier** (§7) quyết, KHÔNG phải LLM.

### Fast lane

**Scope cứng (§13):** docs typo, comment polish, README nhỏ, non-architecture cleanup.
**KHÔNG fast lane:** scaffold, module boundary, CLI entry, config root, public API shape, dependency add.

**Flow:** `Worker → tests → merge` (skip Architect + skip security review)
**Trigger:** classifier output `risk_lane: "fast"`
**Audit:** PR body `Lane: fast` + commit tag `[lane: fast]`

### Normal lane

**Scope:** config schema, state schema, pure internal behavior, scaffold (per §13), refactor non-public, dep version bump minor/patch.

**Flow:** `Architect short → Worker challenge (Turn 1 broad) → Execute → tests → review-lite`
**Architect reading:** `ARCHITECT_BRIEF.md` + ticket + surface-matched docs (qua AGENT_MAP nếu có)
**Worker challenge:** Turn 1 broad, Turn 2 surgical only (§6)
**Security review:** lite (touched files + sliced diff + applicable invariants only)

### Guarded lane

**Scope:** scheduler / launchd / cron, process execution (`Command::spawn`), filesystem persistence, outbound HTTP, token/secret handling, MCP server, deploy automation, auth flow, payment flow, AI prompt change.

**Flow:** `Architect full → Worker CHALLENGE → RESPOND (if objections) → Worker Turn 2 surgical → Execute → security-review-full → approval gate`
**Architect reading:** full ARCHITECT_BRIEF + INVARIANTS + surface docs + applicable RESEARCH (chị Hạ surface)
**Security review:** full (`@agent-boundary-check` invocation mandatory)
**Approval gate:** human required, không auto-merge

### Locked lane

**Scope:** production deploy, secret rotation, destructive migration (DROP TABLE / DELETE / DROP COLUMN), auto-fix PR, schema breaking change, MCP credential change.

**Flow:** Full Guarded + **human approval explicit BEFORE Architect spawn** + **mandatory dry-run** + **rollback plan in ticket**
**Auto-merge:** PROHIBITED
**Hooks:** PreToolUse hook block bash command match destructive pattern (existing `block-unsafe-merge.sh` precedent)

---

## §5. Objection taxonomy 3-loại

[Round 1 raise, round 2-3 thống nhất]

Worker CHALLENGE phase phải tag objection bằng 1 trong 3 loại. Architect respond khác nhau theo loại.

### Mechanical objection

**Triệu chứng:** sai test count, sai path, sai baseline, sai wording nhỏ, KHÔNG đổi behavior/API/security.
**Ví dụ:** "Architect đếm baseline test 13, thật 33."

**Flow [round 3 ChatGPT chỉnh]:**
```
Worker: objection mechanical (tag: [mechanical])
Orchestrator: route to Architect short respond (KHÔNG tự amend)
Architect: ack one-line correction, KHÔNG full re-read doctrine
Worker Turn 2: OPTIONAL skip nếu correction không ảnh hưởng behavior
```

**Anti-pattern:** Orchestrator tự amend technical fact (phá vai — Quản đốc không có verify tool, §14).

### Implementation-shape objection

**Triệu chứng:** function signature sai, enum/newtype shape sai, trait method sai, return type sai, module boundary sai.
**Ví dụ:** "Architect spec inline enum, code thật dùng newtype `Register(register::Args)`."

**Flow:**
```
Worker: objection shape (tag: [shape])
Architect: RESPOND short — chỉ address objection, KHÔNG re-debate scope
Worker Turn 2: SURGICAL verify exact V2 corrections (§6)
Execute: sau khi V2 verify pass
```

**Tarot precedent:** P003 advisory-cron — đúng pattern này, ship clean.

### Design/security objection

**Triệu chứng:** spec không khả thi trên platform thật, mở surface mới, secret/token/logging risk, process treo vô hạn, scheduler/launchd side-effect, network/outbound HTTP, state persistence không an toàn.
**Ví dụ:** "`fire_task` không bọc `tokio::time::timeout` → launchd block vô hạn nếu child treo."

**Flow:**
```
Worker: objection design (tag: [design])
Architect: RESPOND full — re-read INVARIANTS + platform docs + spec doable
Worker Turn 2: BROAD re-verify (full V2 review)
Security review: full nếu chạm Guarded lane surface
Approval gate: required
```

---

## §6. Surgical Turn 2 rule

[Round 1 raise — pattern tốt từ P006 advisory-cron]

```
Turn 1 challenge = BROAD verification (full V1 review)
Architect RESPOND    = ONLY address objections (không mở scope mới)
Turn 2 challenge   = SURGICAL verify exact corrections (chỉ V2 changes)
                     → KHÔNG re-open whole debate
```

**Rule:** Turn 2 chỉ check những thứ Architect đã sửa từ V1 → V2. Mọi assumption đã pass Turn 1 KHÔNG cần re-verify Turn 2 trừ khi V2 changes ảnh hưởng.

**Exception:** Design/security objection (§5) → Turn 2 broad nếu V2 thay đổi scope/surface.

**Anti-pattern:** Worker Turn 2 re-grep tất cả file đã grep ở Turn 1 → bloat token, latency.

---

## §7. Tool availability preflight (Architect Bước 0)

[Round 1 raise — P006 context7 architect không có tool]

Trước khi Architect viết phiếu chứa instruction "use tool X to research":

```
1. Architect verify tool X có trong tool envelope không?
   - List actual tools available (frontmatter `tools:` của agent)
   - Match yêu cầu phiếu với danh sách
2. Nếu KHÔNG có:
   - KHÔNG được ghi "MUST research via tool X"
   - PHẢI ghi "[needs Worker Task 0 verify via <fallback>]"
   - Pass research task xuống Worker (Worker có Bash + Read + Grep)
3. Nếu CÓ:
   - Test query nhỏ trước (smoke test)
   - Nếu smoke fail (vd. MCP not connected) → treat như "không có"
```

**Anti-pattern:** Architect ghi "use context7 to verify rmcp API" trong khi runtime không có context7 → false confidence.

**Tarot precedent:** advisory-cron P006 — Architect đã trung thực mark `[needs Worker verify]`, Worker verify rmcp 1.7.0 viable. Pattern tốt, formalize.

---

## §8. Verifiable triggers — INV-WF-001

[Round 2 em raise, round 3 ChatGPT nâng thành invariant, round 4 em add ID]

### INV-WF-001 — Trigger Verifiability

> **Build hook → test fire → assert behavior.**
> **No trigger = not shipped.**

**Rule:** Mọi workflow tool / hook / classifier / doctor ship phải có:

1. **Trigger structure declared** — hook config / cron config / orchestrator auto-spawn / pre-commit
2. **Trigger fires** — verify bằng smoke test dry-run
3. **Behavior assert** — exit code đúng, output đúng, side-effect đúng

**Layer 2 capability check (mandatory cho mỗi trigger):**

```bash
# Example — block-env-edit.sh hook smoke test
echo '{"tool_name":"Edit","tool_input":{"file_path":".env.production"}}' \
  | bash hooks/block-env-edit.sh
echo "exit=$?"
# expect: exit=2 + stderr contains "blocked: .env*"
```

**Anti-pattern:**
- Hook file tồn tại nhưng chưa `chmod +x` (Sub-mech A instance — "future áo mới")
- Cron declared nhưng `if: false` chưa flip
- MCP configured nhưng OAuth chưa connected
- Function exported nhưng caller chưa import
- ORCHESTRATION rule tồn tại nhưng orchestrator quên triệu (instance #9 P297)

**Home:** `docs/RULES.md` invariant section. ID prefix `INV-WF-` reserved cho workflow invariants. Match tarot pattern INV-001..010 (mechanical) + INV-101..108 (judgment).

**Future invariants (placeholder):**
- INV-WF-002: Classifier output schema stability
- INV-WF-003: Lane override audit completeness
- INV-WF-004: (TBD post-pilot)

---

## §9. Deterministic classifier

[Round 2 em raise, round 3 ChatGPT đồng ý mạnh + spec rõ JSON, round 4 em add audit trail]

### Binary spec

```
lane-classifier classify --base origin/main --head HEAD
```

**Output JSON (stdout):**

```json
{
  "risk_lane": "guarded",
  "touched_surfaces": ["outbound_http", "secret_token", "config_schema"],
  "required_docs": [
    "ARCHITECT_BRIEF.md",
    "docs/security/INVARIANTS.md",
    "docs/security/SURFACE_MAP.md"
  ],
  "security_review": "full",
  "challenge_mode": "required",
  "reason_files": ["src/alert.rs", "Cargo.toml", "src/core/run.rs"],
  "classifier_version": "0.1.0"
}
```

**Logic:** grep file paths diff đối chiếu pattern table:

| Pattern | risk_lane | touched_surfaces |
|---------|-----------|------------------|
| `*.md` only (no INVARIANTS/SECURITY/deploy docs) | fast | docs |
| `*.toml` (config) + behavior code | normal | config_schema |
| `src/.*scheduler.*` / `*.plist` | guarded | scheduler |
| `src/.*alert.*` / `reqwest` add | guarded | outbound_http |
| `*.env*` / token regex | guarded | secret_token |
| `prisma/schema.prisma` migration DROP/DELETE | locked | destructive_migration |

### LLM override rule

LLM có thể request override classifier output **CHỈ KHI:**

1. Ghi lý do explicit trong PR body section `## Lane override` (§11)
2. Bị Worker CHALLENGE catch nếu override không hợp lệ
3. Override không được giảm xuống dưới `normal` nếu classifier output `guarded`/`locked`

**Default:** classifier output authoritative.

### Trigger structure (INV-WF-001 compliance)

```yaml
# .claude/settings.json hook
PreToolUse:
  - matcher: "Bash"
    hooks:
      - command: |
          if [[ "$BASH_COMMAND" =~ gh\ pr\ create ]]; then
            lane-classifier classify --base origin/main --head HEAD > /tmp/lane.json
            cat /tmp/lane.json
          fi
```

Or orchestrator auto-spawn trước Architect:
```
Orchestrator: spawn lane-classifier → read JSON → decide flow
```

---

## §10. AGENT_MAP + validator

[Round 1 raise, round 2 em phản biện scope, round 3 ChatGPT giữ + validator]

### Khi nào dùng AGENT_MAP

- **CÓ map:** repo có > 10 docs/, lịch sử docs phức tạp (Tarot scale)
- **KHÔNG map:** repo Rust nhỏ < 5 docs/ (grep convention trong CLAUDE.md đủ)

Decision khi setup repo. Default cho pilot `advisory-inbox` (Rust nhỏ): **KHÔNG map**, dùng grep convention.

### Schema YAML (khi cần)

```yaml
# AGENT_MAP.yaml
version: 1
surfaces:
  scheduler:
    docs:
      - ARCHITECT_BRIEF.md
      - docs/security/INVARIANTS.md#scheduler
      - docs/ARCHITECTURE.md#runner
    code:
      - src/runner/
      - src/core/run.rs
  outbound_http:
    docs:
      - docs/security/INVARIANTS.md#outbound
    code:
      - src/alert.rs
deprecated_docs:
  - docs/legacy/OLD_ARCHITECTURE.md
```

### Validator subcmd

```
docs-gate validate-map [--map AGENT_MAP.yaml]
```

**Checks:**
1. Mọi path trong `surfaces.*.docs` tồn tại
2. Mọi section anchor `#scheduler` tồn tại trong markdown
3. Map KHÔNG trỏ tới docs trong `deprecated_docs`
4. Mọi path trong `surfaces.*.code` tồn tại

**Exit code:** 0 pass, 1 drift detected.

### Trigger structure (INV-WF-001)

- **Pre-commit hook** (Husky): chạy `docs-gate validate-map` mỗi commit chạm `*.md` hoặc `AGENT_MAP.yaml`
- **CI workflow:** `docs-gate validate-map` trong CI matrix

---

## §11. Knowledge durability home — 4 tầng

[Round 2 em raise, round 3 ChatGPT phân 4 tầng]

Mọi rule mới / fact mới PHẢI có home explicit. KHÔNG có home = chưa ship.

| Tier | Type | Home | Rotate? |
|------|------|------|---------|
| **Durable doctrine** | Rule, invariant, taxonomy, pattern catalog | `ARCHITECT_BRIEF.md` / `docs/RULES.md` / `.claude/agents/*.md` / `AGENT_MAP.yaml` | **KHÔNG** |
| **Operational discoveries** | Instance debug, test failure narrative, anchor mismatch fix | `docs/DISCOVERIES.md` | Khi > 1000 dòng → `docs/archive/DISCOVERIES_<date>.md` |
| **Project debt** | Backlog item, deferred fix, advisory note | `docs/BACKLOG.md` | Khi item done → strikethrough, archive section quarterly |
| **Run-specific facts** | Heartbeat, runlog, scan state | `docs/runlog/*.jsonl` / `.advisory-scan-state` | Rotate per-run hoặc daily |

**Rule:** mỗi rule mới được introduce phải declare home trong commit message:

```
feat: add lane override audit rule

home: docs/RULES.md §11 (durable)
```

**Anti-pattern (Sub-mech D tarot precedent):**
- Ship doctrine vào `DISCOVERIES.md` rồi rotate → lost
- Ship rule vào commit message only (không file) → lost on squash
- Ship rule vào TODO comment trong code (không enforcement) → lost

---

## §12. Runtime state preflight

[Round 2 em raise (Sub-mech F tarot), round 3 ChatGPT add allowlist, round 4 em rename]

### File `.runtime-env.allowlist`

[Round 4 ChatGPT rename — tránh collision với `.env*` block-env-edit hook]

**Location:** `.tools/runtime-env.allowlist` hoặc `.runtime-env.allowlist` (project root).
**Committed:** YES (không chứa value, chỉ key names).
**Format:**

```
# .tools/runtime-env.allowlist
# Declare expected runtime env keys. Used by session-start preflight.

required:
  - OPENROUTER_API_KEY
  - ANTHROPIC_API_KEY

optional:
  - PAYOS_*
  - RESEND_API_KEY
  - SENTRY_DSN

forbidden:
  - GITHUB_TOKEN     # Use env -u to unset; gh CLI uses keychain auth
  - AWS_ACCESS_KEY_ID  # Project doesn't use AWS — inherited = leak risk
```

### Session-start check (3 layer)

```bash
# Layer 1 — Env key count + diff against allowlist
ALLOWLIST=$(yq '.required + .optional | .[]' .tools/runtime-env.allowlist)
FORBIDDEN=$(yq '.forbidden | .[]' .tools/runtime-env.allowlist)
RUNTIME_KEYS=$(printenv | cut -d= -f1)

# Forbidden check (hard block)
for key in $FORBIDDEN; do
  if printenv | grep -q "^${key}="; then
    echo "BLOCK: forbidden runtime key $key detected. Unset before continuing." >&2
    exit 2
  fi
done

# Unexpected key warning (soft)
UNEXPECTED=$(comm -23 <(echo "$RUNTIME_KEYS" | sort -u) <(echo "$ALLOWLIST" | sort -u))
UNEXPECTED_COUNT=$(echo "$UNEXPECTED" | grep -cE 'TOKEN|KEY|SECRET' || true)
if [ "$UNEXPECTED_COUNT" -gt 0 ]; then
  echo "WARN: $UNEXPECTED_COUNT unexpected runtime key(s) matching TOKEN/KEY/SECRET." >&2
fi

# Layer 2 — gh auth status
gh auth status 2>&1 | head -3

# Layer 3 — git config token-in-URL check (P305 tarot precedent)
git config --get-regexp 'http.*extraheader|credential|insteadOf' \
  | grep -qE 'ghp_|gho_|ghu_|ghs_|github_pat_' \
  && echo "BLOCK: token-in-config detected" >&2 && exit 2 \
  || true
```

**KHÔNG log secret value.** Chỉ log key names + counts.

### Trigger structure (INV-WF-001)

- **SessionStart hook** (Claude Code `.claude/settings.json`)
- **Pre-commit hook** for `.tools/runtime-env.allowlist` changes

---

## §13. Lane override audit

[Round 3 ChatGPT raise, round 4 em refine PR body + commit tag]

### PR body required section

`.github/pull_request_template.md` include:

```markdown
## Lane override

- original: N/A
- requested: N/A
- reason: N/A (no override)
- approved_by: N/A
```

Khi override:

```markdown
## Lane override

- original: guarded
- requested: normal
- reason: docs-only change, no runtime/security surface touched (verified: only docs/SECURITY.md unchanged)
- approved_by: orchestrator
```

### Commit tag format

```
feat(P042): foo bar

[lane-override: original=guarded requested=normal reason="docs-only"]
```

### Grep tooling

```bash
# PR body grep
gh pr view <N> --json body | jq -r .body | awk '/^## Lane override/,/^## /' | head -10

# Git log grep
git log --grep="lane-override" --oneline

# Audit rate (overrides per N PRs)
TOTAL=$(gh pr list --state merged --limit 50 | wc -l)
OVERRIDES=$(gh pr list --state merged --limit 50 --json body \
  | jq -r '.[].body' | grep -c "## Lane override" || true)
echo "Override rate: $OVERRIDES / $TOTAL"
```

**Threshold:** override rate > 20% → classifier rule sai, tune (raise to orchestrator).

---

## §14. Orchestrator KHÔNG amend technical fact

[Round 2 em phản biện, round 3 ChatGPT đồng ý]

### Rule

Orchestrator (Quản đốc) **KHÔNG** tự sửa technical content trong spec:
- KHÔNG sửa "test count 13 → 33"
- KHÔNG sửa function signature
- KHÔNG sửa path
- KHÔNG sửa wording technical
- KHÔNG sửa code/config

### Orchestrator được làm

- **Classify** objection (mechanical / shape / design — §5)
- **Route** to đúng agent (Architect short-respond / RESPOND full / spawn boundary-check)
- **Maintain state machine** (DRAFT → CHALLENGE → RESPOND → APPROVAL_GATE → EXECUTE)
- **Halt** nếu phát hiện loop / deadlock
- **Escalate** to Sếp khi gate fail

### Lý do

Orchestrator handbook (tarot pattern) defines vai trò = dàn xếp, KHÔNG verify code. Quản đốc không có guarantee về tool envelope (Bash + Read + Grep) ở mọi context. Nếu mở rule amend → ép Quản đốc gánh verify burden → phá vai → drift toward Worker/Architect.

### Flow chuẩn cho mechanical objection (§5)

```
Worker: objection [mechanical] "Architect đếm test 13, thật 33"
Orchestrator: classify → route to Architect (KHÔNG tự sửa)
Architect: Read source → ack "đúng, thật 33" → 1-line correction in ticket
Worker Turn 2: skip nếu correction không ảnh hưởng behavior
```

---

## §15. Fast lane scope cứng

[Round 2 em phản biện, round 3 ChatGPT đồng ý]

### Fast lane CHỈ cho

- Docs typo fix
- Comment polish (rewording, không đổi semantics)
- README updates không touch invariants/security
- Non-architecture cleanup (vd: trailing whitespace, import order)
- Dev tooling minor (eslint config, prettier config nếu không đổi rule)

### Fast lane KHÔNG cho

- **Scaffold** (chốt module boundary, naming, CLI entry)
- **Module boundary change** (move file giữa folders)
- **CLI entry shape** (add/remove subcommand, change arg signature)
- **Config root** (add config key new, change default)
- **Public API shape** (export new, change signature, deprecate)
- **Dependency add** (cargo add / pnpm add — KHÔNG fast lane kể cả minor)
- **Schema change** (Prisma, SQL, JSON schema)

### Lý do

Scaffold P001 (advisory-cron) — chốt module boundary từ đầu. Sai từ P001 = debt cả sprint. Architect skip ở scaffold → sai shape có thể đã chốt từ đầu, P003 enum/newtype mismatch chỉ là biểu hiện.

**Default cho scaffold:** Normal lane minimum (Architect short brief + Worker Turn 1 challenge).

---

## §16. max_attempts policy

[Round 1 raise, round 2 em recommend 5, round 3 ChatGPT đồng ý]

### Config validation rule

```toml
# Config TOML for any retry-capable component (cron, runner, alert, etc.)
[retry]
max_attempts = 1     # default
backoff_secs = 60    # default, sleep BETWEEN attempts only
```

**Validation:**

| `max_attempts` | Action |
|----------------|--------|
| `< 1` | Hard reject — config invalid |
| `1` | Default OK (no retry) |
| `2..5` | OK |
| `>= 6` | Soft warn — log warning, allow |
| `> 10` | Hard reject — config invalid |

**Backoff:**

| `backoff_secs` | Action |
|----------------|--------|
| `< 0` | Hard reject |
| `0` | OK (immediate retry — use sparingly) |
| `1..3600` | OK |
| `> 3600` | Hard reject (1 hour cap) |

### Behavior rules

- **No sleep after final attempt** — retry chỉ giữa attempts
- **Alert (Telegram, etc.) fires ONCE after final failure** — không spam N alerts per N attempts
- **Heartbeat per attempt** — schema preserved, không add field
- **Retryable exit codes:** `1..127`
- **Non-retryable exit codes:** `>= 128` (signal-like), `-1` (spawn fail)

### Lý do cap 5

- Daily cron 1 lần/ngày → 5 attempt × 60s backoff = 5 phút retry đủ
- Telegram alert 1 lần sau final → max_attempts cao chỉ ép launchd slot block lâu
- Sếp prefer "fail fast + alert" over "retry forever"

---

## §17. Migration plan

```
Phase 1 — Spec write (THIS FILE)               ← em đang ở đây
Phase 2 — Pilot setup: advisory-inbox repo
Phase 3 — Sếp chạy end-to-end zero check (smoke test workflow v2.1)
Phase 4 — Retrospective (em + ChatGPT review giống round 1-4)
Phase 5 — Nhét ngược sos-kit golden template (sửa ~/sos-kit/agents/*, recipes/, etc.)
Phase 6 — Mở rộng 2 repo còn lại (claude-hooks + inv-gate) với v2.1 đã verify
Phase 7 — Re-port lên Tarot (sửa CLAUDE.md tarot inherit từ sos-kit v2.1)
```

### Phase 2 — Pilot setup checklist

```
[ ] gh repo create aspelldenny/advisory-inbox --public --license MIT
[ ] cd ~/advisory-inbox && cargo init
[ ] cargo add clap serde serde_json tokio anyhow
[ ] Copy structure từ ~/advisory-cron:
    - .claude/agents/*
    - .claude/commands/*
    - .claude/hooks/*
    - .claude/settings.local.json
    - docs/PROJECT.md
    - docs/ARCHITECTURE.md
    - docs/ORCHESTRATION.md
    - docs/RULES.md       ← skeleton, sẽ port v2.1 vào
    - docs/BACKLOG.md
    - docs/CHANGELOG.md
    - docs/DISCOVERIES.md
    - docs/ticket/TICKET_TEMPLATE.md
    - scripts/architect-guard.sh
    - scripts/session-start-banner.sh
    - CLAUDE.md
[ ] Adapt nội dung v2.1:
    - CLAUDE.md: subcmd spec (parse-report / dedup / append / migrate-state / state-backfill / serve MCP)
    - docs/RULES.md: §3 The 13 + §4 lanes + §5 objection taxonomy + §8 INV-WF-001 + §14 orchestrator + §15 fast lane + §16 max_attempts
    - docs/ARCHITECTURE.md: subcmd architecture sketch + JSON I/O schema
    - docs/BACKLOG.md: P001..P00N phiếu cho 6 subcmd
    - .claude/agents/architect.md: §7 tool preflight + §6 surgical Turn 2
    - .claude/agents/worker.md: §5 objection taxonomy 3-loại
    - .claude/agents/orchestrator.md: §14 KHÔNG amend technical
    - .tools/runtime-env.allowlist: §12 schema (Rust repo subset)
[ ] Add `.github/pull_request_template.md` với §13 Lane override section
[ ] Initial commit + push
```

### Phase 3 — End-to-end zero check

Sếp chạy autonomous (như advisory-cron hôm qua). Em (orchestrator session) ship full sprint mà KHÔNG ping Sếp giữa chừng. Bug lộ ra qua:
- PR auto-merge sai (classifier output sai lane)
- Hook không fire (INV-WF-001 violation)
- Doctrine lost (Sub-mech D drift)
- Runtime state bẩn (Sub-mech F instance)

### Phase 4 — Retrospective format

Reuse `~/sos-kit/docs/WORKFLOW_V2.1_RETRO_<date>.md` (versioned). 4 round phản biện như v2.1 forge.

### Phase 5 — Nhét ngược sos-kit

Sửa:
- `~/sos-kit/CLAUDE.md` reference WORKFLOW_V2.1.md
- `~/sos-kit/agents/*.md` apply §5/§6/§7/§14
- `~/sos-kit/phieu/TICKET_TEMPLATE.md` add Lane field
- `~/sos-kit/recipes/*` audit cho fast/normal/guarded routing
- `~/sos-kit/hooks/*` apply INV-WF-001 verifiable trigger

---

## §18. Open questions / Future work

### Cần pilot phơi ra

1. **Classifier accuracy** — pattern table §9 đủ cover surface chưa? Override rate sẽ là proxy metric.
2. **AGENT_MAP threshold** — > 10 docs là cứng hay flexible? Pilot Rust < 5 docs sẽ confirm/refute.
3. **Sub-mech C (Migration completeness)** — chưa map trong §2. Pilot có expose không?
4. **Sub-mech E (Environment drift)** — Rust `cargo build` vs local dev có drift như pnpm không? Pilot test.

### v2.2 reserved cho

- New sub-mechanism G/H/... discovered post-pilot
- Cross-project tooling (claude-hooks + inv-gate + advisory-inbox shared lib?)
- sos-kit template generator (1 command bootstrap new Rust repo với v2.1 baked in)

### Không scope v2.1

- AI prompt safety classifier rewrite (defer — domain khác)
- Crisis classifier rewrite (defer)
- Tarot-specific phiếu workflow (chị Hạ Research Gate) — giữ nguyên trong Tarot CLAUDE.md, KHÔNG port lên sos-kit (project-specific)

---

## §19. Acceptance criteria cho v2.1 SHIP

```
[ ] Spec file viết xong (THIS FILE)                                       ← Phase 1 done
[ ] Sếp review + ack
[ ] Pilot advisory-inbox setup hoàn tất (Phase 2 checklist)
[ ] Sếp chạy end-to-end zero check, sprint 5-10 phiếu ship clean
[ ] Retrospective viết xong, không có Sub-mech mới chưa cover
[ ] 13 thứ port trong pilot ALL fire ít nhất 1 lần (verify INV-WF-001)
[ ] sos-kit golden template updated (Phase 5)
[ ] Tarot CLAUDE.md re-port (Phase 7) — optional, làm khi cần
```

---

## §20. Provenance chi tiết — round-by-round

### Round 1 — Agent code retrospective (advisory-cron P001-P010)

10 đề xuất ban đầu:
1. Lane routing (Fast/Normal/Guarded/Locked)
2. Required classifier
3. Architect reading policy
4. Objection taxonomy
5. Surgical Turn 2
6. Tool availability preflight
7. Tool đo, LLM không đếm
8. AGENT_MAP.yaml
9. PR diff fallback helper
10. max_attempts cap (10-20)

### Round 2 — Orchestrator Tarot (em) phản biện

5 phản biện:
1. AGENT_MAP drift risk (Sub-mech D)
2. Orchestrator amend mechanical = phá vai
3. Fast lane scaffold = chốt sai từ đầu
4. Classifier phải tool, không LLM
5. max_attempts cap 5, không 10-20

3 missing:
- M1: Sub-mech A (Trigger gap)
- M2: Knowledge durability routing
- M3: Sub-mech F (Runtime state gap)

### Round 3 — ChatGPT phản biện em

Verdict 8.5/10. Accept hầu hết. 2 chỉnh:
1. AGENT_MAP cho Tarot phải có validator (em đã underscope chỉ Rust nhỏ)
2. PR body audit + commit tag (defense-in-depth squash merge)

Nâng câu invariant: "Build hook → test fire → assert behavior."

### Round 4 — Orchestrator Tarot nuance refinement

3 nuance:
1. Verifiable trigger (Layer 2 capability check, không declared trigger)
2. Audit trail durable cho lane override
3. `.runtime-env.allowlist` file (rename từ `.env.allowlist` em đề — collision với block-env-edit pattern)

ChatGPT round 4 accept 2 chỉnh nhỏ:
1. `.runtime-env.allowlist` rename + chia 3 nhóm required/optional/forbidden
2. PR body + commit tag (cả 2)

Nâng `INV-WF-001` cho câu invariant trigger.

---

**End of spec v2.1.**
