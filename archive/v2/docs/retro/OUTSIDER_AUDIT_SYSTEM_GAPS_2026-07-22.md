# OUTSIDER AUDIT — SOS Kit system gaps (2026-07-22)

> **Mục đích:** ghi lại các khoảng hở được phát hiện khi đọc SOS Kit như một người lạ,
> không chỉ kiểm phần phù hợp với một dự án cụ thể. Đây là backlog evidence để nâng cấp
> sau; tài liệu này **không sửa implementation** và không tuyên bố các finding đã được fix.
>
> **Phạm vi đã kiểm:** doctrine/core, role prompts, phiếu, `sos new/adopt/map/sync`,
> Claude hooks/guards, docs-gate, doctor, ship, installer và Rust `sos-rs` skeleton.
>
> **Quy ước severity:** 🔴 correctness/false-green · 🟠 workflow safety ·
> 🟡 portability/ergonomics · 🟢 cơ chế đã hoạt động đúng cần giữ.

---

## Triage cover

| ID | Sev | Finding | Hướng xử lý đề nghị |
|---|---|---|---|
| OA-01 | 🔴 | `doctor lane-check` yêu cầu `Lane`, template và routing hiện dùng `Tầng` | Chọn một taxonomy canonical hoặc định nghĩa mapping; thêm end-to-end contract test |
| OA-02 | 🔴 → ✅ Rust-only (P077c5, 2026-07-22) | `sos map` có thể map chính asset của kit nhưng bỏ sót source thật; `validate-map` vẫn PASS | FIXED trong `bootstrap/sos-rs` (`map.rs`): stack-aware scanner + `KIT_MANAGED_ROOTS` exclude-list + 3-verdict status. `bin/sos.sh` (Bash) CHƯA sửa — giữ nguyên bug by design, defer tới P077e cutover. Xem `docs/plans/P077c-decomposition.md` P077c5 + `docs/discoveries/P077c5.md`. |
| OA-03 | 🟠 | `ship` cho dirty tree qua preflight rồi chạy `git add -A` | Block dirty tree mặc định hoặc bắt scoped worktree/allowlist explicit |
| OA-04 | 🟠 | docs-sync engine có khả năng tốt nhưng config dogfood để toàn bộ mapping trống | Điền `rules/cross_doc/count_check/doc_structure`; thêm config-semantic gate |
| OA-05 | 🟡 | Current `sos adopt` là Claude-heavy bundle, không phải runtime-neutral adoption | Hoàn thành portable core → adapters; thêm `--runtime`/profile và domain-scoped plan |
| OA-06 | 🟡 | Rust `sos-rs` chưa parity với Bash, không có `new/adopt/map/sync`, hiện có 0 test | Giữ Bash là oracle, xây parity harness trước khi đổi canonical implementation |
| OA-07 | 🟠 | Installer dùng `releases/latest`; checksum cùng trust domain; local fleet có version drift | Pinned manifest + version/status doctor + atomic managed upgrades |
| OA-08 | 🟠 | Marker guards và path patterns chỉ bảo vệ một phần bề mặt, đặc biệt với repo phi chuẩn | Domain/path allowlist sinh từ manifest; fail-closed trên parse lỗi; test Codex/Claude parity |
| OA-09 | 🟡 | `sos init` hiện là dispatcher sang skill, không phải standalone docs generator | Tách deterministic generator khỏi LLM interview; docs nói rõ phần nào machine/LLM-owned |
| OA-10 | 🟢 | Additive/non-clobber adopt, hook wiring, checksums và Rust gates hoạt động đúng ở phạm vi đã test | Giữ làm oracle khi refactor portability |

---

## Môi trường và kiểm chứng cơ học

- SOS Kit HEAD tại lúc audit: `c2c764b` trên `main`, khớp `origin/main`.
- Pre-existing file được giữ nguyên: `docs/retro/DOGFOOD_P071-task6_3OS_2026-06-15.md` đang untracked.
- Rust tests từ source checkout:
  - `doctor`: **40 passed**.
  - `docs-gate`: **115 passed** (95 unit + 11 integration + 9 MCP).
  - `ship`: **54 passed**.
  - `bootstrap/sos-rs`: build/test PASS nhưng **0 tests**.
- `bash -n` PASS cho `bin/sos.sh`, installer, hooks và các shell guards chính.
- `doctor verify-setup --repo .`: CONNECTED.
- `docs-gate --all` trên chính SOS Kit: FAIL ngày audit vì CHANGELOG entry gần nhất
  `2026-07-20` vượt `changelog_max_age_days = 1` vào ngày `2026-07-22`.
- `doctor validate-map --map docs/AGENT_MAP.yaml` trên SOS Kit: không chạy được vì file map
  không tồn tại ở repo gốc.

Hai fixture tạm đã được chạy:

1. Greenfield Rust qua `sos new`: sinh repo buildable, branch `main`, hooksPath=`hooks`,
   `verify-setup` PASS và `docs-gate --all` PASS.
2. Brownfield Rust qua `cargo new` rồi `sos adopt`: manifest gốc giữ nguyên, hooks nối thành
   công, `validate-map` PASS; đồng thời lộ OA-02 và lượng asset Claude-heavy tại OA-05.

---

## OA-01 🔴 — Lane budget gate không tương thích với phiếu hiện hành

### Evidence

- `phieu/TICKET_TEMPLATE.md` khai báo `**Tầng:** 1 | 2`, không có trường `Lane`.
- `agents/orchestrator.md` vừa route theo `Tầng`, vừa bắt chạy
  `doctor lane-check --ticket ...` trước CHALLENGE.
- `doctor/src/cli/lane_check.rs` chỉ nhận regex:
  `**Lane:** Normal|Guarded|Fast`.

### Reproduction

```bash
doctor lane-check --ticket phieu/TICKET_TEMPLATE.md
doctor lane-check --ticket docs/ticket/P071-install-checksum.md
doctor lane-check --ticket docs/ticket/done/P042-giam-sat-boundary-check.md
```

Cả ba trả exit `2`: `ticket missing Lane field`.

### Impact

Hàng rào chống phiếu phình được viết và unit-test tốt nhưng không thể chạy trên mẫu phiếu
canonical. Đây là integration gap: mỗi component tự xanh nhưng state machine không nối được.

### Upgrade direction

1. Quyết định `Tầng` và `Lane` là hai trục độc lập hay một trục đã thay thế nhau.
2. Nếu hai trục độc lập: template bắt buộc cả hai, định nghĩa classifier/owner rõ ràng.
3. Nếu `Tầng` thay `Lane`: sửa Doctor budget theo `Tầng` hoặc tạo mapping canonical.
4. Thêm contract test chạy `doctor lane-check` trên chính `phieu/TICKET_TEMPLATE.md` và
   ít nhất một phiếu được tạo bởi `phieu.sh`.
5. Acceptance phải kiểm exit `0`, không chỉ unit-test fixture tự tạo có trường `Lane`.

---

## OA-02 🔴 — `sos map` false-green về coverage

### Evidence từ brownfield smoke test

Fixture ban đầu là Rust crate chuẩn với `src/main.rs`. Sau `sos adopt`:

- Map sinh surface `frontend` trỏ tới `templates` — đây là thư mục vừa được SOS Kit copy vào.
- Map không ghi `src/main.rs` hay một source surface Rust nào.
- `doctor validate-map` vẫn PASS vì mọi path **đã ghi trong map** đều tồn tại.

Root cause trong `bin/sos.sh`:

- Scanner chỉ biết các pattern `routes/handlers/views/controllers/api`,
  `models/entities/schema`, `services/lib`, `migrations`, `templates/components/static`
  và vài config file.
- Không có generic Rust crate/source surface cho `src/*.rs`.
- Adopt copy kit assets trước rồi mới gọi `sos map`, khiến scanner có thể map asset của kit.
- `validate-map` kiểm soundness của entry đã khai báo, không kiểm completeness của repo.

### Impact

Architect có thể nhận map hợp lệ về cú pháp/path nhưng không thấy phần code load-bearing thật.
`status: draft_needs_review` giảm mức overclaim trong docs, nhưng PASS từ validator dễ tạo
cảm giác map đã đáng tin hơn thực tế.

### Upgrade direction

1. Survey repo **trước** khi cài kit assets, hoặc exclude toàn bộ managed assets theo manifest.
2. Scanner stack-aware: Rust/Python/Node/Go/Swift và monorepo/domain profiles.
3. Phân biệt ba verdict:
   - `PATH_VALID`: mọi entry ghi ra tồn tại.
   - `COVERAGE_UNKNOWN`: chưa có oracle để biết đã bao phủ source.
   - `COVERAGE_REVIEWED`: human/architect đã xác nhận load-bearing surfaces.
4. Không cho `draft_needs_review` được dùng làm routing authority.
5. Thêm fixture acceptance tối thiểu:
   - Rust crate phải map `src/main.rs` hoặc `src/**`.
   - Kit-managed `templates/` không được tự biến thành product frontend.
   - Repo có `sim/`, `tools/`, `migrations/`, nested packages phải có expected surfaces.

### Trạng thái xử lý (P077c5, 2026-07-22)

**✅ FIXED trong Rust `bootstrap/sos-rs` (`crates/sos-cli/src/commands/map.rs`), Rust-only.**
Cả 5 upgrade-direction ở trên đã làm, khớp thứ tự:
1. Không survey-before-install (restructure nặng hơn) — dùng **static exclude-list**
   (`KIT_MANAGED_ROOTS`: `templates`/`phieu`/`scripts`/`hooks`/`.claude`, verified khớp
   `adopt.rs`'s copy targets) áp trong `scan_surface` — giải cả standalone map lẫn
   map-within-adopt bằng MỘT cơ chế.
2. Stack-aware scanner: `STACK_MARKERS` detect Cargo.toml/pyproject.toml,setup.py,
   requirements.txt/package.json/go.mod/Package.swift bất kỳ đâu trong tree (monorepo-aware,
   unbounded depth) → emit `rust_src`/`python_src`/`node_src`/`go_src`/`swift_src` surface.
3. 3-verdict: `status: draft_needs_review` → `PATH_VALID`/`COVERAGE_UNKNOWN`/
   `COVERAGE_REVIEWED`; fresh scan luôn `coverage_unknown`, KHÔNG dùng làm routing authority.
4. (đã làm ở #3)
5. 3 acceptance fixture hard-fail (`oa02_rust_crate_maps_src_main`,
   `oa02_templates_excluded_from_frontend`, `oa02_monorepo_nested_packages_mapped`) trong
   `crates/sos-cli/tests/parity.rs` — negative-test verified (sabotage → fail loud → revert).

**⚠️ `bin/sos.sh` (Bash) CHƯA sửa** — giữ nguyên bug OA-02 by design (P077c invariant:
Bash canonical KHÔNG đổi trong suốt P077c; fix Bash, nếu cần, defer tới P077e cutover
decision — KHÔNG phải Architect tự quyết, cần Chủ nhà xác nhận thời điểm). Người dùng
hiện tại của `bin/sos.sh` (MVP, exposure hẹp) vẫn gặp OA-02 cho tới cutover.

Chi tiết: `docs/plans/P077c-decomposition.md` (P077c5 status line) +
`docs/discoveries/P077c5.md` + `docs/ticket/done/P077c5-oa02-scanner-fix.md`.

---

## OA-03 🟠 — `ship` có thể gom thay đổi ngoài phiếu

### Evidence

- `preflight.rs` đọc `git status`; dirty files chỉ được đưa vào chuỗi mô tả, không fail.
- `commit.rs` chạy `git add -A` trước commit.
- Full pipeline còn tự bump version, sửa changelog, push và tạo/cập nhật PR.
- Default `docs_gate.blocking = false`.

### Impact

Trong shared worktree, một Worker có thể ship cả WIP của Chủ nhà hoặc agent khác. Đây không
phải bug nếu contract bắt buộc clean isolated worktree, nhưng hiện binary không enforce contract đó.

### Upgrade direction

- Mặc định fail nếu preflight thấy dirty paths không thuộc ship plan.
- Cho phép override explicit, có log, ví dụ `--include-dirty`.
- Tốt hơn: ship nhận ticket/worktree manifest và chỉ stage allowlisted paths + generated
  version/changelog files.
- Trước `git add`, in exact candidate set; acceptance kiểm không có unrelated file.
- Với multi-agent: một phiếu/một worktree/một branch là hard precondition.
- Cân nhắc default `docs_gate.blocking = true` cho repo được SOS quản lý.

---

## OA-04 🟠 — docs-sync capability có, nhưng dogfood config chưa dùng

### Evidence

`docs-gate` đã implement:

- file-to-doc mapping `rules`;
- `staleness`;
- `doc_structure`;
- command-backed `count_check`;
- `cross_doc`.

Nhưng `.docs-gate.toml` của chính SOS Kit hiện để cả năm mảng rỗng. Changelog staged check
cũng tắt vì path normalization bug với `docs/../CHANGELOG.md`.

### Impact

Contract “code đổi thì docs source-of-trust phải đổi” mới được enforce một phần. Gate hiện
không tự bảo đảm schema/API/data-flow thay đổi sẽ kéo đúng guide tương ứng. Architect vẫn có
thể đọc docs stale dù pre-commit từng PASS.

### Upgrade direction

1. Sửa path normalization để bật lại `changelog_staged`.
2. Dogfood ít nhất các mapping load-bearing:
   - `bin/sos.sh` / Rust CLI surface → README/GENESIS/BOOTSTRAP docs.
   - role/guard change → LAYERS/ORCHESTRATION/HANDOFF.
   - ticket schema → Doctor lane contract + template docs.
   - installer/tool manifest → SECURITY/SETUP/PORTABILITY architecture.
3. Config parse failure phải fail rõ, không âm thầm fallback default.
4. Thêm end-to-end test: stage source-only → gate FAIL; stage đúng doc → PASS.
5. Không cố suy luận semantic hoàn toàn tự động; mapping là project-curated source of truth.

---

## OA-05 🟡 — adoption hiện tại vẫn Claude-first, chưa phải portable core

### Evidence

Một `sos adopt` lên Rust crate sạch đã thêm hơn 60 file, gồm:

- `.claude/agents`, `.claude/commands`, `.claude/skills`, Claude settings;
- marker permissions;
- Claude-oriented orchestrator and hooks;
- MCP entries cho doctor/docs-gate/ship/guard/vps;
- toàn bộ phieu/templates/scripts generic.

Điều này đúng với product hiện hành nhưng không phù hợp cho:

- Codex-only repo;
- dual-runtime repo;
- monorepo có hai miền code/story;
- adoption chỉ muốn technical control plane trong một subdomain.

`docs/PORTABILITY_ARCHITECTURE.md` và backlog P075–P081 đã nhận đúng vấn đề; P075 portable
core đã xong, Claude parity/Rust adapter/Codex adapter/dogfood vẫn chưa xong.

### Upgrade direction

- Không vá thêm runtime token vào current Bash bundle.
- Đi theo thứ tự đã chốt: core → Claude adapter parity → Rust installer framework → Codex
  adapter → self-dogfood → dual-runtime brownfield dogfood.
- `sos install/adopt` cần `--runtime claude|codex|claude,codex|auto` và profile/domain scope.
- Generated artifact phải có owner/hash/adapter trong managed manifest.
- Với monorepo: cho phép profile chỉ quản `code/**` nhưng hook dispatcher vẫn ở Git root.

---

## OA-06 🟡 — Rust `sos-rs` chưa có parity oracle

### Evidence

- Source tự ghi `Status: skeleton`.
- Có `init/blueprint/contract/apply/recipe/launch/status`.
- Không có current Bash surfaces `new/adopt/map/sync/init security`.
- `cargo test --all-targets` chạy 0 tests.

### Impact

Không thể coi Rust binary là canonical hay thay Bash an toàn. Nếu phát triển song song không
có parity harness, hai implementation sẽ tiếp tục drift.

### Upgrade direction

- Freeze Bash behavior bằng golden fixtures trước.
- Viết parity tests cho dry-run plan, non-clobber conflict, sync provenance, hook collision,
  map generation, rollback và idempotence.
- Rust implementation chỉ thành canonical khi mọi Bash oracle xanh.
- Sau cutover, Bash chỉ là thin launcher hoặc bị xoá; không giữ hai canonical engines.

---

## OA-07 🟠 — distribution/version drift còn hở

### Evidence

- Installer verify `.sha256` và fail-closed cho required binaries: tốt.
- Asset và `.sha256` đều lấy từ cùng GitHub release/account.
- URL dùng `releases/latest`, chưa pin version.
- Trên máy audit:
  - installed `doctor 0.1.1`, source `0.1.3`;
  - installed `ship 0.1.0`, source `0.1.1`;
  - installed `docs-gate 0.1.0`, source `0.1.1`;
  - `inv-gate` không có trên PATH dù installer current xếp nó required.

### Impact

Repo contract có thể giả định tool behavior mới nhưng runtime dùng binary cũ. Checksum chống
corruption/tampering của asset đã chọn, không bảo đảm reproducible version selection.

### Upgrade direction

- `tool-manifest.toml` pin version + platform asset + checksum.
- `sos tools status` so expected/installed version và required/optional availability.
- `sos doctor` fail rõ khi contract cần feature không có trong installed binary.
- Upgrade atomic, có previous-version rollback.
- Sau cùng mới đánh giá signature/provenance mạnh hơn checksum cùng trust domain.

### Status update (P077d3, 2026-07-22) — RESOLVED trong Rust path

`tool-manifest.toml` (kit root) + `sos_install::tools::{check_tools, gate_required}` core +
`sos tools status` + install step-5 gate SHIPPED — pin version+asset+checksum cho đúng 10 tool
(6 required/4 optional), fail-closed cho required (Drift/Missing/Unparseable), reproduce ĐÚNG
evidence audit này live (doctor 0.1.1 vs pinned 0.1.3, inv-gate MISSING, exit 1). Checksum hiện
là honest `TODO` placeholder (chưa có prebuilt asset hash cho pin shape mới — real fill = P081).
Atomic upgrade + previous-version rollback (bullet 4 trên) **CHƯA làm** — đó là P081 tương lai,
d3 chỉ verify-only. `install.sh`/`bin/sos.sh` (Bash legacy path) vẫn dùng `releases/latest`
unpinned nguyên trạng tới khi P077e cutover Rust binary thành canonical — OA-07 coi như đóng
CHO Rust path, Bash path còn hở tới cutover. Chi tiết: `docs/discoveries/P077d3.md`,
`CHANGELOG.md` `[P077d3]`.

### Status update (P078c, 2026-07-22) — resolved-differently: render decoupled từ tool-gate

d3 wired tool-check là HARD-BLOCK render (`resolve_tools()?` chạy TRƯỚC `apply()`/`dry_run()` ở
`install.rs`) — hệ quả: máy có tool drift (evidence trên, live reproduce) thì `sos install`
KHÔNG ghi được adapter file nào, chặn cả P079 Codex dogfood dù render adapter file không hề phụ
thuộc sister-tool version. P078c tách 2 concern: render TRƯỚC/độc lập tool-check; drift surfaced
qua **loud-warn + exit 3** (distinct "installed, tools-not-ready") thay vì abort. `--require-tools`
opt-in flag giữ NGUYÊN hành vi fail-closed d3 (tool-check trước render, abort exit 1, no render)
cho CI/production cần mạnh nhất. OA-07 tín hiệu KHÔNG bị yếu đi: drift vẫn luôn loud + non-zero,
chỉ đổi "block render" → "render + report"; `sos tools status` không đổi. Chi tiết:
`docs/discoveries/P078c.md`, `CHANGELOG.md` `[P078c]`.

---

## OA-08 🟠 — guards là discipline guard, chưa phải complete sandbox

### Evidence

- Architect guard cho phép mọi `.md`; repo có code/config quan trọng dạng Markdown vẫn đọc được.
- Orchestrator guard tập trung `*.swift`, `*.pbxproj`, `src/**`; product code ở `tools/`,
  `sim/`, root scripts hoặc domain phi chuẩn có thể lọt.
- JSON/tool input parsing trong shell guard có đường fail-open khi không parse được path.
- Bash redirect là bypass được tài liệu thừa nhận.
- Marker files có nguy cơ stale/race, đặc biệt background agent và shared worktree.
- Current PreToolUse wiring là Claude-specific; Git hooks mới agent-agnostic.

### Upgrade direction

- Guard theo domain/path manifest, không chỉ extension và tên `src`.
- Architect dùng explicit read allowlist (`docs technical + ticket scope`), không phải “mọi md”.
- Worker dùng ticket-bound edit allowlist.
- Parse structured event bằng typed adapter thay vì regex shell khi Rust framework sẵn sàng.
- Unknown/unparseable target phải fail-closed trong guarded phase.
- Marker lifecycle chuyển thành typed state/lease có owner + expiry; parallel Worker bắt buộc worktree.
- Git hook tiếp tục là invariant backstop chung cho mọi runtime.

---

## OA-09 🟡 — nghĩa của `sos init` dễ bị hiểu quá mức

### Evidence

Current Bash `sos init` chủ yếu khởi tạo state và hướng dẫn gọi `/init`. Skill `/init` mới là
phần phỏng vấn tối đa ba câu và viết `PROJECT.md`/các vision docs. `sos blueprint` cũng in
instruction cho Architect thay vì tự xây blueprint deterministic.

### Impact

Người dùng có thể hiểu `sos init` là một generator hoàn chỉnh, trong khi kết quả phụ thuộc
runtime skill/LLM và adapter availability.

### Upgrade direction

- Docs tách rõ:
  - deterministic CLI output;
  - LLM interview output;
  - human judgment slots;
  - validation gates.
- Rust CLI tạo skeleton/state/manifest deterministically.
- Adapter gọi đúng Story/Technical Architect khi cần judgment.
- Generated docs phải ghi provenance/status: skeleton, draft, human-approved, canonical.

---

## OA-10 🟢 — các oracle tốt cần giữ nguyên khi refactor

Những cơ chế đã hoạt động đúng trong audit:

1. `sos adopt` giữ nguyên manifest brownfield và dùng additive/non-clobber policy.
2. Symlink escape guard trong adopt không cho copy target ra ngoài kit tree.
3. Hook installer nối `core.hooksPath=hooks` và có collision guard.
4. `doctor verify-setup` bắt wiring joints và PASS trên fixture sinh đúng.
5. `docs-gate`, `doctor`, `ship` có test suite tốt ở cấp component.
6. Required binary download có checksum verification và temp→atomic move.
7. `sos new` sinh Rust repo buildable thay vì chỉ tạo `src/` rỗng.
8. Repo tự ghi nhận giới hạn adoption hiện tại (~60–70% fit trong calibration n=1) và đã có
   migration order portability rõ ràng.

Các điểm này nên trở thành golden/parity oracle, không viết lại từ trí nhớ trong P076–P081.

---

## Upgrade order đề nghị

Không xử lý từng finding rời rạc theo độ dễ. Thứ tự giảm nguy cơ sửa chồng:

1. **OA-01:** nối lại ticket schema ↔ Doctor ngay; đây là gate chống scope phình đang chết.
2. **OA-02:** sửa map false-green và định nghĩa mức tin cậy trước khi Architect dùng map.
3. **OA-03:** harden `ship` clean/scoped worktree trước khi giao Worker tự ship.
4. **OA-04:** dogfood docs mappings thật để source-of-trust không drift trong chính đợt refactor.
5. **P076/P077/P078:** giải OA-05/OA-06/OA-08 bằng adapter architecture, tránh vá tiếp Bash.
6. **OA-07:** managed manifest + pinned distribution trước public one-command release.
7. **OA-09:** chuẩn hóa UX/provenance của init sau khi CLI/adapter boundary đã thật.

## System-level acceptance cuối

Một upgrade không được coi là xong chỉ vì unit tests từng binary xanh. Cần một fixture matrix:

| Fixture | Acceptance tối thiểu |
|---|---|
| Greenfield Rust | new → docs/state → ticket → challenge → execute → docs sync → ship dry-run |
| Brownfield Python/TS/Rust | adopt non-clobber, real source mapped, kit assets excluded |
| Codex-only | không sinh `.claude/**`; role/guard/skill native hoạt động |
| Claude-only | parity với behavior golden hiện tại |
| Dual runtime | một core doctrine, hai adapters, không doctrine fork |
| Monorepo code/story | Technical Architect không đọc story; Story Architect không đọc code; sim bridge chỉ qua contract |
| Dirty shared tree | ship bị block hoặc chỉ stage exact allowlist |
| Stale technical docs | code-only staged change bị docs-gate block đúng guide |
| Oversized ticket | ticket được tạo từ canonical template và lane/tier gate chặn thật |

**Done-when:** cả fixture flow xanh từ đầu đến cuối, không chỉ component tests xanh riêng lẻ.
