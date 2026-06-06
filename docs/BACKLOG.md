# sos-kit Backlog

> **Single source of truth for "what to do next on sos-kit itself."**
> Live tracker. SessionStart hook surfaces Active sprint into the model's context on every new Claude Code session in this repo. Pick an item or capture a new idea via `/idea` skill.
>
> **Architect Rule 0:** Only write phiếu for items in **Active sprint**, or for items the maintainer has explicitly promoted from "Next sprint." No phiếu for "Open backlog" / "Park" without explicit promotion.

---

## ✅ COMPLETE sprint: Két dogfood harvest — git-level gates (2 phiếu)

> **Promoted + SHIPPED 2026-06-06** (Sếp explicit pick). Harvest batch P049–P054 born from ket teardown 2026-06-03; ket dogfood CLOSED → proving-ground satisfied. This sprint took the 2 most-grounded items; both shipped on branch `harvest-git-gates` (full state machine each: DRAFT→CHALLENGE→RESPOND→APPROVAL_GATE→EXECUTE). Remaining P049/P052/P054 reviewed after.
> **Doctrine:** dogfood → retro-harvest (the loop that won Két). Locked in the gains before forging the brownfield-adopt blocker (deferred — mold not ripe, brief §E). CHALLENGE earned its keep: caught O1.1 merge-commit hole (P050) + stale-sentinel hole (P053→[P055]).

- [x] ~~**[P050]**~~ SHIPPED + MERGED 2026-06-06 — no-code-on-default-branch pre-commit gate (`scripts/no-code-on-default.sh`). 17/17 test PASS. PR #22 / squash `ae03e5b`. O1.1 fix: `MERGE_HEAD` guard (PR merge not blocked). O1.2 (Chủ nhà): absent `.sos-stack.toml` → ext-union BLOCK. Discovery: `docs/discoveries/P050.md`.
- [x] ~~**[P053]**~~ SHIPPED + MERGED 2026-06-06 — sentinel-vs-silent merge deadlock fix (Option A, emit-side; `block-unsafe-merge.sh` untouched). PR #22 / squash `ae03e5b`. **Self-validated end-to-end:** `/security-review 22` posted clean APPROVE sentinel → `block-unsafe-merge` allowed merge → no deadlock (closes runtime anchor #8). Stale-sentinel limitation documented → deferred to [P055]. Discovery: `docs/discoveries/P053.md`.

---

## 🧭 Inbound brief from tarot orchestrator (2026-06-05) — "quy về 1 mối" + blocker đóng kit

> Capture từ session dogfood tarot (Sếp + Quản đốc). KHÔNG phải Active sprint — đây là **bản đồ + blocker** để Sếp quyết khi quay về sos-kit. Mục tiêu cuối: **1 bộ kit Rust hoàn chỉnh, 1 lệnh cargo cài + tự wire vào repo mới/cũ** (npm-cho-Claude-Code).

### A. Trạng thái toolchain (11 tool, 9 shipped)

- **Shipped Rust + wired:** ship, docs-gate, guard, vps (4 đời đầu) · advisory-inbox, advisory-cron, doctor, doc-rotate (4 mới)
- **Skeleton:** claude-hooks (thay 4 Bash hook), inv-gate (thay ~794 dòng Python security-gate)
- **Lệch chuẩn:** quality-gate (Rust, chưa wire tarot, no CI) · **doc-rotate (Python — port Rust, brief ở repo đó)** · vps + guard (**no README/CLAUDE/docs** — chưa phải template chuẩn, debt nếu thành golden source)

### B. Vision còn thiếu mảnh gì (không phải "thêm tool")

Migration sang Rust ~XONG (9/11). Cái thiếu cho "1 lệnh cài cả bộ + tự chạy":
1. **Installer hợp nhất chưa code.** Hiện chỉ `INSTALL.md` (copy tay 30+ file) + Bash MVP `bin/sos.sh` + skeleton `bootstrap/sos-rs/`. `BOOTSTRAP_AUTOMATION_DRAFT.md §4` đã draft "bash ~50 dòng" (Category A đổ cứng / B default / C khung rỗng + validator) **nhưng chưa implement**. Doctrine §7: bash trước, cargo sau khi mold chín 3 repo.
2. **Trigger wiring** — tool ship mà không nổ (advisory-cron chưa register, doctor không auto-fire). `doctor verify-setup` (validator check wiring) có khung, chưa nhét vào bootstrap.
3. **2 skeleton** chưa code (claude-hooks, inv-gate).

### C. ⛔ BLOCKER THẬT để đóng kit — Adopt "repo nhiễm độc" (Sếp nêu 2026-06-05)

> Genesis (0→1 empty repo) sos-kit giải rồi (`/init`, GENESIS_TEMPLATE). Nhưng **ADOPT brownfield chưa giải** — và kit phải cài được vào repo cũ thật mới gọi hoàn chỉnh.

3 loại repo cũ, độ khó tăng dần:
- **Loại 1 — code-only, no docs:** adopt phải reverse-engineer docs skeleton từ code. "code có độc, docs không có" → còn phải **detect độc trong code** (anti-pattern, AI-bloat ship sẵn, security debt) trước khi tin.
- **Loại 2 — code-lớn + docs-có, CẢ HAI nhiễm độc:** khó nhất. docs **drift khỏi code** (precedent tarot 2026-06-05: SURFACE_MAP/BACKLOG ghi "flask-cors fixed in 5.x" SAI; ARCHITECTURE §3 ghi "8 cột" lệch code 10 cột). Adopt KHÔNG được tin docs mù quáng — phải **reconcile docs↔code** trước khi layer kit lên.

**Đây đúng bài học retro v2.3 phóng to:** "single-source-the-truth — dormant vì 2 nơi khai báo lệch nhau (sentinel mismatch)". Repo nhiễm độc = drift ở quy mô codebase.

**Cần (chưa có recipe/flow):** một **adopt-flow cho brownfield-poisoned**:
1. Scan độc: code anti-pattern + docs↔code drift detect (tận dụng được advisory-watch? doctor validate-map?)
2. Quarantine/flag — không tin docs cho tới khi verify với code
3. Reconcile single-source — chọn code làm oracle, sửa docs theo (hoặc ngược lại có chủ đích)
4. Mới layer sos-kit (agents/hooks/phieu) lên nền đã làm sạch

→ **Không giải được adopt-poisoned thì kit chỉ chạy greenfield, chưa đóng được.**

### D. Roadmap đề xuất (ROI + dependency)

| Tier | Việc | Ghi chú |
|---|---|---|
| 0 | Cho tool đã ship CHẠY (advisory-cron register, advisory-inbox 8-cột, doctor trigger) | Cao nhất, low effort |
| 1 | Bash bootstrap ~50 dòng (BOOTSTRAP_AUTOMATION_DRAFT §4) — "1 lệnh" v0 | Trái tim vision, bash trước |
| 1.5 | **Adopt-poisoned flow** (blocker C) | Bắt buộc trước khi "đóng kit" |
| 2 | Nốt 2 skeleton: inv-gate (ROI cao, thay 794 dòng) > claude-hooks · port doc-rotate Rust | Chuẩn hoá |
| 3 | Cargo unified installer + docs vps/guard | Defer tới khi mold chín 3 repo (doctrine) |

### E. Lưu ý dogfood

Sếp đang dogfood sos-kit trên `~/ket` + tự dogfood trong chính `~/sos-kit` (agent-viết-agent, kiểu Anthropic). Repo mới chưa ổn định → mold còn đang chín. KHÔNG cứng hoá cargo installer trước khi pattern lặp đủ.

---

## ✅ COMPLETE sprint: Tarot port wave 1 — security pipeline + persona codify

> **SPRINT COMPLETE 2026-05-25** — all 4 phiếu shipped. "Done when" criteria verified: `sos init security` detect stack đúng → `/advisory-scan` chạy zero-workaround → `/security-review <PR>` post advisory comment. Sprint closed.

- [x] ~~**[P040]**~~ SHIPPED 2026-05-25 — bootstrap stack detection (`sos init security` subcommand + `.sos-stack.toml` schema + 6 parser stubs underscores). PR #11 / `8047525`. Discovery: `docs/discoveries/P040.md`.
- [x] ~~**[P041]**~~ SHIPPED 2026-05-25 — Trinh sát (advisory-watch) specialist subagent + pnpm/npm parsers. PR #13 / `b253eff`. Discovery: `docs/discoveries/P041.md`.
- [x] ~~**[P042]**~~ SHIPPED 2026-05-25 — Giám sát (boundary-check) specialist subagent — 5 generic INV, ADVISORY mode, sentinel markers, `/security-review` slash command. PR #14 / `ddaa25b`. Discovery: `docs/discoveries/P042.md`.
- [x] ~~**[P043]**~~ SHIPPED 2026-05-25 — Doc drift consolidate (Quản đốc persona codify + alignment engineering + deferred-tool loading). PR #12 / `569e02f`. Discovery: `docs/discoveries/P043.md`.

---

## 🅿️ Paused sprint: Tarot recipe harvest — Tier 1 (5 recipes)

> **Paused 2026-05-25** — refocused to Tarot port wave 1. Item content + Source-DNA paths preserved here. **ID renumbering needed at resume** — original P040-P044 IDs reused by wave 1; harvest items will be assigned fresh `P0NN` upon promotion.

- [ ] **[TBD]** `recipes/payment/payos-webhook-deep.md` — webhook signature verify + idempotency + replay test. Source: `tarot/scripts/{register-webhook,verify-payos,test-webhook}.ts` (191 LOC) + `tarot/src/lib/payment/payos.ts`. Companion to existing `payos-vn.md`.
- [ ] **[TBD]** `recipes/ai/safety-classifier.md` — Pre-AI-response content moderation gate. Source: `tarot/src/lib/ai/safety-classifier.ts` + `tarot/src/lib/safety/`.
- [ ] **[TBD]** `recipes/ops/encryption-at-rest.md` — PII encrypt-in-place migration pattern. Source: `tarot/src/lib/encryption.ts` (+ test) + `tarot/scripts/encrypt-existing-data.ts`.
- [ ] **[TBD]** `recipes/ops/rate-limit.md` — Rate limit Next.js API routes. Source: `tarot/src/lib/rate-limit.ts` + tests.
- [ ] **[TBD]** `recipes/ops/sentry-nextjs.md` — Sentry server vs client instrumentation split. Source: `tarot/instrumentation.ts` + `instrumentation-client.ts` + `tarot/src/lib/sentry.ts`.

---

## 🅿️ Paused sprint: Worker capability + install UX gaps

> **Paused 2026-05-25** per Sếp directive — refocus to tarot port wave 1. P005 + P006 still valid; resume after wave 1 ship. P005 may be implicitly resolved by tarot's architect.md 563-line capability matrix pattern (check khi resume).

- [ ] **[P005]** Worker Skill access — `agents/worker.md:4` `tools:` allowlist không có `Skill`. **DECISION PENDING:**
  - **A.** Add `Skill` vào worker tools allowlist (1-line edit). Pragmatic.
  - **B.** *(em recommend)* Architect/Orchestrator run skill trước CHALLENGE, đổ output vào phiếu. Worker chỉ apply.
  - **C.** Hybrid — Worker invoke skill chỉ khi phiếu có flag `requires_skill: <name>`.
  - Memory ref: `project_tarot_frontend_design_plugin.md`. Existing [P008] DEPENDS on outcome.
- [ ] **[P006]** Pre-commit fresh-install friction — `hooks/pre-commit` shells `docs-gate` failing on fresh repo. **Options:** A (soft-fail), B (bootstrap CHANGELOG/ARCHITECTURE skeleton in INSTALL.md), C (loosen hook). Note: cũng nên xét default `.docs-gate.toml` template trong `templates/`. **Strong P006 evidence accumulated:** P035 + P037 EXECUTE both reported "docs-gate not runnable in sos-kit root (no `.docs-gate.toml`)" — friction confirmed in real motion, not theoretical.

---

## ✅ Recently closed sprint: Worker capability + install UX gaps (2026-04-26 → 2026-05-10)

- [x] ~~**[P005]**~~ SHIPPED 2026-05-10 option B (Skills are Orchestrator-only). PR #10 / `b929bfe`. Codified in `agents/orchestrator.md` + `docs/ORCHESTRATION.md` Hard rule #9 + `phieu/TICKET_TEMPLATE.md` optional `### Skills consulted` subsection. Discovery: `docs/discoveries/P005.md`.
- [x] ~~**[P006]**~~ SHIPPED 2026-05-10 (docs-gate bootstrap). PR #9 / `1086fe2`. Shipped 4 deliverables: `templates/.docs-gate.toml` reference, sos-kit own root `.docs-gate.toml` (kit dogfoods now), hooks/pre-commit graceful-skip guard, `docs/SETUP.md` bootstrap step. Cross-project evidence from media-rating-app P001 EXECUTE (live `docs-gate init` recovery) informed design — schema = flat-string v0.1.0, NOT `[[trigger]]`. Discovery: `docs/discoveries/P006.md`.
- [x] ~~**[P039]**~~ SHIPPED 2026-05-05 (doc drift sweep). PR #8 / `34bafed`. 10 surgical doc edits. Discovery: `docs/discoveries/P039.md`.

**Cross-project event in same window:** `media-rating-app` got full sos-kit migration (PR #4 + #5 in that repo, 2026-05-10) — Tarot-mirror parity. Provided live dogfood evidence for P006 design.

---

## 🎯 Next sprint candidates: Distribution — plugin + Rust CLI cohabit

> **Trigger:** Active sprint (P003 + P004 drift fixes) shipped + maintainer signs off "drift = 0 on fresh install."
> **Theme:** Two install paths, complementary not competing. Plugin = the brain (skills/agents/hooks Claude Code consumes natively). Rust CLI = the hands (filesystem ops, settings.json merge, doctor check, CI scriptable). User picks: plugin-only (simple) or plugin + CLI (recommended for own dogfood).
> **Plan basis:** Claude Code plugin spec confirmed via `claude-code-guide` agent — manifest at `.claude-plugin/plugin.json`, bundles agents + skills + hooks + bash scripts + templates + MCP. Cannot auto-modify user's `.claude/settings.json`, but plugin's bash script (in `bin/`) can with user permission.

- [ ] **[P032]** Phase 1 (MVP) — **sos-kit Claude Code plugin**. Bundle: `agents/architect.md` + `worker.md`, all 9 skills + new `/sos:init` + new `/phieu`, hooks (`hooks/hooks.json` for PreToolUse architect-guard + SessionStart banner), bash scripts in `bin/`, markdown templates (TICKET, BACKLOG, vision/*). Manifest at `.claude-plugin/plugin.json`. User flow: `/plugin install --url https://github.com/aspelldenny/sos-kit` → `/sos:init <project>` → bash script in plugin's `bin/` scaffolds project files + prompts user to merge `.claude/settings.json` (permission gate).
  - **[P032.1]** `/sos:init <project>` skill + companion bash script (creates `docs/BACKLOG.md`, vision templates, `.phieu-counter`, settings.json merge with permission)
  - **[P032.2]** `/phieu <slug>` skill — port `phieu.sh` shell function as cross-platform skill (Windows OK without bash setup)
- [ ] **[P033]** Phase 2 (main, confirmed direction) — **sos-kit Rust CLI**. Standalone binary matching `ship`/`docs-gate`/`guard`/`vps` pattern. Subcommands:
  - `sos-kit init <project>` — clean JSON merge into `.claude/settings.json` + scaffold (no permission prompt friction since it's the user's own CLI)
  - `sos-kit upgrade` — sync project's `.claude/agents/` + scripts from canonical sos-kit, detect + report drift
  - `sos-kit doctor` — verify install (hooks wired, agents register, BACKLOG present, vision docs present)
  - `sos-kit phieu <slug>` — port shell function as proper subcommand with worktree support
  - **Unique value over plugin:** runs outside Claude Code session (useful in CI / scripts / cron), proper JSON merge without per-call permission prompts, cross-project ops, can be invoked from other Rust tools. Companion to existing Rust ecosystem.
- [ ] **[P034]** Distribution channels —
  - **Plugin:** GitHub URL install (immediately works) + marketplace submission via `platform.claude.com/plugins/submit`
  - **Rust CLI:** `cargo install sos-kit` + Homebrew tap `aspelldenny/homebrew-sos` + GitHub Releases pre-built binaries (macOS / Linux / Windows)
  - Documentation in `INSTALL.md` showing both paths side by side

---

## 🌊 Future waves (low commitment)

- [ ] **v2.2 — Debate token optimization.** Park until ≥5 multi-turn phiếu deliver real cost-distribution data. Candidates: skip-CHALLENGE for trivial phiếu (needs criteria), Haiku for Architect DRAFT, inline doc snippets in spawn prompt to skip subagent's Read step. Baseline target: 42k → 25k tokens per multi-turn phiếu.
- [ ] **Multi-project support.** Single sos-kit install serving N projects with centralized `agents/` + `scripts/` + project-local override. Avoids the "8 files copied per project" bootstrap cost. Likely depends on P033 Rust CLI.

---

## 📌 Decision pending mai (2026-05-29)

- [ ] **Bootstrap automation — `sos-init.sh` stopgap → cargo `sos-kit init` doctrine.** Full analysis: `docs/BOOTSTRAP_AUTOMATION_DRAFT.md`. **Diagnosis (Sếp 2026-05-28):** em manual setup doc-rotate 30+ tool call = bệnh "agent copy thiếu" — đang bắt LLM nhớ. Slogan "đừng bắt LLM nhớ, bắt cơ chế nói sự thật" → bootstrap PHẢI là tool. **Framework:** 3 category (BẤT BIẾN đổ cứng / TUNABLE default override / PHẢI KHÁC khung rỗng+validator). **Timing cảnh báo:** đừng cargo hóa NGAY — sos-kit chưa chín, pattern phải lặp ≥3 repo mới biết cái gì thật bất biến. **Stopgap:** bash `sos-init.sh` ~50 dòng, ~20 min viết, giải đau ngay. **Cargo proper:** post pilot vòng 2 + ≥1 repo thứ 3. **6 decision points** for Sếp mai: location / V0 scope / apply first repo / validator integration / --stack values / cargo timeline.

- [ ] **Freeze-tier = BỘ LỌC theo thuộc tính, KHÔNG phải danh sách hạng mục (refine 2026-05-29).** Sửa cách nghĩ 3-category ở trên: đừng hỏi "thứ này LOẠI gì → freeze?", hỏi 2 câu thuộc tính cho mọi artifact (vai/skill/mcp/config/hook): (1) **giống-hệt-mọi-repo** hay **khác-mỗi-repo**? (2) **trong-tầm-kiểm-soát** hay **dependency-ngoài**? → giống+trong-tầm = **Cat-A freeze (copy verbatim)**; khác-mỗi-repo = **Cat-C (khảo-sát/khai-báo, KHÔNG copy)**; dependency-ngoài = ngoài cả 2 tier, cẩn thận credential.
  - **Skills tách 2:** 13 generic SOS skill (plan/verify/review/qa/ship/retro/init/idea/insight/route/decide/forge/apply) = Cat-A freeze, copy. Domain per-repo (media `phase-gate`/`status`, lumi-lighting, tarot-card) = Cat-C, KHÔNG copy cross-repo. ("Skill có freeze ko" = sai câu — lọc TỪNG skill.)
  - **MCP tách theo SỞ HỮU (Denny — sắc hơn Claude Web gộp "all MCP = external"):** tool MÌNH code (`doctor`/`ship`/`guard`/`vps`/`docs-gate`) = native kit, in-control, no-credential, deterministic = "chân tay" → wire (PATH-rel, KHÔNG hardcode `~/.cargo`). Claude Code built-in = không làm gì. MCP-NGOÀI (context7/supabase/canva/gdrive) = **BỎ, không phải việc kit** (repo tự khai = Cat-C). Tinh thần: **LLM suy luận, tool-mình làm chân tay.** Credential-risk chỉ áp MCP-ngoài → mà đã bỏ → risk biến mất. (sos-kit `.mcp.json` đã sạch — chỉ wire doctor.)
  - **GAP THẬT (cùng root với map-lie):** adopt copy được cái-bất-biến (vai) nhưng chưa xử cái-per-repo: map → copy-NHẦM (map tarot), skill/mcp → BỎ-TRỐNG. Fix-direction: adopt cần bước **khảo-sát/khai-báo per-repo** (domain-skill nào, wire tool-mình nào) + wire doctor PATH-rel. **KHÔNG build giờ** — pilot media-rating (privacy) ưu tiên; làm khi curate platform.

- [ ] **KIT LAG SAU FLAGSHIP — sync doctrine tarot→sos-kit (gap-audit 2026-05-29).** Full inventory: `docs/GAP_AUDIT_tarot_to_soskit.md`. **Bệnh trội:** kit ship GATE nhưng rớt DOCTRINE-bật-gate = Sub-mech A "trigger gap" tái sinh TRONG kit (đúng cái làm media collapse hôm nay). **Top-3 nguy (làm hỏng repo mới adopt):** (1) advisory auto-spawn mất (Rule 10 + banner staleness — Trinh sát chết-khi-sinh), (2) pre-merge security-gate doctrine mất (Rule 9 — hook có, lời-bật-hook không, đúng lỗ P297), (3) Architect Layer-1 capability check mất (ship≠chạy 2-lớp mới lắp 1). **Direction (Sếp):** tarot FROZEN đừng-áp-ngược; fix vào kit → media/dự-án-mới làm proving-ground; back-port lên tarot SAU khi kit ngon. **Nuance:** sos-kit ĐI TRƯỚC tarot ở v2.2 (lane/sensor/rubric/multi-stack/INV-4-replay) → sync 2 chiều chọn lọc, KHÔNG copy-tarot-xuống. Effort top-3 phần lớn Small.

- [x] **✅ CURED 2026-06-02 — IN-REPO drift: `.claude/agents/` generate-step chết.** **Resolution:** chọn option (c) ELIMINATE — `.claude/agents/*.md` giờ **symlink** → `agents/`, không copy / không sync-script (đã xoá). SOUL-Q trả lời: "Sếp"=xưng hô, "Chủ nhà"=tên vai → swap là category error. Doctrine: `CLAUDE.md`→Language + `agents/README.md`. Commit `3334a3a`. Bonus: INSTALL.md vá thiếu 2 agent security (gốc lỗ media-no-Giám-sát). _Chẩn đoán gốc bên dưới._ Bệnh RIÊNG, khác chất với gap-audit (đó cross-repo; đây in-repo). `sync-personal-agents.sh` (regen `.claude/agents/` từ canonical `agents/`) **không chạy 1 tháng** → bản run/propagate stale tháng-4 → dogfood + adopt nhận agent cũ = **phần lớn lý do media collapse**. = Sub-mech A (trigger-gap) sống TRONG kit. **Symptom đã vá** (chạy sync 1 lần `796feb2`) — **bệnh CHƯA chữa** (cần-người-nhớ-bấm → rot lại). **Fix (đầu tỉnh):** (a) tự-động pre-commit regen, HOẶC (b) gắn vào action-đã-xảy-ra, HOẶC (c) **bỏ bản generate** (đọc thẳng canonical). **SOUL Q:** có đáng 2 bản `agents/`(Chủ nhà) vs `.claude/agents/`(Sếp) khác 1 chữ + script đồng bộ + điểm-chết-drift không? → single-source 1 bản. **Nhãn:** mọi "dogfood proven" phiên này về bản CŨ; canonical mới (Bước-0/Oracle/Humility) CHƯA chạy = present-not-run. Pattern lặp 3 chỗ (sync-personal-agents / áp-ngược-tay / adopt-copy — đều source→derived→nhớ-bấm).

---

## 💡 Open backlog (triaged, not yet sprinted)

- [ ] **[P007]** *(Tầng 2 housekeeping leftover từ P004)* `bin/sos.sh:94` echo help text vẫn còn literal `docs/CHARACTER.md` — cosmetic, không ảnh hưởng agent envelope rule. 1-line edit thành `docs/CHARACTER*.md` cho consistency. Worker đã classify cosmetic exclusion ở P004 EXECUTE — promote khi rảnh hoặc gom với phiếu housekeeping khác.
- [ ] **[P009]** Notification hook contract — orchestrator fire event sau mỗi state transition (Architect DRAFT/RESPOND done, Worker CHALLENGE/EXECUTE done, APPROVAL_GATE pending), invoke `integrations/notify/notify.sh <event> <payload-json>` nếu exists, no-op nếu không. Kit ship CONTRACT.md + 3 example scripts (`telegram.sh`, `slack.sh`, `macos.sh`); user symlink hoặc copy. **Lý do cần:** subagent runs 2-7 phút (drift sprint P004 RESPOND mất 4:01) → AFK cost cao. Tarot evidence: P040 phiếu 1h29m / 158k tokens, AFK = mất focus block. **Trade-off:** kit complexity +1 hook layer, nhưng pattern-clean (orchestrator chỉ "fire event", không biết Telegram). **Trigger để promote vào Active sprint:** Sếp ship personal Telegram script trước (~30 phút, reuse `integrations/jarvis/` pattern), dùng ≥2 tuần, confirm valuable → kit-level phiếu (~2-3h: CONTRACT.md + orchestrator hook + 3 example + INSTALL.md note + dry-run test). Memory cross-ref: session log 2026-04-26 có full eval (5-yếu-tố matrix + so sánh với github-actions/jarvis pattern).
- [ ] **[P008]** Frontend-design plugin workflow doc — when phiếu touches FE/UI/UX → **Orchestrator** invokes `frontend-design` plugin (claude-plugins-official) BEFORE spawning Architect/Worker, freezes design tokens + component spec into phiếu Context under `### Skills consulted`. **RE-SCOPED 2026-05-10 post-P005 ship:** original draft assumed Worker invokes skill; option B inverts that — workflow doc now documents Orchestrator (main session) trigger criteria + invocation pattern, not Worker handbook entry. Target file: `phieu/FRONTEND_WORKFLOW.md` or section in `docs/ORCHESTRATION.md`.
- [ ] **[P010]** `phieu/AUDIT_TEMPLATE.md` — skeleton fill for AUDIT_PROTOCOL. Currently audit-runner has to build the report structure from scratch; a template halves prep time.
- [ ] **[P011]** Worker AUDIT mode handbook section in `agents/worker.md`. Currently AUDIT mode is documented in `phieu/AUDIT_PROTOCOL.md` only; Worker handbook should declare the mode and trigger phrase.
- [ ] **[P012]** Orchestrator auto-detect "≥N phiếu since last audit" → suggest running AUDIT. State in `docs/ORCHESTRATION.md` or a small `.audit-counter`.
- [ ] **[P013]** Vietnamese 13-checks (diacritics, VND, GMT+7, font rendering, PDF export, etc.) → CI gate that runs pre-deploy. Currently a manual checklist in AUDIT_PROTOCOL.
- [ ] **[P044]** CLAUDE.md AI BIAS WARNINGS section (tarot port wave 1 leftover) — port 4 sub-mechanism failure catalog từ tarot CLAUDE.md §2: Trigger gap / Capability gap / Migration completeness gap / Persistence lifecycle gap + Task 0 capability check matrix. **DEFER promote:** "proven ≥2-tuần" rule (CLAUDE.md sos-kit) — tarot catalog mới ~1 tuần (P281-287 ngày 2026-05-24). Promote sau 2026-06-08 nếu Sếp confirm pattern stable.
- [ ] **[P045]** Skill drift sweep — 6 file drift nhẹ giữa tarot vs sos-kit: `apply` (+2 dòng), `init` (+2 dòng), `forge` `idea` `plan` `retro` (same lines, inline edit). Diff từng file, port relevant content updates. **Tầng 2 surgical.** Run sau wave 1 ship để giảm noise.
- [ ] **[P046]** `block-env-edit.sh` hook port — chống Edit/Write trên `.env*` files (allow `.env.example`). Port từ tarot's `.claude/scripts/block-env-edit.sh`. Add vào `scripts/` + `templates/claude-settings.local.json` PreToolUse matcher `Edit|Write`. **Tầng 2.**
- [ ] **[P047]** Phiếu mid-chat counter UX — `.phieu-counter` file driven ID generation, Architect không hỏi Sếp ID mới mỗi lần. Port từ tarot CLAUDE.md §4. Reduce friction khi DRAFT_PHASE fire trong chat. **Tầng 2.**
- [ ] **[P048]** **[RESEARCH]** `sos new --stack swift` — hỗ trợ stack Swift/macOS thật cho toàn pipeline (03/06/2026). **Bối cảnh:** Két greenfield dogfood lộ `sos new` chỉ biết `python|rust|ts` → app macOS phải mượn placeholder `rust` rồi xoá `Cargo.toml`, khiến `.sos-stack.toml` nói dối (type=rust + cargo_lock parser nhưng repo không có manifest → advisory-scan parse nhầm). Sếp xác nhận sẽ làm **nhiều dự án Swift** → đầu tư thật, không workaround. **Scope:** (1) `--stack swift` đẻ manifest đúng (Package.swift cho SPM / placeholder cho Xcode, KHÔNG Cargo.toml); (2) `sos init security` nhận diện Swift + parser `Package.resolved` (SPM deps) → `.sos-stack.toml` type=swift; (3) `hooks/pre-commit` nhánh type-check Swift (`swift build` cho SPM / skip-with-note cho Xcode vì cần xcodebuild+scheme); (4) `configs/swift.toml` + `docs/SETUP.md` per-stack section. **Nuance:** app Xcode (Két) ≠ SPM package — manifest là `.xcodeproj/.pbxproj` do Xcode sở hữu, khó cat/parse → cân nhắc 2 sub-mode **swift-spm** vs **swift-xcode**. **Feed:** retro WORKFLOW v2.x stack-coverage harvest. Đây là tương lai, không Active sprint.
- [ ] **[P049–P052] HARVEST batch — cross-tool teardown session 2026-06-03.** Born from one session analysing: Tarot P309 (living merge-hook reflex), ket P020 dogfood, media-rating-app multi-agent audit, vibecode-cli (`@nclamvn/vibecode-cli`, Lâm) teardown, + Codex/agent-portability discussion. **Unifying thread (P049/P050/P052):** the most portable layer of the jig is **git-level** — push gates down to git so they survive non-Claude agents (Codex/opencode). Claude PreToolUse hooks die when the agent isn't Claude Code. All 4 are IDEAS, not to-build-now; **proving-ground = ket dogfood** (Sếp to have ket analyse + try, report back before promotion).
  - [ ] **[P049]** Agent-agnostic merge-sentinel (CI/branch-protection version of `block-unsafe-merge.sh`). Current sentinel gate (security-surface → `/security-review` APPROVE before `gh pr merge`) is a **Claude PreToolUse Bash hook** → dies under Codex. Add a CI required-status-check / server-side branch-protection that blocks merging a security-surface PR lacking the sentinel APPROVE comment. **Keep** the PreToolUse version too (faster, in-session); CI = the backstop that survives any agent. Fires on merge, agent-agnostic.
  - [ ] **[P050]** No-code-on-default-branch git gate (pre-commit). Blocks commits touching **CODE** while on the default branch (forces a feature branch); **allows docs-only** commits. **Why:** ket P020 — orchestrator drafted on `main`, forgot to branch; caught by a human, not a gate. "Branch-before-code" is JUDGMENT = coin-flip = `enforce_via_mechanism_not_memory`. **Gate the INVARIANT (no code on main), not the PROCEDURE (branch at setup)** → dissolves the "when to branch" debate. **Constraints (worked out):** (a) distinguish code/docs like `orchestrator-guard.sh`; (b) override marker like `[security-review-skip:]`; (c) fires on `git commit` NOT `git merge` (PR merge into main stays the intended path); (d) git-level = agent-agnostic; (e) sos-kit itself opts out / near-no-op (kit commits maintenance to main directly — would self-block otherwise). **Pairs conceptually with the existing phiếu+Discovery pre-commit gate** (which ket just demonstrated catching a premature phiếu-alone commit).
    - **UPDATE — ket dogfood 2026-06-03 (PROVEN + edge-hole found+fixed):** ket built this as its pre-commit `[0/N]` branch-guard (override marker `.sos-state/allow-code-on-default`), merged, 4 live cases. **Edge-hole found by independent RUN** (not by Giám sát — that's 5-INV security, not gate-logic): a `^Ket/`-style product-**DIR** prefix over-blocks `.md`/non-code under the source dir, contradicting the hook's own "`*.md` allowed" line; the 4 cases tested docs at ROOT (`CHANGELOG.md`), missing docs-under-`Ket/` (the edge). Fixed in ket via `grep -vE '\.md$'` before the product grep + a 15-case edge-test (the missing "gate-logic verifier" role — which then itself had an edge, caught by Giám sát: helper grepped original input not the filtered stream). **Harvest constraints for the sos-kit version:** (a) exempt `.md` — sos-kit's `orchestrator-guard.sh:64 *.md) exit 0` already does this (RUN-confirmed), mirror it; (b) **generalize the product-code pattern from `.sos-stack.toml`**, do NOT hardcode a per-repo dir like `^Ket/` (sos-kit is multi-stack); (c) robustness ket's lacks: detached-HEAD (`git branch --show-current` empty → guard silently passes) + unset `origin/HEAD` (falls back to assuming `main`).
  - [ ] **[P051]** Spec-hash contract-lock (harvest from vibecode-cli). vibecode locks the contract → generates `spec_hash` → "all builds must reference this hash" (drift detection). sos-kit's equivalent is phiếu + Verification Anchors + human APPROVAL_GATE, but **no hash**. Idea: hash the locked phiếu; a hook flags "phiếu changed after lock but EXECUTE/PR doesn't re-reference the new hash." **Open question:** does a hash add value over the human gate, or is it ceremony? vibecode enforced it IN-JS (bypassable) — sos-kit version must be a **hook** to be real. **Lowest priority** — needs a concrete drift incident to justify; don't build on spec.
  - [ ] **[P052]** Git-level `.env` block — **complement to [P046]**. [P046] ports tarot's `block-env-edit.sh` as a Claude PreToolUse Edit/Write guard (dies under Codex). Add a pre-commit hook blocking staging of `.env*` (allow `.env.example`) so the secret-leak guard survives any agent. **Why now real:** media audit SEC-SECRET-01 = `.env.docker` committed to git history — the exact failure a git-level `.env` block prevents at commit-time (PreToolUse only catches Claude's edit-time). Both layers complement.
  - [ ] **[P053]** Sentinel-vs-silent merge **deadlock** (kit interaction bug, ket-surfaced 2026-06-03). `block-unsafe-merge.sh` requires an `APPROVE` sentinel comment to merge a security-surface PR; `boundary-check` (Giám sát) is **silent-when-clean** (P042 design — only posts if it finds something). → security-surface PR + clean review = **no sentinel → merge deadlocks** (only escape = override marker). ket worked around it (WORKFLOW §21: "PR-gated → ALWAYS post sentinel, even clean APPROVE"). **Fix to bake into sos-kit:** either `boundary-check`/`/security-review` always posts a sentinel when the PR is `block-unsafe-merge`-governed (clean APPROVE included), OR `block-unsafe-merge` accepts a "clean-review" signal. The two P042-era gates are in tension — pick one.
  - [ ] **[P054]** **[FINDING — not a build item]** Spawn guards DRIFT from canonical (ket-surfaced 2026-06-03). ket's `orchestrator-guard.sh` had **lost the `*.md) exit 0` exemption** — hand-adapted for Swift (added `Ket/*`) but dropped the `.md` arm → same over-block hole as the branch-guard. **sos-kit's `orchestrator-guard.sh` is NOT affected — it exempts `.md` (line 64, RUN-confirmed). DO NOT "fix" sos-kit's orchestrator-guard; it is already correct.** Corrects the ket harvest-note framing "orchestrator-guard cùng lỗ" — true for ket's *drifted copy*, false for sos-kit's *canonical*. **Real lesson = spawn-drift class** (the root that collapsed media): a spawn hand-adapting a canonical guard silently drops a property. **Direction:** ket re-syncs guards from sos-kit canonical (don't hand-patch); or guards carry a version/hash so drift is detectable. Feeds the existing "KIT LAG / re-sync" theme above.
  - [ ] **[P055]** **[DEBT — surfaced by P053 CHALLENGE 2026-06-06]** SHA-scope the `block-unsafe-merge` APPROVE sentinel. Gate currently greps ANY historical `Verdict: APPROVE` comment on the PR (`block-unsafe-merge.sh:102-106`, no head-SHA binding) → a stale clean APPROVE on commit A can green-light later unreviewed commits B+C on a multi-commit PR. **Pre-existing hole** (gate always grepped any APPROVE), made easier to hit by P053's clean-APPROVE auto-post. **Why separate from P053 (one-disease):** SHA-scoping patches accept-side (`block-unsafe-merge.sh` — out of P053's emit-only scope), needs a new jq filter binding comment→`headRefOid` + slash command posting a `Head SHA:` line. **Direction:** capture `gh pr view <N> --json headRefOid --jq .headRefOid`, require APPROVE sentinel body contain matching `Head SHA: <sha>`, gate rejects sentinel whose SHA ≠ current head. P053 documents the limitation + mitigations (Chủ nhà reads timestamped comment; squash-merge collapses history). Needs grounding before promote (n≥1 multi-commit security PR that actually slipped).
- [x] ~~**CLAUDE.md tree refresh** — current tree in `CLAUDE.md` does not list `CHANGELOG.md`, `DISCOVERIES.md`, `BACKLOG.md`, `docs/ORCHESTRATION.md`.~~ **Shipped via [P039] 2026-05-05** (originally promoted as P038, renumbered after upstream collision).
- [ ] **External (out of sos-kit scope)** — `~/docs-gate` repo: default `valid_types` should include `chore`. Currently every project that uses `chore`-typed phiếu must add it manually to local `.docs-gate.toml` (Tarot fixed in tarot PR #253).

---

## 🅿️ Park / consider further

- [ ] **Slash command `/build <item>` that runs the full state machine** (DRAFT → CHALLENGE → RESPOND → approval → EXECUTE) end-to-end with one user input. Heavy abstraction; may hide useful debate state. Reconsider after P032/P033 ship.
- [ ] **Telemetry** — opt-in usage stats (which skills, which modes, debate-turn distribution). Useful for evidence-based v2.2 optimization. Privacy + complexity trade-off.
- [ ] **Bidirectional Telegram control** — Sếp gửi command từ phone (e.g. `/idea X`, `/status`, `/approve P005`) → bot trigger Claude Code action remote. Depends on P009 (one-way notification) shipping first + Anthropic `RemoteTrigger` deferred tool maturity. Big concept (auth, security, command parsing). Reconsider sau khi P009 + Sếp dùng Telegram one-way ≥1 tháng.
- [ ] **Concept-confusion-in-prose (Q-D6 open half)** — single-source kills *value*-drift (ticket_dir, sentinel) but NOT *concept*-confusion in prose (someone writing "Tầng" where "Lane" is meant, in free text). No tool built (see Rejected: vocab-tool); tracked as unsolved. Don't close the root question with a pretty "self-dissolves" claim — value-half dissolved, concept-in-prose-half open. Reconsider only on a real grounded incident (n≥1, not hypothesized).

---

## 📌 Recurring routines (not items, but reminders for the maintainer)

- **Pre-merge any PR:** run `/ultrareview <PR#>` for multi-agent cloud review.
- **After 5–10 phiếu or wave end:** run AUDIT_PROTOCOL (Worker AUDIT mode, read-only, writes `docs/AUDIT_<wave>.md`).
- **Per phiếu:** Worker writes Discovery Report to `docs/DISCOVERIES.md` (newest on top) before reporting "done."

---

## ✅ Recently shipped

> Quick reference. Full detail in `CHANGELOG.md`.

- ✅ **P050 + P053 / Két-harvest sprint** — (2026-06-06) — git-level gates from ket dogfood. P050 no-code-on-default pre-commit gate (17/17 test) + P053 sentinel-vs-silent merge deadlock fix (self-validated end-to-end). PR #22 / squash `ae03e5b`. CHALLENGE caught a real hole in each (merge-commit + stale-sentinel→[P055]).
- ✅ **P041 / v2.2.2** — (2026-05-25) — Trinh sát (advisory-watch) specialist subagent + pnpm/npm parsers + slash command `/advisory-scan`. PR #13 / `b253eff`. CHALLENGE round caught 3 V1→V2 issues.
- ✅ **P043 / v2.2.1** — (2026-05-25) — Doc drift consolidate: Quản đốc persona codify, alignment engineering expansion, deferred-tool loading, cap raise ≤90→≤105. PR #12 / `569e02f`.
- ✅ **P040 / v2.2.0** — (2026-05-25) — Bootstrap stack detection (`sos init security` + `.sos-stack.toml` schema + 6 parser stubs underscores). PR #11 / `8047525`. First Tarot port wave 1 phiếu shipped.
- ✅ **Inline edit 2026-05-25** — `agents/orchestrator.md` line 9 + 21: main session persona `Kiến trúc sư` → `Quản đốc` (2-line surgical, no phiếu). Trigger: Sếp directive sau tarot recon — tarot dogfood đã đổi sang Quản đốc, sos-kit cần consistent. **Inconsistency tạm thời:** `docs/ORCHESTRATION.md` line 34-37 vẫn nói "Why Kiến trúc sư persona" — sẽ folded into **[P043]** doc drift consolidate.
- ✅ **Foundation v2.2 sprint COMPLETE** — (2026-04-27) — P036 + P035 + P037 shipped same day (PRs #3 + #4 + #5 merged). Total ~632k tokens / ~45m drive time across all 3. **P037 first Tầng 2 dogfood:** ~5min/81k tokens (68% reduction vs Tầng 1 baseline). **Rule B working:** 0 anchor mismatches at EXECUTE across all 3 phiếu — humility markers prevented hallucination cleanly.
- ✅ **P037 / v2.1.6** — (2026-04-27) — `templates/claude-settings.local.json` pre-approves marker file Bash ops + INSTALL.md Step 2.5 (PR #5)
- ✅ **P035 / v2.1.5** — (2026-04-27) — `agents/orchestrator.md` (~88-line condensed handbook) + ORCHESTRATION.md Hard rule #8 (bulk input → 1 gate) + INSTALL anti-patterns + CLAUDE.md contributor section (PR #4)
- ✅ **P036 / v2.1.4** — (2026-04-27) — Workflow tier routing (state machine `tầng==2` skip-CHALLENGE) + Architect humility markers (`[verified]` / `[needs Worker verify]`). Foundation rules specced (PR #3)

---

## ❌ Rejected (kept here so we don't reconsider in 6 months)

- ❌ **Vocab-consistency tool (glossary + forbidden-variant grep pre-commit)** — (Q-D6, WORKFLOW_V2.3 retro 2026-05-29). Proposed to catch term-drift (INV_LOCAL underscore, Tầng-as-lane, sentinel casing skew, `phieu/active` refs). **REJECTED:** forge demolished the evidence — INV_LOCAL underscore n=0 (grep=0); `Tầng:` is the CANONICAL `TICKET_TEMPLATE` field (forbidding it blocks the golden + pre-empts the open Q-D2); sentinel casing is per-repo-internal (emit/grep never cross repos); `phieu/active` refs are intentional legacy. Net live disease ≈ 0 → a blocklist would false-flag → `--no-verify` death. **Correct mechanism = single-source-the-value** (hook reads canonical from 1 declaration, consumers derive — generalizes Q-D1) + fold within-repo sentinel-consistency into `doctor verify-setup` J1 (built Vòng 12). No standalone vocab grep.

---

## 📌 Maintenance rules

1. **New idea** → `/idea` skill → appends to "Open backlog" or directly to "Active sprint" depending on triage.
2. **Phiếu shipped** → move from Active sprint to "Recently shipped" (keep last ~4 entries).
3. **Sprint complete** → summarize in CHANGELOG, prune "Recently shipped" if it grows past 4 entries.
4. **Discovery debt** new → from `DISCOVERIES.md` → append to "Open backlog" with `[DEBT]` prefix.
5. **Architect rule (hard):** no phiếu for items outside "Active sprint" without explicit promotion from the maintainer.
6. **Monthly Park review:** read Park, decide promote / hold / move to Rejected with reason.

---

*This file is LIVE. Maintainer edits it directly. Architect / Worker subagents READ it during sprint planning but never edit it mid-phiếu.*
