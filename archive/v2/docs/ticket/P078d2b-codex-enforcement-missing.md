# PHIẾU P078d2b: Codex in-subagent enforcement — declare MISSING (#4, probe-confirmed) + deprecate dead SubagentStart-marker

---

> **Loại:** Docs + declaration (honesty fix — capability-absence explicit)
> **Ưu tiên:** P1 (last piece of P078d — closes the P079 #4 finding honestly; NOT a behavioral fix, Codex bug is upstream)
> **Tầng:** 1 — chạm security-boundary declaration (`verify()` FindingStatus = machine source-of-truth for capability honesty + SECURITY.md threat-model). Sai (khai SOUND/PARTIAL trong khi thực tế MISSING) → LAN tới mọi Codex adopter tin nhầm architect subagent tự giữ envelope. AUTO Tầng 1 dù không đổi runtime behavior.
> **Lane:** Guarded (no-cap). **Token budget khai trần: ~90k** (docs-heavy, no src read for Architect; Worker reads templates.rs guard/hook + lib.rs verify() + 1 test module). SECURITY-declaration → Worker CHALLENGE bắt buộc dù sprint-delegate.
> **Ảnh hưởng:** `crates/sos-adapter-codex/src/lib.rs` (`verify()` Finding set + count-assert test), `crates/sos-adapter-codex/src/templates.rs` (SubagentStart/Stop marker render — deprecation comment; AGENTS.md content-fn — orchestrator boundary-review guidance), `adapters/codex/CAPABILITY.md`, `SECURITY.md`, `docs/PORTABILITY_ARCHITECTURE.md`, `adapters/codex/MAPPING.md`, `CHANGELOG.md`.
> **Dependency:** P078d2a (multi-path guard + approval bootstrap + spawn caveat #5/#6/#7) SHIPPED. Spec chính = `docs/adapters/SUBAGENT-HOOK-PROBE-2026-07-22.md` (probe thật). Finding = `docs/adapters/P079-CODEX-DOGFOOD-FINDINGS-2026-07-22.md` #4. **Xong phiếu này = P078d DONE (d1 + d2a + d2b).**

---

## Context

### Vấn đề hiện tại

P079 live-dogfood (#4) thấy: architect subagent chạy `apply_patch` bị cấm (Rust src) VẪN thành công **bên trong** spawned agent; marker `.sos-state/architect-active` KHÔNG được tạo. Nghi ngờ ban đầu (d2a defer): SubagentStart-marker hook design hỏng. **Probe d2b (spec `SUBAGENT-HOOK-PROBE-2026-07-22.md`) đã xác định ROOT CAUSE dứt khoát:**

- Codex 0.145.0 dispatch `SubagentStart`/`SubagentStop` **CHỈ cho DEFAULT subagent** (`agent_type="default"`; matcher `"default"`/`"*"`/omitted đều fire — payload captured).
- **Custom-role spawn (architect/worker): KHÔNG hook nào fire** — không `"architect"`, không `"*"`, không omitted. Session metadata CÓ `agent_role:"architect"` nhưng hook không bao giờ chạy.
- Suy ra in-subagent `PreToolUse` cũng không đáng tin cho custom role (consistent với dogfood: architect forbidden `apply_patch` succeeded in-subagent).
- **Đây KHÔNG phải bug mình** — upstream Codex regression/gap: `github.com/openai/codex/issues/21753`.

**Kết luận doctrine (Sếp ratified — "khai MISSING"):** marker-via-SubagentStart-hook CHẾT trên 0.145.0. Cách honest DUY NHẤT = **khai in-subagent role-envelope enforcement = MISSING** (KHÔNG "sửa" cái Codex không cho), deprecate cơ chế đã chết, và tuyên bố rõ 2-3 cổng THẬT sự gánh: main-thread PreToolUse (dogfood ✅) + universal Git pre-commit/pre-push (agent-agnostic) + AGENTS.md guidance.

### Claude vs Codex (nội dung vào CAPABILITY.md)

- **Claude nhốt architect subagent 2 lớp cơ học, tại write-time:** (1) `tools:` frontmatter XOÁ tool per-role — architect KHÔNG có Bash/Edit; (2) PreToolUse guard fire **TRONG** subagent.
- **Codex 0.145.0 KHÔNG có cả 2:** (1) không per-role tool-allowlist (chỉ `sandbox_mode`, gap #1 PARTIAL); (2) custom-role lifecycle hook không fire (#21753) → in-subagent PreToolUse không tin được. → architect subagent = workspace-write + không guard canh → làm gì cũng được **bên trong** subagent. Enforce CHỈ ở boundary (main-thread + Git/CI).

### Giải pháp

Additive docs + declaration. KHÔNG đụng d2a guards (#5/#6), engine, core, adapter-claude. 5 việc:

1. **Deprecate SubagentStart/Stop-marker (GIỮ best-effort, KHÔNG bỏ).** Chốt của Architect: **GIỮ** render marker hook NHƯNG thêm comment loud "does NOT fire on Codex 0.145.0 custom-role subagents (#21753); best-effort only — if upstream fixes #21753 the marker activates. NOT relied upon by any capability claim." Lý do giữ chứ không bỏ: (a) additive/least-destructive khớp constraint P078; (b) nếu Codex fix #21753 thì cơ chế work ngay không cần re-render; (c) DEFAULT subagent marker VẪN fire (probe: default agent_type dispatch OK) nên render không hoàn toàn vô dụng. Chống misleading = declaration loud ở 3 surface (verify MISSING + CAPABILITY + SECURITY) + comment ngay tại render-site. **KHÔNG dựa vào marker cho bất kỳ capability status nào.**
2. **`verify()` → thêm Finding MISSING** "in-subagent role-envelope enforcement" (Codex 0.145.0, cite #21753). Đây là gap MỚI khai rõ, KHÔNG downgrade #1/#3/#5 (chúng PARTIAL nhờ **main-thread** hook + Git/CI VẪN đúng — chỉ làm rõ PARTIAL của chúng KHÔNG bao gồm in-subagent). Count-assert test 5→6.
3. **CAPABILITY.md** — section "In-subagent enforcement: MISSING (Codex 0.145.0)" + bảng Claude-vs-Codex + 3 backstop THẬT + cite #21753.
4. **SECURITY.md** — threat-model note: role-envelope KHÔNG enforce in-subagent trên Codex; boundary (Git/CI + main-thread) là gate thật. "đừng tin architect subagent tự giữ envelope; tin cổng Git".
5. **AGENTS.md render** — guidance: orchestrator (main thread) chịu trách nhiệm boundary review vì in-subagent guards không enforce; optionally orchestrator tự `touch`/`rm` marker quanh delegate (best-effort **main-thread only** — không cứu in-subagent).

### Scope

- **CHỈ sửa:** `lib.rs` verify() Finding set + count test; `templates.rs` marker render comment + AGENTS.md content-fn guidance; `CAPABILITY.md`; `SECURITY.md`; `PORTABILITY_ARCHITECTURE.md`; `MAPPING.md`; `CHANGELOG.md`.
- **KHÔNG sửa:** d2a guards (`architect/orchestrator/env/approval` content-fn — #5/#6 SHIPPED, KHÔNG regress), 3 startup render-fn (config.toml/rules/hooks.json = d1), engine/install-engine/core/adapter-claude. **KHÔNG "sửa" custom-role hook dispatch — Codex bug ngoài tầm, KHÔNG hack workaround giả-fire.**

---

## Task 0 — Verification Anchors

> Architect docs-only (no Bash/Grep/src read). Probe-fact + upstream issue = `[verified: SUBAGENT-HOOK-PROBE-2026-07-22.md]`. Code-site (`lib.rs`/`templates.rs`) = `[needs Worker verify]` (Architect không đọc src). Worker grep-verify TRƯỚC khi sửa.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | Codex 0.145.0 KHÔNG fire SubagentStart/Stop cho custom-role spawn (architect/worker); CHỈ fire cho `agent_type="default"`. Upstream `openai/codex#21753` `[verified: probe spec]` | Đọc `docs/adapters/SUBAGENT-HOOK-PROBE-2026-07-22.md` (probe captured payloads) — KHÔNG cần re-probe; nếu Worker có Codex 0.145.0 tay, optional re-confirm | ⏳ TO VERIFY |
| 2 | `verify()` hiện trả **5** `Finding` với `FindingStatus` (`Sound`/`Partial`/`Missing`) ở `crates/sos-adapter-codex/src/lib.rs` `[needs Worker verify]` (MAPPING.md line 31 nói "5 Findings") | `rg -n "Finding\|FindingStatus\|Missing\|Partial\|fn verify" crates/sos-adapter-codex/src/lib.rs` → đếm Finding + xem shape struct/enum | ⏳ TO VERIFY |
| 3 | Có test assert số Finding == 5 (count-oracle) trong `lib.rs` `#[cfg(test)]` `[needs Worker verify]` | `rg -n "verify\|Finding\|len() ==\|assert" crates/sos-adapter-codex/src/lib.rs` (test module) → tìm count assert | ⏳ TO VERIFY |
| 4 | SubagentStart/Stop marker render-site ở `crates/sos-adapter-codex/src/templates.rs` (~`:302` per d2a Task-0, `:315-317` per brief — drift `[needs Worker verify]`); marker hook trong `.codex/hooks.json` content-fn hoặc guard emit | `rg -n "SubagentStart\|SubagentStop\|architect-active\|worker-active\|agent_type" crates/sos-adapter-codex/src/templates.rs` → xác nhận render-site + matcher | ⏳ TO VERIFY |
| 5 | AGENTS.md content-fn = `agents_md()` `templates.rs:145-162`, có bullet-list orchestrator guidance (insertion point ~`:155-157`) `[needs Worker verify — d2a anchor #5 confirmed :145-162 nhưng d2a Task 3 đã chèn spawn caveat → line drift]` | `rg -n "fn agents_md\|orchestrator\|spawn\|delegate\|boundary" crates/sos-adapter-codex/src/templates.rs` → tìm bullet-list orchestrator section | ⏳ TO VERIFY |
| 6 | `CAPABILITY.md` seeded FROM `verify()` (separation-invariant #5: capability absence explicit); có 5 gap sections + "P078b3 enforcement status" + "Multi-path bypass CLOSED (d2a)" `[verified: đã đọc CAPABILITY.md]` | Đã đọc `adapters/codex/CAPABILITY.md` (5 gaps §1-5 + b3 status). Section MISSING mới thêm SAU §5, TRƯỚC/quanh enforcement-status | ✅ Confirmed (Architect read) |
| 7 | `SECURITY.md` có Invariants + trust-anchor; CHƯA có Codex-adapter threat-model note in-subagent `[verified: đã đọc SECURITY.md:1-60]` | Đã đọc `SECURITY.md` top; thêm Codex in-subagent note ở section threat-model (Worker xác nhận section phù hợp, `[needs Worker verify]` exact anchor cuối file) | ✅ Confirmed top; ⏳ exact insert section |
| 8 | `docs/PORTABILITY_ARCHITECTURE.md` có mô tả enforcement PARTIAL cho Codex (P078b) cần thêm "có MISSING (in-subagent)" `[needs Worker verify: exact section]` | `rg -n "PARTIAL\|enforce\|Codex\|in-subagent\|MISSING" docs/PORTABILITY_ARCHITECTURE.md` → tìm section enforcement để thêm MISSING note | ⏳ TO VERIFY |

**Nếu ❌ (verify() không phải 5 Finding / không có count test / marker render khác vị trí) → Worker DISCOVERY_REPORT + điều chỉnh anchor, KHÔNG "sửa mò".**
**Nếu anchor #1 sai (Worker có Codex 0.145.0 tay + custom-role hook FIRE) → ESCALATE NGAY: cả tiền-đề phiếu sập, có thể marker design cứu được → dừng, báo orchestrator, KHÔNG khai MISSING.**

---

## Debate Log

> Cap = 3 turns. SECURITY-declaration → Worker CHALLENGE bắt buộc dù sprint-delegate.

**Phiếu version:** V1 (initial draft)

---

### Turn 1 — Worker Challenge (phiếu V1)

**Anchor verification:**
- #1 ✅ `docs/adapters/SUBAGENT-HOOK-PROBE-2026-07-22.md` confirms: DEFAULT subagent `agent_type="default"` fires (payload captured); custom-role (architect/worker) fires NO SubagentStart/Stop at all. Cross-checked against `docs/adapters/P079-CODEX-DOGFOOD-FINDINGS-2026-07-22.md:26` (finding #4: "SubagentStart/Stop hooks NEVER created `.sos-state/architect-active`... architect's forbidden Rust apply_patch succeeded") — this is DIRECT dogfood evidence, not just inference from the probe.
- #2 ✅ `crates/sos-adapter-codex/src/lib.rs:116-171` `verify()` returns exactly 5 `Finding` (Partial/Missing/Partial/Missing/Partial), matches MAPPING.md line 31 "5 Findings".
- #3 ✅ count-oracle test at `lib.rs:200-215` `verify_reports_exactly_five_gaps_none_sound` asserts `findings.items.len() == 5` and no `Sound` status.
- #4 ⚠️ drift confirmed as anticipated: actual render-site is `templates.rs:335-341` (inside the `hooks_json()` JSON string literal, function starts `:320`), not `:302`/`:315-317` cited in brief/dogfood finding #4 (P079 finding cited `:302` — pre-P078d2a line numbers, now shifted). Site is a raw JSON string embedded in a Rust string — a Rust `//` comment directly above the `"SubagentStart"` block (consistent with existing style at `:320-325` for the same function) is the correct low-risk placement Task 2 already specifies.
- #5 ⚠️ drift confirmed: `agents_md()` at `templates.rs:164-179` (not `:145-162`); orchestrator/spawn-caveat guidance is at `:174-177` (P079 #7 spawn-caveat bullet), matching Task 0's own "post-d2a-drift" caveat. Insertion point = after line 177, before `Full role contract:` (line 179).
- #6/#7 ✅ Architect's own reads confirmed (CAPABILITY.md 5-gap sections + enforcement-weakness note at bottom; SECURITY.md not independently re-read by Worker but Architect's `[verified]` tag is docs-only claim, no code dependency — acceptable, no Tầng 1 risk).
- #8 ✅ confirmed: `docs/PORTABILITY_ARCHITECTURE.md:62` has the P078b3 status paragraph ("PARTIAL-honest 3-surface... `CAPABILITY.md`/`MAPPING.md`/`SECURITY.md` đều ghi rõ bypassable") — correct insertion point for a new P078d2b status line (style-consistent with existing b1/b2/b3/c status paragraphs at lines 58/60/62/64), before `## Ownership và dependency direction` (line 66).

**MISSING-accuracy check (critical, per brief):** Finding #6 "MISSING" is well-supported — not just probe-inference but direct dogfood proof (P079 #4: forbidden `apply_patch` actually succeeded inside the architect subagent). This is stronger than "PARTIAL" territory (PARTIAL implies a weaker-but-real enforcement path exists in-subagent; none does). **MISSING is correct, not an over-claim.**

Findings #1/#3/#5 (existing PARTIAL) do NOT need downgrading — their PARTIAL status is grounded in main-thread PreToolUse enforcement + Git/CI backstop, both of which remain real and dogfood-confirmed (P079 finding: "MAIN-THREAD guards ENFORCED: orchestrator-guard blocked src probe; block-env-edit blocked .env"). However their current prose (`lib.rs:127-132`, `:150-155`, `:159-166`) does not explicitly distinguish "main-thread" from "in-subagent" — Task 1's own instruction to tighten 1 clause per Finding if wording implies in-subagent coverage is warranted and should be applied (minor wording risk, not a blocking objection — Worker will resolve at EXECUTE time per Task 1 note).

**Deprecate-marker:** GIỮ + comment is not misleading — DEFAULT-subagent marker genuinely still fires (probe-confirmed), so the render is not fully dead code; the proposed comment explicitly scopes "fires for DEFAULT subagents only" and points to the 3 declared surfaces (verify/CAPABILITY/SECURITY) rather than asserting the marker itself proves anything. No false-confidence risk as drafted.

**3 backstops — real, not rhetorical:**
(a) Main-thread PreToolUse guards — dogfood-confirmed (P079 SOUND section, line 10).
(b) Git pre-commit/pre-push — confirmed present at `hooks/pre-commit`/`hooks/pre-push`, fires at the `git commit`/`git push` boundary regardless of which agent/runtime authored the diff (git-level, not Codex/Claude-specific) — genuinely agent-agnostic.
(c) AGENTS.md guidance — correctly declared guidance-only, no enforcement claim.

**Oracle:** count-assert test update (5→6) is a clean, sound structural oracle — same pattern as the existing `verify_reports_exactly_five_gaps_none_sound` test, extend or add a companion asserting the new `Missing` finding's content/status. Task 2's "Rust-comment-only, zero JSON byte change" default keeps d1's hooks.json parse-validity regression risk at zero; only if Worker chooses to touch the JSON `description` field does the d1 regression test need re-running (correctly flagged in the phiếu).

**Objections:** none — Tầng-1 premise (probe root cause, MISSING classification, additive-only scope, no core/adapter-claude touch) all check out against real code and real dogfood evidence.

**Worker accepted V1 — no challenges.** Ready for Chủ nhà approval gate.

---

### Turn 2 — Chủ nhà APPROVE + Worker EXECUTE (V1)

**Chủ nhà:** CHALLENGE APPROVE V1 → EXECUTE mode. Sprint delegated on branch `P078d2b-codex-enforcement-missing`.

**Worker EXECUTE result:** All 5 Nhiệm vụ shipped as spec'd (site drift at Task 2/Task 5 resolved as anticipated in Turn 1 — see `docs/discoveries/P078d2b.md`). `verify()` 5→6 Findings, count-test + content-test both green. Marker deprecation = Rust-comment-only, zero JSON byte change. CAPABILITY §6 / SECURITY note / AGENTS guidance / MAPPING all landed. `cargo build/test --workspace` green, 48/48 `sos-adapter-codex` tests, ×20 = 0 flaky, dep-direction green, additive verified (d2a guards + engine/core untouched), d1 render regression green, trust-gate clean (no rebaseline needed). **P078d = DONE (d1+d2a+d2b).** Full Discovery: `docs/discoveries/P078d2b.md`.

**Status:** ✅ SHIPPED (commit pending — see hand-back).

---

## Nhiệm vụ

### Task 1: `verify()` — thêm Finding MISSING "in-subagent role-envelope enforcement" (#4)

**File:** `crates/sos-adapter-codex/src/lib.rs` — `verify()` (`[needs Worker verify]` vị trí Finding set).

**Tìm:** hàm `verify()` trả về vec 5 `Finding` (gap #1-#5, mỗi cái có `FindingStatus`).

**Thay bằng / Thêm:** thêm **1 Finding MỚI** (thứ 6):
- Status = `Missing`.
- Nội dung: "in-subagent role-envelope enforcement (custom-role architect/worker) — Codex 0.145.0 does NOT fire SubagentStart/Stop or in-subagent PreToolUse for custom-role spawns (upstream `openai/codex#21753`); role envelope is NOT enforced inside spawned agents. Backstops: main-thread PreToolUse guards (dogfood-confirmed) + universal Git pre-commit/pre-push + AGENTS.md guidance."
- **KHÔNG downgrade** #1/#3/#5 (chúng PARTIAL nhờ main-thread hook + Git/CI — vẫn đúng). Nếu wording #1/#3/#5 hiện HÀM Ý in-subagent enforcement → chỉnh 1 câu để rõ "PARTIAL = main-thread + Git/CI, KHÔNG in-subagent" (minimal, `[needs Worker verify]` wording thật).

**Lưu ý:** giữ `FindingStatus` enum như hiện có (đừng thêm variant mới — dùng `Missing`). Đảm bảo Finding mới cite issue `#21753`. Đây là machine source-of-truth mà CAPABILITY.md seed từ đó — wording phải khớp CAPABILITY section (Task 3).

### Task 2: SubagentStart/Stop marker render — deprecation comment (GIỮ best-effort, #4)

**File:** `crates/sos-adapter-codex/src/templates.rs` — marker render-site (`[needs Worker verify]` `~:302`/`:315-317`).

**Tìm:** đoạn render SubagentStart/Stop hook tạo `.sos-state/architect-active`/`worker-active`.

**Thay bằng / Thêm:** GIỮ nguyên render output NHƯNG thêm comment (Rust `//` cạnh render-site + nếu có `description`/`_partial_note` field trong hook JSON thì cập nhật): "DEPRECATED best-effort: Codex 0.145.0 does NOT dispatch SubagentStart/Stop for custom-role (architect/worker) subagents — only agent_type=\"default\" (upstream openai/codex#21753). This marker fires for DEFAULT subagents only; NOT relied upon for any capability claim (see verify() Missing finding + CAPABILITY.md in-subagent section). Retained so it activates automatically if #21753 is fixed upstream."

**Lưu ý:** **KHÔNG đổi bytes render output** trừ khi thêm vào một field description hợp lệ (hooks.json schema d1 chỉ chấp `description`/`hooks` — nếu chèn note vào `description` phải giữ schema valid, KHÔNG tái tạo lỗi `_provenance` d1 #3). An toàn nhất: comment Rust cạnh render-fn, KHÔNG đụng JSON output → 0 risk regress d1 hook parse. Nếu đổi JSON `description` → phải chạy lại d1 hook-parse regression test. Chốt: **ưu tiên Rust-comment-only** (0 output change); chỉ đụng JSON nếu Worker thấy giá trị rõ + test hook-parse xanh.

### Task 3: CAPABILITY.md — section "In-subagent enforcement: MISSING"

**File:** `adapters/codex/CAPABILITY.md` — thêm section SAU gap §5 (`:66-86`), quanh/trước "P078b3 enforcement status".

**Thêm:** section mới:
- Tiêu đề: "## 6. In-subagent role-envelope enforcement (custom-role subagents) — MISSING".
- **What's missing:** Codex 0.145.0 không fire SubagentStart/Stop CHO custom-role spawn (chỉ `agent_type="default"`); in-subagent PreToolUse không tin được → architect/worker subagent workspace-write + không guard in-subagent → làm gì cũng được bên trong. Cite `openai/codex#21753`.
- **Bảng Claude vs Codex:**

| Lớp nhốt architect subagent | Claude | Codex 0.145.0 |
|---|---|---|
| Per-role tool-removal (write-time) | ✅ `tools:` frontmatter xoá Bash/Edit per-role | ❌ chỉ `sandbox_mode` (gap #1 PARTIAL) — không per-role allowlist |
| In-subagent PreToolUse guard (runtime) | ✅ guard fire TRONG subagent | ❌ custom-role hook không fire (#21753) |
| → Enforce point | write-time + in-subagent runtime | **CHỈ boundary** (main-thread + Git/CI) |

- **Backstops THẬT (rely on these, declare clearly):** (a) main-thread PreToolUse guards — dogfood-confirmed P079 (orchestrator-guard chặn src probe, block-env chặn `.env` trên main thread); (b) **universal Git pre-commit/pre-push** — agent-agnostic, gánh thật bất kể runtime; (c) AGENTS.md role guidance (guidance-only). **Status = MISSING, KHÔNG PARTIAL** — in-subagent thực sự không enforce, không simulate SOUND bằng prose.

**Lưu ý:** wording khớp Finding Task 1 (CAPABILITY seeded từ verify()). Giữ giọng doc hiện có (status + why + backstop). KHÔNG đổi 5 section cũ trừ khi #1/#3/#5 hàm ý sai in-subagent (Task 1 đồng bộ).

### Task 4: SECURITY.md — Codex in-subagent threat-model note

**File:** `SECURITY.md` — thêm threat-model note (`[needs Worker verify]` exact section — sau Invariants / trust-anchor phù hợp).

**Thêm:** một note ngắn: "**Codex adapter — in-subagent role-envelope NOT enforced (0.145.0):** trên Codex, architect/worker subagent chạy workspace-write và custom-role lifecycle/PreToolUse hooks KHÔNG fire (`openai/codex#21753`) → role envelope KHÔNG enforce **bên trong** spawned agent. Gate THẬT = boundary: universal Git pre-commit/pre-push + main-thread PreToolUse (dogfood-confirmed). **Đừng tin architect subagent tự giữ envelope trên Codex; tin cổng Git.** (Claude khác: `tools:` frontmatter + in-subagent guard nhốt tại write-time — xem `adapters/codex/CAPABILITY.md` §6.)"

**Lưu ý:** SECURITY.md là auto-exec-surface-doc → đổi nó KÍCH `trust-gate` INV-TRUST-01 rebaseline? KHÔNG — SECURITY.md là doc (không phải script/hook auto-exec); INV-TRUST-02 scan hidden-unicode CÓ áp SECURITY.md → giữ ASCII sạch, không zero-width. `[needs Worker verify]` trust-gate có require rebaseline khi SECURITY.md đổi (per CLAUDE.md scripts list trust-gate covers SECURITY.md content). Nếu có → chạy `scripts/trust-gate.sh rebaseline` + commit `.sos-trust-baseline` (ghi Discovery).

### Task 5: AGENTS.md render — orchestrator boundary-review guidance (#4)

**File:** `crates/sos-adapter-codex/src/templates.rs` — `agents_md()` (`~:145-162`, `[needs Worker verify]` post-d2a-drift).

**Tìm:** phần AGENTS.md orchestrator/delegation guidance (nơi d2a Task 3 đã chèn spawn caveat).

**Thêm:** một dòng guidance orchestrator-facing: "On Codex, in-subagent role guards do NOT enforce (custom-role hooks don't fire, openai/codex#21753) — the orchestrator (main thread) is responsible for boundary review of subagent-authored diffs before merge; the Git pre-commit/pre-push gate is the real backstop. Optionally the orchestrator may `touch .sos-state/<role>-active` before delegating and `rm` after (best-effort, main-thread only — does NOT enforce inside the spawned agent)."

**Lưu ý:** doc-string trong render, KHÔNG đụng hook/guard logic. Giữ AGENTS.md dưới 32KiB (discovery limit). Wording khớp probe #21753. KHÔNG mâu thuẫn spawn-caveat d2a Task 3 (bổ sung, không ghi đè).

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-adapter-codex/src/lib.rs` | Task 1: thêm Finding MISSING "in-subagent enforcement" (#4, cite #21753); count-assert test 5→6 |
| `crates/sos-adapter-codex/src/templates.rs` | Task 2: deprecation comment cạnh SubagentStart/Stop marker render (GIỮ output, Rust-comment-only ưu tiên); Task 5: AGENTS.md orchestrator boundary-review guidance |
| `adapters/codex/CAPABILITY.md` | Task 3: section §6 "In-subagent enforcement: MISSING" + bảng Claude-vs-Codex + 3 backstop |
| `SECURITY.md` | Task 4: Codex in-subagent threat-model note ("tin cổng Git") |
| `docs/PORTABILITY_ARCHITECTURE.md` | enforcement section: PARTIAL → thêm "có MISSING (in-subagent, #21753)" `[needs Worker verify: exact section]` |
| `adapters/codex/MAPPING.md` | cập nhật "5 Findings" → 6 (line ~31) + note in-subagent MISSING row nếu phù hợp |
| `CHANGELOG.md` | entry P078d2b |
| `docs/discoveries/P078d2b.md` | Discovery Report (mới) |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `crates/sos-adapter-codex/src/templates.rs` — d2a guards (architect/orchestrator/env/approval content-fn) | #5/#6 SHIPPED — KHÔNG regress output/logic |
| `crates/sos-adapter-codex/src/templates.rs` — 3 startup render-fn (config.toml/rules/hooks.json) | d1 SHIPPED — KHÔNG regress. Task 2 nếu đụng hooks.json `description` PHẢI giữ parse-valid |
| `crates/sos-install/**`, `crates/sos-core/**`, `crates/sos-adapter-claude/**` | Untouched |
| Custom-role hook dispatch (Codex runtime) | Bug upstream #21753 — KHÔNG hack giả-fire, KHÔNG workaround. Chỉ KHAI MISSING |

---

## Luật chơi (Constraints)

1. **KHAI MISSING, KHÔNG SIMULATE.** In-subagent enforcement = `Missing`, không PARTIAL/SOUND-bằng-prose. Separation-invariant #5: capability absence phải explicit ở machine (`verify()`) + human (CAPABILITY/SECURITY).
2. **KHÔNG "sửa" Codex bug.** #21753 upstream — KHÔNG hack marker giả-fire, KHÔNG workaround custom-role dispatch. Phiếu này = declaration + deprecate + guidance, KHÔNG behavioral fix.
3. **Marker GIỮ best-effort, KHÔNG bỏ.** Render output KHÔNG đổi (ưu tiên Rust-comment-only); marker KHÔNG được dựa vào cho bất kỳ capability status. Nếu Architect-sau/Worker thấy giữ = misleading dù đã comment → escalate Chủ nhà, KHÔNG tự bỏ.
4. **Additive, dep-direction giữ** — adapter→core, no core→adapter import. KHÔNG đổi `FindingStatus` enum shape (dùng variant `Missing` có sẵn). KHÔNG đổi public signature.
5. **KHÔNG regress d1/d2a** — 3 startup file + d2a guards render/behavior giữ nguyên (regression test xanh). Nếu Task 2 đụng hooks.json → chạy hook-parse test.
6. **Dựa trên probe THẬT** (`SUBAGENT-HOOK-PROBE-2026-07-22.md`) + issue #21753 — KHÔNG đoán. Nếu Worker re-probe thấy custom-role hook FIRE → ESCALATE (tiền-đề sập), KHÔNG khai MISSING.

---

## Nghiệm thu

### Automated
- [ ] `cargo check` clean
- [ ] `cargo test -p sos-adapter-codex` pass (gồm count-assert 5→6 + verify() Missing finding present)
- [ ] **Oracle STRUCTURAL/declaration:** test assert `verify()` trả Finding với `FindingStatus::Missing` cho "in-subagent enforcement" + tổng Finding == 6; assert CAPABILITY/SECURITY chứa declaration nếu có content-test (hoặc verify tay); render 3 startup + marker vẫn parse valid (d1 regression) — config.toml/rules/hooks.json/marker output KHÔNG đổi bytes (trừ comment nếu Rust-only)
- [ ] **KHÔNG behavioral** (Codex #21753 ngoài tầm — KHÔNG test custom-role hook fire)
- [ ] Flake gate: `cargo test -p sos-adapter-codex` ×20 → 0-flaky
- [ ] Dep-direction guard xanh (adapter→core)

### Manual Testing
- [ ] (optional, nếu có Codex 0.145.0 tay) re-confirm probe: custom-role spawn KHÔNG fire SubagentStart/Stop — nếu FIRE → ESCALATE (không khai MISSING)
- [ ] CAPABILITY.md §6 render đọc được: bảng Claude-vs-Codex + 3 backstop + cite #21753
- [ ] AGENTS.md render chứa orchestrator boundary-review guidance, dưới 32KiB

### Regression
- [ ] 3 startup-file (config.toml/rules/hooks.json — d1) render output KHÔNG đổi (trừ optional `description` note, phải parse valid)
- [ ] d2a guards (multi-path #6 + approval bootstrap #5) render/behavior KHÔNG đổi
- [ ] SubagentStart/Stop marker render output KHÔNG đổi (Rust-comment-only)

### Docs Gate
- [ ] `adapters/codex/CAPABILITY.md` — §6 In-subagent MISSING + bảng + backstop (seeded từ verify())
- [ ] `adapters/codex/MAPPING.md` — "5 Findings" → 6 + in-subagent row nếu phù hợp
- [ ] `SECURITY.md` — Codex in-subagent threat-model note ("tin cổng Git"); nếu trust-gate cover SECURITY.md → `scripts/trust-gate.sh rebaseline` + commit `.sos-trust-baseline` (`[needs Worker verify]`)
- [ ] `docs/PORTABILITY_ARCHITECTURE.md` — enforcement PARTIAL → thêm MISSING (in-subagent, #21753)
- [ ] `CHANGELOG.md` — entry P078d2b
- [ ] `docs/discoveries/P078d2b.md`

### Discovery Report
- [ ] Write to `docs/discoveries/P078d2b.md`
  - Anchor #1–8 — CORRECT / WRONG (file:line thật cho verify() Finding set + count test + marker render-site + agents_md; số Finding thật; PORTABILITY enforcement section)
  - **Chốt deprecate-marker:** GIỮ best-effort + comment (Rust-only vs JSON-description) — ghi lựa chọn thật
  - **#4 handoff CLOSED:** probe #21753 xác định root cause; marker-via-hook chết cho custom-role; khai MISSING đã hạ 3 surface (verify/CAPABILITY/SECURITY). P078d (d1+d2a+d2b) = DONE
  - trust-gate SECURITY.md rebaseline — có chạy không, tại sao
  - Tier escalations (None expected — nếu re-probe thấy hook FIRE → escalate, tiền-đề sập)
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`
