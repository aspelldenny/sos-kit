# PHIẾU P078d1: Codex adapter startup-schema fixes (3 STARTUP-BLOCKERS)

---

> **Loại:** Bugfix
> **Ưu tiên:** P1
> **Tầng:** 1 — render là contract surface (adapter emit artifact mà runtime THẬT phải parse). Sai → untouched install KHÔNG khởi động = LAN tới mọi Codex user. AUTO Tầng 1.
> **Lane:** Guarded
> **Ảnh hưởng:** `crates/sos-adapter-codex/src/templates.rs` (3 content-fn: config.toml, rules, hooks.json) + test module.
> **Dependency:** P078b3 (Codex enforcement render) SHIPPED. Spec = `docs/adapters/P079-CODEX-DOGFOOD-FINDINGS-2026-07-22.md`.

---

## Context

### Vấn đề hiện tại

P079 live-dogfood chạy trọn một phiếu thật trên adapter Codex đã cài (Codex CLI **0.145.0**). Kiến trúc adapter **SOUND** (install 17 artifact, workflow DRAFT→…→DELIVERED, main-thread guard enforce). Nhưng dogfood tìm **7 bug thật**, trong đó **3 STARTUP-BLOCKER**: một install chưa đụng gì **không khởi động được** vì 3 render-fn của `templates.rs` emit sai **schema Codex 0.145.0** (đúng cú pháp TOML/JSON chung chung, sai shape Codex cần). Đây là 3 lỗi Codex báo lỗi rõ ràng + đưa fix chính xác — KHÔNG đoán.

3 bug (tất cả `crates/sos-adapter-codex/src/templates.rs`):

1. **#1 config.toml table-scope** (`templates.rs:258` `[needs Worker verify]`) — root settings `sandbox_mode` + `approval_policy` emit **SAU** một table header → theo luật TOML table-scope chúng bind vào table đó → Codex deserialize thành member của `AgentRoleToml`. Lỗi thật: `.codex/config.toml:8: invalid type string "workspace-write", expected struct AgentRoleToml`. **FIX:** emit `sandbox_mode` + `approval_policy` (mọi root key) **TRƯỚC table header ĐẦU TIÊN trong file** (kể cả `[mcp_servers.doctor]`, không chỉ `[agents]` — xem Turn 1 O1.1).
2. **#2 rules list-pattern** (`templates.rs:687` `[needs Worker verify]`) — `pattern` emit dạng **string**; Codex 0.145.0 cần **token LIST**. Lỗi thật: `pattern doesn't match, expected list, actual string`. **FIX:** emit list form, ví dụ `pattern = ["git", "push", "--force"]`.
3. **#3 hooks.json fields** (`templates.rs:291` `[needs Worker verify]`) — emit field `_provenance` + `_partial_note` không hợp lệ; Codex hooks schema chỉ nhận `description` / `hooks`. Lỗi thật: `unknown field _provenance, expected description or hooks`. **FIX:** gộp nội dung 2 note vào top-level `description` (string), bỏ 2 field custom.

### Giải pháp

Sửa **CHỈ format render** trong 3 content-fn của `templates.rs` (config.toml root-before-table, rules list-pattern, hooks.json fields) + **thêm per-file schema-shape parse test** bắt đúng 3 bug này. Additive: chỉ `templates.rs` + test module. KHÔNG đụng enforcement logic (marker lifecycle, approval gate, multi-path guard = P078d2).

**Oracle mấu chốt (V2 — cập nhật sau Turn 1 O1.2):**

- **config.toml** → parse bằng **`toml` crate THẬT** + assert root-key ở ROOT (không nested dưới bất kỳ table). (real-parse)
- **hooks.json** → parse bằng **`serde_json` THẬT** + assert top-level keys ⊆ {`description`,`hooks`}. (real-parse)
- **rules** → **structural-string / regex assert** (KHÔNG parse). Lý do: `.codex/rules/*.rules` là **STARLARK** (`docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:12`), KHÔNG phải TOML — `toml::from_str` trên `prefix_rule(pattern=...)` sẽ **parse ERROR cú pháp** (không phải schema), test toml-parse **bất khả thi viết**. Structural-string là oracle **yếu hơn parse** nhưng đủ bắt list-vs-bare-string; thêm Starlark-parser = Tầng-1 new-dep KHÔNG đáng. Ground-truth cuối = **live Codex 0.145.0** (đã note b2/b3-gap).

Đây là cái b2/b3 THIẾU: test cũ chỉ assert valid-TOML/JSON **chung chung** → PASS trong khi cả 3 blocker còn sống (bug là *cú pháp hợp lệ, sai schema Codex*). Test mới phải assert **shape Codex-cụ thể**, và **negative-test**: revert fix → test FAIL.

### Scope

- CHỈ sửa: `crates/sos-adapter-codex/src/templates.rs` — 3 content-fn (config.toml, rules, hooks.json) + test module cùng crate.
- KHÔNG sửa: enforcement logic (SubagentStart/Stop marker `templates.rs:302`, approval-gate `templates.rs:620`, multi-path guard `templates.rs:379/481/538/606`, spawn caveat) = **P078d2**. engine / install / core / adapter-claude / other content-fn = untouched.

---

## Task 0 — Verification Anchors

> Architect docs-only (no Bash/Grep/src read) — line số + code-shape từ P079 findings + Codex discovery report; Worker grep-verify TRƯỚC khi sửa. Codex-format facts = `[verified]` từ discovery + dogfood errors (spec đã đọc); code-site = `[needs Worker verify]`.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | config.toml content-fn ở `templates.rs:258` emit `sandbox_mode`/`approval_policy` SAU một table header `[needs Worker verify]` | `rg -n "sandbox_mode\|approval_policy\|\[agents\]\|\[mcp_servers" crates/sos-adapter-codex/src/templates.rs` → xác nhận thứ tự emit; xác nhận `[mcp_servers.doctor]` (`:259` per Turn 1) là table header ĐẦU TIÊN | ⏳ TO VERIFY |
| 2 | rules content-fn ở `templates.rs:687` emit `pattern` dạng string `[needs Worker verify]` | `rg -n "pattern" crates/sos-adapter-codex/src/templates.rs` → xác nhận string, không phải array | ⏳ TO VERIFY |
| 3 | hooks.json content-fn ở `templates.rs:291` emit `_provenance` + `_partial_note` `[needs Worker verify]` | `rg -n "_provenance\|_partial_note\|description" crates/sos-adapter-codex/src/templates.rs` | ⏳ TO VERIFY |
| 4 | Codex 0.145.0: root config keys phải TRƯỚC **table header ĐẦU TIÊN** (kể cả `[mcp_servers.doctor]`, không chỉ `[agents]`) `[verified: discovery §Codex-surfaces #4 + dogfood err "expected struct AgentRoleToml" + Turn 1 O1.1]` | Rendered `.codex/config.toml` → `toml::from_str::<toml::Value>` → `value.get("sandbox_mode").is_some()` ở ROOT (không nested dưới `mcp_servers`/`agents`) | ⏳ TO VERIFY |
| 5 | Codex 0.145.0: rules `pattern` = token list. **Oracle = structural-string** (rules là STARLARK `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:12`, KHÔNG toml-parse được) `[verified: dogfood err "expected list, actual string" + Turn 1 O1.2]` | Rendered `.codex/rules/*.rules` (String) → assert `content.contains("pattern = [")` (list-form) **AND** `!content.contains("pattern = \"")` (không còn bare-string form). KHÔNG parse Starlark, KHÔNG thêm dep | ⏳ TO VERIFY |
| 6 | Codex 0.145.0: hooks.json top-level chỉ nhận `description` / `hooks` `[verified: dogfood err "unknown field _provenance, expected description or hooks"]` | Rendered `.codex/hooks.json` → `serde_json` → top-level keys ⊆ {`description`,`hooks`} | ⏳ TO VERIFY |
| 7 | b2/b3 test hiện chỉ assert valid-TOML/JSON **chung chung**, KHÔNG assert schema-shape Codex-cụ thể `[needs Worker verify]` | `rg -n "from_str\|toml::\|serde_json\|assert\|contains" crates/sos-adapter-codex/src/templates.rs` (test module) → xác nhận thiếu root-key / list-pattern / field-subset assert | ⏳ TO VERIFY |
| 8 | Fix #1 chỉ đổi thứ tự emit (root-before-first-table), KHÔNG đổi giá trị/nội dung config `[needs Worker verify]` | Rendered output diff: cùng key/value, chỉ khác vị trí; các member table (`[mcp_servers.doctor]`, `[agents]`) vẫn nguyên | ⏳ TO VERIFY |

**Nếu Result có ❌ (ví dụ line số lệch, hoặc pattern đã là array sẵn) → Worker DISCOVERY_REPORT + điều chỉnh; nếu bug KHÔNG tồn tại như spec mô tả → dừng + báo (findings có thể đã partially-fixed).**

---

## Debate Log

**Phiếu version:** V2 (Turn 1 — Worker CHALLENGE + Architect RESPONSE, 2 ACCEPT)

### Turn 1 — Worker Challenge (phiếu V1)

Worker chạy CHALLENGE empirical (grep + real-parse trial), tìm 2 objection:

- **[O1.1] (self-closed, note-only):** Task 1 nói "emit root keys TRƯỚC `[agents]`" — nhưng `config_toml()` có `[mcp_servers.doctor]` (`:259`) là table header **ĐẦU TIÊN** trong file. Đặt root keys giữa `[mcp_servers.doctor]` và `[agents]` → vẫn nest dưới `[mcp_servers.doctor]` (sai). Test Task 4.1 (`value.get("sandbox_mode").is_some()` ở root) đã bắt được → **không block, chỉ sửa WORDING**.
- **[O1.2] BLOCKING:** Task 0 anchor #5 + Task 4.2 + Constraint #4 bắt parse `.codex/rules/*.rules` bằng `toml` crate. **NHƯNG `.rules` là STARLARK** (`docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md:12`), KHÔNG phải TOML. `toml::from_str` trên `prefix_rule(pattern=...)` → **parse ERROR cú pháp** (không phải schema). Test như spec **bất khả thi viết**.

**Status:** ⏳ AWAITING ARCHITECT RESPONSE

### Turn 1 — Architect Response (phiếu V2)

- **[O1.1] → ACCEPT (wording).** Worker đúng: `[mcp_servers.doctor]` (`:259`) là table header đầu tiên, "trước `[agents]`" chưa đủ. Sửa Task 1 + Context #1 + anchor #1/#4/#8: root keys emit **TRƯỚC table header ĐẦU TIÊN trong file** (trước cả `[mcp_servers.doctor]`). Test Task 4.1 (`.is_some()` ở root, không nested) **giữ nguyên — đã đúng, đã bắt được**.
- **[O1.2] → ACCEPT (blocking, đổi oracle).** Worker đúng: rules = Starlark, KHÔNG toml-parse được → test toml-parse bất khả thi. Đổi oracle cho rules:
  - **config.toml** → **`toml` crate THẬT** + root-key-ở-root assert. (GIỮ real-parse)
  - **hooks.json** → **`serde_json` THẬT** + top-level-keys-subset assert. (GIỮ real-parse)
  - **rules** → **structural-string / regex assert** (KHÔNG parse Starlark, KHÔNG thêm dep): assert content chứa `pattern = [` (list-form) VÀ KHÔNG chứa `pattern = "` (bare-string). Consistent repo style (hooks test hiện dùng `content.contains(...)`). Negative-test: revert #2 → bare-string xuất hiện → assert FAIL.
  - Ghi rõ: rules oracle = structural-string (**yếu hơn parse**) vì Starlark-parser = Tầng-1 new-dep không đáng; live-Codex 0.145.0 vẫn ground-truth cuối (b2/b3-gap note giữ).
- Amendments áp: Context #1 + Oracle-block; Task 1 wording; Task 4.2 + anchor #5 + Constraint #4 + Nghiệm thu oracle → rules string-structural. config.toml + hooks.json giữ real-parse. Negative-test giữ cho cả 3 (rules dùng string-assert).

**Status:** ✅ RESPONDED — phiếu bumped to V2. Recommend orchestrator spawn Worker CHALLENGE lại verify consensus (blocking O1.2 đã resolve, không DEFER).

### Turn 2 — Worker Challenge (phiếu V2)

**Worker accepted V2 — no further challenges.** Both amendments verified sound before EXECUTE:
- O1.1 wording fix confirmed applied (root keys now emit before `[mcp_servers.doctor]`, the first table header).
- O1.2 oracle swap confirmed sound: re-ran the empirical `toml::from_str` trial against the Starlark `prefix_rule(...)` syntax in a scratch cargo project — parse errors deterministically, confirming a real-parser oracle for `.rules` is impossible without a new dependency. Structural-string oracle (`pattern = [` present / `pattern = "` absent) accepted as the correct Tầng-2 implementation choice.

Ready for Chủ nhà approval gate → orchestrator delegated directly to EXECUTE (sprint delegation, marker `worker-active` set, branch `P078d-codex-dogfood-fixes`).

**Status:** ✅ ACCEPTED V2 — EXECUTED.

### Final consensus
- Phiếu version: V2
- Approved by Chủ nhà: 2026-07-22 (orchestrator-delegated EXECUTE)

---

## Nhiệm vụ

### Task 1: config.toml — emit root settings TRƯỚC table header ĐẦU TIÊN (#1)

**File:** `crates/sos-adapter-codex/src/templates.rs` (config.toml content-fn, quanh `:258` `[needs Worker verify]`)

**Tìm:** đoạn build nội dung `.codex/config.toml` nơi các root-level key (`sandbox_mode` / `approval_policy` / `web_search`) được emit **SAU** một table header (theo Turn 1 O1.1: table header đầu tiên là `[mcp_servers.doctor]` ở `:259` `[needs Worker verify]`, KHÔNG chỉ `[agents]`).

**Thay bằng / Thêm:** re-order — mọi **root-level key** (`sandbox_mode`, `approval_policy`, `web_search`, và bất kỳ root key khác) emit **TRƯỚC table header ĐẦU TIÊN trong file** (tức trước cả `[mcp_servers.doctor]`, trước MỌI `[table]`). Root keys ở đầu file. Giữ nguyên key/value — CHỈ đổi thứ tự.

**Lưu ý:** luật TOML — key sau một table header bind vào table đó (bất kỳ table nào, không riêng `[agents]`). Đây là gốc lỗi `expected struct AgentRoleToml`. **Chú ý bẫy Turn 1 O1.1:** đặt root keys giữa `[mcp_servers.doctor]` và `[agents]` → vẫn nest dưới `[mcp_servers.doctor]` = SAI. Phải ở đầu file. Nếu content ghép từ nhiều fragment string, fragment root-settings nối TRƯỚC mọi fragment table. KHÔNG đổi giá trị `workspace-write` / policy — chỉ vị trí (anchor #8).

### Task 2: rules — emit `pattern` dạng token LIST (#2)

**File:** `crates/sos-adapter-codex/src/templates.rs` (rules content-fn, quanh `:687` `[needs Worker verify]`)

**Tìm:** nơi emit `pattern = "<...>"` (string form) cho `.codex/rules/*.rules`.

**Thay bằng / Thêm:** list form — `pattern = ["git", "push", "--force"]` (mỗi token là một phần tử array, tách theo token của lệnh). Áp cho mọi rule mà content-fn phát ra.

**Lưu ý:** token-split phải khớp cách Codex 0.145.0 match prefix_rule (mỗi argv token một phần tử). Nếu pattern nguồn hiện là một chuỗi space-joined, split theo whitespace-token; nếu có token chứa space thật thì giữ nguyên phần tử đó — `[needs Worker verify]` cách source lưu pattern. Most-restrictive-wins semantic (discovery §6) KHÔNG đổi — chỉ format.

### Task 3: hooks.json — bỏ `_provenance`/`_partial_note`, gộp vào `description` (#3)

**File:** `crates/sos-adapter-codex/src/templates.rs` (hooks.json content-fn, quanh `:291` `[needs Worker verify]`)

**Tìm:** nơi object hooks.json set field `_provenance` và `_partial_note`.

**Thay bằng / Thêm:** bỏ 2 field custom; gộp nội dung 2 note thành một chuỗi và đặt vào top-level `description` (string). Top-level object chỉ còn `description` + `hooks`.

**Lưu ý:** nếu top-level đã có `description` sẵn thì nối provenance + partial-note vào cuối (giữ thông tin, không mất provenance trace). Field bên trong cây `hooks` (matcher/handler) KHÔNG đụng — chỉ top-level fields. Codex hooks schema (discovery §5) chỉ chấp `description`/`hooks` ở top-level.

### Task 4: per-file schema-shape parse tests + negative-test

**File:** `crates/sos-adapter-codex/src/templates.rs` (test module cùng crate `[needs Worker verify]` vị trí test — có thể là `#[cfg(test)] mod tests` trong file hoặc `tests/`).

**Thêm 3 test** (render → assert Codex-shape). **Oracle-mix V2 (Turn 1 O1.2): config.toml + hooks.json real-parse; rules structural-string:**

1. **config.toml (real-parse):** render → `toml::from_str::<toml::Value>(&s)` (must Ok) → assert `value.get("sandbox_mode").is_some()` **ở root** AND `value.get("approval_policy").is_some()` ở root (KHÔNG nằm dưới `value["mcp_servers"]` hoặc `value["agents"]`). Bắt #1.
2. **rules (structural-string — KHÔNG parse Starlark):** render → assert `content.contains("pattern = [")` (list-form present) AND `!content.contains("pattern = \"")` (bare-string form absent) cho mọi rule content phát ra. **KHÔNG** `toml::from_str` (rules là Starlark `CODEX_ADAPTER_DISCOVERY_2026-07-22.md:12` → toml-parse ERROR cú pháp). Bắt #2. *Oracle này yếu hơn parse — có chủ ý (xem Lưu ý).* 
3. **hooks.json (real-parse):** render → `serde_json::from_str::<serde_json::Value>(&s)` (must Ok) → assert top-level object keys ⊆ {`description`, `hooks`} AND `!contains_key("_provenance")` AND `!contains_key("_partial_note")`. Bắt #3.

**Negative-test (bắt buộc — chứng minh test có răng, cả 3):** trong Discovery Report, Worker ghi rõ đã verify: revert từng fix (tạm) → test tương ứng **FAIL**. Cụ thể rules: revert #2 → bare-string `pattern = "` xuất hiện → structural-string assert FAIL. Nếu revert mà test vẫn PASS → test vô dụng, phải siết assert.

**Lưu ý:** đây chính là cái b2/b3 THIẾU — test cũ assert valid-TOML/JSON chung chung nên câm với schema mismatch. Test mới assert SHAPE Codex-cụ thể. **Ghi chú trung thực (bắt buộc vào Discovery + docs):**
- config.toml + hooks.json shape-assert = **xấp xỉ hand-coded** schema Codex 0.145.0 — KHÔNG phải chính struct Codex.
- rules oracle = **structural-string, YẾU HƠN parse** (Starlark-parser = Tầng-1 new-dep không đáng). Chỉ bắt list-vs-bare-string, không validate ngữ nghĩa Starlark.
- Structural-valid + shape-assert PASS vẫn KHÔNG bảo chứng Codex-accepts; chỉ **live Codex 0.145.0** mới là ground-truth. (Bài học P079: b2/b3 oracle structural đã PASS trong khi 3 blocker còn sống.)

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `crates/sos-adapter-codex/src/templates.rs` | Task 1: config.toml root-before-first-table; Task 2: rules `pattern` list; Task 3: hooks.json drop `_provenance`/`_partial_note` → `description`; Task 4: 3 schema-shape test (config.toml+hooks.json real-parse, rules structural-string) + negative-test |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `crates/sos-adapter-codex/src/templates.rs` — enforcement content-fn (`:302` marker, `:620` approval, `:379/481/538/606` guard) | KHÔNG đụng — đó là P078d2. Render 3 file này KHÔNG được đổi hành vi hook/guard/approval |
| `crates/sos-install/**`, `crates/sos-core/**`, `crates/sos-adapter-claude/**` | Untouched — d1 additive, chỉ codex templates + test |

---

## Luật chơi (Constraints)

1. **CHỈ format render** — 3 fix là thứ tự-emit (#1), format-token (#2), field-set (#3). KHÔNG đụng enforcement logic (d2). Nếu phát hiện muốn sửa marker/approval/guard → DỪNG, đó là d2.
2. **Additive** — chỉ `templates.rs` (3 content-fn + test). Không đổi signature public, không đổi engine/install/core.
3. **Test phải có răng** — mỗi fix có một schema-shape assert bắt đúng bug; revert fix → test FAIL (negative-test verify, ghi Discovery).
4. **Oracle-mix (V2 — Turn 1 O1.2):** config.toml parse bằng **`toml` crate THẬT**, hooks.json parse bằng **`serde_json` THẬT** — KHÔNG assert 2 file này bằng string-contains thô. **rules là NGOẠI LỆ có chủ ý:** dùng **structural-string assert** (`content.contains("pattern = [")` + `!contains("pattern = \"")`), KHÔNG parse — vì `.rules` là **Starlark** (`CODEX_ADAPTER_DISCOVERY_2026-07-22.md:12`), toml-parse = ERROR cú pháp; thêm Starlark-parser = Tầng-1 new-dep không đáng. Ghi rõ oracle-yếu-hơn vào Discovery.
5. **Dựa trên Codex 0.145.0 errors thật** (findings), KHÔNG đoán. Nếu bug không khớp spec (đã partially-fixed / line lệch) → DISCOVERY + báo, không "sửa mò".
6. **Dependency-direction giữ** — adapter→core, không tạo core→adapter import.

---

## Nghiệm thu

### Automated
- [ ] `cargo check` clean
- [ ] `cargo test -p sos-adapter-codex` pass (gồm 3 test mới)
- [ ] Oracle (V2 oracle-mix): rendered `.codex/config.toml` parse bằng **`toml` crate** (Ok + root-key ở ROOT, không nested); `.codex/hooks.json` parse bằng **`serde_json`** (Ok + top-level keys ⊆ {description,hooks}); `.codex/rules/*.rules` **structural-string assert** (`pattern = [` present, `pattern = "` absent — KHÔNG parse, rules là Starlark)
- [ ] **Negative-test:** revert từng fix → test tương ứng FAIL (config.toml + hooks.json + rules; rules dùng string-assert). Ghi kết quả vào Discovery
- [ ] Flake gate: `cargo test -p sos-adapter-codex` ×20 → 0-flaky
- [ ] Dep-direction guard vẫn xanh (adapter→core)

### Manual Testing
- [ ] (Nếu có Codex 0.145.0 tay) render fresh install → `codex` khởi động KHÔNG lỗi `AgentRoleToml` / `expected list` / `unknown field _provenance` — đây là **ground-truth** vượt trên structural oracle (đặc biệt cho rules, nơi oracle chỉ structural-string)
- [ ] Rendered config.toml: root settings đứng TRƯỚC table header đầu tiên (`[mcp_servers.doctor]`); các member table nguyên vẹn

### Regression
- [ ] Enforcement render (marker/approval/guard content) KHÔNG đổi output (chỉ 3 startup-file đổi)
- [ ] Các artifact Codex khác (AGENTS.md, agents/*.toml, skills) render như cũ

### Docs Gate
- [ ] `CHANGELOG.md` — entry P078d1 (3 Codex startup-schema fix)
- [ ] `docs/discoveries/P078d1.md` — gồm **note structural-oracle-gap**: b2/b3 oracle structural KHÔNG đủ (đã PASS khi 3 blocker sống); chỉ live-Codex 0.145.0 bắt được schema mismatch; schema-shape assert mới = xấp xỉ hand-coded; **rules oracle = structural-string yếu hơn parse** (Starlark, không parse)
- [ ] `docs/adapters/` — nếu cần một format-note cho Codex render (config table-order / list-pattern / hooks field-set / rules=Starlark): thêm vào adapter MAPPING doc **nếu tồn tại** (`docs/adapters/codex/MAPPING.md` hoặc tương đương — `[needs Worker verify]` file tồn tại; nếu chưa có, ghi note vào `docs/adapters/CODEX_ADAPTER_DISCOVERY_2026-07-22.md` phần format hoặc tạo mục ngắn). KHÔNG bắt buộc tạo file mới nếu chưa có surface phù hợp — ghi "N/A, note để trong Discovery" nếu vậy

### Discovery Report
- [ ] Write to `docs/discoveries/P078d1.md`
  - Anchor #1–8 — CORRECT / WRONG (file:line thật cho 3 bug-site; xác nhận `[mcp_servers.doctor]` là table đầu tiên)
  - Negative-test kết quả (revert→fail cho từng fix; rules string-assert)
  - **structural-oracle-gap note** (bắt buộc — bài học P079) + **rules structural-string weaker-than-parse note** (Turn 1 O1.2)
  - Docs updated (hoặc "None"/"N/A" explicit cho adapter format-note)
  - Tier escalations (None expected — nếu chạm enforcement → escalate d2)
- [ ] Append 1-line index vào `docs/DISCOVERIES.md`
