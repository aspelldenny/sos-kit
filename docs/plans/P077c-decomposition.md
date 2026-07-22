# P077c decomposition — Rust `new/adopt/map/sync` → parity + OA-02 fix

> **Status:** P077c1 SHIPPED (2026-07-22, branch `P077c-rust-parity-impl`, not yet merged). `docs/ticket/P077c1-rust-map-parity.md` (V2 — file-output parity gap ACCEPTED Turn 1).
> **Status:** P077c2 SHIPPED (2026-07-22, branch `P077c2-rust-sync-parity`, not yet merged). `docs/ticket/P077c2-rust-sync-parity.md` (V1 accepted, no revision — CHALLENGE found `sync.golden` re-froze from real-HEAD-pin 0-change to a synthetic self-contained fake-kit fixture exercising all 4 outcomes; new `sync.tree.golden` two-fixture oracle; Bash's unsorted spine `find` replicated bug-for-bug in Rust, NOT sorted like `map`). Feed-forward for c3/c4: reuse the synthetic-self-contained-fixture pattern whenever a command's golden would otherwise depend on real repo/kit state that isn't the command's own fixture.
> **Status:** P077c3 SHIPPED (2026-07-22, branch `P077c3-rust-new-parity`, not yet merged). `docs/ticket/P077c3-rust-new-parity.md` (V1 accepted, no revision — CHALLENGE live-probed all 3 escape hatches, none triggered). Shipped Rust `new` + a **3-layer fixture** (`new.golden` stdout, `new.tree.golden` path-shape, `new.gen.golden` gen-content-hash), synthetic fake-kit SIMPLER than c2's (plain dir tree, no git needed — `new` only copies verbatim). **Critical finding**: the pre-existing `new.golden` had been captured with a real `doctor` on the machine's PATH (CONNECTED-path artifact, not a design choice) — re-froze with `DOCTOR_BIN=/nonexistent/doctor` forcing the deterministic skip branch. Two new normalization gotchas found+fixed: (1) full ISO-8601 timestamps in `.sos-stack.toml` need `strip_timestamp` BEFORE the bare-date rule (order bug, half-stripped `<DATE>T14:23:01Z` otherwise); (2) locale-dependent `sort` mismatches Rust's byte-order sort (`claude` vs `INVARIANTS` ordering) — pinned `LC_ALL=C sort` on both `capture.sh` sort sites. Feed-forward for c4: reuse the 3-layer split (tree-shape for copied/path-only, content-hash for GENERATED-authored only) — `adopt` also writes files as its real product; watch for the same timestamp/locale-sort gotchas if `adopt` also touches `.sos-stack.toml` or similar generated content.

## ⚠️ Feed-forward note for c2/c3/c4 — two-fixture oracle (from P077c1)

P077c1's Worker CHALLENGE (Debate Log Turn 1, V1→V2) found `map.golden` (the stdout
freeze) contained NO surface data — `sos_map`'s real work-product is a **file**
(`<target>/docs/AGENT_MAP.yaml`), not stdout; the original stdout-only parity oracle
was blind to scan-correctness. Fixed additively for `map`: `capture.sh` now ALSO
freezes the file content (`map.agent_map.golden`), and the harness hard-fails on
either mismatch (see `crates/sos-cli/tests/README.md` "two-fixture oracle" section).

**c3 (`new`) and c4 (`adopt`) have the SAME shape of gap** — their real work-product
is the files they generate/write, and their current goldens (`new.golden`,
`adopt.golden`) only freeze stdout. Before flipping `new`/`adopt` into
`PARITY_ENFORCED`, c3/c4 should check whether their stdout also fully echoes the
generated content (if so, stdout-only may be sufficient) or whether — like `map` —
a second file-content fixture is needed to avoid the same false-green class. This is
a **check to make**, not a fix already required; do not assume either way without
looking.
> **Parent:** P077c row trong `docs/plans/P077-decomposition.md` — "Impl các Rust command còn thiếu tới parity: `new/adopt/map/sync` + `init security`. Parity-harness → hard-fail on diff. Sửa OA-02."
> **Depends:** P077a (golden oracle + informational harness), P077b (crate boundary) — both landed (`docs/discoveries/P077a.md`, `P077b.md`).

## ⚠️ Feed-forward (P077c1 V2 Debate Log Turn 1) — golden phải freeze WORK-PRODUCT, không chỉ stdout

Worker CHALLENGE trên c1 phát hiện: `map.golden` = **1 dòng stdout confirmation** (123b), nhưng work-product THẬT của `sos map` là **file** `<target>/docs/AGENT_MAP.yaml` (surface data). `capture.sh` freeze stdout-only → parity oracle **mù với scan-correctness** (false-green class OA-02: oracle chống-drift tự drift). c1 fix bằng **two-fixture**: thêm `map.agent_map.golden` (freeze file-content) + harness diff cả stdout LẪN file, hard-fail cả hai.

**Áp cho các sub-phiếu sau — kiểm tra "golden freeze work-product hay chỉ stdout?":**
- **c3 (new):** work-product = **files sinh ra** (greenfield gen). `new.golden` (1.8k) có thể là stdout summary → cần fixture freeze **nội dung file được gen** (hoặc tree listing + key file contents), diff trong harness. KHÔNG chỉ trust stdout "created N files".
- **c4 (adopt):** work-product = **file writes onboarding** (AGENT_MAP.yaml qua map bug-for-bug từ c1 + hooks/guards wired + non-clobber merges). `adopt.golden` (4.7k) stdout-only sẽ bỏ lọt các file-mutation THẬT → cần file-output fixtures (AGENT_MAP.yaml đã có pattern từ c1; thêm hook-install + conflict-merge results). adopt kế thừa c1 map file-fixture — tái dùng pattern.
- **c2 (sync):** provenance/list output — nếu sync ghi file (provenance record) thì cũng cần file fixture; nếu thuần stdout list thì stdout golden đủ. Worker verify ở c2 EXECUTE.
- **c5 (OA-02):** correctness fixtures Rust-only vốn đã là file-content (crate map `src/main.rs`, templates excluded) — pattern two-fixture của c1 áp thẳng.

**Nguyên tắc chung:** mỗi sub-phiếu, hỏi "golden có freeze work-product THẬT chưa, hay chỉ freeze confirmation stdout?" Nếu command mutate filesystem → BẮT BUỘC file-output fixture, không stdout-only. Đây là containment đúng: c1 fix c1, feed-forward c2–c5 tự verify khi tới lượt (KHÔNG phình scope c1).

## Tại sao P077c phải chia tiếp

P077c gộp: (1) impl 4 Rust command tới parity với Bash golden, (2) flip parity-harness informational → hard-fail, (3) fix OA-02 (scanner stack-aware + survey-before-install + 3-verdict). Nhồi cả 3 vào một delivery unit vi phạm **one-ticket-one-delivery** và, quan trọng hơn, **trộn hai oracle mâu thuẫn nhau** (xem finding dưới). `adopt.golden` một mình đã 4.7k (fixture lớn nhất); cộng OA-02 scanner-rewrite = khối un-CHALLENGE-able, phá lane budget.

### ⚠️ Finding cốt lõi — parity oracle vs OA-02 oracle MÂU THUẪN

- **Parity oracle** (P077c mandate): `Rust <cmd> output == Bash golden fixture` (frozen P077a), harness hard-fail on diff.
- **OA-02 fix** (`bin/sos.sh` map bug): scanner phải stack-aware (map `src/*.rs`), survey-before-install (exclude managed kit assets), emit 3-verdict (`PATH_VALID`/`COVERAGE_UNKNOWN`/`COVERAGE_REVIEWED`).
- **Constraint bất di:** `bin/sos.sh` KHÔNG đổi (additive). Nên Bash **giữ nguyên bug OA-02**.

Hệ quả: nếu Rust map fix OA-02, Rust output **cố ý khác** Bash golden (Bash vẫn buggy) → **parity FAIL by design**. Cả 3 nhánh OA-02 đều làm lệch stdout: exclude-assets đổi surface set · stack-aware thêm `src/*.rs` surface · 3-verdict thêm field. Không thể vừa "== Bash golden" vừa "beat Bash" trên cùng command khi Bash frozen buggy.

**Resolution:** tách OA-02 thành sub-phiếu RIÊNG (P077c5) với **oracle khác**: KHÔNG phải "== Bash golden" mà là **correctness fixtures Rust-only** (Rust crate map `src/main.rs`; kit `templates/` bị exclude; 3-verdict emit đúng) — đúng OA-02 upgrade-direction §5. Mỗi phiếu parity giữ **một oracle nhất quán**; phiếu OA-02 giữ oracle correctness riêng. Không phiếu nào có gate tự-mâu-thuẫn.

## Sub-phiếu (5) — thứ tự cứng

| ID | Deliverable | Oracle | Harness flip | Lane | Dep |
|---|---|---|---|---|---|
| **P077c1** | Rust `map` → parity (bug-for-bug, KHÔNG OA-02) + build real comparison trong harness (**stdout + file-content two-fixture**) + flip HARD_FAIL từ single-bool → **per-command set**, thêm `map` vào set | `Rust map stdout == map.golden` (123b, 1-dòng) **VÀ** `Rust AGENT_MAP.yaml == map.agent_map.golden` (mới) | `map` → hard-fail (cả 2 assert); new/adopt/sync giữ informational | Guarded | P077a,b |
| **P077c2** | Rust `sync` → parity (sync provenance, OA-06 list) — verify work-product (stdout + file nếu sync ghi provenance record) | `Rust sync == sync.golden` (439b) [+ file fixture nếu mutate fs] | +`sync` hard-fail | Guarded | P077c1 (harness per-cmd) |
| **P077c3** | Rust `new` → parity (greenfield gen, idempotence/non-clobber) — **file-output fixture** (nội dung file được gen, KHÔNG chỉ stdout) | `Rust new == new.golden` (1.8k) + gen-file-content fixture | +`new` hard-fail | Guarded | P077c1 |
| **P077c4** | Rust `adopt` + `init security` → parity (non-clobber conflict, hook collision, rollback, dry-run plan). adopt gọi **bug-for-bug map từ c1**; tái dùng file-fixture pattern (AGENT_MAP.yaml + hook-install + merge results) | `Rust adopt == adopt.golden` (4.7k, nặng nhất) + **file-output fixtures** | +`adopt` hard-fail | Guarded | P077c1 (map), c3 |
| **P077c5** | **OA-02 fix** trên map+adopt: scanner stack-aware (Rust/Py/Node/Go/Swift), survey-before-install (exclude managed assets per manifest), 3-verdict. Cố ý diverge khỏi buggy Bash | **Correctness fixtures Rust-only** (NOT Bash golden): Rust crate map `src/main.rs`; kit `templates/` excluded; verdict đúng | map/adopt golden **re-froze/mở rộng** sang correctness set | Guarded | P077c1, c4 |

### Thứ tự + lý do

`c1(map) → c2(sync) → c3(new) → c4(adopt+init-sec) → c5(OA-02)`:

1. **c1 map trước** — nhỏ nhất (123b golden) → pilot rẻ để dựng **real comparison** trong harness + refactor HARD_FAIL single-bool → per-command set (P077a để lại `HARD_FAIL` là 1 const; per-command flip là design point của c1). c1 cũng dựng **two-fixture pattern** (stdout+file) mà c3/c4 kế thừa. `adopt` (c4) **gọi map** nên Rust map bug-for-bug phải có trước.
2. **c2 sync / c3 new** — độc lập, parity thuần, không phụ thuộc adopt. Làm sau c1 vì c1 đã dựng cơ chế per-command flip + file-fixture cho chúng dùng lại.
3. **c4 adopt + init security** — nặng nhất; gọi map bug-for-bug từ c1. `init security` gộp đây vì adopt = brownfield onboarding, security-wiring (hooks/guards) đậm nhất ở bước này.
4. **c5 OA-02 CUỐI** — chỉ khi map (c1) + adopt (c4) đã parity. c5 cố ý diverge → phải re-froze/mở rộng golden của map+adopt sang correctness fixtures. Đặt cuối để không phá parity gate của c1/c4.

## Bất biến xuyên suốt

- **Bash `bin/sos.sh` GIỮ canonical + KHÔNG đổi** — Rust chứng minh bằng nhau (c1–c4) hoặc tốt hơn có chủ đích (c5). Chưa switch entrypoint (P077e).
- Additive: user vẫn dùng `bin/sos.sh`; không đổi `CLAUDE.md` repo contract (P077e).
- Tất cả Tầng 1, Lane Guarded (impl + parity = móng, sai thì LAN).
- **Golden freeze work-product THẬT** — command mutate filesystem → file-output fixture bắt buộc, không stdout-only (xem Feed-forward section trên).
- DOCS-GATE mỗi phiếu: `bootstrap/sos-rs/README.md` command-status + `docs/plans/P077c-decomposition.md` (đánh dấu sub-phiếu done) + CHANGELOG. c5 thêm `docs/retro/OUTSIDER_AUDIT_SYSTEM_GAPS_2026-07-22.md` OA-02 (mark addressed) + AGENT_MAP semantics doc.

## Ngoài scope P077c (đừng kéo vào)

- Install framework / manifest / rollback-record / doctor = **P077d**.
- Cutover canonical + flip repo contract = **P077e**.
- OA-01 (Lane↔Tầng), OA-03 (ship dirty), OA-04 (docs-sync config), OA-05 (runtime-neutral adopt), OA-07 (version pin) — upgrade order riêng, không block P077c.
