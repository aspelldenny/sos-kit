# Bootstrap Automation — Draft Doctrine (2026-05-28)

> **Status:** DRAFT (2026-05-28) → **ARC KICKOFF 2026-05-29** (see §7). Script-first build greenlit post Q-D5/canary; §5 decisions resolved in §7.5. **Cargo still deferred** (share-time, future — not a scale condition).
> **Why draft (not full doctrine):** sos-kit chưa chín cho cargo hóa. Pattern phải lặp qua ≥2-3 repo mới biết cái gì thật bất biến. Doc-rotate là data source vòng 2. — *Update 2026-05-29: load-bearing pieces have now RUN (§7.1); the bash SCRIPT (§4) is greenlit, cargo stays deferred.*

---

## §0. Diagnosis (Sếp catch 2026-05-28)

Em manual setup doc-rotate **30+ tool call** (mkdir + Write × 15 + cp × 7 + chmod + git + commit × 3). Đáng lẽ là 1 lệnh `sos-kit init`.

**Bệnh gốc (Sếp word):** *"Việc agent copy sos-kit vào repo mới = đang bắt LLM NHỚ phải copy đủ file gì, đặt đúng đâu, config ra sao. Và nó quên — đúng như mọi lần LLM được giao việc-phải-nhớ. Mày không gặp lỗi mới. Mày gặp lại đúng con bệnh cũ ở tầng bootstrap."*

**Slogan áp dụng:** "Đừng bắt LLM nhớ, bắt cơ chế nói sự thật." Bootstrap là việc deterministic nhất đời — PHẢI là tool, không phải prompt. 6 binary trước (ship/docs-gate/guard/vps/quality-gate/advisory-inbox) đã apply pattern này. Cargo hóa bootstrap = nhận ra cái bootstrap cũng là một binary.

**Đối xứng với v2.2 §2 SOUND/PARTIAL:** Bootstrap là việc SOUND (deterministic). Đổ cứng bằng tool = SOUND oracle. Phần PARTIAL (mỗi repo khác) giữ cho agent + validator gác ranh.

---

## §1. Phân loại 3 category (refinement em add — analysis gốc chỉ 2)

Sếp + Claude Web phân 2 category. Em thêm 3rd để full spectrum:

### A. BẤT BIẾN — đổ cứng (SOUND, tool gánh)

Mỗi repo GIỐNG NHAU. Cargo-hóa thắng lớn ở đây — hết quên, hết drift.

| Item | Source | Why bất biến |
|---|---|---|
| `agents/architect.md`, `worker.md`, `orchestrator.md`, `advisory-watch.md`, `boundary-check.md` | sos-kit canonical | Subagent contract — workflow v2.X-bound, không repo-tune |
| `hooks/pre-commit` | sos-kit canonical | Hook chain integrity |
| `scripts/block-env-edit.sh`, `block-unsafe-merge.sh`, `architect-guard.sh`, `session-start-banner.sh` | sos-kit canonical | Universal security/orchestration hooks |
| `scripts/install-hooks.sh`, `pre-push-hook.sh` | sos-kit canonical | Bootstrap helpers |
| Directory structure: `src/`, `tests/`, `docs/`, `docs/ticket`, `docs/ticket/done`, `.claude/agents`, `.claude/commands`, `templates/` | sos-kit pattern | Convention (phiếu live in `docs/ticket/`, unified 2026-05-29) |
| `phieu/TICKET_TEMPLATE.md`, `phieu.sh`, `AUDIT_PROTOCOL.md`, `DISCOVERY_PROTOCOL.md`, `GENESIS_TEMPLATE.md`, `LAUNCH_CHECKLIST.md`, `RELAY_PROTOCOL.md` | sos-kit canonical | Phiếu workflow contract |
| `phieu/VISION_TEMPLATES/*` | sos-kit canonical | Vision skeleton |
| `.claude/commands/advisory-scan.md`, `security-review.md` | sos-kit canonical | Slash command spec |
| Doctor binary install (`cargo install --path ~/Doctor`) | external bootstrap | Required dep |

### B. TUNABLE — default sensible, project override OK (NEW em add)

Sinh default value, project có thể tune. Validator KHÔNG block nếu giữ default.

| Item | Default | Tune example |
|---|---|---|
| `.docs-gate.toml` `changelog_max_age_days` | 1 | Long-running project tune 7 |
| `.docs-gate.toml` `[architecture] file` | `LAYERS.md` (sos-kit), `ARCHITECTURE.md` (downstream) | Per repo |
| `.docs-gate.toml` `[ticket] ticket_dir` | `docs/ticket` | Per repo if different convention |
| Hook chain SECTION list (which gate fires in pre-commit) | type-check + docs-gate + v2 sos-kit + security gate | Per stack disable/add |
| `.mcp.json` server list | minimal core (doctor + docs-gate + ship + guard + advisory-cron) | Add github/sentry per project need |
| Lane budget thresholds (v2.2 §1 — Normal 250 dòng / 5 anchors) | per v2.2 default | Per repo tune via override marker |

### C. PHẢI KHÁC — khung rỗng + validator gác (PARTIAL, agent điền)

Mỗi repo bắt buộc tự điền. Validator BLOCK nếu còn placeholder.

| Item | Template form | Validator check |
|---|---|---|
| `docs/AGENT_MAP.yaml` | Skeleton với example surface COMMENTED OUT + `# TODO: fill at least 1 surface` marker | Grep `# TODO` → fail. + `doctor validate-map` path/anchor exist. |
| `templates/INVARIANTS-template.md` → `docs/security/INVARIANTS.md` (per-repo) | 5 generic INV + section `## INV-LOCAL-*` empty với placeholder | Grep `<INV-LOCAL-N>` placeholder → warn (allow zero local INV if explicit `# No local INV` marker). |
| `CLAUDE.md` per-project section | Skeleton "## Project context" + `# TODO: fill stack + role + constraint` | Grep `# TODO` → fail. |
| `docs/BACKLOG.md` Active sprint | Empty section with `# TODO: pick from Open backlog` | sos-kit v2 hook đã check Active sprint not empty — extends validator. |
| `.sos-stack.toml` (if not auto-detected) | Skeleton `[[stack]] type = "?"` | `sos init security` detect stack → write OR fail-and-prompt. |
| `pyproject.toml` / `Cargo.toml` / `package.json` (stack-specific) | Per-stack template | Stack-detect from CLI flag `--stack python|rust|ts`. |
| Pilot-specific rules in CLAUDE.md (if pilot mode) | Optional section "## Pilot rules" | If `--pilot true` flag → include section template. |

---

## §2. `sos-kit init` should do 3 things (Sếp + Claude Web framework)

```
sos-kit init <project> --stack <python|rust|ts> [--pilot true]

1. ĐỔ CỨNG Category A (BẤT BIẾN)
   - Copy agents/, hooks/, scripts/, phieu/, templates/, .claude/commands/
   - Copy doctor binary symlink check (fail if cargo install not done)
   - Copy directory structure scaffold
   - No prompts, no agent decisions.

2. SINH KHUNG RỖNG Category C (PHẢI KHÁC)
   - Generate AGENT_MAP.yaml skeleton with commented examples + # TODO markers
   - Generate INVARIANTS.md skeleton with INV-LOCAL placeholder section
   - Generate CLAUDE.md skeleton with project-context placeholder
   - Generate BACKLOG.md skeleton (sos-kit template)
   - Generate pyproject.toml/Cargo.toml/package.json from per-stack template (if --stack flag)

3. ĐỔ DEFAULT Category B (TUNABLE)
   - .docs-gate.toml via `docs-gate init` (proper bootstrap)
   - .mcp.json with minimal core server list
   - Hook chain default sections in pre-commit
   - Lane budget defaults

4. CHẠY VALIDATOR — gác ranh A+B+C
   - doctor validate-map (path/anchor exist)
   - doctor verify-setup (hooks wired + agents register + BACKLOG present)
   - Grep <TODO> / # TODO markers in C templates → list to user
   - Output: "✅ A+B đổ cứng X files. ⚠️ C còn N placeholder cần điền: <list>"

5. Optional: --commit flag → git init + first commit if no .git exists
```

---

## §3. Timing cảnh báo (Sếp word — CRITICAL)

> *"Đừng build cái này NGAY BÂY GIỜ. Mày đang giữa nhịp 3 (doctor) và sắp chạy nhịp 4 (pilot doc-rotate). Cargo-hóa-sos-kit là một dự án riêng — nếu mày nhảy vào nó bây giờ, mày bỏ dở pilot, mất luôn cái thí nghiệm partial-oracle đang cần. Tệ hơn: sos-kit chưa chín, v2.2 chưa được pilot doc-rotate kiểm chứng. Nếu mày cargo-hóa bây giờ, mày đóng băng một cái khuôn có thể còn sai, rồi mọi repo sau đẻ ra từ cái khuôn sai đó."*

**Pattern doctrine:** Mày chỉ biết cái gì NÊN đổ-cứng SAU KHI đã thấy nó GIỐNG NHAU qua vài repo.

**Sequence đúng:**
1. ✅ Pilot vòng 1 advisory-inbox (Rust SOUND) — done
2. 🔄 Pilot vòng 2 doc-rotate (Python PARTIAL) — đang chạy
3. ⏸ Pilot vòng 3 repo thứ 3 (TBD, có thể TS partial-oracle) — pending
4. After 3 repo: pattern lặp đủ để biết cái gì THẬT bất biến
5. **Then** cargo-hóa `sos-kit init` proper

**Lý do:** Cargo crate là contract đóng cứng. Mỗi update v2.X → cargo bump → migrate downstream. Nếu schema còn evolving rapidly (v2.1 → v2.2 ship 2 ngày trước, v2.3 sẽ ship sau pilot vòng 2), cargo-hóa = lock vào schema rồi force-migrate khi update.

**Sub-mech reference:** Đây là pattern Sub-mech B (capability gap) — ship doctrine TRƯỚC khi verify đủ instance. Em đã sai pattern này nhiều lần (P285 documented). Cargo-hóa-sớm = lặp pattern ở scale lớn hơn.

---

## §4. Stopgap — bash 50 dòng (Sếp đề xuất)

Pre-cargo solution để giải đau ngay:

```
~/sos-kit/bin/sos-init.sh <project> [--stack python|rust|ts] [--pilot true]

1. Đổ cứng (Category A): cp/mkdir batch
2. Sinh khung rỗng (Category C): cp templates với # TODO markers
3. Đổ default (Category B): chạy docs-gate init + .mcp.json minimal copy
4. Chạy validator: doctor validate-map + grep TODO
5. Output diagnostic: list ✅ A+B done + ⚠️ C TODO
```

**~50 dòng bash, ~20 phút viết.** Đủ chữa đau cho repo TEST mới + future pilot. Doc-rotate vẫn dùng manual setup hiện tại (đang chạy pilot, không động).

**Cargo proper:** roadmap post-pilot doc-rotate + ≥1 repo thứ 3, khi pattern đã lặp đủ.

---

## §5. Decision points cho mai (2026-05-29)

Em chờ Sếp quyết:

1. **Location:** `~/sos-kit/bin/sos-init.sh` (extend existing sos.sh) hay `~/sos-kit/bootstrap/sos-init.sh` (new dir)?
2. **Scope V0:** Full 4-bước (A+B+C+validator)? Hay phase 0 chỉ A (đổ cứng), B+C+validator sau?
3. **Apply first:** repo TEST mới (verify clean baseline) hay defer to doc-rotate re-bootstrap post-pilot?
4. **Validator integration:** invoke `doctor` binary subcmds? Hay inline grep TODO trong bash?
5. **--stack flag values:** chỉ Python+Rust+TS (current 3 pilot stacks) hay open list?
6. **Cargo timeline:** sau pilot vòng 2 only? Hay đợi vòng 3?

---

## §6. Cross-reference

- `~/sos-kit/docs/WORKFLOW_V2.2.md §2` — SOUND vs PARTIAL doctrine (đối xứng bootstrap)
- `~/sos-kit/docs/WORKFLOW_V2.2.md §7` — Sub-mech B capability gap (pattern "ship trước verify đủ")
- `~/sos-kit/docs/BACKLOG.md` — P032 (plugin) + P033 (Rust CLI `sos-kit init`) đã có vision, doctrine này refine
- `~/doc-rotate/` — data source vòng 2, pilot đang chạy
- Tarot evolution — em manual backport P230/P273/P285+/P297/P305/P306 vào sos-kit 30+ commit. Cargo hóa sẽ giảm future backport cost.

---

**Provenance:** Sếp 3 messages 2026-05-28 chiều + Claude Web analysis (forwarded by Sếp) + em (Quản đốc) 3 refinement add.

---

## §7. Reconciliation 2026-05-29 — ARC KICKOFF (post Q-D5 verify-setup + tarot canary)

> Added after the v2.3 retro shipped 4 pain-cures. **Goal shifted:** from "relieve manual-setup pain" → **make spawning a new product cheap + safe = the scale foundation** (the 2-dead-weeks investment so future products don't re-pay infra cost). This section reconciles the 2026-05-28 draft with what matured, applies the freeze-filter discipline, resolves §5, and sets the done-when. **Bounded kickoff — spec only, no build yet.**

### 7.1 What changed — the §3 timing warning, re-judged
§3 warned "don't cargo — mold not mature, need 3 repos." Re-judged: the §4 **bash script was ALWAYS allowed** (§3 only warned against CARGO). The load-bearing pieces have now RUN across repos: spine A1 (5 repos / 3 gens), single-source `ticket_dir` (Q-D1 canary), sentinel (fixed + canary), `verify-setup` (discrimination-passed Vòng 12), boundary canary (tarot v166 real-gate). → **Build the SCRIPT now** (scale-unlock). **Cargo stays deferred** — §3's core holds; cargo is share-time (future), not a scale condition. The script is no longer a "stopgap" — it's the deliberate scale foundation.

### 7.2 Freeze-filter — apply the verify-setup-saving discipline to Category A
Category A "đổ cứng" must be filtered **joint-by-joint ("did THIS run")**, NOT frozen as a bundle — else new repos inherit dormant members, multiplied (the cargo-too-much trap):
- ✅ **Freeze (proven-cắn):** agent `.md` files (spine A1), scripts (`block-env-edit`, `block-unsafe-merge` **with the FIXED sentinel**, `architect-guard`, `session-start-banner`), `phieu/` workflow, `templates/`, `.claude/commands/`, dir structure.
- ⚠️ **`hooks/pre-commit` — freeze the FIRING sections only:** docs-gate + security-gate + ticket_dir single-source. `lane-check` / doctor-as-blocker were **DORMANT** (retro Cột B) → do NOT freeze them as "active"; keep graceful-skip or exclude until actually wired.
- 🔌 **`verify-setup` is NOT in any hook yet** (CLI/MCP only — grep of `hooks/pre-commit` confirms). Wiring it as the post-spawn gate (§2 step 4) + optionally pre-commit IS new work, not "already have it."

### 7.3 verify-setup ≠ the whole validator (scope clarity)
The 2026-05-28 §2 step-4 imagined `doctor verify-setup` = "hooks wired + agents register + BACKLOG present." After Q-D5, verify-setup is specifically the **Giám sát (boundary-check) wiring chain** (J1 sentinel / J2 rubric / J4 invariants / J5 merge-gate / J6 verdict). **Keep it narrow.** The **bootstrap validator** is a COMPOSITE of separate checks:
- `doctor verify-setup` — boundary-check role wired (if security enabled)
- `doctor validate-map` — AGENT_MAP path/anchor exist
- grep `# TODO` / placeholders in Category-C templates
- BACKLOG Active-sprint present (existing sos-kit v2 hook check)

### 7.4 DONE-WHEN — the acceptance test = discrimination test for the spawn mechanism
NOT "the script runs without error." The proof:
1. `sos new <test-repo> --stack <X>` on an empty dir →
2. `doctor verify-setup` returns **CONNECTED zero-hand-fix for the WIRING joints (J1/J2/J5/J6)** — copied from golden, must be intact with no manual repair.
3. **J4** (`docs/security/INVARIANTS.md`) is EXPECTED-empty on a fresh spawn → the validator must report it as a **Category-C "fill this" TODO**, NOT a hard wiring-DORMANT. → **this REQUIRES the J4-conditional hardening** (the deferred residual surfaces HERE as required work: distinguish "wiring broken" from "content-slot awaiting fill"). *(This is the tool-vs-product principle in action: do the completion when the pain it cures actually recurs — bootstrap is where J4-conditional recurs.)*
4. **Negative oracle:** the spawned repo must NOT resemble doc-rotate (sentinel mismatch / missing wiring). If verify-setup flags a WIRING joint broken on a fresh spawn → bootstrap is incomplete, and that's the find.

### 7.5 §5 decision points — RESOLVED (recommendations)
1. **Location:** extend `bin/sos.sh` (already has `init`/`blueprint`/`contract`) — add/beef `sos new` (full A+B+C+validator). NOT a new dir.
2. **Scope V0:** full 4-step (A đổ-cứng + C sinh-rỗng + B default + composite validator), as **BASH** (§4). Each step small.
3. **Apply first:** a fresh THROWAWAY test repo (clean baseline = the §7.4 acceptance test). Do **NOT** re-bootstrap doc-rotate (live pilot — leave it; one-measurement discipline).
4. **Validator:** invoke `doctor verify-setup` + `doctor validate-map` (both built now) + inline grep TODO. Hybrid.
5. **--stack:** Python + Rust + TS (current 3 pilot stacks). Open list later.
6. **Cargo:** deferred — script first; cargo when SHARING (future). §3 core holds for cargo.

### 7.6 NOT in this arc (completion, not scale-blocker — tool-vs-product discipline)
Q-D7 adversarial canary, advisory→block, Q-D2 doctrine mầm-bệnh, J7/J9 beyond bootstrap need — none block "spawn a repo that runs." **J4-conditional IS pulled in** (§7.4 needs it — completion surfacing at the real use-case). **Wave D** (lower doctrine to golden) = step-1 of bootstrap = *defining the freeze-filtered Category A* = happens AS the script is built (one stream, not a separate task).

### 7.7 RESULT — built + ran (2026-05-29, same session)
`sos new` built in `bin/sos.sh` (a do-er like `sos init security`, not a printer). **Acceptance test PASSED:**
- `sos new <repo> --stack python` AND `--stack rust` on empty dirs → `doctor verify-setup` = **CONNECTED, 5/5 joints WIRED, exit 0, ZERO hand-fix.**
- TODO-grep surfaces the Category-C fill list (INV-LOCAL, AGENT_MAP, CLAUDE.md) — separately from the wiring verdict.
- **Teeth test (negative oracle):** corrupt the spawned repo's sentinel (mimic doc-rotate's death) → verify-setup flips to **DORMANT (J1 BROKEN, exit 1).** The gate discriminates correct-spawn (CONNECTED) from tampered-spawn (DORMANT) — NOT rubber-stamping its own output.
- **Build revealed a cleaner design than §7.4 predicted:** bootstrap **LAYS DOWN** `docs/security/INVARIANTS.md` (generic-5 universal + empty INV-LOCAL + TODO) → J4 PASSES (file exists) with **no verify-setup change**. "Fill INV-LOCAL" is caught by the separate TODO-grep. So **J4-conditional hardening is NOT required for the bootstrap MVP** (it stays optional polish — distinguishing template-only from filled — for when that pain actually recurs). This is RUN-beats-forge: building found the cleaner split (wiring=verify-setup, content=TODO-grep).
- **Caveat:** test used `DOCTOR_BIN` → the freshly-built release binary (PATH still has doctor 0.1.0). Real use needs `cargo install --path ~/doctor` to land `verify-setup` on PATH. Cargo-packaging of sos-kit still deferred (the script is proven sufficient for scale).

### 7.8 Cross-check fixes + honest labels (2026-05-29)
Post-build cross-check (Claude Web + Codex) → fixed 2 gaps + cargo-installed + re-ran:
- **Gap 1 (non-empty guard):** `sos new` now refuses a non-empty target (verified: occupied dir untouched, exit 1) unless `--force`. A command named `new` must not silently overwrite (adopting an existing repo = future `sos adopt`).
- **Gap 2 (born-ready repo):** spawned repo failed its OWN docs-gate (missing ARCHITECTURE/CHANGELOG + a double-`docs/` path bug). Fixed: generate `docs/ARCHITECTURE.md` + root `CHANGELOG.md` (dated) skeletons + a `.docs-gate.toml` **mirroring sos-kit's working config** (`docs_dir`, `changelog="../CHANGELOG.md"`, architecture `file="ARCHITECTURE.md"`, `required_sections=0`). **RESULT: a fresh spawn now passes its ENTIRE pre-commit chain (docs-gate + BACKLOG + security-gate, exit 0)** — born-ready, not just verify-setup-CONNECTED.
- **cargo install done:** `doctor verify-setup` now on PATH (closes the DOCTOR_BIN caveat).
- README sos-CLI row updated to list `sos new` (Tầng-1).

**Honest labels (Codex cross-check — accepted, no overclaim):**
- **(a) J4-pass is a harmless tautology, NOT a design-win.** `sos new` CREATES `docs/security/INVARIANTS.md` then verify-setup checks it exists → it passes because the file was just laid down. ALL joints tautologically pass on a *correct* spawn (that's what "correct" means); the **teeth test** (break→DORMANT) is what proves the joints aren't blind. J4 = "file exists" ≠ "INV-LOCAL filled" → a fresh spawn is **wiring-ready, NOT guard-ready** (INV-LOCAL still TODO; TODO-grep flags it). The right design is the wiring/content SPLIT (verify-setup vs TODO-grep), not "J4 passes."
- **(b) Proven = wiring-correct-at-spawn, NOT spawned-repo-guards-well.** verify-setup checks the WIRE is connected; it does NOT check the spawned repo's boundary-check actually CATCHES violations (that's canary/Q-D7, not run on spawned repos — guard-quality is inherited from golden, which is only partial-canaried at tarot). "Scale foundation proven" = **wiring proven**; guard-quality on spawned repos = untested.

**Remaining (the one open fork — Sếp call):** wire `doctor verify-setup` into `hooks/pre-commit` so drift-AFTER-spawn is caught (not just at-spawn). Tradeoff: Codex → WARN-mode (not BLOCK — brittle gate that blocks invites `--no-verify` death); Claude Web → not yet (don't add hook surface before 1-2 real repos). **Wrinkle:** wiring now would fire DORMANT-J4 on sos-kit-self (kit ships template, no real INVARIANTS.md) = noise that trains ignore-the-warn → pair it with the **J4-conditional** (distinguish kit/awaiting-fill from broken) so it's never sủa-bừa. → Defer until J4-conditional, or pair them as the next bounded step. STOP-line unchanged: no `WORKFLOW_V2.3.md` golden ratification until Q-D2 mầm-bệnh + Sếp approve.

### 7.9 Flatten — the A/B fork was MIS-FRAMED (3-eye convergence, 2026-05-29)
Sếp chose B (defer). Codex then caught that the shared premise of BOTH A and B was wrong — and that reframe is RIGHT, with two corrections from the RUN evidence:
- **Premise error (Codex, accepted):** A and B both assumed "wire verify-setup into **sos-kit's** pre-commit." But sos-kit is the TEMPLATE/golden, not an operating repo — *you don't drift-gate the mould, you drift-gate what's cast from it.* verify-setup's DORMANT-J4 on sos-kit is ACCURATE (the kit ships `templates/INVARIANTS-template.md`, no live `docs/security/INVARIANTS.md`) but **not actionable for a template.** → Don't wire verify-setup into sos-kit's pre-commit. The drift-gate belongs in the CHILD repo, wired BY `sos new` at spawn.
- **Correction 1 (mechanism — RUN-grounded, refutes Codex's block-first-commit worry):** Codex feared a fresh child's first commit would be BLOCKED because "J4 sees INV-LOCAL still TODO." **FALSE — verified:** a fresh spawn is verify-setup **CONNECTED** *while* the TODO-grep simultaneously lists `docs/security/INVARIANTS.md`. J4 checks the file EXISTS (sos new lays it down) — NOT whether INV-LOCAL is filled (that's TODO-grep's separate job). So a child is CONNECTED from birth; verify-setup in its pre-commit PASSES and only fires on REAL drift (sentinel changed / INVARIANTS deleted / registration removed). → **No J4-conditional needed** (it solved a non-problem); no WARN→BLOCK graduation machinery needed. The 5 implemented joints (J1/J2/J4/J5/J6) are all copied-from-golden + proven; J7/J9 aren't implemented so can't false-fire. WARN default (arm-not-decree); BLOCK is also defensible (a bootstrapped child is verify-setup's PROVEN known-pattern case, not the brittle edge).
- **Correction 2 (mirror-config = copy, and that's CORRECT):** Gap-2's child `.docs-gate.toml` is a GENERATED COPY (bash heredoc), not a single-source reference. For Category B (tunable per-repo, §1.B) that's the INTENDED design — children evolve config independently. The only real coupling is generator↔docs-gate-schema, caught by the acceptance test (spawn → docs-gate pass). NOT a hidden single-source-drift.
- **Convergence (3 eyes, each one piece):** B = right TIMING (defer to pilot — don't polish platform in a vacuum, per tool-vs-product); Codex = right PLACEMENT (child via sos new, not sos-kit); orchestrator = right MECHANISM (fresh child CONNECTED → clean wire, no J4-conditional). **Lesson: before arguing A-vs-B, check the premise A and B SHARE.** Here the shared premise ("gate the template") was the bug.
- **RESOLUTION:** DEFER (B) — get evidence from the real pilot first. WHEN the pilot surfaces real drift, wire verify-setup into the CHILD's pre-commit via `sos new` (WARN default), NOT into sos-kit, NO J4-conditional. Recorded so we wire it RIGHT when the time comes.
