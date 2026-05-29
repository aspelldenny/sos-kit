# PHIẾU P006: Pre-commit fresh-install friction — `docs-gate` bootstrap

> **Loại:** Bugfix (install UX)
> **Ưu tiên:** P1
> **Tầng:** 2 (lặt vặt — ≤4 files mới/sửa, anchor rõ, surgical)
> **Ảnh hưởng:** `templates/.docs-gate.toml` (NEW), `/.docs-gate.toml` (NEW, sos-kit root dogfood), `hooks/pre-commit` (resilience preamble), install doc (`docs/SETUP.md` hoặc `INSTALL.md` — Worker xác minh)
> **Dependency:** None
> **Branch:** `fix/P006-docs-gate-bootstrap`
> **Date:** 2026-05-10

---

## Context

### Vấn đề hiện tại

`hooks/pre-commit` shell `docs-gate` binary. Trên repo fresh (chưa có `.docs-gate.toml`), hook fail ungracefully → user mới install kit lần đầu thấy hook đỏ mà không biết bootstrap như thế nào.

**Bằng chứng tích lũy (3 incident):**
1. **P035 EXECUTE** — Worker report "docs-gate not runnable in sos-kit root (no `.docs-gate.toml`)".
2. **P037 EXECUTE** — same friction.
3. **2026-05-10, media-rating-app P001 EXECUTE (cross-project dogfood)** — Worker hit `docs-gate` v0.1.0 schema mismatch: prior Architect (in P001) bịa `[[trigger]]` table syntax, reality là **flat-string** (`docs_dir = "..."`, `changelog = "..."`, `[architecture]` section). Worker recovery in-flight bằng `docs-gate init` + chỉnh tay → `/home/sep/media-rating-app/.docs-gate.toml` chính là working artifact.

**Sos-kit kit-shoot-foot:** `test -f /home/sep/sos-kit/.docs-gate.toml` → **false** (đã verify ở recon). Kit ship hook nhưng không dogfood chính nó. Cần đóng vòng.

### Giải pháp

4 deliverables surgical, không động vào `~/docs-gate` binary repo:

1. **`templates/.docs-gate.toml`** (NEW) — Reference template generic cho downstream sos-kit-style projects. Worker generate bằng `docs-gate init` trong tmp dir + curate cho sos-kit canonical layout (docs/ + CHANGELOG.md + ARCHITECTURE expectation).
2. **`/.docs-gate.toml`** (NEW, sos-kit root) — Dogfood. Tuned cho sos-kit's own docs/ structure (`docs/` docs_dir, root `CHANGELOG.md` changelog — đã verify root-level qua Glob, KHÔNG `docs/CHANGELOG.md`, Discovery Reports tại `docs/discoveries/P*.md`).
3. **`hooks/pre-commit` resilience preamble** — Thêm guard `[ -f .docs-gate.toml ]` trước docs-gate invocation. Nếu missing → in yellow warning "docs-gate skipped: no .docs-gate.toml — run `docs-gate init` to bootstrap" + skip step (không fail), các checks khác (type-check + v2 BACKLOG/Discovery) vẫn chạy độc lập.
4. **Install doc** — Thêm 1 step "after copying `hooks/pre-commit`, run `docs-gate init` (hoặc copy `templates/.docs-gate.toml`)". Worker xác minh đúng file (`docs/SETUP.md` vs `INSTALL.md` — cả hai tồn tại, gắn step vào file documenting hook setup).

### Scope

- CHỈ tạo/sửa: `templates/.docs-gate.toml`, `/.docs-gate.toml`, `hooks/pre-commit`, install doc (1 trong 2: `docs/SETUP.md` / `INSTALL.md`)
- KHÔNG sửa: `~/docs-gate` binary repo, `phieu/phieu.sh`, agents, skills, `docs/HANDOFF.md`, `docs/PHILOSOPHY.md`
- KHÔNG redesign hook structure, không thêm check categories mới, không auto-init từ hook (giữ deterministic + side-effect-free)

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `docs-gate` binary có trên PATH | `which docs-gate` | ✅ `/home/sep/.cargo/bin/docs-gate` (CHALLENGE recon) |
| 2 | `docs-gate --version` = `0.1.0` | `docs-gate --version` | ✅ `docs-gate 0.1.0` (CHALLENGE recon) |
| 3 | `docs-gate init` subcommand tồn tại | `docs-gate --help` lists `init` | ✅ Commands include `check-discovery, serve, init, help` (CHALLENGE recon) |
| 4 | Sos-kit root chưa có `.docs-gate.toml` | `test -f /home/sep/sos-kit/.docs-gate.toml` | ✅ absent (recon) — phiếu task tạo mới |
| 5 | `templates/.docs-gate.toml` chưa tồn tại | `test -f /home/sep/sos-kit/templates/.docs-gate.toml` | ✅ absent (Glob: chỉ có `BACKLOG_template.md` + `claude-settings.local.json`) |
| 6 | `templates/` là thư mục dùng để chứa template (precedent: P037 ship `claude-settings.local.json` ở đây) | Glob `templates/*` | ✅ `BACKLOG_template.md`, `claude-settings.local.json` — convention rõ |
| 7 | `hooks/pre-commit` tồn tại | Glob `hooks/*` | ✅ `hooks/pre-commit` |
| 8 | `hooks/pre-commit` shell `docs-gate` ở đâu (line range) | Worker greps `grep -n "docs-gate" hooks/pre-commit` | `[needs Worker verify]` — Architect không peek source |
| 9 | Tổng số dòng `hooks/pre-commit` (anchor cho resilience edit) | `wc -l hooks/pre-commit` | `[needs Worker verify]` |
| 10 | `hooks/pre-commit` dùng `set -uo pipefail` (semantics quan trọng cho missing-file branch) | Worker greps `grep -n "set -" hooks/pre-commit` | `[needs Worker verify]` |
| 11 | Sos-kit có `CHANGELOG.md` ở root, KHÔNG ở `docs/` | Glob | ✅ `/home/sep/sos-kit/CHANGELOG.md` exists, `docs/CHANGELOG.md` absent |
| 12 | Sos-kit có `docs/discoveries/` directory cho per-phiếu Discovery Reports (P038 pattern) | Worker `test -d /home/sep/sos-kit/docs/discoveries` | `[needs Worker verify]` — nếu absent thì root `.docs-gate.toml` không reference được |
| 13 | Install instruction doc tồn tại | Glob | ✅ Cả `docs/SETUP.md` và `INSTALL.md` đều có. Worker đọc cả hai, chọn file documenting hook copy step |
| 14 | Working schema reference từ P001 cross-project | `cat /home/sep/media-rating-app/.docs-gate.toml` | `[needs Worker verify]` — file trên máy Sếp, Worker đọc để reuse schema, **KHÔNG re-derive** |
| 15 | `templates/` là canonical home cho template assets (xác nhận qua docs reference) | Worker greps `templates/` trong `CLAUDE.md` + `README.md` + `INSTALL.md` | `[needs Worker verify]` — sanity check trước khi đổ file vào |

**Anchor #8, #9, #10, #12, #14, #15 = `[needs Worker verify]`.** Architect không peek source per `feedback_architect_no_hallucination.md`. Tất cả là Worker grep-first ở EXECUTE.

### Pre-phiếu snapshot (Worker auto first-step)

```bash
PHIEU_ID=$(basename "$(git rev-parse --show-toplevel)" | grep -oE 'P[0-9]+')
mkdir -p ".backup/${PHIEU_ID}"
cp .claude/settings.local.json ".backup/${PHIEU_ID}/" 2>/dev/null || true
cp hooks/pre-commit ".backup/${PHIEU_ID}/" 2>/dev/null || true
git rev-parse HEAD > ".backup/${PHIEU_ID}/main-head.txt"
echo "✓ Snapshot at .backup/${PHIEU_ID}/"
```

Rollback: `cp .backup/${PHIEU_ID}/pre-commit hooks/` + `git reset --hard $(cat .backup/${PHIEU_ID}/main-head.txt)` (worktree only).

---

## Debate Log

> Tầng 2 surgical phiếu — per `feedback_challenge_selectivity_by_tier.md`, Worker MAY skip CHALLENGE và đi thẳng EXECUTE nếu Task 0 anchors verify clean. Nếu phát hiện anchor mismatch hoặc schema bịa lúc verify → CHALLENGE bình thường.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge

*(Skip nếu anchors verify clean. Nếu CHALLENGE: ghi objections theo schema chuẩn.)*

**Status:** ⏳ AWAITING WORKER (CHALLENGE optional cho Tầng 2)

### Final consensus
- Phiếu version: V<N>
- Total turns: <count>
- Approved by Chủ nhà: [date]

---

## Nhiệm vụ

### Task 1: Generate `templates/.docs-gate.toml` (reference template)

**File:** `templates/.docs-gate.toml` (NEW)

**Tìm:** N/A (file mới)

**Thay bằng / Thêm:**

```bash
# Worker chạy trong tmp dir để lấy real v0.1.0 schema:
TMPDIR=$(mktemp -d)
cd "$TMPDIR"
docs-gate init
cat .docs-gate.toml   # → đây là default v0.1.0 output
cd -
```

Sau đó so sánh với `/home/sep/media-rating-app/.docs-gate.toml` (Worker P001 đã hard-earned schema này) để curate sensible defaults cho **canonical sos-kit-style downstream project**:

- `docs_dir = "docs"`
- `changelog = "CHANGELOG.md"` (default theo precedent media-rating-app — root-level changelog) HOẶC `"docs/CHANGELOG.md"` nếu `docs-gate init` recommend như vậy. Worker ghi commented hint cho cả hai variant.
- `[architecture]` section — required docs that should exist (e.g. `docs/PROJECT.md`, `docs/SOUL.md`). Worker curate dựa trên VISION_TEMPLATES convention (`PROJECT_template.md`, `SOUL_template.md`, `CHARACTER_template.md` đã có trong `phieu/VISION_TEMPLATES/`).
- Header comment: `# Reference template for sos-kit-style projects. Copy to repo root as .docs-gate.toml, then tune. Generated from docs-gate v0.1.0 init.`
- Generic — **KHÔNG hardcode** path từ media-rating-app hay tarot.

**Lưu ý:**
- Schema = **flat-string v0.1.0** (`docs_dir`, `changelog`, `[architecture]`). KHÔNG dùng `[[trigger]]` table syntax (đã chứng minh bịa ở P001).
- Nếu `docs-gate init` output có field Worker không chắc nghĩa → giữ default + comment `# leave as-is unless you know what you're doing`.
- File này là REFERENCE; consumer downstream sẽ copy + tune cho repo của họ.

### Task 2: Generate sos-kit's own root `/.docs-gate.toml` (dogfood)

**File:** `/.docs-gate.toml` (NEW, root sos-kit)

**Tìm:** N/A (file mới)

**Thay bằng / Thêm:**

Tương tự Task 1 nhưng tuned cho sos-kit's own layout:
- `docs_dir = "docs"`
- `changelog = "CHANGELOG.md"` (verify anchor #11 — sos-kit changelog ở root)
- `[architecture]` — point đến sos-kit's foundation docs (`docs/LAYERS.md`, `docs/HANDOFF.md`, `docs/PHILOSOPHY.md`, `docs/SETUP.md`). Worker curate dựa trên existing sos-kit layout.
- Discovery Reports path: `docs/discoveries/P*.md` (verify anchor #12 — nếu directory chưa có Worker `mkdir -p docs/discoveries/` + commit empty `.gitkeep`).

**Lưu ý:**
- Mục tiêu: sos-kit dogfood chính hook của nó. Sau khi commit, `git commit` trên sos-kit phải continue working — **smoke test mandatory** (Task 5).
- Nếu schema bắt buộc một required doc mà sos-kit chưa có → relax constraint thay vì tạo doc giả. Phiếu này là install UX fix, không phải doc-coverage fix.

### Task 3: Edit `hooks/pre-commit` — config-existence guard

**File:** `hooks/pre-commit`

**Tìm:** Line(s) where script invokes `docs-gate` binary (anchor #8 — Worker greps `grep -n "docs-gate" hooks/pre-commit` để locate exact line range).

**Thay bằng / Thêm:** Wrap docs-gate invocation block với guard preamble. Pattern (Worker apply tại đúng vị trí):

```bash
# --- docs-gate (P006: graceful skip on fresh install) ---
if [ -f .docs-gate.toml ]; then
  # existing docs-gate invocation (whatever the current line(s) are)
  docs-gate <existing-args>
else
  # Yellow warning (ANSI 33) on stderr; exit 0 to keep hook flow alive
  printf '\033[33m[pre-commit] docs-gate skipped: no .docs-gate.toml — run `docs-gate init` to bootstrap.\033[0m\n' >&2
fi
# --- /docs-gate ---
```

**Lưu ý:**
- Giữ nguyên existing flag/args của `docs-gate` invocation — chỉ wrap, không rewrite.
- Hook đang dùng `set -uo pipefail` (anchor #10 verify). `if [ -f ... ]; then ... fi` an toàn với cả hai semantics.
- KHÔNG auto-run `docs-gate init` từ hook (deterministic + no surprise side-effect, theo decision đã chốt trong Context).
- Type-check + v2 BACKLOG/Discovery checks chạy **độc lập** với block này — đừng làm exit early.
- Nếu hook hiện shell `docs-gate` ở >1 chỗ (anchor #8 báo nhiều match) → wrap mỗi invocation cùng pattern, hoặc gom thành 1 block ở đầu rồi dùng flag biến `DOCS_GATE_OK=1` cho các block dưới. Worker chọn approach cleanest.

### Task 4: Update install doc — bootstrap step

**File:** `docs/SETUP.md` HOẶC `INSTALL.md` (Worker đọc cả hai, chọn file documenting `hooks/pre-commit` copy step)

**Tìm:** Section/step copy `hooks/pre-commit` vào `.git/hooks/` của downstream project.

**Thay bằng / Thêm:** Ngay sau step copy hook, thêm:

```markdown
### Bootstrap `docs-gate` config

The pre-commit hook invokes `docs-gate` to verify documentation hygiene. On a fresh repo, generate the config:

```bash
docs-gate init
```

Or copy the reference template:

```bash
cp <sos-kit>/templates/.docs-gate.toml .docs-gate.toml
# then tune docs_dir / changelog / [architecture] for your repo
```

If `.docs-gate.toml` is absent, the hook prints a yellow warning and skips the docs-gate check (other checks still run). No hard fail.
```

**Lưu ý:**
- Match existing doc voice (English public-facing per `CLAUDE.md` "Language" rule).
- Reference đúng path `templates/.docs-gate.toml` (relative tới sos-kit repo root).
- Document graceful-skip behavior một dòng để user mới hiểu warning là expected.

### Task 5: Smoke test (mandatory cho Tầng 2 + Risk #1)

**Không phải file edit — verification step.**

```bash
# 5a. Sos-kit's own hook chạy clean với .docs-gate.toml mới:
cd /home/sep/sos-kit
git add -A   # stage Task 1-4 changes
bash hooks/pre-commit   # phải pass (or fail only on legitimate doc issues)

# 5b. Graceful path — temporarily move config aside:
mv .docs-gate.toml /tmp/.docs-gate.toml.bak
bash hooks/pre-commit   # phải in yellow warning + exit 0 (or fail only on non-docs-gate checks)
mv /tmp/.docs-gate.toml.bak .docs-gate.toml

# 5c. Templates file syntactically valid (best-effort — docs-gate có thể không có --validate flag):
docs-gate --config templates/.docs-gate.toml check-discovery 2>&1 | head -20
# nếu binary error "command requires .docs-gate.toml in cwd" thì copy tạm vào tmp dir + chạy ở đó

# 5d. Restore + commit
git status   # confirm only intended files modified
```

**Pass criteria:**
- 5a: hook không fail vì `docs-gate` schema/config issue.
- 5b: hook in `[pre-commit] docs-gate skipped: ...` yellow + tiếp tục → exit 0 (assuming type-check + v2 checks pass).
- 5c: templates file parse-able (docs-gate không complain syntax).

**Nếu 5a fail:** root `.docs-gate.toml` mis-configured — chỉnh `[architecture]` required docs về set sos-kit thực có, đừng tạo placeholder docs.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `templates/.docs-gate.toml` | NEW — Task 1: reference template, generic, schema v0.1.0 flat-string |
| `.docs-gate.toml` | NEW — Task 2: sos-kit root dogfood config |
| `hooks/pre-commit` | Task 3: config-existence guard preamble around docs-gate invocation |
| `docs/SETUP.md` HOẶC `INSTALL.md` | Task 4: 1 paragraph + 1 command snippet for bootstrap step |
| `docs/discoveries/.gitkeep` | Conditional (anchor #12) — nếu directory chưa có |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `~/docs-gate` repo | KHÔNG động vào binary source; out-of-scope per BACKLOG "External" note |
| `phieu/phieu.sh` | Không liên quan; hook flow độc lập với phiếu shell function |
| `agents/architect.md`, `agents/worker.md`, `agents/orchestrator.md` | Không liên quan |
| `docs/PHILOSOPHY.md`, `docs/HANDOFF.md`, `docs/LAYERS.md` | Stable foundation docs; không chạm |
| `templates/BACKLOG_template.md`, `templates/claude-settings.local.json` | Đã ship trước; phiếu này chỉ thêm file thứ 3 |

---

## Luật chơi (Constraints)

1. **Tầng 2 surgical** — KHÔNG scope creep: không redesign hook, không thêm check categories, không reformat docs.
2. **Schema = flat-string v0.1.0** (`docs_dir`, `changelog`, `[architecture]`). KHÔNG dùng `[[trigger]]` table — đã verified bịa ở P001 incident.
3. **Hook stays deterministic + side-effect-free** — không auto-run `docs-gate init` từ pre-commit. Warning + skip only.
4. **Generic templates** — `templates/.docs-gate.toml` không hardcode path từ project nào; sos-kit root `.docs-gate.toml` chỉ tuned cho sos-kit.
5. **Don't break sos-kit's own commit flow** — smoke test 5a + 5b mandatory trước khi PR.
6. **Reuse Worker P001's hard-earned schema** từ `/home/sep/media-rating-app/.docs-gate.toml`. Đừng re-derive bằng đọc `~/docs-gate` source.
7. **English public-facing docs** (per `CLAUDE.md` Language rule) — install doc + template comments tiếng Anh. Phiếu prose Vietnamese OK (internal).

---

## Nghiệm thu

### Automated
- [ ] `bash hooks/pre-commit` exit 0 với `.docs-gate.toml` present (Task 5a)
- [ ] `bash hooks/pre-commit` exit 0 với `.docs-gate.toml` aside, prints yellow warning (Task 5b)
- [ ] `templates/.docs-gate.toml` parse-able by `docs-gate` binary (Task 5c)
- [ ] `git diff --name-only` chỉ list 4-5 files thuộc scope (no unintended changes)

### Manual Testing
- [ ] Stage một sample doc-only change (e.g. typo fix trong `docs/PHILOSOPHY.md`) + `git commit -m "test: P006 hook smoke"` → hook chạy, docs-gate check trên config mới của sos-kit pass. Revert commit sau test.
- [ ] Đọc lại `templates/.docs-gate.toml` — comments rõ, default sensible cho người clone fresh.

### Regression
- [ ] Type-check section trong `hooks/pre-commit` vẫn chạy (không bị ảnh hưởng bởi guard preamble).
- [ ] v2 BACKLOG/Discovery checks (nếu hook đang có) vẫn chạy độc lập.
- [ ] Existing 2 templates trong `templates/` (`BACKLOG_template.md`, `claude-settings.local.json`) không bị touch.
- [ ] Sos-kit's existing `git commit` flow (Sếp dùng hằng ngày) không broken.

### Docs Gate
- [ ] `CHANGELOG.md` — entry P006 dòng `### Fixed` hoặc `### Changed` (sos-kit ship at root CHANGELOG, anchor #11)
- [ ] `INSTALL.md` hoặc `docs/SETUP.md` — bootstrap step thêm vào (Task 4)
- [ ] `README.md` — nếu có Install table reference docs-gate, verify vẫn match. Nếu chưa có, không add (out of scope).

### Discovery Report
- [ ] Write `docs/discoveries/P006.md` (P038 per-phiếu pattern) gồm:
  - Anchors `[needs Worker verify]` resolved (#8, #9, #10, #12, #14, #15) — actual values + file:line citations
  - Schema notes: confirm v0.1.0 flat-string, list any field Worker thấy trong `docs-gate init` output mà phiếu chưa cover
  - Decision log: `templates/.docs-gate.toml` curate choices (vd. chọn `CHANGELOG.md` vs `docs/CHANGELOG.md` default)
  - Hook edit approach: 1 wrap vs N wraps (depend trên anchor #8)
  - Smoke test results (5a/5b/5c outputs)
  - Tier escalation: write "None" nếu phiếu giữ Tầng 2 từ DRAFT đến ship
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`
- [ ] PR opened on branch `fix/P006-docs-gate-bootstrap` referencing P006 + linking BACKLOG entry

---

## Risks (Worker heed during EXECUTE)

1. **Sos-kit root `.docs-gate.toml` mis-configured → blocks Sếp's future commits.** Mitigation: Task 5a + 5b smoke test mandatory; rollback bằng `cp .backup/P006/pre-commit hooks/` + `rm .docs-gate.toml`.
2. **`hooks/pre-commit` shell logic** — bash `set -uo pipefail` nghiêm. Test guard với `bash -n hooks/pre-commit` (syntax check) + execute cả 2 branch.
3. **`templates/` naming convention** — anchor #15. Worker confirm qua grep `templates/` trong CLAUDE.md/README.md/INSTALL.md trước khi đổ file vào, tránh tạo precedent sai.
4. **`docs-gate` v0.1.0 schema bịa** — đã có 1 incident (P001). Worker reuse `/home/sep/media-rating-app/.docs-gate.toml` làm source-of-truth, KHÔNG re-derive. Nếu thấy field lạ trong `docs-gate init` mà media-rating-app không có → hỏi qua Discovery Report, đừng silently add.
