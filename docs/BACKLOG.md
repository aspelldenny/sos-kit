# sos-kit Backlog

> **Single source of truth for "what to do next on sos-kit itself."**
> Live tracker. SessionStart hook surfaces Active sprint into the model's context on every new Claude Code session in this repo. Pick an item or capture a new idea via `/idea` skill.
>
> **Architect Rule 0:** Only write phiếu for items in **Active sprint**, or for items the maintainer has explicitly promoted from "Next sprint." No phiếu for "Open backlog" / "Park" without explicit promotion.

---

## 🔒 Active sprint — runtime-portability foundation (Sếp ratified 2026-07-20)

> **Mục tiêu:** rút SOS Kit ra khỏi Claude Code thành một core độc lập runtime, sau đó cắm lại Claude Code như adapter đầu tiên, viết Codex adapter và dogfood trước khi đóng gói.
>
> **Scope gate vòng này:** CHỈ Claude Code + Codex. Cursor, OpenCode và Antigravity để future adapters; không implement hoặc tuyên bố support khi chưa có môi trường dogfood thật.
>
> **Thứ tự cứng:** portable core → Claude parity → Rust adapter framework → Codex adapter → sos-kit self-dogfood → brownfield dogfood → packaging. Chưa qua gate trước thì không khởi công gate sau.

### Active items (in order)

- [x] **[P074] Runtime boundary inventory** — DONE 2026-07-20. Ownership A: một monorepo/version/`sos`; core + Claude/Codex adapters module hóa; sister tools qua managed manifest để giữ one-command UX. Docs-only, không đổi runtime. Phiếu: `docs/ticket/P074-runtime-boundary-inventory.md`.
- [ ] **[P075] Portable SOS Core** — tạo `SOS.md`, role/policy/workflow trung lập; core không chứa runtime-specific tool/model/env.
- [ ] **[P076] Claude Code adapter parity** — tái dựng wiring Claude từ cùng core; golden behavior trước/sau phải tương đương.
- [ ] **[P077] Rust adapter framework** — adapter contract, policy engine, manifest, dry-run/non-clobber/rollback/sync/doctor và lifecycle state.
- [ ] **[P078] Codex native adapter** — `AGENTS.md`, `.codex/agents`, hooks, skills, MCP và permission envelope.
- [ ] **[P079] Codex self-dogfood** — Codex chạy trọn một phiếu thật ngay trên sos-kit qua DRAFT→CHALLENGE→APPROVAL→EXECUTE→DISCOVERY→MERGE.
- [ ] **[P080] Dual-runtime brownfield dogfood** — Claude + Codex cùng repo, fresh + brownfield, regression và cross-platform.
- [ ] **[P081] Distribution** — Rust prebuilt/checksum trước; npm/pnpm wrapper và native plugins chỉ sau dogfood xanh.

---

## ⏸ Carry-over acceptance — open-source-hardening (không block P074)

> **Trạng thái:** implementation cả 3 leg đã ship. Còn acceptance P071 trên Linux + Windows Git Bash; log tại `docs/retro/DOGFOOD_P071-task6_3OS_2026-06-15.md`. Carry-over này không mở lại scope portability.
>
> Threat model the 3 legs cover (distinct, complementary):
> - **Leg 1 — privacy/portability:** ✅ DONE (`b20aa0c`) — `.mcp.json` dùng PATH-relative `doctor`.
> - **Leg 2 — download integrity `[P071]`:** ✅ IMPLEMENTED (`ef04ea4`) — installer verify `.sha256`; ⏳ Linux/Windows discrimination acceptance còn treo.
> - **Leg 3 — auto-exec integrity `[P073]`:** ✅ DONE (`2ab5b64`) — trust baseline + hidden-unicode gate + `SECURITY.md`.

### Remaining acceptance items
- [x] **[P071 implementation]** Release `.sha256` + installer fetch-verify shipped. Version pinning tách sang `[P-pin]` follow-up.
- [ ] **[NEW]** Dogfood P071 + kit trên **cả Linux + Windows** (không chỉ macOS) — Sếp có sẵn 2 máy. **Acceptance-gate của P071, KHÔNG phải dogfood chung:** P071 Task 6 (2-OS discrimination test) chỉ coi là xong khi `$SHA` probe verify thật trên Linux (`sha256sum`) + Windows Git Bash (cả 2) + macOS (`shasum -a 256`). Bonus bắt: install.sh curl|sh + sos adopt/new line-ending / command-availability gaps; advisory-cron win-skip phải warn-skip sạch. → **P071 done = probe xanh trên 2-3 OS.** (15/06/2026)
- [x] **[P073]** Auto-exec baseline-diff + hidden-unicode gate shipped.

> **Follow-up (NOT a public gate — reproducibility, not integrity):**
> - [ ] **[P-pin]** Version-pinning manifest cho install.sh (`releases/latest` → pinned tag + `versions.env`). Split khỏi P071 tại APPROVAL_GATE 2026-06-15 (checksum một mình đã đóng supply-chain HIGH). Edge đã khảo sát: manifest-before-clone ordering → Approach A (inline vars trong install.sh). Recurring bump cost mỗi release. Làm khi n-user lớn cần reproducible install. (15/06/2026)
> - [ ] **[CI-CLEANUP]** docs-gate + ship CI đỏ (pre-existing, surfaced P071 fan-out): **docs-gate** `cargo fmt -- --check` fail · **ship** `cargo test` fail. Re-triggered bởi P071 version-bump commit (không phải P071 gây ra). KÈM: ci.yml của 2 repo này còn Node20 `cache@v4→v5` + `upload-artifact@v4→v7` + `download-artifact@v4→v8` (deferred khỏi P072 release.yml-scope — targets đã verify tồn tại). Làm 1 thể: fix fmt/test + bump ci.yml Node20. (15/06/2026)

---

## ▶ NEXT SESSION — Tier-3 ĐÓNG; còn lại = optional/polish (updated 2026-06-11 EOD) [RESET POINT]

> **Session 2026-06-11 — steps 2+3+4 ALL DONE (trọn plan 2026-06-09):**
> - **JARVIS foreign-adopt** (steps 2+3): verify-setup CONNECTED; 2 adopt bugs fixed (JA-01 🔴 symlink agents + realpath escape-guard; JA-02 dup changelog); 12-finding friction log → 9/12 đóng trong ngày. JA-12 PAT revoked + script sang env var, gate re-run 2/2 PASS. Jarvis session sống tốt (Quản đốc bên đó tự nhận diện phiếu treo, 115/115 test, đề xuất commit+deploy đúng bài).
> - **"1 lệnh" SHIPPED (step 4, P064+P065):** adopt born-wire (JA-03/04/05/08/09) + INSTALL.md 3-command rewrite (JA-07) → `113d1ac`. Release-CI doctor v0.1.0 + claude-hooks v0.9.1 (3-target, xanh first-run) + `install.sh` curl|sh (fail-closed, no-Rust, JA-06) + B+3 shim vào `block-unsafe-merge.sh` (live-fire: gate chặn chính lệnh test của Quản đốc giữa session) + `templates/setup-dev.sh` golden → `0f5ffaf`. Giám sát NEEDS_REVIEW → hardened same-branch (curl/git timeouts + trusted-path exec giết PATH-spoof); **[P071]** checksum/pinning = deferral explicit. **Acceptance: public curl|sh thật từ GitHub → sos adopt → CONNECTED (máy-trắng-giả-lập).**
>
> **Phần 2 cùng ngày (sau test tay của Sếp):** doctor **v0.1.1** (verify-setup B+3-shim-aware — Thợ execute qua orchestrator-guard block đúng quy trình, 40/40 test) · manifest **2→9 binary** (release-CI rải đủ 9 repo; advisory-cron matrix mac+linux — Windows bites #1 đúng dự báo) · adopt jq-merge đủ 5 MCP server · sos-kit tự seed INVARIANTS.md (J4) · **jarvis spine restore qua sos sync n=3** (session anh em xoá scripts/+agents — 55 file phục hồi, 3 agents take-newer về background:true) · installer chia core(fail-closed)/optional(warn-skip) vì **4 repo private**: scan history → advisory-cron SẠCH (chờ Sếp gật public), guard/vps/doc-rotate lộ IP+port+root@ VPS thật trong history → GIỮ PRIVATE (scrub-rồi-public khi có user ngoài). **Public test chốt: rc=0, 5 core + sos, 4 optional skip rõ lý do.**
>
> **Cuối ngày:** advisory-cron PUBLIC hoá (Sếp gật, scan sạch) — installer giờ tải được 6/9 ẩn danh. **inv-gate project DỰNG XONG chờ Sếp vào chạy** (`~/inv-gate`: sos adopt CONNECTED rc=0 + doctor v0.1.1; CLAUDE.md golden-oracle method; BACKLOG Sprint 1 = P001 pin oracle → P002-P005 port per INV; golden/ = tarot 5-file 797 LOC frozen; release.yml sẵn; 5 agents background:true; pushed `f2946c4`). Bonus finding: macOS SIGKILL khi overwrite binary in-place (cp đè inode) — install.sh dùng mv nên đã an toàn sẵn. 
>
> **📥 HARVEST IN (2026-06-11 tối, Sếp gửi): inv-gate ĐÃ CHẠY XONG TRỌN DỰ ÁN** — 2 sprint 8 phiếu cùng ngày, **v0.1.0 released** (3-target, asset khớp contract install.sh sẵn). Friction log đầy đủ: `docs/retro/FRICTION_inv-gate_e2e-dogfood_2026-06-11.md` — 11 findings (IG-01→11) + biểu điểm 23-point + **6 HARVEST ACTIONS** ở đầu file (join BINARIES line 36 · pre-commit [4/7] swap python3→binary · platform gaps Intel-Mac/ARM-Linux · quarantine xattr UX · ⚠️ Node20 CI deadline 16/06 ẢNH HƯỞNG CẢ FAMILY · doctrine items đã formalize). Dự báo "small harvest" ở mục Hàng đợi SAI — e2e full-flow đầu tiên trên repo adopt-born lộ 11 findings thật, 4 cái Tầng-1-class cho kit.
>
> **🔥 Active — cụm skills-audit (Sếp ratified 2026-06-11):** 3-giữ-10-park + định danh caller. Kèm:
> - [ ] **[NEW]** UserPromptSubmit hook bắt idea-smell → nhắc invoke /idea — trigger "idea mới" mờ, model lúc nhớ lúc quên (ghi BACKLOG thẳng không qua skill); regex các cụm "ghi vào backlog/anh nghĩ ra/ý tưởng/thêm vào backlog" → inject reminder. Coin-flip → deterministic, đúng enforce-via-mechanism. (11/06/2026)
>
> - [ ] **[NEW]** Register weekly `/retro` cron cho sos-kit — caller đã khai trong SKILL.md, register BLOCKED on advisory-cron `fire_task` no-timeout DEBT (repo nó tự cảnh báo `claude -p` hang). Fix debt đó trước → register `0 18 * * 0`-form. (11/06/2026)
> **📐 SPEC GHIM — "adopt-hiểu-repo" (Principle 7, Sếp-ratified 2026-06-11 — COMPASS, CHƯA LÀM):**
> Chuẩn: survey → classify → wire-what-matches → map-from-reality → ask-judgment-only → validate. Điểm hiện tại: map-from-reality ~80% (`sos map`) + validate ~85% (verify-setup trong adopt) ĐẠT; survey ~30% (chỉ manifest+dirs, thiếu framework/DB/CI/cron/deploy-surface); classify ~10% (chỉ new/adopt/sync); wire-what-matches ~15% (jarvis bị nhồi spine y hệt — vụ guard/vps "sida"); judgment-slots ~25% (TODO-dump thay vì interview). Thứ tự khi làm: (a) tier-wiring đã duyệt (gỡ guard/vps khỏi default .mcp.json) + (b) post-adopt interview 4 câu judgment (Quản đốc AskUserQuestion sau adopt: production surface / load-bearing / secret quan trọng / app cấm làm gì → điền INVARIANTS+AGENT_MAP) — 2 món rẻ làm trước; (c) survey/classify engine CHỜ n adopt đủ lớn (dogfood-BEFORE-infra). Đích: ráp vào chạy 70-80% ngay, phần còn lại repo tự mài dao trong quá trình chạy. **Calibration n=1 (inv-gate, small-tool-repo): Quản đốc bên đó ước ~60-70% fit — 6/10 cụm adopt là noise cho repo tool nhỏ (advisory/guard/vps/ship-MCP/recipes/INV-LOCAL chưa đụng), 4 thứ phải tự mài (biểu điểm template, gate-delegation, parity-harness pattern → candidate recipe `rust/golden-oracle-port`, per-check swap).**
>
> **🌾 HARVEST inv-gate Sprint 1 (2026-06-11 — 5/5 phiếu ship 1 ngày, biểu điểm 19/23 tick, 6 findings):** marker-symmetry fix + GATE_DELEGATION + Debate-Log-as-state + cite-ranges đã hạ vào contract cùng ngày. Items còn treo:
> - [ ] **[DEBT]** Security gate KHÔNG stack-aware (W16 ngược): chỉ quét .ts/.js — repo Rust không được bảo vệ chính source của nó. Route: inv-gate Sprint 2 profile-mode + gate config per-stack. (11/06/2026)
> - [ ] **[RESEARCH]** `phieu.sh` CLI + `.phieu-counter` 0 lần dùng trong full sprint (W22 — Architect tự đặt tên file đúng convention). Caller-law case: wire vào architect handbook hay park? (11/06/2026)
> - [ ] **[NEW]** Sprint-item "assumption note" (IG-06): BACKLOG item viết trước khi có oracle nên khai giả định môi trường — guidance vào TICKET_TEMPLATE/idea skill khi sprint sau xác nhận lại pattern. (11/06/2026)
> - Carry sang sprint sau: W4 idea-smell live, W14 exit-code contract, W18 CHANGELOG ngày-2, W21 Sếp chấm % fit. W5 nghĩa-vụ-chặn B+3 = test khi merge sprint branch inv-gate (gh pr merge sẽ đòi security-review APPROVE).
>
> **🌾 HARVEST inv-gate Sprint 1+2 (2026-06-11 — 11 findings, source: docs/retro/FRICTION_inv-gate_e2e-dogfood_2026-06-11.md). 6 actions DONE/flagged this session:** (1) inv-gate → install.sh BINARIES ✓ · (2) security-gate.sh binary-first per-check (kill python dep, python fallback, both branches fire-tested) ✓ · (3) platform IG-10: Intel-Mac→Rosetta + 3-platform-arch documented ✓ · (4) quarantine xattr-strip Darwin ✓ · (6) doctrine items already synced ✓. Items still OPEN:
> - [x] **[FLEET-NODE] ✅ DONE 2026-06-15 — P072 shipped.** Node20-deprecation bump 10 repo release.yml — all 10/10 rc-oracle PASS (3/3 green, 0 Node20 annotations, prerelease=true, latest unchanged), all rc tags deleted. guard/vps/doc-rotate/advisory-cron upgraded to full draft/publish+sha256 template (P071+P072 combined). See `docs/discoveries/P072.md`.
> - [ ] **[DECISION] IG-07** — local-merge lách sentinel: `block-unsafe-merge` chỉ canh PR-comment (gh), `git merge main + push` tay bỏ qua nghĩa vụ Giám-sát. Fix: pre-push check sentinel APPROVE cho security-surface push, HOẶC document "local merge = Chủ nhà tự chịu".
> - [ ] **[DECISION] IG-01-deeper** — guard phân vai bằng marker file, không phân biệt main-session-orchestrator edit `.claude/` vs architect-subagent. Symmetric-rm đã vá triệu chứng; gốc vẫn còn (orchestrator sửa config hợp lệ bị chặn). Fix: whitelist `.claude/` edit cho orchestrator HOẶC signal khác ngoài marker.
> - [ ] **[DECISION] platform-targets** — thêm `x86_64-apple-darwin` (Intel native, bỏ Rosetta) + `aarch64-unknown-linux-gnu` (VPS ARM/Graviton) = 5-target, HAY giữ 3 + Rosetta/document? (IG-10 phần 2.)
> - [ ] **[DEBT] W16** — security gate quét `.ts/.js`, KHÔNG quét `.rs/.py` → repo Rust không bảo vệ chính source nó (kế thừa tarot). inv-gate Sprint 2 profile-mode.
> - W21 fit ~65% (Quản đốc chấm, Sếp chỉnh): spine+workflow+gates gánh tốt; trừ điểm IG-01/07/09. IG-08 (non-contiguous synthetic secret) + IG-09 (per-phiếu merge) đã landed vào worker.md.
>
> **NEXT (Sếp chọn):**
> 1. **inv-gate Sprint 1** — mở Claude Code ở `~/inv-gate`, banner hiện P001 → brief 1 câu là chạy
> 2. **[P071]** checksum/signing khi mở public rộng (release.yml +1 step ×9 repo + install.sh verify)
> 3. **media-rating brownfield** adopt-poisoned pass (P068 follow-up)
> 4. Quay về **PRODUCT** (tarot/jarvis) — kit đã đủ răng.

---

## ▶ PREVIOUS plan (2026-06-09, Sếp + Quản đốc) — steps 1-3 superseded above

> **Session 2026-06-09 was huge** — built `sos sync` (KIT-LAG cure, n=2 proven) + P069 (Architect Write-envelope enforce) + F09 (install-hooks security-hijack guard); closed Tier-0; de-risked Blocker C trọn; ratified B+3 (fail-closed binary deploy); harvested 3 friction logs (~20 findings, claude-hooks + doc-rotate ×2 + tarot). Details: CHANGELOG v2.3 forge + items below.
>
> **🔒 LOCKED PRINCIPLE — dogfood BEFORE infra:** adopt the kit into a FOREIGN repo *before* building the "1 lệnh" installer — the foreign-adopt reveals what the installer actually needs. (The rule that won this whole session: `sos sync` + F09 both came from dogfood-first; "build installer first = đoán mò.")

**Order (Sếp-ratified — note the end was self-corrected from "build infra → adopt" to "adopt → build infra"):**
1. **inv-gate** (optional) — last Rust tool, replaces `security-gate.sh` Python (~794 LOC). Clean golden-oracle port like doc-rotate → **family dogfood = small harvest** (family-friction largely mined over claude-hooks + doc-rotate). Do for "đủ bộ Rust" or defer; don't expect big learning.
2. **Adopt into JARVIS first** (foreign, HIGH-signal) — `~/jarvis` = fresh-foreign Python bot, near-no-kit (only CLAUDE.md + .docs-gate.toml, no agents/hooks/phieu) = **the portrait of a real new-user repo** → reveals TRUE installer requirements. (`~/media-rating-app` = brownfield-poison + v2.0-kit-stale stress — already de-risked via sos sync + P068; do it LATER, paired with code-poison build.)
3. **Tổng kết** the jarvis foreign-adopt friction → **this is where installer requirements surface** (like media-stress surfaced sos sync).
4. **Build "1 lệnh"** (Tier-3 distribution) WITH the real requirements: prebuilt binary (GitHub Releases 3-target mac/win/linux) + `curl|sh` bootstrap + **B+3 fail-closed shim** (ratified [P064]) + golden `setup-dev.sh` ([P065]). **Build-heavy + cross-platform — Windows WILL bite** (P059 was the warm-up).

**⚠️ Mode shift for next session:** this session = doctrine/harvest (reflect-heavy). Next = **BUILD/INFRA** (port inv-gate · release-CI · curl|sh installer · cross-platform test). Different muscle — more code, less reflection.

**Still open (mapped, non-urgent):** doc-rotate re-sync when DR05 done (`sos sync` brings it) · code-poison orchestration build (HOLD, n=1, P068) · F11/F12 recipe-lesson (when forging AI-tool recipes) · install-hooks/banner/architect-guard re-sync to downstream via `sos sync` when those repos cool.

---

## ✅ COMPLETE sprint: Két dogfood harvest — git-level `.env` block (1 phiếu)

> **Promoted 2026-06-08 → SHIPPED + MERGED 2026-06-09** (Sếp explicit pick). Continuation of the git-gates harvest thread (P049–P052). Most-grounded remaining mechanical item — **n≥1 real incident** (media audit SEC-SECRET-01: `.env.docker` committed to git history). Complement to [P046] PreToolUse `block-env-edit.sh` (edit-time, Claude-only) — this is the commit-time, agent-agnostic layer. Full state machine: DRAFT→CHALLENGE([O1.1] `.envrc`)→RESPOND→APPROVAL_GATE→EXECUTE.

- [x] ~~**[P052]**~~ SHIPPED + MERGED 2026-06-09 — git-level `.env*` commit block (`scripts/block-env-commit.sh`, pre-commit `[7/7]`). PR #23 (merge-commit `542df0f`). Fire-test 9/9 PASS (orchestrator re-ran independently incl. false-positive guard `.environment.ts`). `/security-review 23` → clean APPROVE sentinel → `block-unsafe-merge` allowed merge (**P053 self-validated #2** — merge-commit path). `.envrc` deliberately excluded ([O1.1]). Discovery: `docs/discoveries/P052.md`.

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
2. **Trigger wiring** — ~~tool ship mà không nổ~~ **✅ LARGELY DONE (status verified 2026-06-09 — snapshot trên đã stale):** `doctor` wired đủ — binary cài + MCP (`.mcp.json` serve) + handbook GỌI thật (`orchestrator.md` lane-check pre-CHALLENGE · `worker.md` runtime-scan Task 0 · `boundary-check.md` validate-map/runtime-scan) + `adopt` gọi `verify-setup` (`bin/sos.sh:567,811`; lệnh CÓ thật, Q-D5 wiring-check). `advisory-cron` registered cho **tarot** (launchd `com.advisorycron.advisory-scan-tarot` daily 09:00), per-project opt-in. **Residual (optional, low-prio):** `advisory-cron register` chưa nằm trong `sos adopt`/setup-dev → repo deps-nặng mới phải register tay; **arguably đúng** (repo mỏng không cần quét CVE hằng ngày → auto-register mọi adopt = noise). Nếu wire thì gộp vào [P065] setup-dev golden template, GATED opt-in.
3. **2 skeleton** chưa code (claude-hooks, inv-gate).

### C. ⛔ BLOCKER THẬT để đóng kit — Adopt "repo nhiễm độc" (Sếp nêu 2026-06-05)

> Genesis (0→1 empty repo) sos-kit giải rồi (`/init`, GENESIS_TEMPLATE). Nhưng **ADOPT brownfield chưa giải** — và kit phải cài được vào repo cũ thật mới gọi hoàn chỉnh.

3 loại repo cũ, độ khó tăng dần:
- **Loại 1 — code-only, no docs:** adopt phải reverse-engineer docs skeleton từ code. "code có độc, docs không có" → còn phải **detect độc trong code** (anti-pattern, AI-bloat ship sẵn, security debt) trước khi tin.
- **Loại 2 — code-lớn + docs-có, CẢ HAI nhiễm độc:** khó nhất theo chiều drift. docs **drift khỏi code** (precedent tarot 2026-06-05: SURFACE_MAP/BACKLOG ghi "flask-cors fixed in 5.x" SAI; ARCHITECTURE §3 ghi "8 cột" lệch code 10 cột). Adopt KHÔNG được tin docs mù quáng — phải **reconcile docs↔code** trước khi layer kit lên.
- **Loại 3 — CONSTELLATION / fleet, lệch-stack, lệch-độ-chín, một-mục-đích** (POD-agent pilot 2026-06-09). KHÔNG phải 1 khối (media/tarot shape) mà **N tool rời** dưới một mái phục vụ một business. Vỡ đúng tiền-đề ngầm lớn nhất của kit: **1 repo = 1 product** (1 PROJECT/SOUL/CHARACTER · 1 BACKLOG/Active-sprint · 1 phiếu-stream · 1 `.sos-stack.toml` `type`). Xem finding bên dưới.

**Đây đúng bài học retro v2.3 phóng to:** "single-source-the-truth — dormant vì 2 nơi khai báo lệch nhau (sentinel mismatch)". Repo nhiễm độc = drift ở quy mô codebase.

**Cần (chưa có recipe/flow):** một **adopt-flow cho brownfield-poisoned**:
1. Scan độc: code anti-pattern + docs↔code drift detect (tận dụng được advisory-watch? doctor validate-map?)
2. Quarantine/flag — không tin docs cho tới khi verify với code
3. Reconcile single-source — chọn code làm oracle, sửa docs theo (hoặc ngược lại có chủ đích)
4. Mới layer sos-kit (agents/hooks/phieu) lên nền đã làm sạch

→ **Không giải được adopt-poisoned thì kit chỉ chạy greenfield, chưa đóng được.**

#### FINDING — POD-agent = pilot Loại 3 (constellation/fleet) — 2026-06-09 (capture-only, KHÔNG build, mold chưa chín §E)

> Sếp đưa `github.com/aspelldenny/POD-agent` làm dogfood. Repo = chòm sao ~6 tool POD trên Creative Fabrica: `Auto-bundle` (Python batch) · `cf-image-replacer`/`cf-product-deleter`/`cf-trademark-checker` (3 extension) · `Creative-brain-api` (Flask+Telegram-bot+ext = "não" hiện tại) · `siêu-CF-rút-xiền` (ext rút tiền) · `run/` (Puppeteer 5-account). Một business, nhiều tool, lệch stack + độ chín. Vision cuối của Sếp: **tool ngon + một bộ-não-agent điều phối** = chính mô hình **Quản đốc** ở quy mô fleet.

**Bất ngờ:** repo ĐÃ có sos-kit nguyên thuỷ — root `CLAUDE.md` có DoD + DOCS GATE + vai thợ-xây/kiến-trúc-sư + "docs = bộ nhớ giữa session". → Adopt = **nâng-cấp bản mini**, không phải genesis 0→1.

**Độc đã thấy (grounded):** data-dump commit thẳng (5× `*_ids_to_delete.txt`, `tm_blacklist_FINAL.csv`; `.gitignore` mới tạo 2026-06-09 → rác lọt trước) · báo-cáo-session-vứt-tại-chỗ giả-docs (`ANALYSIS_REPORT`+`FINAL_REFACTOR_SUMMARY`+`REFACTOR_SUMMARY` chồng nhau 1 folder + `spell-check-report`) · docs tản mác (docs/ 9 + stencil/ 5 + siêu-CF/ 4 + per-tool) · **DOCS GATE mù** (chỉ route `docs/*_GUIDE.md`, KHÔNG phủ 3 cf-* extension → cả mảng tool ngoài doctrine) · doc-drift (`PROJECT_OVERVIEW.md` cập nhật cuối 2026-03-25, lệch CHANGELOG đang chạy).

**3 thứ Loại 3 ĐÒI mà kit hiện chưa có (đây là cái doctrine cần, không phải tool):**
1. **Multi-stack `.sos-stack.toml`** — 1 `type` không đủ cho Python-batch + extension-JS + Flask + Puppeteer-node trộn. Cần per-tool stack (hoặc fleet-manifest liệt kê N tool × stack).
2. **Per-tool phiếu-stream / BACKLOG có tool-tag** — 1 Active-sprint cho 6 tool vô nghĩa ("tool nào active?"). Fleet cần luồng song song, phiếu model muốn 1 luồng tụ → tension thật.
3. **Mixed-maturity tiered-adoption** — đừng áp full doctrine đồng đều (đánh thuế script throwaway = completeness-bias). Tier: (a) hygiene repo-level trước → (b) layer kit thật ở 1 tool xương sống (`Creative-brain-api`+`Auto-bundle`) làm dogfood → (c) cf-*/rút-xiền = vệ tinh light-touch tới khi earn full kit → (d) bộ-não-điều-phối = đích Quản đốc, sau khi ≥2 tool ổn định.

**Security note:** cf-*/rút-xiền đụng credential + tiền thật → `block-env-commit` + security-gate có giá trị thật ở repo này (không phải ceremony).

**Trạng thái:** capture-only. POD-agent là **pilot ứng viên** cho adopt-flow Loại 3 KHI Sếp quay lại đóng kit (Tier 1.5 roadmap §D). Có thể chạy thử pass #1 (hygiene) để lấy máu Loại-3 thật — nhưng đó là việc Ở repo POD-agent, ngoài scope sos-kit, cần Sếp gật riêng.

### D. Roadmap đề xuất (ROI + dependency)

| Tier | Việc | Ghi chú |
|---|---|---|
| 0 | Cho tool đã ship CHẠY: ~~advisory-cron register~~ ✅(tarot) · ~~doctor trigger~~ ✅(MCP+handbook+adopt, verified 06-09) · **advisory-inbox 8-cột = CÒN** (chưa verify, item riêng) | Cao nhất, low effort — **trigger-wiring 2/3 done** |
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

## 🛡️ Next sprint candidate: Trust gate port từ thanhtra v1.2 (12/06/2026 — Sếp duyệt ghi backlog, "để anh tính tiếp")

> **Threat model:** sos-kit là ĐÚNG class "Rules File Backdoor" — skills/agents/templates markdown được Claude Code load thẳng vào context người dùng, tức **PR độc hại vào repo này = prompt injection tới mọi user**, payload có thể vô hình (Unicode Tags U+E0000–E007F, zero-width, bidi). Vendors (Cursor, GitHub) đã phán class này là "user responsibility" → ecosystem không đỡ hộ, repo tự phòng thủ. Tier-1 GitHub hardening đã bật 12/06 (secret scanning + push protection, ruleset, fork-PR approval, immutable releases); item này là tầng content.
> **Oracle = thanhtra v1.2** (cùng owner, đã chạy thật): `thanhtra/core/trust.py` — 3 lớp detector deterministic (hidden-unicode / auto-exec / injection-marker; regex không bị prompt-inject, Trail of Bits đã bypass mọi scanner LLM nên gate PHẢI deterministic) + `scripts/validate-trust.py` (self-test payload trồng + self-scan + baseline `tests/trust-baseline.json` — corpus quote cụm tấn công hợp lệ thì diff baseline, marker MỚI = FAIL tới khi review) + SECURITY.md 5 invariants + `.github/workflows/gate.yml`.

- [ ] **[NEW]** Port trust gate — **KHÔNG copy nguyên xi, 3 chỗ phải lệch oracle:** (a) thanhtra hard-fail mọi auto-exec config, nhưng sos-kit **CỐ Ý ship** `.claude/settings.json` hooks + `.mcp.json` (đó là sản phẩm) → auto-exec chuyển sang baseline-diff: thay đổi hooks/mcp = FAIL cho tới khi review + rebaseline, diff hiện trong PR; (b) U+FEFF literal `phieu/DISCOVERY_PROTOCOL.md:196` (quote kỹ thuật BOM trong tài liệu) sẽ trip hidden-unicode gate → đổi sang escape/mô tả trước khi bật; (c) SECURITY.md threat model riêng cho kit: hooks chạy gì khi user trust folder, invariants (không fetch URL runtime, không giấu gì khỏi user, install chỉ symlink/copy khai báo rõ). Gotcha từ thanhtra: rebaseline dùng `git ls-files` → `git add` file mới TRƯỚC khi rebaseline; không bao giờ commit Unicode ẩn literal trong code/test — dùng escape. Làm xong copy pattern sang **claude-hooks [P012]** (share phieu/ sync). ~1 session.

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
  - [→] **[P052]** **PROMOTED → Active sprint 2026-06-08** (see top). Git-level `.env` block — complement to [P046].
  - [ ] **[P053]** Sentinel-vs-silent merge **deadlock** (kit interaction bug, ket-surfaced 2026-06-03). `block-unsafe-merge.sh` requires an `APPROVE` sentinel comment to merge a security-surface PR; `boundary-check` (Giám sát) is **silent-when-clean** (P042 design — only posts if it finds something). → security-surface PR + clean review = **no sentinel → merge deadlocks** (only escape = override marker). ket worked around it (WORKFLOW §21: "PR-gated → ALWAYS post sentinel, even clean APPROVE"). **Fix to bake into sos-kit:** either `boundary-check`/`/security-review` always posts a sentinel when the PR is `block-unsafe-merge`-governed (clean APPROVE included), OR `block-unsafe-merge` accepts a "clean-review" signal. The two P042-era gates are in tension — pick one.
  - [ ] **[P054]** **[FINDING — not a build item]** Spawn guards DRIFT from canonical (ket-surfaced 2026-06-03). ket's `orchestrator-guard.sh` had **lost the `*.md) exit 0` exemption** — hand-adapted for Swift (added `Ket/*`) but dropped the `.md` arm → same over-block hole as the branch-guard. **sos-kit's `orchestrator-guard.sh` is NOT affected — it exempts `.md` (line 64, RUN-confirmed). DO NOT "fix" sos-kit's orchestrator-guard; it is already correct.** Corrects the ket harvest-note framing "orchestrator-guard cùng lỗ" — true for ket's *drifted copy*, false for sos-kit's *canonical*. **Real lesson = spawn-drift class** (the root that collapsed media): a spawn hand-adapting a canonical guard silently drops a property. **Direction:** ket re-syncs guards from sos-kit canonical (don't hand-patch); or guards carry a version/hash so drift is detectable. Feeds the existing "KIT LAG / re-sync" theme above.
  - [ ] **[P055]** **[DEBT — surfaced by P053 CHALLENGE 2026-06-06]** SHA-scope the `block-unsafe-merge` APPROVE sentinel. Gate currently greps ANY historical `Verdict: APPROVE` comment on the PR (`block-unsafe-merge.sh:102-106`, no head-SHA binding) → a stale clean APPROVE on commit A can green-light later unreviewed commits B+C on a multi-commit PR. **Pre-existing hole** (gate always grepped any APPROVE), made easier to hit by P053's clean-APPROVE auto-post. **Why separate from P053 (one-disease):** SHA-scoping patches accept-side (`block-unsafe-merge.sh` — out of P053's emit-only scope), needs a new jq filter binding comment→`headRefOid` + slash command posting a `Head SHA:` line. **Direction:** capture `gh pr view <N> --json headRefOid --jq .headRefOid`, require APPROVE sentinel body contain matching `Head SHA: <sha>`, gate rejects sentinel whose SHA ≠ current head. P053 documents the limitation + mitigations (Chủ nhà reads timestamped comment; squash-merge collapses history). Needs grounding before promote (n≥1 multi-commit security PR that actually slipped).
- [ ] **[P056–P058] HARVEST batch — ket dogfood WORKFLOW retro (5 bài học) 2026-06-06.** Born from ket dogfood teardown — the "câu-hỏi-vàng soi vào CHÍNH nghi thức" reflection. **Distinct from the 2026-06-03 git-gates batch above:** that was teardown-of-tools; this is **doctrine harvest from the run itself**. 3/5 bài đã có trong sos-kit v2.2 (lane / `research_gate` surface-field / mechanical-first) → chỉ 3 món dưới là thật-mới. **Cả 3 là doctrine change → BẮT BUỘC qua retro v2.3** (memory `project_workflow_v23_retro_state.md`, đang dang dở), **KHÔNG sửa ad-hoc** (CLAUDE.md "Edit Workflow doctrine"). Capture-only ở đây; promote qua retro fold. Tất cả **surface-gated, KHÔNG universal** (chống chính-mình: đừng đánh thuế-nghi-thức lên mọi phiếu — đó cũng là completeness-bias ở tầng quy-trình).
  - [ ] **[P056]** Pre-merge **DOGFOOD gate** cho phiếu chạm UI-surface. **Sếp-decided 2026-06-06: CÓ — dogfood trước merge, đồng ý → mới merge.** Evidence: ket dogfood bắt ~12 UI bug mà 137 test xanh + BUILD SUCCEEDED bỏ lọt (ô vuông trắng menu-bar, panel import không hiện, vô-hạn-số-0, đã-trả-vẫn-notify, secret-orphan sau re-sign) — **agent mù GUI = tiên đề kiến trúc, không phải lưu ý.** **Design (tôn trọng "không phải lúc nào cũng dogfood"):** conditional state `EXECUTE → [DOGFOOD nếu phiếu touch UI/UX] → DONE`; pre-merge cần **human dogfood-ack** (Sếp xác nhận "đã dogfood, OK"); mirror escape `[security-review-skip:]` pattern → `[dogfood-skip:<reason>]` cho phiếu non-UI / thật-sự-không-cần, để KHÔNG false-block. TICKET_TEMPLATE field `### Dogfood check` chỉ bắt buộc khi lane chạm UI. **Nuance:** sos-kit tự nó không có GUI → đây là doctrine cho **consumer downstream** (ket/tarot/media) inherit, KHÔNG phải để kit tự-dogfood. Hạ vào WORKFLOW_V2.3 state machine + TICKET_TEMPLATE + `block-unsafe-merge` family.
    **[Evidence n=2 — tarot 2026-06-08, repo+stack khác ket]** P336 (free-deep ×3, web/Next.js): 906 test xanh + "shipped" hô lên như validation, NHƯNG copy-reviewer ("Người giữ đền") khi soi giọng modal vô tình bắt **off-by-one hiển thị số buổi free** (`page.tsx` dùng `freeDeepRemaining` thô thay vì `-1` → "còn ba lần" khi đáng "hai"; buổi cuối remaining=1 hiển thị sai hẳn). **Cùng bệnh ket:** lớp render người-dùng-đọc nằm NGOÀI vùng test phủ (test = invariant backend `cost 0 vs 15`, không assert chuỗi render). **Nhưng nó MAY** — copy-review không có nhiệm vụ bắt logic; nếu copy tình cờ đúng số thì bug ship thẳng. Root được tarot-orchestrator truy: logic "số→câu chữ" **nhân đôi 2 file** (modal `-1` đúng / page thiếu `-1`) → drift; không test nào quét chuỗi theo {3,2,1,0}. **Refinement cho P056 design:** tách 2 subclass — **(a) text/số/copy hiển thị** → một *agent-reviewer đọc render* bắt được (rẻ, tự-động-hoá-được, KHÔNG cần human); **(b) GUI visual/tương-tác** (ô vuông trắng, panel không hiện) → vẫn cần *human dogfood-ack*. P056 hiện nghiêng hết về (b); tarot chứng minh (a) bắt được rẻ hơn bằng agent. **Khe vai cốt lõi nó phơi ra:** dây chuyền không có ai sở hữu "giá-trị-người-dùng-đọc có đúng nghĩa không" — Worker tưởng test phủ, copy-review tưởng chỉ lo giọng → rơi vào khe giữa. P056 (render-review) = cái lấp khe đó. **Anti-over-build:** KHÔNG đẻ vai "UI-number reviewer" riêng (ceremony tax); giá trị của con-mắt-thứ-hai là *đọc artifact thật với kỳ vọng ngữ nghĩa*, không phải thêm checklist.
  - [ ] **[P057]** **Verify-cò rule** — "phiếu sinh gate/hook mới → BẮT BUỘC kèm fire-test (kéo thử xem cò nổ) trong CÙNG phiếu." Codify thực-hành đã chứng minh (P050 17/17 test, P053 self-validated end-to-end) thành luật cứng Definition-of-Done để sống qua mất-trí-nhớ-session. Root: **"ship ≠ chạy"** (hook viết ≠ hook fire — cần restart load settings; gate dựng ≠ được gọi). Lỗ kinh điển: build cò rồi tưởng nó sống. Hạ vào WORKFLOW §13 test-acceptance + DoD. **Rẻ** (codify, không build).
    **[Sibling extension — tarot P336 2026-06-08]** "Kéo cò" mở rộng sang **giá-trị-hiển-thị dẫn-xuất**, KHÔNG chỉ gate/hook. Khi bắt được một display-bug (off-by-one số buổi free, xem [P056] evidence): hardening đúng = **1 nguồn dùng chung + test quét BIÊN**, KHÔNG patch nhiều site cho khớp (vẫn để drift). Tarot fix: gom `freeDeepCopy(remaining)` helper + unit test sweep — **nhưng phải quét {3,2,1,0,≥quota} chứ không chỉ {3,2,1}**: off-by-one và state "buổi-cuối/hết-buổi" sống ở rìa; {3,2,1} đơn thuần vẫn xanh mà nhánh `0`/paywall sai. Đây là **vaccine** (chống tái-phát cho giá trị đó), *hệ quả* của P057 áp lên display value — KHÔNG phải gate thứ 3 (giữ §0.1 một-bệnh-một-cơ-chế: [P056] render-review = lưới bắt-lần-đầu, P057-display-test = vaccine không-tái). **Anti-over-build:** "đừng nhân-đôi display logic + test biên" là vệ-sinh-kỹ-thuật generic → KHÔNG nâng thành recipe kit; nó là fix-tại-chỗ ở consumer (tarot), kit chỉ ghi nguyên-tắc DoD.
  - [ ] **[P058]** **Câu-hỏi-vàng = dòng-1 phiếu Tầng-1.** TICKET_TEMPLATE field `### Scope question` Architect PHẢI điền trước khi draft tiếp ("cái gì có thể over-engineer / quá tay ở đây?"). Tách cấu trúc **scope-decision (hỏi sớm/nhẹ/nhiều, trong DRAFT)** khỏi **approval (1 gate trước EXECUTE)** — hiện gộp → hoặc hỏi-quá-ít (over-engineer trôi) hoặc làm-phiền-gate. Evidence: câu-hỏi-vàng cứu ket ≥3 lần (per-item SecAccessControl→at-the-door · currency ×1000→full-units · biometric-per-item→access-group) — **completeness-bias = failure-mode #1, neo-người là thuốc giải.** Đang là §0 `[guidance]` slogan → nâng thành phiếu field. **Rẻ.**
  - **KHÔNG harvest (giữ Park / watchlist):** auto-bump-lane-on-surface + auto `research_gate` heuristic (bài 3+5 — cơ chế đã có, chỉ thiếu auto-trigger; chưa có incident sos-kit nào → sensor, đừng build). Spec-hash [P051] + SHA-scope [P055] giữ Park (ceremony, chưa có máu — build = process-completeness-bias chính retro tự cảnh báo).
- [ ] **[P059] HARVEST — Windows/cross-platform script portability (POD-agent install dogfood 2026-06-09).** Phát hiện khi cài kit vào POD-agent trên **Windows Git Bash** (máy Sếp Windows; kit gốc viết macOS). Sibling của FINDING POD-agent §C cùng session. **Grounded n≥1** — test hành vi THẬT trên máy, KHÔNG giả lập. **Headline: 2 security guard ĐANG fail-open im lặng trên MỌI máy Windows** — cài xong nhìn như có răng, thực ra không chặn gì. **Audit này CHỈ Windows** (Sếp directive 2026-06-09): Sếp tự quét mac/linux trên máy thật 2 bên — có env thật fix dễ hơn em giả lập BSD.

  | # | File:line | Construct | Hành vi Windows | Mức | Fix direction |
  |---|---|---|---|---|---|
  | 1 | `block-env-edit.sh:26` | `python3 -c` parse JSON | python3 = MS Store shim (exit≠0) → `FILE_PATH=""` → **exit 0 = fail-open**, `.env` edit KHÔNG bị chặn | 🔴 HIGH | sed extract `file_path` như `architect-guard.sh` (zero-dep, đã proven 3-OS) |
  | 2 | `block-unsafe-merge.sh:34` | `python3 -c` parse JSON | same → **fail-open**, force-push/merge ẩu KHÔNG chặn | 🔴 HIGH | grep pattern nguy hiểm trên raw JSON (`push.*--force`, `reset --hard`…) — khỏi parse JSON, zero-dep |
  | 3 | `security-gate.sh:46,50` + `check-*.py` shebang `#!/usr/bin/env python3` | gọi `python3 scripts/*.py` | python3 fail → secret gate skip/sai | 🟠 MED | resolver đầu file: `python3`→`python`→`py`, **test chạy thật** (`"$c" -c ""`) để loại shim, KHÔNG chỉ `command -v` |
  | 4 | `hooks/pre-commit:57` | `python3 -c ast.parse` | python3 fail → false "Syntax error" → chặn nhầm commit (CHỈ khi repo có `pyproject.toml`/`requirements.txt` ở ROOT; POD-agent ko có → thoát nạn lần này) | 🟠 MED | resolver như #3 |
  | 5 | **toàn bộ `*.sh` + `hooks/*`** | CRLF (`core.autocrlf=true` + KHÔNG có `.gitattributes`) | index=LF nhưng worktree=CRLF; shebang `…bash\r`. Git Bash nhân nhượng nên hiện vẫn CHẠY nhưng fragile | 🟠 MED | thêm `.gitattributes`: `*.sh text eol=lf` · `*.py text eol=lf` · `hooks/* text eol=lf` · `bin/* text eol=lf` (vô hại mac/linux, fix dứt điểm Win) |

  **Đã PASS trên Windows (ghi để khỏi nghi oan):** `realpath` ✅ (GNU coreutils 8.32 trong Git Bash) · `mktemp` ✅ · `session-start-banner.sh` date đã có sẵn GNU+BSD fallback ✅ · `architect-guard.sh` (sed-based) ✅ · không thấy `grep -P` / `stat -c|-f` / `mapfile`.
  **Note cho audit mac/linux của Sếp (em KHÔNG test được ở Win):** `bin/sos.sh:20 realpath` (mac không có mặc định) · `phieu/phieu.sh:122 sed -i.bak` (BSD vs GNU — `-i.bak` có thể OK, verify) · nhánh date BSD trong banner.
  **Doctrine Q cho Sếp:** guard khi KHÔNG tìm được parser nên **fail-closed (chặn+báo)** hay fail-open? Guard bảo mật → em nghiêng fail-closed. (Ảnh hưởng fix #1–#3.)
  **Liên hệ:** [P046] block-env-edit + family đã ship nhưng **rớt cross-platform** = cùng họ "KIT LAG / ship≠chạy". POD-agent đã cài bản lỗi này → re-copy sau khi fix (Sếp directive: fix sos-kit trước, quay lại POD-agent sau).
- [ ] **[P060–P063] HARVEST batch — downstream BUILD dogfood (claude-hooks + doc-rotate Rust port) 2026-06-09.** **VERDICT: sos-kit dogfood ĐẠT.** Workflow cõng trọn 1 port Rust nhiều phiếu (doc-rotate RP01→RP07b COMPLETE, 72/72 test, cargo install chạy, Python retired) + 1 partial (claude-hooks P001-P002). **Cú cứu thật (F04):** ARCHITECTURE ghi "byte-exact" nhưng code lưu **char offset** → Architect docs-only nuốt giả định sai vào phiếu RP02 → nếu lọt sẽ corrupt MỌI entry sau ký tự tiếng Việt đầu; Worker CHALLENGE + oracle-3-field BẮT pre-ship (v2.2 §2 HOLD trên PARTIAL oracle). Friction logs: `~/claude-hooks/docs/SOS_KIT_FEEDBACK.md` (F-001..F-003) + `~/doc-rotate/docs/WORKFLOW_FRICTION_LOG.md` (F01..F08 + pilot scoreboard).
  - **✅ FIXED 2026-06-09 (3 cheap, cùng commit này):** F-001 banner thiếu `touch worker-active` (Worker Write đầu bị chặn) → banner sync 2-chiều · F-003 pre-commit `[3/7]` hardcode `docs/CHANGELOG.md` false-positive → grep `^(docs/)?CHANGELOG\.md$` · F02 `target/` 167MB không ignore khi scaffold → `sos adopt` seed `.gitignore` (target/__pycache__/node_modules/dist/egg-info + runtime markers).
  - [x] ~~**[P060]**~~ ✅ SHIPPED 2026-06-09 — `sos adopt` now copies `docs/ORCHESTRATION.md` (F01: orchestrator was "blind handbook" downstream). `adopt_item "docs/ORCHESTRATION.md"` added to spine; temp-adopt verified 298-line spec lands.
  - [x] ~~**[P061]**~~ ✅ SHIPPED 2026-06-09 (Sếp chốt **option b = track**) — phiếu = audit trail, committed (NOT gitignored). `sos adopt` seeds `.phieu-counter` (0); `agents/worker.md` "You MUST track the phiếu file (`git add`, not rewrite)". Fixes F-002/F03 (phiếu untracked → `git mv active→done` FAIL + debate trail invisible).
  - [x] ~~**[P062]**~~ ✅ SHIPPED 2026-06-09 — CLAUDE.md DOCS GATE row strengthened: `hooks/pre-commit` SECTION add/remove now flags it changes the **phase COUNT `[N/M]`** → update M everywhere (labels + `# Runs in order` header + prose "Phase N"/"N phases" in CLAUDE.md + ARCHITECTURE). Guidance not gate (§0.1 — comparing count to prose isn't soundly mechanizable). Fixes F08 silent 3-phiếu drift.
  - [ ] **[P063]** **[DOCTRINE → RETRO v2.3, KHÔNG ad-hoc]** Pattern C — Architect docs-only dễ tổn thương trước doc mơ hồ/aspirational (F04 + F06 + F07): (1) tag doctrine claim `[intent]` vs `[verified-impl]` (chặn "byte-exact" aspirational lọt vào phiếu); (2) TICKET_TEMPLATE golden-snapshot field spec rõ UNIT (byte/char — F06 Rust `String::len()`=byte vs Python `len(str)`=char); (3) disambiguate hard-rule "fixture tự chế" = file `.md` hand-author (CẤM) vs synthetic in-code instance khi oracle SINH (OK) — F07 tốn 1 vòng CHALLENGE vì wording. Oracle-3-field là LƯỚI đã giữ → mấy cái này làm nó rẻ hơn, không thay nó.
  - **→ RETRO v2.3:** F04 + pilot scoreboard Q1-Q4 (oracle PARTIAL self-stop HOLD · M1=char/byte UTF-8 thật · lưới=Worker+oracle KHÔNG chỉ docs · hook cap enforce YES) = **bằng chứng v2.2 §2 sống trên port thật** → ghi vào retro. Wins ngân hàng: F05 (no-code-on-default chặn main, không bypass) + F04-net (workflow chặn được corruption-class bug).
- [ ] **[P064–P066] HARVEST batch — tarot tool-adoption KIT items (2026-06-09).** **WIN trước: tarot tool-dogfood ĐẠT** — doc-rotate (P343/PR#627) + claude-hooks (P344/PR#628, Giám sát APPROVE) chạy OK trong tarot prod thật. Tool-findings đã route về repo tool (F-* trong `~/doc-rotate/docs/TAROT_DOGFOOD_FEEDBACK.md` + `~/claude-hooks/docs/SOS_KIT_FEEDBACK.md`). Đây CHỈ item KIT (distribution/convention). Source: `docs/TAROT_ADOPTION_HANDOFF.md`. **Timing (Sếp): kit làm SAU khi tool ổn ở NHIỀU dự án** (tarot mới 1 adoption thật) → capture-only. Cụm này = phần CỤ THỂ của §B.1 "installer hợp nhất chưa code" + §D Tier-1/Tier-3, KHÔNG phải item mới rời.
  - [x] ~~**[P064]**~~ ✅ **SHIPPED 2026-06-11** — `install.sh` (curl|sh) + release-CI doctor v0.1.0 / claude-hooks v0.9.1 (3-target GitHub Releases) + B+3 fail-closed shim hạ vào `scripts/block-unsafe-merge.sh`. Acceptance: fresh-machine sim → `sos adopt` CONNECTED. Chi tiết CHANGELOG 2026-06-11. ~~🔴 **Installer cross-OS + cho NGƯỜI-KHÔNG-RUST** (K-1)~~ — **chỉnh lại vision "1 lệnh"**: `cargo install` *giả định Rust đã cài* (như `npm` giả định Node) → share cho bạn-không-code = bắt cài cả Rust toolchain = rào. → "1 lệnh" THẬT = **prebuilt binary (GitHub Releases 3-target mac-arm64/win-x64/linux-x64) + `curl … | sh` bootstrap**; `cargo install`/crates.io chỉ là đường-DEV. Cross-ref `docs/INSTALL.md` + `docs/BOOTSTRAP_AUTOMATION_DRAFT.md`. **Đây là Tier-3 "đóng kit" — sau trigger-wiring + adopt-poisoned.**
    **✅ RATIFIED DECISION (Sếp 2026-06-09) — canonical deploy pattern cho binary hook (từ claude-hooks F-006 fail-closed):** binary có thể VẮNG PATH (máy mới) → hook fail-CLOSED (`block-unsafe-merge`, gác merge security) vắng → exit 127 → harness allow → **gác mở im lặng** (case nguy hiểm duy nhất; 3 hook fail-open kia vắng=allow=đúng default → wire binary thẳng). **Chốt = B + 3:** **(B)** fail-closed shim `command -v claude-hooks || { echo BLOCKED >&2; exit 2; }` rồi `exec` — vắng → block LOUD, không open SILENT; **(3)** prebuilt binary (= chính P064) → "phiền" của B co lại còn 1 dòng `curl|sh`. **KHÔNG chọn A (bash-fallback)** vì: tái sinh bash mà port sinh ra để giết (P059 bash block-unsafe-merge ĐÃ fail-open im lặng trên Windows = bằng chứng F-005 bash-rot); transitional không durable (vỡ khi retire bash); đòi nuôi bash-parity vĩnh viễn ([[feedback_memory_dependent_sync_dies]]). Áp đúng fail-closed doctrine của kit (§B "guard bảo mật → fail-closed"). **Hạ vào:** adoption doctrine (`docs/SETUP.md`/`INSTALL.md` khi build P064) + update claude-hooks README/handoff từ "adopter's choice" → "sos-kit canonical = B+3" (claude-hooks-side, khi nguội).
  - [ ] **[P071]** 🟡 **Release-asset integrity — checksum + pinning (Giám sát finding trên install.sh, 2026-06-11).** `install.sh` tải binary HTTPS-enforced nhưng KHÔNG checksum/signature, và `releases/latest` không pin version → trust anchor = GitHub account. **Deferral là quyết định explicit** (threat model hiện tại: account 2FA, repo private-ecosystem, n-user=1). Cure khi mở public: release CI publish `<asset>.sha256` (1 step thêm vào `release.yml` cả doctor + claude-hooks) + install.sh fetch-verify trước chmod +x (pattern rustup/mise). Timeout + PATH-spoofing hardening đã fix tại chỗ cùng ngày.
  - [x] ~~**[P065]**~~ ✅ **SHIPPED 2026-06-11** — `templates/setup-dev.sh` golden (generalize tarot P344: Rust check → cargo install TOOLS → F09-guarded hook arm; advisory-cron opt-in comment). ~~**`setup-dev.sh` → golden template** (K-2)~~ — P344 đẻ `scripts/setup-dev.sh` (check Rust→warn nếu thiếu · `cargo install` cả bộ binary · reinstall doc-rotate cap-check vào `.git/hooks/pre-commit`). Pattern "bootstrap mỗi máy" mà MỌI repo adopt sẽ lặp → nâng thành golden template sos-kit (lắp vào, không tự viết lại). Lõi của bootstrap, liên quan P064.
  - [ ] **[P066]** ⚪ **Chuẩn hoá tên + format file dogfood-feedback** (K-3, minor) — kênh dogfood project→tool→kit đang lệch tên (`TAROT_DOGFOOD_FEEDBACK.md` vs `SOS_KIT_FEEDBACK.md`). Chốt 1 tên + section format, port vào golden template. **Meta-note:** kênh feedback CÓ TÊN CHUẨN = phần load-bearing của loop (friction-log discipline) — đáng chuẩn hoá dù minor.
- [ ] **[P067] FINDING — `sos adopt` KHÔNG có upgrade-path → cần `sos sync` (kit-adopt stress test, media-rating-app 2026-06-09).** **Bài test KIT thật đầu tiên trên repo NGOẠI-LAI** (foreign Flask app + kit **v2.0** cũ — 4 agent no-orchestrator, pre-commit `[1/3]`), chạy trên clone `/tmp` (repo thật có 28 uncommitted → không đụng). **n=1 cứng cho giả thuyết KIT-LAG.**
  **Kết quả adopt v2.3 đè v2.0:** 35 file ADD sạch (spine v2.3 mới: orchestrator.md, security-gate, no-code-on-default, block-env-commit, parsers, …) + **27 CONFLICT** stage vào `.sos-adopt-incoming/` đòi merge tay.
  **Phát hiện cốt lõi (đã phân loại):** **~tất cả 27 conflict là STALE-CANONICAL (lấy-bản-mới), KHÔNG phải media-customization.** Bằng chứng: `skills/qa/SKILL.md` = **0-diff** (y hệt canonical) · `hooks/pre-commit` `[1/3]` chỉ **thiếu 5 section mới** (không có section media-riêng) → take-newer đúng · `phieu/*`, `skills/*`, `ORCHESTRATION.md`, scripts = canonical-đã-tiến-hoá. File media-RIÊNG thật (`CLAUDE.md`, `BACKLOG`, `INVARIANTS`, `.docs-gate.toml`, `AGENT_MAP`) thì adopt **giữ đúng** (không conflict).
  **Bài học:** `sos adopt` đúng cho **fresh-adopt** (additive, POD-agent kiểu mostly-add) nhưng **SAI cho UPGRADE** — nó coi MỌI file tồn tại là conflict, không phân biệt được "bản-cũ-canonical" vs "repo-tự-sửa" → đổ 27 đống merge-tay mà thực ra ~26 là take-newer máy-móc. **Không ai merge tay 27 file → kit-cũ kẹt mãi (KIT-LAG).**
  **Direction (cái `sos sync` cần):** **provenance manifest** — adopt ghi lúc copy: mỗi spine file đến từ canonical-hash nào. Sync: nếu file-hiện-tại == origin-hash đã ghi → *chưa sửa* → **take-newer an toàn**; khác → merge thật. Có manifest → 27 conflict co lại còn ~1-2 thật (settings.json). **Cheap-leverage cao** (manifest rẻ, giết 27 hand-merge).
  **KHÔNG test được (flag):** đây mới là **KIT-LAG / version-drift** half. **Blocker C code-poison half CHƯA probe** — adopt chỉ chạm file-kit, không scan code/docs Flask của media tìm độc/drift. Cần probe riêng (advisory-watch / doctor validate-map / docs↔code reconcile) cho nửa kia.
  **Trạng thái:** ✅ **CURE BUILT + PROVEN 2026-06-09** — `sos sync` shipped (`bin/sos.sh`). Provenance oracle = **sos-kit git history** (không cần manifest, chạy retroactive trên repo đã adopt): downstream file khớp blob lịch sử nào của canonical-path → unmodified → take-newer; khớp không-bản-nào → customized → flag. **Fire-test trên media clone v2.0→v2.3: 27 merge-tay → CÒN 2.** (9 UPDATED auto take-newer gồm `pre-commit [1/3]→[1/7]`+ORCHESTRATION+phieu.sh · 18 already-current · 2 FLAGGED = `architect.md`+`worker.md` media tự sửa thật → đúng phải merge tay.) Identity files KHÔNG đụng · `doctor verify-setup` exit 0 sau sync. **KIT-LAG half của §C = ĐÓNG.** **n=2 PROVEN (2026-06-09):** chạy thật trên **claude-hooks production** (commit `a448563`) — ADD ORCHESTRATION.md + take-newer 3 (banner F-001 · pre-commit F-003 · orchestrator doctrine) + flag 2 customized (worker.md background:true, architect-guard 140-line custom) + 56 already-current. Đóng luôn deferred re-sync. doc-rotate skip (hot — branch DR05). Caveat MVP: git-history scan chậm trên history khổng lồ; eol/gitattributes có thể ảnh hưởng hash-match cross-platform (mac/linux LF OK, verify Win sau). **Còn mở:** code-poison half của §C → xem [P068].
- [ ] **[P068] FINDING — code-poison half của §C: poison-scan pass trên media (2026-06-09).** Chạy battery tool-CÓ-SẴN lên media clone (committed) đối chiếu lỗ-đã-biết làm oracle. **Reframe xác nhận đúng: KHÔNG cần build flow khổng lồ — mostly reuse + 2 gap nhỏ + 1 phần judgment bất khả-cơ-học.**
  **Catch vs MISS (đo thật):**

  | Loại độc | Tool | Kết quả |
  |---|---|---|
  | Secret committed | `check-runtime-secrets.py` | ✅ **BẮT** — DB password (`postgres...ord@`) trong `.github/workflows/ci.yml` + `docker-compose.yml` (finding THẬT của media) |
  | Secret full-repo | `check-hardcoded-secrets.py` | ⚠️ **MÙ** — scan **staged-only** (`git diff --cached`) → 0 file trên repo có-sẵn. Cần `--full-repo` audit mode |
  | CVE/dep | advisory-scan | ⏭ surface có (`requirements.txt` pinned) — cần subagent+network, không chạy headless |
  | docs↔code drift | `doctor validate-map` | ⏭ N/A — media không có AGENT_MAP; `sos map` build trước rồi mới validate |
  | **Logic/authz (privacy)** | — | ❌ **MISS bởi MỌI tool cơ học** — confirmed `app/routes/media/detail.py`: route `media_detail` (L128) KHÔNG `@login_required` (public) + review query (L169) không filter visibility → guest thấy review private |

  **Gap chính (2 thứ + 1 judgment):**
  1. **Tool kit là COMMIT-GATE, không phải REPO-AUDITOR.** `check-hardcoded` scan staged-only (mù trên brownfield); `security-gate.sh` không portable (chạy từ trong repo adopted). → cần **`--full-repo`/audit mode** cho dùng poison-scan (`check-runtime-secrets` thì đã scan full-tree → bắt được).
  2. **Logic/authz poison BẤT KHẢ cơ-học** — NHƯNG kit ĐÃ CÓ đúng detector: **Giám sát `boundary-check` INV-3 (cross-user resource → ownership binding)** = chính class lỗ privacy này. Gap: boundary-check soi **PR-DIFF**, không soi code-có-sẵn. → cho brownfield cần chạy Giám sát/`/review` **full-repo audit mode**, không diff-only.
  3. Phần **irreducible-judgment** (logic poison) → route về Giám sát/review, **KHÔNG build authz-checker cơ học mới** (over-build trap).
  **→ "Adopt-poisoned flow" = orchestrate tool-có-sẵn thành 1 scan pass (secret full-repo + advisory CVE + sos map→validate-map drift) + route logic-poison sang Giám sát full-repo audit.** Build = nhỏ (2 audit-mode flag + 1 orchestration), KHÔNG phải flow khổng lồ. Capture-only, chờ promote.
  **→ ROUTE VỀ MEDIA (finding của media, không phải kit):** DB password committed trong CI/compose + lỗ privacy `detail.py:169` (parked) — fix ở repo media, không phải sos-kit.
- [ ] **[P069] FINDING — Architect Write-envelope KHÔNG được hook enforce (doc-vs-hook drift, surfaced từ claude-hooks F-004 2026-06-09).** `agents/architect.md:22` nói Architect "**only Write new phiếu files** (docs/ticket/P*.md)" — nhưng KHÔNG hook nào enforce: `architect-guard.sh` fire trên **Read|Glob** (không Write); `orchestrator-guard.sh` fire trên Write nhưng dùng **denylist product-source** (`*.swift`/`*.pbxproj`/`src/**`) + allow-list kit-maintenance (`scripts/`/`docs/`/`*.sh`/`*.md`). → khi architect-active, Architect Write `scripts/foo.sh` / `docs/random.md` (non-product, non-phiếu) **trôi không bị chặn**. Bản combined CŨ (tarot deployed + claude-hooks port) CÓ Write-allowlist branch trong architect-guard; **split design canonical (architect-guard=read / orchestrator-guard=write-product) ĐÁNH RƠI cái allowlist** cho Architect. **Severity: latent** (Architect subagent theo prompt; n=0 incident) NHƯNG đúng class "mechanical backstop lỏng hơn doctrine" ([[feedback_enforce_via_mechanism_not_memory]]). **Quyết định cần:** (a) thêm lại Write-allowlist branch vào `architect-guard.sh` (fire trên Write khi architect-active → block ngoài `docs/ticket/P*.md`), HOẶC (b) chấp nhận (prompt + tool-restriction đủ, hook là belt-and-suspenders). Em nghiêng (a) — security-surface, rẻ, đối xứng orchestrator-guard. (claude-hooks giữ bản combined của mình = port reference đúng; sync flag đã preserve.) **✅ SHIPPED (a) 2026-06-09:** `architect-guard.sh` giờ dispatch theo `tool_name` — Read/Glob giữ read-block; **Write/Edit/MultiEdit → allowlist CHỈ phiếu** (`basename` khớp `P[0-9]*-*.md`), mọi thứ khác (code/scripts/docs-khác) BLOCK exit 2. settings.json matcher `Read|Glob` → `Read|Glob|Write|Edit`. **Fire-test 9/9 PASS** (Write phiếu→allow · Write scripts/docs-random→BLOCK · Read .md→allow · Read src→block · no-marker→allow-all). Downstream: doc-rotate sync sau (hot); claude-hooks đã có bản combined riêng (port reference).
- [ ] **[P070] HARVEST — doc-rotate Rust-port friction log (F09-F13 NEW; F01-F08 đã xử) 2026-06-09.** Source: `docs/retro/FRICTION_doc-rotate_rust-port_2026-06-09.md` (13 finding, RP01-RP07 + DR01-06 dogfood adopt-tarot). **F01-F08 đã action session này** (F01→P060 · F02→adopt-gitignore · F03→P061 · F04/F06/F07→P063 · F05 win · F08→P062). Dưới = 5 NEW:
  - [x] ~~**[F09]**~~ 🔴 ✅ **FIXED 2026-06-09** — `install-hooks.sh` **hijack `core.hooksPath` + nuốt security-hook của adopter IM LẶNG** (`git config core.hooksPath hooks` đè + rename `.git/hooks/*` → `.bak`; adopter có hooksPath/security-gate riêng như tarot P275 → mất gác). 1 installer 2 audience = class lỗi nuốt-hook-security. **Fix:** thêm GUARD — detect `core.hooksPath` đã set ≠ `hooks` HOẶC `.git/hooks/pre-commit` thật → confirm (TTY) / **ABORT exit 1 (non-TTY)** trước khi hijack. Fire-test 3/3 (clean→proceed · adopter-hooksPath non-TTY→ABORT không clobber · idempotent→proceed). (DR01 doc-rotate-local cũng thêm `install-cap-check.sh` additive — doc-rotate-specific, kit không cần.)
  - [x] ~~**[F13]**~~ 🟡 ✅ **FIXED 2026-06-09** — scaffold `version="0.0.0"` drift. (1) `bin/sos.sh:499-500` scaffold giờ set `0.1.0` (cả python+rust), không `0.0.0`; (2) DOCS-GATE row mới "CHANGELOG version bump → `Cargo.toml`/`pyproject.toml [package] version` sync" (mechanizable: grep Cargo vs first `## [` CHANGELOG).
  - [x] ~~**[F10]**~~ 🟡 ✅ **FIXED 2026-06-09** — DOCS-GATE row mới "**Language port / module rename / file move** → `docs/AGENT_MAP.yaml` paths update + re-run `doctor validate-map`" (validate-map là lưới cơ học, nhưng phải RUN). doc-rotate đã CLOSED local DR06.
  - [ ] **[F11/F12]** 🟡 (doc-rotate product, fixed DR02/DR03 local) **Heuristic-tool ÉP content bẻ theo tool** = vision violation. F11 classifier token-exact (`Sub-mech` vs `Sub-mechanism` miss → adopter phải rename) → fix = explicit keep-marker escape-hatch. F12 hard-block cross-ref đánh nhau "broken-OK doctrine" → fix = default-SOFT + `--strict` opt-in. **Takeaway kit (recipe-level):** heuristic/destructive tool nên có **escape-hatch (override marker) + default-permissive (warn không block) NGAY TỪ ĐẦU** — token-only forcing content-bend = class adopter-friction. Product-specific nhưng pattern chung cho recipe AI-tool.
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
