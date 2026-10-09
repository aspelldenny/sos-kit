# SOS Kit Portability Architecture

> P074 target decision, 2026-07-20. Current reality nằm ở `docs/RUNTIME_BOUNDARY_INVENTORY.md`.

## Product decision

SOS Kit là **một product trong một monorepo**, phát hành theo một version và có một user-facing binary: `sos`.

```text
one repository → one release/version → one `sos` entrypoint
                                      ├─ portable core
                                      ├─ Claude Code adapter
                                      ├─ Codex adapter
                                      ├─ Git hooks/gates
                                      └─ managed sister tools
```

Một repo không đồng nghĩa một module. Boundary được giữ bằng Rust crates và dependency rules; người dùng không phải hiểu hoặc cài từng crate/tool.

## Target workspace

Tên crate là target shape để P077 chốt, không phải cam kết public API trong P074:

```text
sos-kit/
├── Cargo.toml                    # workspace + version chung
├── crates/
│   ├── sos-cli/                  # binary duy nhất: `sos`
│   ├── sos-core/                 # state, policy, roles, workflow, config schema
│   ├── sos-install/              # plan/apply/sync/rollback/manifest
│   ├── sos-adapter-claude/       # Claude renderer + detector + lifecycle binding
│   ├── sos-adapter-codex/        # Codex renderer + detector + permission/config mapping
│   └── sos-hooks/                # universal Git hook dispatcher
├── core/                         # canonical Markdown/templates/recipes, runtime-neutral
├── adapters/
│   ├── claude/                   # adapter-owned templates/assets
│   └── codex/
└── tool-manifest.toml            # managed external tool versions/assets/checksums
```

Rust workspace (repo-root `Cargo.toml`+`crates/`, relocated from `bootstrap/sos-rs/` P077f) là nguồn khởi đầu được nâng vào workspace này ở P077, không extract sang repo khác. **P077e đã sửa `CLAUDE.md` contract** ("Runtime monorepo (as of P077e)" thay "Not a runtime binary source") cùng lúc workspace trở thành canonical cho 6 heavy subcommand.

**P077b status (crate boundary carved):** 5 crate skeleton dựng tại `crates/` (relocated from `bootstrap/sos-rs/crates/` P077f) với đúng dependency direction (adapter→core, core zero adapter dep, CLI composition root — xem `crates/README.md` "Module layout"). Deviation so target — đúng 3 điểm đã biết trước:
1. ~~Workspace root ở `bootstrap/sos-rs/` (target: repo-root)~~ — **RESOLVED P077f**: workspace relocated to repo-root, layout khớp target tree (dòng 24-39).
2. ~~`sos-adapter-codex` chưa tạo~~ — **RESOLVED P078b1**: crate created, `detect()`/`verify()` live (foundation status line dòng 57).
3. `sos-install`/`sos-adapter-claude`/`sos-hooks` là skeleton rỗng — logic P077d.

**P077d1 status (adapter contract + manifest schema carved):** trait `Adapter` (5 method — `detect/plan/render/verify/uninstall`, khớp dòng 63-69 dưới) render tại `sos-core/src/adapter.rs` cùng type runtime-neutral (`Capabilities`/`Plan`/`Artifact`/`Findings`/`RemovalPlan`); `ManagedManifest` (6 field, khớp `core/ASSETS.md:57-64`) render tại `sos-core/src/manifest.rs`, serde TOML round-trip tested. `sos-adapter-claude` implement trait với stub bodies (zero fs mutation, zero install logic — đó là P077d2). Dependency-direction giữ nguyên (adapter→core one-way, guard test vẫn xanh).

**P077d3 status (tool-manifest live, OA-07 RESOLVED trong Rust path):** `tool-manifest.toml` (kit root, committed — KHÁC `.sos-manifest.toml` d2 generated) pin 10 sister tool (`name`/`version`/`required` + `[tool.asset]`/`[tool.checksum]` per 3 target-triple), 6 required (`doctor`/`claude-hooks`/`docs-gate`/`ship`/`advisory-inbox`/`inv-gate`, khớp `install.sh:40`) / 4 optional (`guard`/`vps`/`doc-rotate`/`advisory-cron`, khớp `install.sh:48`). Version fill từ source `Cargo.toml` mỗi tool (KHÔNG `releases/latest` — chính là bug OA-07), checksum = honest `TODO-sha256-<tool>-P081` placeholder (chưa có prebuilt asset hash cho pin shape này — E2, real fill = P081). `sos_install::tools` (NEW module) — `check_tools()` chạy `<tool> --version` (E1: format `"<name> <x.y.z>"` đồng nhất trên cả 9 tool live-test, không tiền tố `v`), hand-rolled dotted-tuple compare (KHÔNG có crate `semver` trong workspace — anchor #8), `Verdict` (Ok/Newer/Drift/Missing/Unparseable), `gate_required()` fail-closed core dùng CHUNG bởi 2 surface: **`sos tools status`** (table + exit 1 nếu required drift/missing, optional = warn-only exit 0) và **step 5 `resolve_tools()`** (nay ĐỌC manifest thật, KHÔNG còn no-op — fail trước khi mutate fs, nên "rollback" trivial vì chưa ghi gì). **KHÔNG có `sos doctor` subcommand** trong sos-cli (mọi `doctor` reference là shell-out `Command::new("doctor")` gọi binary ngoài) → fail-clear surface qua 2 đường trên, không tạo subcommand xung đột. d3 GIỚI HẠN: verify-only, KHÔNG tải/atomic-upgrade/auto-fetch — đó là P081. Live smoke trên dev machine reproduce đúng OA-07 evidence (doctor 0.1.1 vs pinned 0.1.3, inv-gate MISSING, exit 1). `install.sh`/`bin/sos.sh` legacy `releases/latest` giữ nguyên tới P077e cutover.

**P077d2 status (install engine live):** `sos-install::engine` implement transaction step 1-4+6-7 (step 5 tool-resolve NAY ĐÃ FILL bởi P077d3, xem status line trên — trước đó là stub seam `// SEAM P077d3`, `resolve_tools()` trả về empty), driven THUẦN qua `Adapter` trait — engine zero host token. **Narrow d1 amendment** (Worker CHALLENGE Turn 1 O1.1, Architect ACCEPT Alt A): `ManagedOperation` (`sos-core/src/adapter.rs`) thêm field `content: String` — `plan()` trước đó chỉ trả path+description, không đủ cho engine biết ghi bytes gì; Adapter 5-method + ManagedManifest 6-field KHÔNG đổi. Non-clobber discrimination = 4-way (`Create`/`NoOp`/`Update`/`Conflict`): target absent → CREATE; target hiện có + hash == desired → NoOp (idempotence, zero mutation); hash khác desired nhưng khớp `content_hash` đã ghi trong `.sos-manifest.toml` → UPDATE cho phép; khác + không khớp/không record → CONFLICT, stage `.sos-install-incoming/<path>` (mirror `.sos-adopt-incoming`), original giữ nguyên. Rollback = snapshot-before-mutate (CREATE→delete on fail, UPDATE→restore prior bytes), `.sos-manifest.toml` không commit khi fail, exit non-zero. Manifest artifact: `.sos-manifest.toml` tại project root, `[[managed]]` array-of-tables. Oracle = 5 correctness fixtures (`crates/sos-install/tests/install.rs`, `MockAdapter`) — KHÔNG parity-vs-Bash (lệnh mới). `sos install --runtime <auto|claude|codex>[--dry-run]` wired trong `sos-cli`; `claude` dùng `ClaudeAdapter` (vẫn stub d1 → plan tối thiểu/rỗng, render thật vẫn defer); `codex` → lỗi rõ "not yet available (P078)". `bin/sos.sh`/`install.sh` zero-touch.

**P077e status (CUTOVER LIVE — approach A, founder-confirmed):** Rust `sos` binary nay canonical cho 6 heavy subcommand (`new`/`adopt`/`sync`/`map`/`install`/`tools`) — `bin/sos.sh` case-block `exec` thẳng binary (resolver: `SOS_RUST_BIN` → build-dir → `cargo build` on-demand → fail-LOUD, KHÔNG `command -v sos` chống recursion với launcher cùng tên). 7 guidance subcommand (`init`/`blueprint`/`contract`/`apply`/`recipe`/`launch`/`status`) giữ nguyên Bash tới P078 render per-runtime. Bash `sos_new`/`sos_adopt`/`sos_sync`/`sos_map` giữ DORMANT (rollback safety + oracle, không xóa). Repo contract flipped: root `CLAUDE.md` "What this repo is NOT" #1 + Rules #1 + repo-structure nay khai runtime monorepo. Workspace tại thời điểm này GIỮ NGUYÊN ở `bootstrap/sos-rs/` (relocate repo-root Cargo.toml = **P077f, deferred**, orthogonal + reversible, tách khỏi cutover irreversible-ish này). `install.sh` **zero-touch** (prebuilt-binary bootstrap của `sos` = P081, chưa đổi). Cutover smoke: heavy cmd qua `bin/sos.sh` == gọi thẳng Rust binary == golden (verified). `cargo test --workspace` ×20 = 0 flaky.

**P077f status (RELOCATE DONE — pure move, KHÔNG đổi logic/behavior):** workspace (`Cargo.toml`+`Cargo.lock`+`crates/`) `git mv` từ `bootstrap/sos-rs/` → repo-root; `bootstrap/` gỡ (rỗng sau move). Layout nay khớp "Target workspace" tree (dòng 24-39) — deviation #1 ở P077b RESOLVED. Path-fix: `crates/sos-cli/tests/parity.rs` CARGO_MANIFEST_DIR depth (4→2 hop tới `scripts/install-hooks.sh`); `crates/sos-install/src/tools.rs` `include_str!` depth (5→3 hop tới `tool-manifest.toml` — site ngoài `CARGO_MANIFEST_DIR` family, phát hiện ở Worker CHALLENGE Turn 1 O1.1); `bin/sos.sh` resolver `_sos_workspace_root` → repo-root; `scripts/orchestrator-guard.sh` allow-list glob thêm `crates/*` (Worker CHALLENGE Turn 1 O1.2 — security-surface preserve-behavior fix, giữ `bootstrap/*` như dead glob); `.gitignore` `bootstrap/*/target/` → `/target/`; trust-baseline rebaseline (`bin/sos.sh` + `orchestrator-guard.sh` hash đổi). `cargo build/test --workspace` từ root xanh, ×20 = 0 flaky, cutover smoke + guard smoke verified. **Reversible** (`git revert`). **P077 HOÀN TOÀN DONE (a-f).**

**P078b1 status (Codex adapter foundation — decompose 1/3, b2/b3 follow):** `crates/sos-adapter-codex` (NEW, deps CHỈ `sos-core`) — `CodexAdapter` implement trait `Adapter`. `detect()` structural: static Codex 0.145.0 facts (`docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:32`) + fail-safe `codex --version` probe (absence không panic — structural oracle chạy trên máy KHÔNG cài codex, Decision 5). `verify()` = machine surface PARTIAL-declaration mechanism: trả đúng 5 `Finding` (mỗi cái mang `FindingStatus::Partial`/`Missing`, KHÔNG có `Sound` — `crates/sos-adapter-codex/src/lib.rs` test `verify_reports_exactly_five_gaps_none_sound`), khớp 5 gap từ report (per-role tool allowlist / repo slash commands / skill allowed-tools / native ticket-approval / Read-Glob interception). `FindingStatus` enum (`Sound`/`Partial`/`Missing`, additive, `crates/sos-core/src/adapter.rs`) thêm vào `Finding` — trước đó KHÔNG có field status; `ClaudeAdapter` không cần compile-fix (nó không construct `Finding`, chỉ `Findings::default()`). `plan()`/`render()`/`uninstall()` = minimal-honest stub (empty) — render bytes thật = P078b2/b3. Wired `sos-cli`: `install --runtime codex` không còn lỗi "not yet available" — construct `CodexAdapter`, drive `sos-install::engine` giống hệt nhánh `claude` (engine nhận `&Plan`/`project_root`, KHÔNG `Adapter`-typed param — zero engine change, Worker CHALLENGE Turn 1 anchor #12 confirmed). Declarative docs `adapters/codex/{README,MAPPING,CAPABILITY}.md` (NEW, mirror `adapters/claude/`) — `CAPABILITY.md` = frozen 5-gap declaration seeded từ `verify()`. **Oracle STRUCTURAL** (Decision 5): `cargo build/test --workspace` xanh ×20 = 0 flaky; dep-direction guard xanh (`sos_adapter_codex` substring-matched bởi forbidden-token scan sẵn có, zero regex change cần); `install.sh`/`bin/sos.sh`/`sos-install/engine.rs` diff rỗng (additive verified). `install --runtime codex --dry-run` reach ĐÚNG code path như `claude` (bail string biến mất, grep confirm) — full exit-0 trên dev machine này bị chặn bởi tool-manifest pin drift KHÔNG LIÊN QUAN (P077d3 step-5 gate, xảy ra GIỐNG HỆT với `--runtime claude`, pre-existing trước phiếu này, KHÔNG phải regression). **Behavioral** (Codex CLI thật chạy đúng output) = P079, ngoài scope b1.

**P078b2 status (Codex declarative render — decompose 2/3, b3 follow):** `CodexAdapter::render()`/`plan()` (`crates/sos-adapter-codex/src/lib.rs` + NEW `src/templates.rs`) nay LIVE — sinh 10 artifact declarative: `AGENTS.md` (root), 4× `.codex/agents/{architect,worker,advisory-watch,boundary-check}.toml`, 4× `.agents/skills/{idea,forge,apply,retro}/SKILL.md`, `.codex/config.toml`. Trait shape confirmed coherent (Worker CHALLENGE Turn 1 anchor #1/#3): `render(&self, asset:&Asset, capabilities:&Capabilities)->Artifact` là per-Asset; `plan()` tự enumerate 10-Asset cố định rồi gọi `render()` từng cái, map `Artifact{target_path,content}` → `ManagedOperation{description,target_path,content}` — engine (`sos-install::engine`) tiêu thụ generic, KHÔNG cần đổi. Content-source = template/format-string crate-embedded (`templates.rs`) — render() KHÔNG đọc `core/**` filesystem lúc chạy; mỗi artifact mang **pointer** core ID (`core/ROLES.md#<role_id>` etc.), KHÔNG copy semantics (`core/ASSETS.md:51`, mirror Claude P076 provenance marker). PARTIAL-honest: `architect.toml` `developer_instructions` mang marker "envelope PARTIAL — enforce qua PreToolUse hook P078b3 + prose, KHÔNG structural tool-removal", khớp `verify()` Finding #1; `advisory-watch.toml`/`boundary-check.toml` render honest `sandbox_mode="read-only"` (không PARTIAL marker — sandbox thật enforce read-only cho 2 role này). Skill set = 4 (`idea`/`forge`/`apply`/`retro`) — `init` **deferred, không silently drop** (`MAPPING.md`: `caller` Claude = `sos init CLI` — Claude-CLI-bound, chưa có Codex-native trigger). Escape hatch chưa đóng: advisory-watch cần network (GHSA query) nhưng `sandbox_mode="read-only"` — report không xác nhận sandbox có gate network hay không; render giữ read-only (honest cho filesystem dimension) + comment UNCONFIRMED, behavioral resolve = P079. **Oracle STRUCTURAL** (Decision 5): 13 unit test mới (TOML parse OK cho 4 `.toml`+config.toml; frontmatter name+description cho 4 SKILL.md; AGENTS.md non-empty + chứa pointer; mỗi 10 artifact chứa đúng core-ID pointer; PARTIAL marker present đúng chỗ, absent đúng chỗ; `plan()` sinh đúng 10 `ManagedOperation`) — chạy KHÔNG cần Codex cài. `cargo build/test --workspace` xanh ×20 = 0 flaky. Dep-direction guard xanh (`grep -rE 'adapters/|\.codex' crates/sos-core/src/` → 0). Additive verified: `git diff bin/sos.sh install.sh crates/sos-install/src/engine.rs crates/sos-core` rỗng; `.codex/`/`AGENTS.md`/`.agents/` KHÔNG xuất hiện trong `git status` của sos-kit (Decision 6 — render chỉ tới target project qua `install --runtime codex`, chưa test end-to-end vào 1 dir thật ngoài repo). `install --runtime codex --dry-run` bị chặn bởi CÙNG tool-manifest pin drift như `--runtime claude` (pre-existing, KHÔNG regression, giống b1 precedent) — 10-artifact enumerate verify qua unit test thay vì CLI print trực tiếp. **Behavioral** (Codex CLI thật đọc/chạy artifact) = P079. b3 (enforcement: `.codex/hooks.json`/`.codex/rules/**`/rewritten guard scripts) = phần cuối P078b.

**P078b3 status (Codex enforcement — decompose 3/3, P078b DONE):** `all_assets()` mở rộng 10→17 — thêm 7 enforcement artifact (`crates/sos-adapter-codex/src/templates.rs`): `.codex/hooks.json` (wire SessionStart/UserPromptSubmit/SubagentStart-Stop/PreToolUse/Stop → 5 guard script), `.codex/rules/exec-policy.rules` (Starlark `prefix_rule` force-push/destructive deny), 5× `scripts/codex/*.sh` guard REWRITE (architect/orchestrator/block-env/approval-gate/idea-smell — KHÔNG copy Claude bytes, viết lại cho payload shape Codex). **Anchor #1 CRITICAL (apply_patch marker syntax) RESOLVED via Debate Log Turn 2**: Worker CHALLENGE Turn 1 escalate (report không khai marker syntax, chỉ có training-prior chưa verified) → Sếp chạy live Codex CLI (gpt-5.6) probe, capture 4 payload thật → xác nhận đúng V4A envelope (`*** Begin Patch\n*** Add|Update|Delete File: <path>\n...*** End Patch`, embedded newline = literal `\n`) — fixture commit `crates/sos-adapter-codex/tests/fixtures/codex-apply-patch-payloads.jsonl`, KHÔNG đoán. **fail-CLOSED (Decision 1, đảo cực Claude fail-open):** mọi guard xử lý apply_patch — path extract fail (grep no-match) → BLOCK exit 2, KHÔNG allow. Path extraction: `grep -oE '\*\*\* (Add|Update|Delete|Move) File: [^\\"]+'` trên raw JSON line (dừng ở escape `\n` hoặc closing quote) — verify khớp cả 3 real Add/Update/Delete fixture. **approval-gate (E5, Codex native gap #4 guard-BUILT):** đọc projection `.sos-state/ticket-state.env` (`version`/`approved_version`, Worker Tầng-2 layout decision — `core/STATE.md` không chỉ định format cụ thể), read-compare ONLY, KHÔNG mutate; fail-CLOSED khi state file missing. **block-unsafe-merge: DEFER semantic** (Decision 4) — mechanical force-push qua rules, PR-sentinel gate vẫn ở `claude-hooks` binary ngoài (Git/CI backstop). **PARTIAL-honest 3-surface:** `.codex/hooks.json` `_partial_note` field + mọi guard header + `CAPABILITY.md`/`MAPPING.md`/`SECURITY.md` đều ghi rõ bypassable (untrusted repo/disabled hook/obfuscated command) → Git/CI backstop retained, KHÔNG tuyên bố security boundary kín. **Oracle STRUCTURAL + mock-payload (Decision 5):** hooks.json valid JSON + đúng event set; mọi guard `bash -n` clean; 20 mock-payload test (`crates/sos-adapter-codex/src/lib.rs` `mock_payload_oracle` module, `#[cfg(unix)]`) feed real+path-substituted-real fixture → assert block(exit2)/allow(exit0) đúng cho architect/orchestrator/block-env/approval-gate — dùng CẢ real fixture lines LẪN synthetic payload giữ nguyên envelope thật, chỉ đổi path. `plan()` giờ enumerate 17 (10+7) — test cũ 10-count amend thành 17, KHÔNG duplicate. `cargo build/test --workspace` xanh, ×20 = 0 flaky. Dep-direction guard xanh (`grep -rn 'adapters/|\.codex|apply_patch' core/` → 0). Additive verified: diff `bin/sos.sh`/`install.sh`/`crates/sos-install/src/engine.rs`/`crates/sos-core` rỗng; `.codex/`/`scripts/codex/` KHÔNG xuất hiện `git status` sos-kit. `install --runtime codex --dry-run` bị chặn bởi CÙNG tool-manifest pin drift pre-existing (KHÔNG regression, giống b1/b2). **Behavioral** (Codex CLI thật enforce hook/apply_patch/rules) = P079, ngoài scope. **P078b HOÀN TOÀN DONE (b1+b2+b3).** Codex adapter build coi như đủ — phần còn lại của P078 là behavioral live-dogfood (P079, Sếp chạy).

**P078c status (render-before-toolgate reorder — unblock P079 dogfood):** `install --runtime <codex|claude>` step order đổi TỪ `plan()→resolve_tools()?→apply()/dry_run()` (tool-drift HARD-BLOCK render qua dấu `?`) SANG `plan()→apply()/dry_run()→tool-check-report` (render TRƯỚC, tool-check SAU, KHÔNG abort). Concern-conflation gốc (`docs/discoveries/P077d3.md`): render adapter files KHÔNG phụ thuộc sister-tool version — chỉ tool-manifest gate mới cần đo "workflow-ready". Reorder = ZERO thay đổi ở `sos-install::engine`/`sos-install::tools` — cả 2 concern vốn đã độc lập callable (`engine::apply()`/`engine::dry_run()` không nhận tool-status làm input, tự thân đã document rõ "Step 5 tool-resolve is intentionally NOT called here"; `tools::check_tools()`/`tools::required_drift()`/`tools::describe_failure()` vốn đã `pub` + non-`Result` — install.rs chỉ đổi CÁCH GỌI, tái dùng nguyên các hàm này thay vì `resolve_tools()?`). 2 hàm `run_claude()`/`run_codex()` refactor thành 1 hàm chung `run_adapter(&dyn Adapter, owner, runtime_label, dry_run, require_tools)` (symmetric tự thỏa qua shared path, KHÔNG duplicate logic). **Exit-code 3-way contract (CHỐT):** `0` = render OK + tools ready; `3` = render OK NHƯNG tool-drift/missing (installed-but-tools-not-ready — WARNING loud stderr + tool list + pointer `sos tools status`, KHÔNG bao giờ nuốt silent); `1` = render/apply THẬT SỰ lỗi (nguyên trạng, rollback). Cơ chế: `std::process::exit(3)` gọi trực tiếp trong `report_tool_drift()` sau khi in warning — tái dùng pattern có sẵn (`commands/tools.rs:51`, `commands/launch.rs:34`), KHÔNG cần đổi `main.rs` return-type/`ExitCode` plumbing, KHÔNG phá `?`-error-handling ở chỗ khác. **`--require-tools` opt-in (CHỐT: CÓ)** — clap flag mới trên `Install` (`main.rs`), khi set khôi phục hành vi PRE-P078c y hệt: `resolve_tools()?` chạy TRƯỚC apply/dry_run, drift → abort exit 1, KHÔNG render (fail-closed mạnh nhất, dành CI/production). `--require-tools --dry-run` cùng lúc: vẫn gate trước (Tầng-2 CHỐT — CI muốn cùng tín hiệu fail-closed dù dry-run không mutate). **dry-run show-both:** giữ nguyên plan-print, cộng thêm tool-drift warning (report-only, không abort) trừ khi `--require-tools` đã gate ở trên. **OA-07 preserved:** drift luôn loud + non-zero (exit 3 mặc định, exit 1 khi `--require-tools`); `sos tools status` UNCHANGED (vẫn exit 1 on drift, dedicated check). Smoke thật trên máy dev (doctor 0.1.1<0.1.3 DRIFT, inv-gate MISSING — pre-existing OA-07 evidence, KHÔNG mock): `install --runtime codex` → 17 file ghi thật (`AGENTS.md`+`.codex/**`+`scripts/codex/**`+`.agents/skills/**`) + WARNING loud liệt kê 6 tool + exit **3**; `--require-tools` → exit 1, ZERO file ghi; `--runtime claude` symmetric (ClaudeAdapter plan rỗng → chỉ `.sos-manifest.toml`, tool-drift warning + exit 3 vẫn đúng); `--dry-run` in cả 17 would-CREATE cả warning, zero mutation. Additive verified: `git diff bin/sos.sh install.sh crates/sos-install/` rỗng. **P079 Codex dogfood giờ UNBLOCK** — tool-drift trên máy dev không còn chặn render nữa. Oracle: `cargo build/test --workspace` xanh, ×20 = 0 flaky, clippy 0 warning mới (pre-existing `sync.rs:102` giữ nguyên), dep-direction guard xanh.

**P078d2b status (in-subagent enforcement — declare MISSING, P078d DONE):** live probe
(`docs/adapters/SUBAGENT-HOOK-PROBE-2026-07-22.md`) xác định dứt khoát: Codex 0.145.0 KHÔNG
dispatch `SubagentStart`/`SubagentStop` (và suy ra `PreToolUse` in-subagent) cho custom-role
spawn (`architect`/`worker`) — chỉ `agent_type="default"` fire; upstream
`openai/codex#21753`. Dogfood P079 #4 xác nhận trực tiếp (không chỉ suy luận): architect
subagent's forbidden `apply_patch` trên `src/` THÀNH CÔNG in-subagent, marker
`architect-active` KHÔNG BAO GIỜ được tạo. **Khai MISSING (KHÔNG PARTIAL, KHÔNG simulate):**
`CodexAdapter::verify()` (`crates/sos-adapter-codex/src/lib.rs`) thêm Finding #6 (5→6,
`FindingStatus::Missing`, cite `#21753`) — Findings #1/#3/#5 (PARTIAL) GIỮ NGUYÊN status, chỉ
thêm 1 câu làm rõ "enforced on MAIN THREAD only". `templates.rs` `hooks_json()`:
SubagentStart/Stop marker GIỮ render (byte-identical, DEFAULT subagent vẫn fire) + deprecation
comment (Rust-comment-only, 0 JSON output change). `agents_md()` thêm 1 bullet orchestrator
boundary-review guidance (main-thread chịu trách nhiệm review vì in-subagent guard không
enforce). `adapters/codex/CAPABILITY.md` §6 mới (bảng Claude-vs-Codex + 3 backstop thật:
main-thread `PreToolUse` dogfood-confirmed + universal Git pre-commit/pre-push agent-agnostic +
AGENTS.md guidance) + `SECURITY.md` threat-model note ("đừng tin architect subagent tự giữ
envelope; tin cổng Git") + `MAPPING.md` "5 Findings" → 6. **KHÔNG hack/workaround Codex bug —
declaration + deprecate only.** Additive verified: `git diff` d2a guards (#5/#6 multi-path +
approval bootstrap), `crates/sos-install/engine.rs`, `crates/sos-core` = rỗng. Oracle
STRUCTURAL: `cargo build/test --workspace` xanh ×20 = 0 flaky, dep-direction guard xanh (0 hit
trong `sos-core`), d1 render regression xanh (`hooks_json_is_valid_json_with_expected_events` +
`hooks_json_top_level_keys_are_description_and_hooks_only` PASS — declaration KHÔNG phá render).
**P078d (d1 startup-schema + d2a multi-path-guard + d2b MISSING-declaration) = DONE. P078 tổng
thể = code build đủ, phần còn lại là re-dogfood round-2 (P079) do Sếp chạy.**

## Ownership và dependency direction

### Portable core

Sở hữu role semantics, workflow/state machine, policy, phiếu, recipes, canonical skill body, universal Git gates và state/config schemas. Core không được chứa:

- runtime event/tool/model name;
- `CLAUDE_*`, Claude/Codex manifest path;
- permission serialization;
- lệnh spawn một agent host cụ thể.

### Runtime adapters

Mỗi adapter implement cùng contract tối thiểu:

```text
detect() → capabilities
plan(core, project) → managed file/tool/hook operations
render(asset, capabilities) → runtime-native artifact
verify(project) → findings
uninstall(manifest) → safe removal plan
```

- Claude adapter sở hữu `CLAUDE.md`, `.claude/**`, Claude agent frontmatter, lifecycle events, tool/model/env mapping.
- Codex adapter sở hữu `AGENTS.md`, `.codex/**`, Codex skills/agents/config/MCP/permission mapping.
- Adapter được phụ thuộc core contract; core không import adapter. CLI là composition root chọn một hoặc nhiều adapter.

### Generated artifacts

Install/sync ghi manifest gồm owner, source version, target path và content hash. Generated file không là source of truth. Luật apply:

- mặc định additive/non-clobber;
- file chưa đổi kể từ lần cài có thể update;
- file người dùng đã sửa phải báo conflict hoặc ghi incoming copy;
- `--dry-run` luôn cho thấy plan;
- mutation có backup/rollback record;
- uninstall chỉ xóa artifact vẫn khớp managed hash.

## Một lệnh cài toàn bộ

Public UX cuối cùng:

```bash
# bootstrap không cần Rust toolchain
curl -fsSL <official-installer> | sh

# hoặc npm/pnpm convenience wrapper tải cùng prebuilt binary
pnpm dlx sos-kit install --runtime codex
```

Bootstrap chỉ tải binary `sos` đúng platform và verify checksum. Sau đó:

```bash
sos install --runtime auto        # detect Claude/Codex; cài common + adapter tương ứng
sos install --runtime claude      # common + Claude
sos install --runtime codex       # common + Codex
sos install --runtime claude,codex
```

`sos install` lập một transaction plan:

1. Detect project, OS/arch và runtime capabilities.
2. Cài/update common core assets.
3. Render runtime adapter assets.
4. Cài Git hook stubs gọi `sos hooks run pre-commit|pre-push`.
5. Resolve tool manifest, tải đúng version/checksum của required và optional tools.
6. Ghi managed manifest/state.
7. Chạy `sos doctor`; lỗi required component làm install fail rõ ràng và rollback.

## Hooks, gates và sister tools

Tất cả được expose qua một namespace ổn định:

```text
sos doctor
sos gate inv
sos gate docs
sos docs rotate
sos ship
sos hooks install
sos hooks run pre-commit
sos hooks run pre-push
sos tools status|install|upgrade
```

Ban đầu `doctor`, `inv-gate`, `docs-gate`, `doc-rotate`, `ship` và các tool hiện hữu có thể tiếp tục là binary/repo riêng. `sos` khóa version/checksum trong `tool-manifest.toml`, tải và dispatch chúng. Như vậy one-command UX không bị block bởi một cuộc nhập source lớn. Chỉ nhập một tool thành workspace crate khi có lý do kỹ thuật/release rõ ràng; đó không phải điều kiện portability.

Git hook trong repo đích là stub nhỏ và ổn định:

```sh
exec sos hooks run pre-commit "$@"
```

Policy và phase logic sống trong Rust/tool manifest, nên `sos upgrade` cập nhật hành vi mà không rải lại shell logic tùy tiện.

## Migration order và gates

| Phiếu | Deliverable | Gate để đi tiếp |
|---|---|---|
| P075 | Portable core (`SOS.md`, roles/policy/workflow/assets neutral) | Xóa mọi adapter path vẫn đọc hiểu và validate được core; zero runtime token trong canonical core allowlist |
| P076 | Claude adapter từ cùng core | Golden Claude behavior trước/sau tương đương: agents, skills, hooks, permissions, new/adopt/sync |
| P077 | Rust workspace + adapter/install contract + manifest/dry-run/non-clobber/rollback/sync/doctor | Rust CLI là canonical; Bash oracle parity xanh; repo contract đổi sang runtime monorepo |
| P078 | Codex native adapter | Fresh install sinh đúng Codex-native files và doctor xác nhận; không copy doctrine fork — **P078b DONE** (b1 foundation + b2 declarative render + b3 enforcement, 17 artifact tổng); phần còn lại của P078 = behavioral live-dogfood, xem P079 |
| P079 | Dogfood Codex trên chính sos-kit | Một phiếu thật đi đủ DRAFT→CHALLENGE→APPROVAL→EXECUTE→DISCOVERY→MERGE |
| P080 | Dual-runtime brownfield dogfood | Claude + Codex cùng repo, fresh/brownfield, repeated install/sync, conflict/rollback, platform matrix xanh |
| P081 | Distribution | Prebuilt/checksum/release provenance xanh; installer và pnpm wrapper gọi cùng binary; one-command acceptance |

Không khởi công packaging P081 trước hai dogfood gates P079/P080. Cursor, OpenCode và Antigravity là future adapters; không nằm trong acceptance hiện tại.

## Non-goals của vòng này

- Không tạo doctrine copy riêng cho Claude và Codex.
- Không ép mọi sister tool nhập source vào monorepo trước khi installer dùng được.
- Không giữ Bash và Rust là hai canonical implementations sau parity.
- Không hứa runtime chưa có môi trường dogfood.
