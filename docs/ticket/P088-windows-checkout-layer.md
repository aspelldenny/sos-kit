# PHIẾU P088: Windows checkout/EOL layer — CRLF breaks parity goldens + dead `.claude/skills/*` symlinks

> **ID format:** `P088` — assigned manually (Windows bug-fix wave, Sếp direct approval — override Architect Rule 0).
> **Filename:** `docs/ticket/P088-windows-checkout-layer.md`
> **Branch:** `fix/P088-windows-checkout-layer`

---

> **Loại:** Bugfix (distribution / checkout contract)
> **Ưu tiên:** P1 (Windows dogfood 2026-07-24: 3 parity fail CRLF-only + skills chết trên Windows)
> **Tầng:** 1 — `.gitattributes` + `.claude/skills/*` symlink là **distribution contract**: sai thì LAN sang MỌI Windows checkout (parity goldens hash-mismatch, skills không load) và tương tác với security baseline (trust-gate hash của `.claude/settings.json`/`.mcp.json`). Checkout-layer touch ảnh hưởng consumer + không cục-bộ → AUTO Tầng 1 (per `docs/LAYERS.md` §2-tier).
> **Lane:** Guarded — budget axis no-cap (2 item gộp + 1 open decision cần Chủ nhà gate + tương tác P087 trust-gate + git-shell-out patch trong Rust). Def: `docs/WORKFLOW_V2.2.md` §1.
> **Ảnh hưởng:** `.gitattributes`, `crates/sos-cli/src/commands/{new,adopt}.rs` (git shell-out flags), `docs/SETUP.md` + `INSTALL.md` (Windows section), có thể `doctor`/verify check (nếu chọn option (c) symlink), CHANGELOG.
> **Dependency:** Tương tác P087 BUG 2 (trust-gate hash format). P088 ITEM 1 giải CRLF-hash-mismatch **structurally** (force LF cho text) — có thể là điều kiện đủ để full `[8/8]` xanh trên Windows mà P087 một mình chưa đóng. Thứ tự merge khuyến nghị: P087 trước (format fix) → P088 (EOL fix), nhưng 2 phiếu độc lập về file đụng.

---

## Context

### Vấn đề hiện tại

Windows dogfood 2026-07-24: trong 6 parity fail, 3 cái là **CRLF-only** (`parity_sync_enforced`, `parity_new_enforced`, `parity_adopt_enforced`) + phát hiện `.claude/skills/*` symlink chết trên Windows. P088 gộp 2 item (Chủ nhà-ratified).

**ITEM 1 — CRLF phá parity goldens + capture noise.**
- `.gitattributes` hiện chỉ force LF cho `*.sh`, `*.bash`, `*.py`, `hooks/*`, `bin/*` (P059). Với `core.autocrlf=true` (Windows default), `crates/sos-cli/tests/golden/*.golden` checkout thành CRLF → parity string-compare fail với diff `\r\n` vs `\n` thuần: `parity_sync_enforced`, `parity_new_enforced`, `parity_adopt_enforced`.
- Fix: mở rộng `.gitattributes` (`*.golden text eol=lf` — hoặc rộng hơn `* text=auto eol=lf`). Cân nhắc thêm `*.toml`, `*.json`, `*.md`, `*.yaml` mà trust-gate hash (tương tác P087 BUG 2 note — CRLF checkout của `.claude/settings.json`/`.mcp.json` đổi sha256 vs POSIX-seeded baseline; force LF repo-wide cho text giải structurally).
- ALSO: `sos new`/`adopt` chạy `git add` trong fixture; trên Windows git in `warning: in the working copy of 'X', LF will be replaced by CRLF...` → lọt vào captured output → parity mismatch DÙ đã fix golden EOL. Fix hướng: code Rust shell-out git nên truyền `-c core.autocrlf=false` (hoặc `-c advice.*=false` / tách stderr khỏi stream parity-capture — Worker verify stream nào test bắt) để output platform-invariant.
- Contributor-env note: checkout hiện có cần `git add --renormalize .` hoặc re-clone sau khi đổi `.gitattributes`.

**ITEM 2 — `.claude/skills/*` symlink chết trên Windows.**
- Pull này thêm symlink `.claude/skills/{apply,forge,idea,init,retro} → ../../skills/<name>` (mirror convention `.claude/agents/ → agents/`). Trên Windows default `core.symlinks=false` → checkout thành FILE TEXT PHẲNG chứa đường link (verified: file 17-18 byte). Skills KHÔNG load trong Claude Code trên Windows; copy logic mong dir thật có thể misbehave (Rust `new.rs` deref-copy xử POSIX symlink OK; trên Windows sẽ copy file stub text).
- **Decision cần Chủ nhà gate** (Architect đề xuất, KHÔNG tự quyết vì đây là vision/distribution decision):
  - (a) document requirement — Windows Developer Mode + `git config core.symlinks true` + re-clone — trong `docs/SETUP.md` + `INSTALL.md` Windows section;
  - (b) drop symlinks, dùng copy thật + sync mechanism;
  - (c) hybrid: giữ symlink, thêm doctor/verify check phát hiện stub-text-file checkout + in fix.
  - **Architect recommend (c) hoặc (a).** (b) đảo một kit decision có chủ đích — copy+sync CŨ đã rot (CLAUDE.md "Language" section history: `sed 's/Chủ nhà/Sếp/g'` + per-repo copy + `sync-personal-agents.sh` conflate 2 layer, rot thành drift, bị gỡ ưu tiên symlink). Flag history đó trong phiếu.

### Giải pháp

1. **ITEM 1 (mechanical, no open Q):** mở rộng `.gitattributes` force LF cho `*.golden` (tối thiểu) + text families trust-gate hash (`*.toml`, `*.json`, `*.md`, `*.yaml`) — Architect cân blast radius, **ưu tiên targeted line theo kit style** (P059 precedent = targeted, không `* text=auto` toàn cục). Thêm `-c core.autocrlf=false` cho git shell-out trong `new.rs`/`adopt.rs` (hoặc tách stderr — Worker verify stream). Task line + doc mention `git add --renormalize .`.
2. **ITEM 2 (BLOCKED trên Chủ nhà gate):** Architect đề xuất (c) hybrid = giữ symlink + `doctor` (hoặc verify-setup) detect stub-text-file → in fix. Chủ nhà chọn (a)/(b)/(c) tại APPROVAL_GATE trước khi Worker EXECUTE ITEM 2. ITEM 1 KHÔNG bị block bởi decision này (có thể ship trước).

### Scope
- CHỈ sửa `.gitattributes` + git shell-out flags trong `crates/sos-cli/src/commands/{new,adopt}.rs` + docs Windows section + (nếu Chủ nhà chọn (c)) `doctor`/verify check.
- KHÔNG đụng `map.rs` (P087), KHÔNG đụng `trust-gate.sh` logic (P087) — nhưng `.gitattributes` LF cho text families HỖ TRỢ P087 BUG 2 (CRLF-hash mismatch giải structurally).
- KHÔNG rewrite skills content, KHÔNG revert `.claude/agents/ → agents/` symlink convention.

---

## Task 0 — Verification Anchors

> Architect envelope chặn source-read → file:line từ Quản đốc insight briefing. Marker: `[unverified]` (chưa tự Read), `[needs Worker verify]` (số dòng/stream/signature).

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `.gitattributes` hiện force LF chỉ cho `*.sh`,`*.bash`,`*.py`,`hooks/*`,`bin/*` (P059) — KHÔNG có `*.golden` | `grep -n "eol=lf\|text=auto\|golden" .gitattributes` | ✅ `[verified: .gitattributes:6-10 exactly *.sh/*.bash/*.py/hooks/*/bin/*, no *.golden, no *.json, no *.toml — confirms both P088 ITEM1 gap AND P087 BUG2's CRLF-hash-mismatch note: .claude/settings.json + .mcp.json (SURFACE_GLOBS) are NOT LF-forced today]` |
| 2 | Golden files tại `crates/sos-cli/tests/golden/*.golden` | `ls crates/sos-cli/tests/golden/` | ✅ `[verified: 10 .golden files present incl. new.golden, adopt.golden, sync.golden, map.golden, *.tree.golden, *.gen.golden]` |
| 3 | Failing CRLF-only tests: `parity_sync_enforced`, `parity_new_enforced`, `parity_adopt_enforced` | `grep -n "fn parity_sync_enforced\|fn parity_new_enforced\|fn parity_adopt_enforced" crates/sos-cli/tests/parity.rs` | ⚠️ `[verified: actual lines parity.rs:392(new), :557(adopt), :664(sync) — briefing said :405/:577/:677, drifted ~13-20 lines, names correct]` |
| 4 | `sos new`/`adopt` shell-out `git add`; git in `warning: ... LF will be replaced by CRLF` vào captured stream | `grep -n "git\|add\|Command::new" crates/sos-cli/src/commands/new.rs crates/sos-cli/src/commands/adopt.rs` | ✅ `[verified: new.rs:480 git init, :482 symbolic-ref, :500 git add -A, :508 git add baseline — ALL use .status() = inherit parent stdio; adopt.rs:742 git add loop — same .status(). Test harness (parity.rs:62-95 run_rust*) captures via .output() and does `stdout + stderr` combined (parity.rs:66-67) → confirms git's stderr warning DOES leak into the compared stream. "tách stderr" alternative in Task 2 is NOT viable — test explicitly merges both streams; -c core.autocrlf=false is the only correct fix direction.]` |
| 5 | `.claude/skills/{apply,forge,idea,init,retro}` là symlink `→ ../../skills/<name>` | `ls -la .claude/skills/` (POSIX) — trên Windows là file text 17-18 byte | ✅ `[verified on this Windows checkout RIGHT NOW: 5 files, 17-18 bytes each (apply=18,forge=18,idea=17,init=17,retro=18), content is literal relative-path text e.g. "../../skills/apply" via `cat -A`; `git config core.symlinks` = false locally — reproduces exactly as briefed]` |
| 6 | Symlink convention mirror `.claude/agents/ → agents/` (đừng revert) | đọc `agents/README.md` | ✅ `[verified: agents/README.md:9,28-31,39,42 — explicit convention + explicit history note about the OLD copy+sync rot (sync-personal-agents.sh) that led to the symlink-over-copy decision; matches CLAUDE.md Language section reference]` |
| 7 | `new.rs` deref-copy xử POSIX symlink (Windows copy stub text file) — nếu chọn (b)/(c) cần biết copy path | `grep -n "symlink\|copy\|read_link\|deref" crates/sos-cli/src/commands/new.rs` | ⚠️ `[verified + SCOPE CORRECTION — see O1.1 below: only new.rs is exposed (adopt.rs sources skills from canonical skills/ dir directly, NOT via .claude/skills symlinks, so adopt.rs is NOT affected)]` |
| 8 | `core.autocrlf=false` truyền được qua `git -c` flag ở shell-out call site | `grep -n "Command::new(\"git\")\|args(" crates/sos-cli/src/commands/new.rs crates/sos-cli/src/commands/adopt.rs` | ✅ `[verified: all git invocations are Command::new("git").arg(...) chains — `.arg("-c").arg("core.autocrlf=false")` insertable before the subcommand arg at each of new.rs:480,482,500,508 and adopt.rs:742(+related seed-baseline calls). Standard git per-invocation -c flag, no env-var mechanism needed.]` |

**⏳ ghi nhận:** mọi anchor `[unverified]`/`[needs Worker verify]` do envelope. ITEM 2 anchors (#5-7) chỉ actionable SAU Chủ nhà chọn option — nếu (a) document-only thì #7 (copy path) N/A.

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
[ -d .sos-state ] && cp -r .sos-state ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.
> **⚠️ ITEM 2 có 1 open decision (a/b/c) — Architect đề xuất (c)/(a). Đây là DEFER TO CHỦ NHÀ candidate: distribution/vision decision, không phải technical. Orchestrator dùng AskUserQuestion tại APPROVAL_GATE.**

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

**Anchor verification (recap Task 0):** 8/8 confirmed real (file:line). Anchors #3 (test line numbers) drifted vs briefing (~13-20 lines, function names correct — self-closed, oracle: `grep -n "fn <name>"`, SOUND). Anchor #7 needs a scope correction (below).

**Objections (Tầng 1 only):**
- [O1.1] **Anchor #7 scope is narrower than the "relevant chỉ khi (b)/(c)" framing implies — `new.rs` is the ONLY affected call site, and this matters even under option (a).** Evidence: `crates/sos-cli/src/commands/new.rs:310` calls `copy_tree(&kit.join(".claude/skills")..., true)` — `deref=true` walks `.claude/skills/*` via `WalkDir::follow_links(true)` (`new.rs:101,105`) and `fs::copy`s (`new.rs:114`) whatever it finds. On a Windows checkout of the **kit repo itself** used as the `--kit`/source for `sos new`, `.claude/skills/*` are the 17-18 byte stub text files (verified anchor #5) — `follow_links` has nothing real to dereference, so `copy_tree` would literally copy the 18-byte stub as the target's `.claude/skills/apply` file, not the skill content. By contrast `crates/sos-cli/src/commands/adopt.rs` does NOT go through this path at all — `adopt_skills()` (`adopt.rs:264-293`) walks the canonical `skills/` dir directly (comment at `adopt.rs:262-263`: "skills/ has no symlinks today"), sidestepping `.claude/skills/*` entirely.
  - **Consequence for option (a) (document-only):** the SETUP.md/INSTALL.md guidance ("enable Developer Mode + `core.symlinks true` + re-clone") must be scoped correctly — it's not just about the *consumer's* checkout loading skills in Claude Code, it ALSO has to apply to whichever machine holds the **sos-kit source repo** being passed as `--kit` to `sos new` (since that's what `copy_tree` reads from). If a Windows contributor keeps sos-kit itself checked out with `core.symlinks=false` and runs `sos new` from it, `new`-scaffolded projects will silently ship dead skill stubs regardless of the *target*'s own settings. Phiếu Task 3's (a) doc note currently reads generically ("Windows section") without calling out this source-vs-consumer distinction.
  - **Consequence for option (c) (hybrid detect):** the proposed `doctor`/verify stub-file check needs to run against BOTH the target checkout's `.claude/skills/*` AND — separately — be documented as a pre-flight check on the kit repo itself before it's used as a `sos new --kit` source, or `new.rs:310`'s `copy_tree` will propagate the stub silently with no detection at scaffold time.
  - This does not change WHICH option Chủ nhà should pick — still a/b/c per phiếu — it only changes what "done" looks like for whichever option is chosen (doc/check must cover 2 locations, not 1).

**Proposed alternatives (for whichever option a/c is chosen):**
- A. (Worker lean) Task 3 for (a): explicitly split doc guidance into "if you are a *consumer* running `sos new`/`sos adopt` against a pre-built kit checkout" vs "if you *are* the kit maintainer/contributor whose checkout is the `--kit` source" — both need `core.symlinks true`.
- B. Task 3 for (c): `doctor` check runs standalone (any repo with `.claude/skills/*`), so it's already reusable against the kit source dir too — just document "run `doctor` on your sos-kit checkout too, not only generated projects" in SETUP.md.

**Status:** ✅ Worker challenge complete — 1 non-blocking scope-clarification objection (O1.1), does not block APPROVAL_GATE on ITEM 2's a/b/c decision. Recommend: fold O1.1 into whichever Task 3 branch text Architect/Chủ nhà finalize, no need for a full Architect RESPOND round-trip unless Chủ nhà wants it addressed explicitly before approval.

### Final consensus
- Phiếu version: V1 (no bump — objection is a scope clarification, not a redirection)
- Total turns: 1
- ITEM 2 option chosen by Chủ nhà: **(c) hybrid** — keep symlinks, add mechanical stub-detection check + docs coverage (SETUP.md/INSTALL.md), covering both the maintainer-checkout-as-`--kit`-source case (O1.1) and the consumer-checkout case.
- Approved by Chủ nhà: **✅ APPROVED 2026-07-24** — Chủ nhà approved P087+P088 at APPROVAL_GATE via AskUserQuestion ("Approve cả 2"), decided ITEM 2 = option (c) hybrid.

### APPROVAL RECORD

Chủ nhà approved P087+P088 at APPROVAL_GATE 2026-07-24 via `AskUserQuestion` — "Approve cả 2", and DECIDED P088 ITEM 2 = **option (c) hybrid**: keep symlinks, add a mechanical stub-detection check (detects `.claude/skills/*` checked out as text stub files on Windows) that prints the fix (Windows Developer Mode + `git config core.symlinks true` + re-clone/checkout), plus docs coverage in SETUP.md/INSTALL.md — including the maintainer-checkout-as-`--kit`-source propagation case from O1.1.

---

## Nhiệm vụ

### Task 1: Mở rộng `.gitattributes` force LF cho text checkout (ITEM 1)

**File:** `.gitattributes`

**Tìm:** khối eol=lf hiện có (P059: `*.sh`, `*.bash`, `*.py`, `hooks/*`, `bin/*`) `[needs Worker verify]` nội dung thực.

**Thay bằng / Thêm:** thêm dòng targeted (kit style = targeted, theo P059 precedent — KHÔNG `* text=auto` toàn cục trừ khi Worker CHALLENGE chứng minh blast an toàn hơn):
```gitattributes
*.golden text eol=lf
*.toml   text eol=lf
*.json   text eol=lf
*.md     text eol=lf
*.yaml   text eol=lf
```

**Lưu ý:** `*.golden` là fix TRỰC TIẾP cho 3 parity fail. `*.toml/*.json/*.md/*.yaml` phủ text families mà trust-gate hash (`.claude/settings.json`, `.mcp.json`, docs) → giải CRLF-hash-mismatch của P087 BUG 2 **structurally**. Worker cân: nếu bất kỳ binary/generated file khớp glob này → loại trừ (`-text`). Nếu Worker thấy `* text=auto eol=lf` an toàn + đúng kit intent hơn (ít drift tương lai) → propose tại CHALLENGE, Architect cân blast.

### Task 2: Git shell-out platform-invariant output (ITEM 1)

**File:** `crates/sos-cli/src/commands/new.rs` + `crates/sos-cli/src/commands/adopt.rs`

**Tìm:** chỗ shell-out `git` (git-init / `git add` / arm-hooks) mà output lọt vào stream parity-capture `[needs Worker verify]` — Worker xác nhận (a) call site, (b) test bắt stdout hay stderr.

**Thay bằng / Thêm:** truyền `-c core.autocrlf=false` cho các `git` invocation của born-wire/adopt (per-invocation `git -c core.autocrlf=false <subcmd>`), HOẶC tách stderr khỏi stream parity-capture nếu warning đi stderr và test chỉ bắt stdout — Worker chọn theo stream thực.

**Lưu ý:** mục tiêu: `warning: ... LF will be replaced by CRLF` KHÔNG lọt captured output → parity byte-invariant cross-platform. Nếu test bắt CẢ stderr thì `-c core.autocrlf=false` là fix đúng (chặn warning tại nguồn). Verify KHÔNG đổi hành vi POSIX (`core.autocrlf=false` là default POSIX rồi → no-op). Nếu có nhiều git call site trong born-wire, phủ tất cả các call in ra warning.

### Task 3 (ITEM 2 — CHỜ CHỦ NHÀ CHỌN OPTION): symlink checkout trên Windows

**⚠️ BLOCKED trên APPROVAL_GATE decision. Worker KHÔNG execute Task 3 tới khi Debate Log "Final consensus" ghi option (a)/(b)/(c).**

**Nếu (a) document-only (Architect co-recommend):**
- **File:** `docs/SETUP.md` + `INSTALL.md` — thêm Windows section: yêu cầu Developer Mode + `git config --global core.symlinks true` + re-clone (symlink checkout đúng thành link thật). Note: không có Developer Mode → skills không load, dùng bản copy thủ công hoặc WSL.
- **Lưu ý:** rẻ nhất, không đụng code; nhược: dựa vào user config đúng.

**Nếu (c) hybrid (Architect co-recommend):**
- **File:** `doctor` verify check (hoặc `verify-setup` trong adopt) `[needs Worker verify]` vị trí — detect `.claude/skills/*` là file text 17-18 byte (stub) thay vì symlink/dir → in fix guidance (`core.symlinks true` + re-clone). + docs `docs/SETUP.md` Windows note.
- **Lưu ý:** giữ symlink convention; thêm net phát hiện. Tương tác `new.rs` deref-copy (anchor #7) — verify copy path xử stub-file đúng trên Windows.

**Nếu (b) drop symlinks + copy+sync:**
- **⚠️ Architect flag history:** copy+sync CŨ đã rot (CLAUDE.md "Language" section — per-repo copy + `sync-personal-agents.sh` conflate role-name/address, drift, bị gỡ). Chọn (b) = đảo kit decision có chủ đích → cần Chủ nhà xác nhận rõ đã đọc history này. Worker chỉ execute nếu Final consensus ghi rõ "(b) — history acknowledged".

### Task 4: Contributor-env renormalize note (ITEM 1)

**File:** `docs/SETUP.md` (hoặc `INSTALL.md` Windows/contributing section) `[needs Worker verify]` vị trí section.

**Tìm:** section setup/contributing Windows.

**Thay bằng / Thêm:** note — sau khi pull `.gitattributes` mới, checkout hiện có cần `git add --renormalize .` (rồi commit) HOẶC re-clone để EOL áp đúng.

**Lưu ý:** `.gitattributes` chỉ áp cho checkout mới / renormalize — không tự sửa working tree đang có. Thiếu note này = contributor tưởng fix không chạy.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `.gitattributes` | Task 1: force LF cho `*.golden` + text families (toml/json/md/yaml) |
| `crates/sos-cli/src/commands/new.rs` | Task 2: git shell-out `-c core.autocrlf=false` (hoặc stderr split) |
| `crates/sos-cli/src/commands/adopt.rs` | Task 2: cùng |
| `docs/SETUP.md` | Task 3 (nếu (a)/(c)) + Task 4: Windows symlink note + renormalize note |
| `INSTALL.md` | Task 3 (nếu (a)): Windows Developer Mode + core.symlinks note |
| `doctor`/verify-setup | Task 3 (CHỈ nếu Chủ nhà chọn (c)) — stub-file detect check `[needs Worker verify]` vị trí |
| `CHANGELOG.md` | Docs Gate: entry P088 |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `crates/sos-cli/tests/parity.rs` | `parity_sync/new/adopt_enforced` xanh sau fix (cả Windows + POSIX byte-stable) |
| `.claude/agents/ → agents/` symlink | KHÔNG revert — convention giữ nguyên |
| `skills/{apply,forge,idea,init,retro}/SKILL.md` | Content KHÔNG đổi — chỉ checkout/symlink cơ chế |
| `scripts/trust-gate.sh` | KHÔNG đụng (P087) — nhưng LF-cho-text hỗ trợ trust-gate hash invariance |

---

## Luật chơi (Constraints)

1. `.gitattributes` change là distribution contract Tầng 1 — targeted lines ưu tiên (P059 style); nếu mở `* text=auto` phải qua CHALLENGE + loại trừ binary/generated.
2. POSIX behavior BẤT BIẾN — `core.autocrlf=false` là default POSIX (no-op); LF eol là hiện trạng POSIX.
3. ITEM 2 (Task 3) BLOCKED tới khi Chủ nhà chọn (a)/(b)/(c) tại APPROVAL_GATE. Worker KHÔNG tự chọn — distribution/vision decision.
4. (b) drop-symlink = đảo kit decision có chủ đích → chỉ execute nếu Final consensus ghi "history acknowledged".
5. KHÔNG revert `.claude/agents/ → agents/` symlink convention. KHÔNG đụng `map.rs`/`trust-gate.sh` logic (P087).
6. Sau `.gitattributes` change: nếu file nào bị renormalize trong repo này → commit renormalize riêng/rõ ràng, KHÔNG lẫn logic change.

---

## Nghiệm thu

### Automated
- [x] `cargo check --workspace` clean (Task 2 đụng code — clean build confirmed).
- [x] Windows (this checkout, post `git add --renormalize .` + forced re-checkout of goldens): `cargo test --workspace` → `parity_sync_enforced`, `parity_new_enforced`, `parity_adopt_enforced` **PASS**. Kết hợp P087 + P088 → cả 10 `parity.rs` tests xanh (former 6-fail baseline fully closed).
- [ ] POSIX (linux + macOS): not run in this environment (Windows-only machine) — no code path is platform-conditional in a way that should regress POSIX (all fixes are either Windows-only `warn`/heuristic additions or `Path::components()` normalization that behaves identically on POSIX). Flagging as unverified-on-POSIX, not skipped-by-choice.

### Manual Testing
- [x] Windows: `git status` clean sau renormalize + forced golden re-checkout (no pending CRLF diff for the LF-forced families).
- [x] Windows: `sos new`/`sos adopt` output (via `cargo test`) KHÔNG chứa `warning: ... LF will be replaced by CRLF` in the ASSERTED stream (test harness's own `run_git()` fixture-setup helper still prints it to inherited stdio, uncaptured — out of scope, see Discovery).
- [x] ITEM 2 — option (c) chosen: `find_symlink_stubs`/`warn_symlink_stubs` (new.rs, reused by adopt.rs) detects stub-file `.claude/{agents,commands,skills}/*` → prints Developer Mode + `core.symlinks true` + re-clone fix. Not live-tested against an actual Developer-Mode-enabled re-clone in this session (no 2nd Windows env available) — logic verified via the heuristic's unit-level reasoning + non-interference with existing test fixtures (see Discovery).

### Regression
- [x] POSIX: no symlink-handling logic changed — `find_symlink_stubs` is purely additive (new function, new call sites), existing `copy_tree`/`adopt_item` deref behavior untouched.
- [x] Trust-gate baseline (P087) supported: LF-forcing `.claude/settings.json`/`.mcp.json` closes the CRLF-hash-mismatch class structurally per Debate Log; full `cargo test --workspace` green including P086/P087's own acceptance tests (`new_first_commit_passes_all_hooks_zero_seed`, `adopt_does_not_stage_users_untracked_file`).

### Docs Gate
- [x] `CHANGELOG.md` — entry P088 (above P087, v2.3 forge section).
- [x] `docs/SETUP.md` — new §5b Windows checkout/EOL/symlink section, incl. renormalize + forced-recheckout note.
- [x] `INSTALL.md` — new Windows checkout section.

### Discovery Report
- [x] `docs/discoveries/P088.md` — assumptions CORRECT/WRONG (file:line), ITEM 2 option (c) rationale, git stream (combined stdout+stderr, confirmed), `.gitattributes` glob blast reasoning, P087 interaction (CRLF-hash mismatch closed structurally), tier notes (Tầng 2 self-decide for is_noise/attic fix + stub-check scope correction).
- [x] 1-line index appended to `docs/DISCOVERIES.md`.
