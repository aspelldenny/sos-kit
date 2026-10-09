# PHIẾU P078b3: Codex adapter enforcement — render `.codex/hooks.json` + `scripts/codex/*` rewritten guards + `.codex/rules/*.rules` (apply_patch-aware, fail-CLOSED, honest-PARTIAL). Xong = **P078b DONE**.

> **Loại:** Feature (SECURITY surface)
> **Ưu tiên:** P1
> **Tầng:** 1 (móng — enforcement/security surface. Guard sai LAN sang mọi Codex-runtime project adopt adapter + P079 behavioral dogfood; **security/auth/envelope touch → AUTO Tầng 1** bất kể LOC, `CLAUDE.md` Rule #8. Fail-OPEN guard = silent bypass = worst-case, KHÔNG reversible sau khi adopter tin.)
> **Lane:** Guarded — **security surface → CHALLENGE BẮT BUỘC kỹ** (fail-closed correctness + apply_patch parse là điểm chết người).
> **Token (khai trần):** est ~120-170k (7 guard-script content template + hooks.json + rules + mock-payload oracle với fixture); nếu vượt 200k → STOP + split rules/oracle sang sub-phiếu, báo Chủ nhà.
> **Ảnh hưởng:** `crates/sos-adapter-codex/src/{templates.rs,lib.rs}` (mở rộng `all_assets()` 10→N + content-fn enforcement + mock-payload oracle), `adapters/codex/MAPPING.md` (fill 3 b3 enforcement row), `adapters/codex/CAPABILITY.md` (bypass-PARTIAL + fail-closed + block-unsafe-merge defer), `docs/PORTABILITY_ARCHITECTURE.md` (P078b3 status + **P078b DONE**), `SECURITY.md` (Codex enforcement PARTIAL note), `CHANGELOG.md`.
> **Dependency:** P078b2 DONE (render()/plan()/`templates::all_assets()` LIVE — 10 declarative artifact; extend, KHÔNG rewrite). P078a DONE (`core/STATE.md` approval-record + edit-allowlist + review-trigger canonical — guard enforce theo). b3 = phần CUỐI của P078b (b1 foundation → b2 declarative → **b3 enforcement**).

---

## Context

### Vấn đề hiện tại

P078b2 render 10 artifact **declarative** (AGENTS.md + 4 `.codex/agents/*.toml` + 4 skill + config.toml) — Codex CLI có contract để đọc, NHƯNG **zero enforcement**: không hook nào wire, không guard nào chạy. Envelope architect = chỉ prose trong `architect.toml` (PARTIAL, `CAPABILITY.md` gap #1); approval-gate (Codex native gap #4) = KHÔNG tồn tại; product-source write / `.env` edit / architect-đọc-source = KHÔNG bị chặn ở tool-call layer.

**KEY constraint (`docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:7,13,20` + `CAPABILITY.md:12-75`):** Claude guards KHÔNG copy được:
- Codex writes qua **`apply_patch`** (`tool_name="apply_patch"`, patch trong `tool_input.command`) — KHÔNG `file_path`. Claude guards đọc `file_path` → **fail-OPEN trên Codex patch trừ khi rewrite** (report:13).
- Codex reads qua **shell** (`rg`/`sed`/`cat`/`head`) — KHÔNG `Read`/`Glob` tool. Architect read-restriction phải **inspect shell command TEXT**, heuristic (gap #5 PARTIAL).
- Project hooks chỉ chạy repo **TRUSTED**; non-managed hooks cần `/hooks` trust; user disable được (report:11,22). → **KHÔNG unbypassable → giữ Git/CI backstop**.

### Giải pháp

Mở rộng `CodexAdapter::plan()`/`templates::all_assets()` (b2 confirmed shape `docs/discoveries/P078b2.md:14-16`) từ 10 → N artifact, thêm **enforcement artifact** (mỗi cái = 1 Asset + content-fn trong `templates.rs`, render tới TARGET project — KHÔNG commit vào sos-kit):

| # | Artifact | Nội dung | Core ID pointer |
|---|----------|----------|-----------------|
| E1 | `.codex/hooks.json` | wire events → `scripts/codex/*` handler: SessionStart, SubagentStart/Stop (set/clear role marker), PreToolUse (apply_patch+Bash-read), PermissionRequest, UserPromptSubmit, Stop. Deny = `{permissionDecision:"deny"}` HOẶC exit-2+stderr (report:11) | `core/POLICY.md` (enforcement) + `core/WORKFLOW.md` (state gate) |
| E2 | `scripts/codex/architect-guard.sh` | architect-active → block writes (**parse apply_patch payload** → extract path → allowlist = phiếu `P<NNN>-*.md` only) + block source-reads (**inspect shell cmd text** `rg`/`sed`/`cat`/`head`/`less` trên `src/`,`crates/**/src`) | `core/POLICY.md` (edit-and-verify) + `core/ROLES.md#architect` |
| E3 | `scripts/codex/orchestrator-guard.sh` | no `worker-active` marker → block product-source apply_patch write | `core/POLICY.md` + `core/ROLES.md#orchestrator` |
| E4 | `scripts/codex/block-env-edit.sh` | block apply_patch touch `.env*` (allow `.env.example`) | `core/POLICY.md` (secret guard) |
| E5 | `scripts/codex/approval-gate.sh` | **(Codex native gap #4 — guard-built)** block EXECUTE/source-write khi machine-state `approved_version` ≠ ticket `version` (`core/STATE.md:29-43`) | `core/STATE.md` (approval-record) + `core/WORKFLOW.md` (approval→execute gate) |
| E6 | `scripts/codex/idea-smell.sh` | UserPromptSubmit → regex idea-smell → inject reminder (stdout→context) | `core/WORKFLOW.md` (intake) |
| E7 | `.codex/rules/exec-policy.rules` | Starlark `prefix_rule` — force-push deny + destructive-command policy (most-restrictive wins, report:12) | `core/POLICY.md` (authority/scope, outside-sandbox commands) |

(Role-marker set/clear cho SubagentStart/Stop = inline command trong hooks.json HOẶC 1 tiny rendered helper — **Worker chốt, Tầng 2 layout**. Nếu tách helper → thêm 1 Asset, count test tracks.)

**Decision 1 — apply_patch parse = pure-shell fail-CLOSED (CRITICAL).** Guard đọc Codex hook stdin JSON `{"tool_name":"apply_patch","tool_input":{"command":"<patch>"}}`, extract path bị đổi từ patch body, check allowlist. Cơ chế = **pure sed/grep, NO jq** (mirror Claude guards `scripts/architect-guard.sh:22-23,44-52` — cross-platform Windows msys2). **Đảo cực fail-safe:** Claude guards fail-OPEN trên unparseable ("don't block on weird input", `architect-guard.sh:51-52`) — với Codex đó = **bypass**. Codex guard PHẢI **fail-CLOSED**: khi đang bị guard (architect-active / no-worker-marker / unapproved) VÀ tool là apply_patch mà **KHÔNG extract được path an toàn** → **BLOCK (exit 2)**, KHÔNG allow. Report:13 "fail OPEN on Codex patch unless rewritten" → nghĩa vụ b3 = fail-CLOSED đúng.

**Decision 2 — exact apply_patch patch-format = anchor #1 `[needs Worker verify]`, ESCALATE nếu không xác nhận được.** Report khai `tool_name`+`tool_input.command` NHƯNG **KHÔNG khai chi tiết marker syntax bên trong patch** (`*** Update File:`/`*** Add File:`/`*** Delete File:` vs unified-diff `+++/---`). Đây là **security-critical**: sed-pattern sai → extract path sai → fail-open thật (trái ý). **Nếu Worker KHÔNG xác nhận được exact patch file-path marker từ nguồn thẩm quyền (report / real Codex 0.145.0 apply_patch sample) → ESCALATE, ĐỪNG ĐOÁN.** Mock fixture oracle vô nghĩa nếu fixture format là đoán. Fallback an toàn nếu format uncertain: guard fail-CLOSED trên MỌI apply_patch khi bị guard (block-all khi không parse chắc) + note gap — nhưng đó làm Codex gần vô dụng, nên escalate để lấy sample thật là đường đúng.

**Decision 3 — approval-gate = deterministic, đọc machine-state projection (`core/STATE.md`).** Codex KHÔNG có native ticket-version approval (`CAPABILITY.md:46-53`). Guard E5 đọc state artifact (`core/STATE.md:29-36` fields: `ticket`/`version`/`state`/`approved_version`) — nếu `state` sắp vào execute-intent (apply_patch trên source) VÀ `approved_version` ≠ `version` (hoặc rỗng) → BLOCK. Mutation-authority (`core/STATE.md:41-42`): chỉ owner/bounded-delegate tạo approval-record; guard KHÔNG tự set approved_version, chỉ đọc-so-sánh. Debate-log vẫn authoritative (`STATE.md:36`) — projection sai → sửa projection, guard đọc projection.

**Decision 4 — block-unsafe-merge: DEFER semantic PR-sentinel, render mechanical force-push qua rules.** Report:13 "block-unsafe-merge maybe portable (Bash tool_input.command) but needs adapter test." Chốt: **(a)** force-push / destructive `rm` = **mechanical prefix → render `.codex/rules/exec-policy.rules`** (E7, Starlark, đủ). **(b)** PR-merge-without-security-APPROVE = **semantic** (đọc PR sentinel — `scripts/block-unsafe-merge.sh:1-9` là shim → `claude-hooks` binary parse Claude JSON shape) → **KHÔNG rewrite được trong phiếu này** (binary ở repo ngoài `~/claude-hooks`; parse Codex Bash payload = binary-side + behavioral). → **DEFER**, note `CAPABILITY.md` là gap với **Git/CI backstop** (merge trên GitHub vẫn qua review gate ở CI). KHÔNG giả có gate merge-sentinel Codex khi chưa có.

**Decision 5 — Oracle STRUCTURAL (không behavioral), có mock-apply_patch-payload.** b3 oracle chạy KHÔNG cần Codex cài (mirror b2 Decision 5):
- hooks.json = valid JSON (parse OK, event/matcher/command tree đúng report shape).
- Mỗi guard `bash -n` clean (syntax).
- **Mock-payload parse correctness (fixture-driven):** feed guard sample Codex apply_patch/Bash stdin JSON → assert đúng block/allow:
  - architect-active + apply_patch ghi `crates/x/src/y.rs` → **BLOCK**; ghi `docs/ticket/P099-x.md` → **ALLOW**; apply_patch unparseable/no-path → **BLOCK** (fail-closed).
  - architect-active + Bash `rg foo crates/sos-core/src` → **BLOCK**; `rg foo docs/` → **ALLOW**.
  - orchestrator-guard: apply_patch product-source + no worker-marker → **BLOCK**; worker-marker present → **ALLOW**.
  - block-env-edit: apply_patch `.env` → **BLOCK**; `.env.example` → **ALLOW**.
  - approval-gate: state `approved_version≠version` + apply_patch source → **BLOCK**; `approved_version==version` → **ALLOW**.
- rules = Starlark prefix_rule shape present (force-push deny).
- **PARTIAL-honest present:** CAPABILITY + hooks.json/architect-guard header note bypassable + Git/CI backstop.

Behavioral (Codex CLI 0.145.0 thật enforce hook/apply_patch/rules) = **P079**, ngoài scope.

**Decision 6 — Additive, render tới TARGET, KHÔNG commit vào sos-kit.** `.codex/hooks.json`/`.codex/rules/`/`scripts/codex/*` chỉ land ở target project qua `install --runtime codex` (Decision 6 b2, `docs/discoveries/P078b2.md:54-56`). Test = in-memory Artifact / temp dir. sos-kit repo KHÔNG có `.codex/`/`scripts/codex/` sau phiếu. `install.sh`/`bin/sos.sh`/`engine.rs` zero-touch (engine tiêu thụ ManagedOperation generic).

### Scope
- **CHỈ sửa:** `crates/sos-adapter-codex/src/templates.rs` (thêm content-fn E1-E7 + mở `all_assets()`), `crates/sos-adapter-codex/src/lib.rs` (mock-payload oracle tests; plan() count update auto qua all_assets()), `adapters/codex/MAPPING.md` (fill 3 b3 row), `adapters/codex/CAPABILITY.md` (bypass-PARTIAL + fail-closed + block-unsafe-merge defer), `docs/PORTABILITY_ARCHITECTURE.md` (P078b3 status + P078b DONE), `SECURITY.md` (Codex enforcement note), `CHANGELOG.md`.
- **KHÔNG sửa:** `crates/sos-core/src/**` (Asset/Artifact/ManagedOperation đủ chở — nếu enforcement buộc thêm core-type field → STOP escalate), `crates/sos-install/src/engine.rs` (generic consume), `crates/sos-adapter-claude/**` (P078c), `bin/sos.sh`/`install.sh` (additive), `core/**` docs semantics (pointer-only), `scripts/{architect,orchestrator,block-env-edit,idea-smell}-guard.sh` **của sos-kit** (đó là Claude runtime của kit — CHỈ ĐỌC để hiểu intent, KHÔNG copy, KHÔNG sửa; Codex guard là bản REWRITE trong templates.rs).
- KHÔNG commit artifact render (`.codex/`, `scripts/codex/`) vào sos-kit.

---

## Task 0 — Verification Anchors

> Architect docs-only (KHÔNG đọc được `crates/**/src` — architect-guard chặn, đúng envelope). `[verified]` = đọc từ report/discovery/STATE/Claude-guard thật; `[needs Worker verify]` = Worker grep/mở src/nguồn xác nhận TRƯỚC impl. Cite RANGE, KHÔNG count. **Security-critical anchor → escape hatch = ESCALATE, KHÔNG đoán.**

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | **CRITICAL — apply_patch patch-format:** stdin `{"tool_name":"apply_patch","tool_input":{"command":"<patch>"}}`; path bị đổi extract được từ patch body qua **known marker syntax** (`*** Update/Add/Delete File:` HAY unified `+++/---`) | Xác nhận exact marker từ `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:7,13` + real Codex 0.145.0 `apply_patch` payload sample (Sếp có thể chạy `codex`) | ⏳ `[needs Worker verify]` — report:13 khai `tool_name`+`tool_input.command` NHƯNG **KHÔNG khai marker syntax bên trong**. **ESCALATE nếu không lấy được sample thật** (Decision 2) — sed-pattern sai = fail-open thật, đừng đoán security-critical. |
| 2 | hooks.json format: event list (SessionStart/SubagentStart/SubagentStop/PreToolUse/PermissionRequest/PostToolUse/UserPromptSubmit/Stop) + matcher/handler tree; PreToolUse deny = `{permissionDecision:"deny"}` OR exit-2+stderr (Claude-compatible) | Read `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:11,25-26` (README owns list) | ✅ `[verified]` |
| 3 | `core/STATE.md` machine-state fields: `ticket`/`version`/`state`/`approved_version`/`previous_state`/`blocked_reason`; approval-record binds exactly-one version, mutation = owner/delegate only; debate-log authoritative, projection derived | Read `core/STATE.md:27-43` | ✅ `[verified]` |
| 4 | edit-allowlist glob/normalization: repo-relative pattern, `*` trong segment / `**` cross-segment; normalize = strip leading `./`, no trailing slash; edit-outside-set → stop tới amend+version-bump | Read `core/STATE.md:45-50` | ✅ `[verified]` |
| 5 | Claude guard intent (rewrite-target): architect-guard = marker `.sos-state/architect-active` + Write-allowlist `P[0-9]*-*.md` + Read-block `src/`,`crates/*/src/` + fail-OPEN unparseable; orchestrator-guard = `worker-active` marker gate product-source; block-env-edit = `^\.env($\|\.)` allow `.env.example`; idea-smell = regex→stdout | Read `scripts/{architect-guard,orchestrator-guard,block-env-edit,idea-smell}.sh` | ✅ `[verified]` (đã đọc — intent, KHÔNG copy: Codex parse apply_patch/shell KHÔNG file_path; fail-CLOSED KHÔNG open) |
| 6 | b2 render mechanism để extend: `templates.rs` có `all_assets()->Vec<Asset>`, `target_path_for(identity)`, `content_for(identity)` (pure match, 1 fn/surface); `plan()` enumerate → render() per-Asset → `ManagedOperation`; engine generic zero-change | Read `docs/discoveries/P078b2.md:14-17` + `grep -n "all_assets\|content_for\|target_path_for\|fn plan\|fn render" crates/sos-adapter-codex/src/{templates.rs,lib.rs}` | ✅ `[verified]` (discovery) / ⏳ `[needs Worker verify]` exact fn names + có `Asset.identity` enum/const nào để thêm 7 identity mới |
| 7 | Codex reads qua shell (rg/sed/cat) KHÔNG Read/Glob → architect read-restriction = inspect shell cmd TEXT (heuristic, PARTIAL gap #5); apply_patch = write path | Read `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:13,20` + `adapters/codex/CAPABILITY.md:58-68` | ✅ `[verified]` |
| 8 | `.codex/rules` = Starlark `prefix_rule(pattern, decision allow\|prompt\|forbidden)`, most-restrictive wins, cho commands OUTSIDE sandbox (force-push/destructive), KHÔNG in-sandbox tool allowlist | Read `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:12` | ✅ `[verified]` |
| 9 | Enforcement bypassable: hooks chỉ chạy repo TRUSTED; non-managed cần `/hooks` trust; user disable được; hosted tools không hook-visible → PARTIAL, giữ Git/CI backstop | Read `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:22` + `adapters/codex/CAPABILITY.md:70-75` | ✅ `[verified]` |
| 10 | block-unsafe-merge: shim → `claude-hooks` binary parse stdin JSON (Claude shape); Codex Bash cũng `tool_input.command` NHƯNG binary parse Codex payload = external-repo + behavioral | Read `scripts/block-unsafe-merge.sh:1-28` | ✅ `[verified]` → Decision 4 DEFER semantic, render mechanical force-push qua rules |
| 11 | Oracle exec test cần `bash` trên PATH (Windows CI = Git Bash). `bash -n` + mock-payload-run test có chạy được cross-platform CI không, hay cần `#[cfg(unix)]` gate + content-assertion fallback? | `grep -rn "bash\|cfg(unix)\|Command::new" crates/sos-adapter-codex/ crates/*/tests/` + check CI matrix | ⏳ `[needs Worker verify]` — escape hatch: nếu Windows CI thiếu bash cho exec-test → `#[cfg(unix)]` gate exec-based tests + giữ content-pattern assertion cross-platform (KHÔNG bỏ correctness test, chỉ chọn harness) |
| 12 | Role-marker cho SubagentStart/Stop: inline command trong hooks.json (`touch/rm .sos-state/<role>-active`) đủ, HAY cần rendered helper script? Codex hooks command chạy shell được? | Read `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:11` (matcher/command tree) | ⏳ `[needs Worker verify]` — Tầng 2 layout Worker chốt; nếu tách helper → +1 Asset, count test tracks |

**Anchor ❌:** không có. **`[needs Worker verify]`** = #1(CRITICAL-ESCALATE), #6(fn-names), #11(bash-harness), #12(marker-layout). **CRITICAL #1:** apply_patch marker syntax quyết fail-closed đúng/sai — Worker xác nhận sample thật TRƯỚC impl; không chắc → ESCALATE (Decision 2), fail-open security-critical KHÔNG được đoán.

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+[a-z0-9]*')
mkdir -p ".backup/${PHIEU_ID}"
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

Rollback: `git reset --hard $(cat .backup/${PHIEU_ID}/main-head.txt)` (trong worktree phiếu, KHÔNG main).

---

## Debate Log

> Schema: 1 turn = Worker Challenge + Architect Response. Cap = 3 turns. **Security surface → CHALLENGE bắt buộc kỹ** (đặc biệt anchor #1 apply_patch fail-closed).

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Anchor verification:** #6 ✅ (`all_assets()`/`target_path_for`/`content_for` fn-names confirmed exact, `templates.rs:32,54,72`, `lib.rs:91,105,111-112`; identity is plain `&str`, not enum). #11 ⚠️ (no bash-exec test in `sos-adapter-codex` yet, but `#[cfg(unix)]` pattern already precedented in `crates/sos-install/tests/tools.rs:50,147,171,204,221` + `crates/sos-cli/tests/parity.rs:249` — safe to reuse). #12 open, correctly deferred to Worker Tầng-2 layout call — no objection.

**Objection [O1.1] — Anchor #1 CRITICAL, apply_patch marker syntax unconfirmed:** report (`CODEX_ADAPTER_DISCOVERY_2026-07-22.md:13`) states only `tool_name="apply_patch"` + patch text in `tool_input.command` — it does NOT document the marker syntax inside the patch body. My own training-data prior (OpenAI's public "V4A" apply_patch format: `*** Begin Patch`/`*** Add File:`/`*** Update File:`/`*** Delete File:`/`*** End Patch`) is a plausible guess, but per Oracle-first self-close rule there is no SOUND oracle in this session to verify it against real Codex CLI 0.145.0 — cannot self-close a security-critical claim on recollection alone.
**Verdict:** RECOMMEND ESCALATE — need a real captured `apply_patch` PreToolUse hook payload before writing sed extraction patterns (matches phiếu's own Decision 2 stance). No other Tầng-1 objections found.

**Status:** ⏳ AWAITING ARCHITECT RESPONSE / CHỦ NHÀ DECISION (anchor #1 factual gap, not a design ambiguity)

### Turn 2 — Escape-hatch RESOLVED (Chủ nhà)

Sếp ran a live Codex CLI (gpt-5.6) probe and captured 4 real `apply_patch`/`Bash` PreToolUse hook payloads → saved as ground-truth fixture. Confirms:
- stdin = 1 JSON object/line: `hook_event_name`, `permission_mode`, `cwd`, `tool_name`, `tool_input.command`, `tool_use_id`. No `$CLAUDE_PROJECT_DIR` equivalent — use `cwd` field / `git rev-parse` fallback.
- apply_patch envelope confirmed = exactly the V4A format Worker's Turn-1 prior guessed: `*** Begin Patch\n*** Add|Update|Delete File: <path>\n...*** End Patch`, embedded newlines are literal `\n` (backslash-n) inside the JSON string, not real newlines.
- Path extraction: `grep -oE '\*\*\* (Add|Update|Delete|Move) File: [^\\"]+'` on the raw JSON line is sound (stops at the escaped-newline or closing quote).
- Reads fire as `tool_name="Bash"`, `tool_input.command` = plain shell string — heuristic command-text inspection required (gap #5, unchanged from report).

O1.1 CLOSED — real sample obtained, Worker's V4A prior confirmed correct. **Mode: EXECUTE.** Fixture committed at `crates/sos-adapter-codex/tests/fixtures/codex-apply-patch-payloads.jsonl` with provenance note. Proceeding to Task 1-6 per phiếu V1 (no further amendment needed — Decision 1-6 all stand as written).

**Status:** ✅ RESOLVED — EXECUTE

---

## Nhiệm vụ

### Task 1: Thêm 7 enforcement Asset + content template (`templates.rs`)

**File:** `crates/sos-adapter-codex/src/templates.rs` (extend — b2 pattern anchor #6)

**Thêm:** 7 Asset-identity mới (E1-E7 bảng Context) vào `all_assets()`; `target_path_for` map identity → path (`.codex/hooks.json`, `scripts/codex/architect-guard.sh`, …); `content_for` thêm 1 arm/surface (`hooks_json()`, `guard_architect_sh()`, `guard_orchestrator_sh()`, `guard_block_env_sh()`, `guard_approval_gate_sh()`, `guard_idea_smell_sh()`, `rules_exec_policy()`).

**Lưu ý:**
- Guard content = **bash script string literal** (Rust raw-string). Cơ chế parse = **pure sed/grep, NO jq** (anchor #5 Claude precedent). Codex-only token (`apply_patch`, `.codex`, `tool_input`, `permissionDecision`) CHỈ trong crate này (dep-direction — core zero host token).
- **apply_patch parse (Decision 1, CRITICAL):** extract path từ `tool_input.command` patch body theo marker anchor #1 (`[needs Worker verify]` — ESCALATE nếu không chắc). **fail-CLOSED:** bị-guard + apply_patch + no-safe-path → BLOCK.
- **Read-restriction (E2, anchor #7):** inspect shell cmd text (`rg`/`sed`/`cat`/`head`/`less` + path `src/`,`*/src/*`,`crates/*/src/*`) — heuristic PARTIAL, note honest trong header.
- Provenance pointer core ID (Decision, mirror b2) trong comment mỗi artifact — KHÔNG copy semantics.

### Task 2: `.codex/hooks.json` wire (E1)

**File:** `templates.rs` `hooks_json()` (Task 1)

**Thêm:** JSON wire event→handler (anchor #2): SessionStart, SubagentStart/Stop (set/clear role marker — inline `touch/rm .sos-state/<role>-active` HOẶC helper anchor #12), PreToolUse matcher `apply_patch`+`Bash` → 4 guard (architect/orchestrator/block-env/approval-gate) + UserPromptSubmit → idea-smell. Deny = `{permissionDecision:"deny"}` HOẶC exit-2 (guard dùng exit-2 stderr, Claude-compatible).

**Lưu ý:** valid JSON (oracle parse). Path handler = `scripts/codex/*.sh` (relative target). PostToolUse/PreCompact/Stop = wire tối thiểu hoặc bỏ nếu không dùng (KHÔNG wire event rỗng).

### Task 3: approval-gate guard (E5 — Codex native gap #4, guard-BUILT)

**File:** `templates.rs` `guard_approval_gate_sh()` (Task 1)

**Thêm:** đọc machine-state projection (`core/STATE.md:29-43` fields) — apply_patch trên source + (`approved_version` rỗng HOẶC ≠ `version`) → BLOCK exit-2 "chưa approve exact version". Đọc-so-sánh thôi, KHÔNG set approved_version (mutation-authority = owner/delegate, `STATE.md:41-42`). State-file path/format = projection integration-defined (`STATE.md:36,66`) — dùng `.sos-state/` convention nhất quán marker; nếu path/format chưa chốt ở codebase → `[needs Worker verify]` + escape hatch (KHÔNG bịa schema).

**Lưu ý:** đây là cái Codex KHÔNG có native (`CAPABILITY.md:46-53`) → guard là replacement, Git/CI backstop retained. fail-CLOSED: state-file thiếu/unparseable + đang execute-intent → BLOCK (an toàn hơn allow-execute-unapproved).

### Task 4: `.codex/rules/exec-policy.rules` (E7) + block-unsafe-merge DEFER (Decision 4)

**File:** `templates.rs` `rules_exec_policy()` (Task 1)

**Thêm:** Starlark `prefix_rule` (anchor #8): force-push (`git push --force`/`-f`/`+`) → `forbidden`; destructive (`rm -rf`/`git reset --hard` outside-worktree nếu biểu diễn được) → `prompt`/`forbidden`. Most-restrictive wins.

**Lưu ý:** PR-merge-sentinel gate = **DEFER** (Decision 4) — note CAPABILITY gap + Git/CI backstop. KHÔNG render giả gate merge-sentinel.

### Task 5: Mock-payload structural oracle (Decision 5)

**File:** `crates/sos-adapter-codex/src/lib.rs` `#[cfg(test)]` (+ fixture)

**Thêm** test (chạy KHÔNG cần Codex cài):
- `plan_renders_expected_artifact_count` — all_assets() = 10 + N enforcement (assert đúng set, KHÔNG hard-count nếu Worker tách marker-helper — track thực tế).
- `hooks_json_is_valid_json` — parse OK (`serde_json`/`toml`? dùng dep sẵn workspace, anchor — KHÔNG thêm dep nặng).
- `guard_scripts_bash_n_clean` — mỗi guard `bash -n` (anchor #11 harness — `#[cfg(unix)]` gate nếu Windows CI thiếu bash + content-assertion fallback cross-platform).
- **`guard_blocks_and_allows_correctly` (fixture-driven, CORE oracle):** feed mock apply_patch/Bash stdin (Decision 5 bảng) → assert block(exit2)/allow(exit0):
  - architect-guard: apply_patch src→BLOCK, phiếu.md→ALLOW, unparseable→BLOCK; Bash rg-on-src→BLOCK, rg-on-docs→ALLOW.
  - orchestrator-guard: product-source no-marker→BLOCK, marker→ALLOW.
  - block-env-edit: `.env`→BLOCK, `.env.example`→ALLOW.
  - approval-gate: version-mismatch→BLOCK, match→ALLOW.
- `enforcement_artifacts_carry_core_pointer_and_partial_note` — hooks/architect-guard header có bypass-PARTIAL + Git/CI backstop note.

**Lưu ý:** mock fixture PHẢI reflect apply_patch format THẬT (anchor #1) — nếu #1 escalate/uncertain, fixture = đoán → oracle vô nghĩa (Decision 2). Behavioral (Codex thật) = P079.

### Task 6: Docs gate (Tầng 1)

- `adapters/codex/MAPPING.md` — 3 b3 row (`:21-23` hooks.json / rules / scripts-codex) Physical-render → "P078b3 DONE"; thêm "Enforcement coverage (P078b3)" subsection.
- `adapters/codex/CAPABILITY.md` — update: (a) gap #1/#4/#5 backstop nay có guard (PARTIAL, KHÔNG SOUND) + **bypassable note** (`:70-75` mở rộng: rendered guard = fast-feedback, KHÔNG unbypassable, Git/CI = boundary); (b) **fail-CLOSED design note** (Codex guard đảo cực vs Claude fail-open); (c) **block-unsafe-merge DEFER** (Decision 4, gap + Git/CI backstop).
- `docs/PORTABILITY_ARCHITECTURE.md` — thêm "P078b3 status" đoạn (sau `:60` P078b2): 7 enforcement artifact, apply_patch fail-closed parse, approval-gate guard-built, PARTIAL bypassable, structural mock-payload oracle, behavioral P079. **Đánh dấu P078b DONE (b1+b2+b3)**; migration table row P078 → tùy P079/P080 còn (b DONE ≠ P078 DONE toàn phần — đọc row hiện tại, chỉ bump phần b).
- `SECURITY.md` — thêm subsection ngắn "Codex adapter enforcement (rendered to target, PARTIAL)": guard render tới target project, bypassable (untrusted-repo/disabled hook), Git/CI backstop retained; KHÔNG phải auto-exec surface của CHÍNH sos-kit (`.sos-trust-baseline` không cover — render-only). Giữ honest 3-surface (verify/CAPABILITY/SECURITY).
- `CHANGELOG.md` — entry P078b3 (+ note P078b DONE).

**Lưu ý:** `core/**` semantics KHÔNG đổi. Regression: `grep -rn 'adapters/\|\.codex\|apply_patch' core/` → zero (dep 1 chiều).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-adapter-codex/src/templates.rs` | Task 1-4: 7 enforcement Asset + content-fn (hooks/5 guards/rules) |
| `crates/sos-adapter-codex/src/lib.rs` | Task 5: mock-payload oracle + count test |
| `adapters/codex/MAPPING.md` | Task 6: 3 b3 enforcement row DONE + coverage subsection |
| `adapters/codex/CAPABILITY.md` | Task 6: bypass-PARTIAL + fail-closed + block-unsafe-merge defer |
| `docs/PORTABILITY_ARCHITECTURE.md` | Task 6: P078b3 status + P078b DONE |
| `SECURITY.md` | Task 6: Codex enforcement PARTIAL note |
| `CHANGELOG.md` | Task 6: entry |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `crates/sos-core/src/adapter.rs` | Asset/Artifact/ManagedOperation đủ chở guard bytes — KHÔNG thêm field (buộc → STOP escalate) |
| `crates/sos-install/src/engine.rs` | Generic consume ManagedOperation — zero-touch |
| `crates/sos-adapter-claude/**` | P078c — zero-touch |
| `scripts/{architect,orchestrator,block-env-edit,idea-smell}-guard.sh` | ĐỌC intent (anchor #5), KHÔNG copy/sửa — Codex rewrite trong templates.rs (apply_patch+shell, fail-CLOSED) |
| `scripts/block-unsafe-merge.sh` | ĐỌC (anchor #10) → DEFER semantic (Decision 4) |
| `core/STATE.md` | Fields approval/edit-allowlist đúng (anchor #3,#4) — pointer, KHÔNG sửa |
| `bin/sos.sh`, `install.sh` | Zero-touch (additive) |

---

## Luật chơi (Constraints)

1. **apply_patch fail-CLOSED** (Decision 1) — bị-guard + apply_patch + no-safe-path-extract → BLOCK. Claude fail-OPEN precedent KHÔNG áp cho Codex write-guard. Pure sed/grep, NO jq.
2. **apply_patch marker = verify TRƯỚC, ESCALATE nếu uncertain** (Decision 2, anchor #1) — KHÔNG đoán security-critical sed-pattern.
3. **approval-gate đọc-so-sánh, KHÔNG mutate** (Decision 3, `core/STATE.md:41-42`) — guard không tạo approval-record; owner/delegate only. fail-CLOSED khi state thiếu.
4. **block-unsafe-merge DEFER semantic** (Decision 4) — render mechanical force-push qua rules; PR-sentinel = Git/CI backstop, note gap. KHÔNG giả gate.
5. **PARTIAL honest bypassable** (anchor #9) — CAPABILITY/hooks/SECURITY ghi rõ "hook bypassable → Git/CI backstop retained; envelope PARTIAL". KHÔNG tuyên bố Codex adapter = security boundary kín.
6. **Structural mock-payload oracle** (Decision 5) — chạy KHÔNG cần codex; fixture reflect format THẬT (anchor #1). Behavioral = P079.
7. **Additive + KHÔNG commit artifact** (Decision 6) — render tới target; `.codex/`/`scripts/codex/` KHÔNG vào sos-kit; engine/install.sh/bin/sos.sh zero-touch.
8. **Dep-direction bất di** — Codex token (`apply_patch`,`.codex`,`tool_input`) CHỈ `sos-adapter-codex`; core zero host token; `grep -rn 'adapters/\|\.codex\|apply_patch' core/` → zero; `dep_direction.rs` XANH.
9. **KHÔNG thêm core-type field** — buộc → STOP escalate.
10. **Cite RANGE** (`core/POLICY.md`), KHÔNG count.

---

## Nghiệm thu

### Automated (oracle STRUCTURAL — Decision 5)
- [ ] `cargo build --workspace` xanh. `[oracle: cargo build --workspace SOUND]`
- [ ] `cargo test --workspace` xanh incl. b3 mock-payload oracle: hooks.json valid JSON + guard `bash -n` clean + **mock-apply_patch-payload block/allow correct** (architect/orchestrator/block-env/approval-gate — bảng Decision 5) + rules Starlark-shape + PARTIAL-note present + count. `[oracle: structural — hooks.json valid + guard bash -n + mock-apply_patch-payload parse correct (block/allow) + rules-shape + PARTIAL-honest SOUND]`
- [ ] `cargo test --workspace` ×20 = 0 flaky. `[oracle: ×20 flaky-check SOUND]`
- [ ] dep-direction: `grep -rn "sos_adapter_codex" crates/sos-core/src/` → zero + `grep -rn 'adapters/\|\.codex\|apply_patch' core/` → zero + `dep_direction.rs` XANH. `[oracle: dep-direction SOUND]`
- [ ] `install --runtime codex --dry-run` LIỆT 10+N artifact (incl 7 enforcement path+preview), reach đúng path như b2/claude (tool-manifest pin drift chặn exit-0 = pre-existing, KHÔNG regression — b1/b2 precedent). `[oracle: dry-run structural — PARTIAL nếu env pin drift, precedent]`

**Ranh giới oracle (in rõ):** b3 KHÔNG verify Codex CLI 0.145.0 thật enforce hook/apply_patch/rules — đó behavioral, **P079**. b3 verify = artifact valid + guard parse mock-payload đúng (block/allow) + fail-closed + PARTIAL honest. Structural-vs-behavioral = Decision 5.

### Manual Testing
- [ ] Đọc `scripts/codex/architect-guard.sh` render out: apply_patch parse (extract path) + shell-read inspect + Write-allowlist phiếu-only + **fail-CLOSED** unparseable + header bypass-PARTIAL note.
- [ ] `.codex/hooks.json` render out: event→guard wire đúng, valid JSON, deny mechanism (permissionDecision/exit-2).
- [ ] approval-gate render out: đọc state `approved_version` vs `version`, BLOCK mismatch, KHÔNG mutate.
- [ ] CAPABILITY/SECURITY: "bypassable → Git/CI backstop retained; PARTIAL" hiện diện (honest 3-surface).

### Regression
- [ ] `install --runtime claude --dry-run` KHÔNG regress (Claude adapter zero-touch).
- [ ] b2 10-artifact structural tests vẫn xanh (extend KHÔNG break).
- [ ] `crates/sos-core/**` diff rỗng (KHÔNG thêm core-type field).
- [ ] `.codex/`/`scripts/codex/` KHÔNG xuất hiện trong `git status` của sos-kit (Decision 6).
- [ ] `scripts/{architect,orchestrator,block-env-edit,idea-smell}-guard.sh` (Claude, của kit) diff rỗng (KHÔNG đụng).

### Docs Gate (Tầng 1)
- [ ] `adapters/codex/MAPPING.md` — 3 b3 enforcement row DONE + coverage subsection
- [ ] `adapters/codex/CAPABILITY.md` — bypass-PARTIAL + fail-closed design + block-unsafe-merge defer
- [ ] `docs/PORTABILITY_ARCHITECTURE.md` — P078b3 status + P078b DONE
- [ ] `SECURITY.md` — Codex enforcement PARTIAL note (render-only, không cover bởi `.sos-trust-baseline`)
- [ ] `CHANGELOG.md` — entry P078b3

### Discovery Report
- [ ] Write `docs/discoveries/P078b3.md`:
  - **Anchor #1 (CRITICAL):** apply_patch marker syntax THẬT — lấy được sample chưa / escalate? sed-pattern extract path đúng chưa. Fail-open risk đóng chưa.
  - #6 fn-names + Asset-identity mechanism (enum/const thêm 7 identity); #11 bash-harness (cfg(unix) hay cross-platform); #12 marker layout (inline vs helper).
  - fail-CLOSED: các branch đảo cực vs Claude — liệt path unparseable→BLOCK.
  - approval-gate: state-file path/format thật; đọc-so-sánh; fail-closed khi thiếu.
  - block-unsafe-merge DEFER: gap ghi CAPABILITY chưa; force-push rules cover gì.
  - Artifact count thật (10+N); mock-payload oracle: fixture nào, block/allow verify nào.
  - PARTIAL honest: 3-surface (CAPABILITY/hooks-header/SECURITY) khớp chưa.
  - **Symmetry flag** (P078c Claude): guard-rewrite pattern có bake Codex-only nào cần reconcile.
  - **Behavioral defer:** liệt cái gì để P079 verify (Codex thật enforce hook/apply_patch/rules).
  - Docs updated (list) / Tier escalations ("None" nếu không) / Trust-gate (N/A nếu chỉ Rust+MD, không đụng `scripts/*.sh` của kit).
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`.
- [ ] **P078b DONE** — note trong Discovery + BACKLOG (b1+b2+b3 complete; P078 phần còn = P079 behavioral).
