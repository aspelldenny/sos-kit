# WORKFLOW FRICTION LOG — JARVIS foreign-adopt (dogfood-BEFORE-infra step 2)

> **Mục đích:** Ghi mọi friction khi adopt sos-kit vào `~/jarvis` (fresh-foreign Python bot — portrait
> của repo người-dùng-mới thật: chỉ có CLAUDE.md + .docs-gate.toml trước adopt, không agents/hooks/phieu).
> Mỗi finding ở đây = 1 **requirement cho installer "1 lệnh"** (Tier-3 distribution build).
> **Format mỗi mục:** `[ID] severity — triệu chứng → root cause → đề xuất fix`.
> Sev: 🔴 block/correctness · 🟡 friction/ergonomics · 🟢 worked-as-intended (ghi để giữ).
> ID prefix `JA-` (jarvis-adopt) — tránh đụng F01-F13 của log doc-rotate 2026-06-09.

---

## 📬 TRIAGE COVER — installer requirements (2026-06-11)

| ID | Sev | Trạng thái | Installer requirement |
|---|---|---|---|
| **JA-01** | 🔴 | ✅ FIXED session này | `sos adopt` drop CẢ 4 spawnable agents (symlink + `find -type f`) → fix `find -L` trong `adopt_item` |
| **JA-02** | 🟡 | ✅ FIXED session này | adopt đẻ root `CHANGELOG.md` thừa khi repo để changelog ở `docs/` → check `docs/CHANGELOG.md` trước khi generate |
| **JA-03** | 🟡 | OPEN | adopt KHÔNG arm hooks (`sos new` arm, adopt chỉ copy) — installer phải chạy `install-hooks.sh` (đã có F09 guard, an toàn) |
| **JA-04** | 🟡 | OPEN | adopt KHÔNG chạy `sos init security` (`sos new` có) → thiếu `.sos-stack.toml` cho advisory-scan/security-review |
| **JA-05** | 🟡 | OPEN | JSON merge tay ×2 (`.mcp.json` thêm doctor entry; `settings.local.json` thêm 5 marker permissions) — installer cần auto-merge (jq) hoặc in chính xác block cần dán |
| **JA-06** | 🟡 | OPEN | `sos` không có trên PATH — ngay cả máy chính chủ kit. Installer phải symlink/alias `sos` (curl\|sh step 1) |
| **JA-07** | 🟡 | OPEN | INSTALL.md (copy tay 5') và `sos adopt` (1 lệnh) là 2 đường song song chưa thống nhất — INSTALL.md không nhắc `sos adopt`. New user không biết theo đường nào |
| **JA-08** | 🟡 | OPEN | adopt không warn khi target dirty tree — output adopt lẫn WIP của user trong `git status` (jarvis có 7 modified + 9 untracked lúc adopt) |
| **JA-09** | 🟡 | OPEN | First-commit-after-adopt sẽ bị docs-gate block (changelog stale 2026-04-24) — expected onboarding moment nhưng cần message hướng dẫn "thêm CHANGELOG entry" thay vì FAIL khô |
| **JA-10** | 🟢 | giữ | F09 install-hooks guard worked-as-intended trên foreign repo: jarvis có sẵn `.git/hooks/pre-commit` → move `.bak` + báo rõ, không hijack im lặng. n=1 foreign |
| **JA-11** | 🟢 | giữ | `doctor verify-setup` 3-way triage dẫn thẳng tới root cause: J6 BROKEN = boundary-check.md vắng mặt = JA-01. Không có verify-setup thì agents-mất-tích sẽ im lặng tới lần spawn đầu |
| **JA-12** | 🔴 | → JARVIS repo | INV-010 bắt **GitHub PAT thật** hardcode trong `scripts/setup_obsidian_server.sh:8,36` (committed 17804e7 + pushed, repo PRIVATE) — kit-side = worked-as-intended 🟢; jarvis-side = incident: revoke token + chuyển env var |

**Ưu tiên installer build:** JA-01/02 đã fix → JA-03+04 (adopt nên "born-wired" như `sos new`: arm hooks + init security trong 1 lần chạy) → JA-05 (JSON merge) → JA-06+07 (PATH + 1 đường install duy nhất) → JA-08/09 (UX warning).

**Kết luận lớn cho "1 lệnh":** `sos adopt` hiện = "copy đúng nhưng chưa wired" — còn 4 bước tay (arm hooks, init security, merge 2 JSON, fill BACKLOG). Installer 1-lệnh = `sos adopt` + tự động JA-03/04/05 + verify-setup cuối + in đúng 2 việc còn lại cho Chủ nhà (fill BACKLOG Active sprint + restart Claude Code).

---

## JA-01 🔴 — adopt drop cả 4 spawnable agents (symlink bị `find -type f` bỏ qua) — ✅ FIXED

- **Triệu chứng:** Sau `sos adopt ~/jarvis`, `.claude/agents/` chỉ có `orchestrator.md`. Thiếu architect/worker/advisory-watch/boundary-check → workflow 3-role chết từ spawn đầu. `doctor verify-setup` báo `J6 BROKEN — agent does not emit a Verdict: line` (vì boundary-check.md không tồn tại).
- **Root cause:** Kit's `.claude/agents/*.md` là **symlink** → `../../agents/*.md` (drift-fix 2026-06-01). `adopt_item` duyệt dir bằng `find "$src" -type f` — symlink là type `l`, bị skip toàn bộ. `sos new` đã né bằng `cp -RL` (deref), adopt sót. Bug nằm im từ khi symlink-hóa vì mọi adopt trước đó (media stress 2026-06-09) target đã có agents → conflict path, không lộ absent path.
- **Fix (session này):** `find -L` trong `adopt_item` (bin/sos.sh) — deref symlink, `cp` không `-P` tự deref content. Verify: re-run adopt → 4 agents land; `doctor verify-setup --repo ~/jarvis` → **CONNECTED rc=0**; run 3 → ADDED (none) = idempotent.

## JA-02 🟡 — duplicate root CHANGELOG khi repo để changelog ở `docs/` — ✅ FIXED

- **Triệu chứng:** jarvis để changelog ở `docs/CHANGELOG.md` (đúng theo `.docs-gate.toml` của nó: `docs_dir="docs"` + `changelog="CHANGELOG.md"`). Adopt chỉ check root → generate thêm root `CHANGELOG.md` skeleton → 2 changelog, root là noise và docs-gate không đọc nó.
- **Root cause:** adopt hardcode vị trí root, không consult `.docs-gate.toml` của repo.
- **Fix (session này):** check thêm `$target/docs/CHANGELOG.md` trước khi generate (80% case). Dài hạn (installer): parse `changelog` path từ `.docs-gate.toml` thật.

## JA-03 🟡 — adopt không arm hooks (copy nhưng không wire)

- **Triệu chứng:** Sau adopt, `git config core.hooksPath` trống — `hooks/pre-commit` đã copy vào repo nhưng KHÔNG chạy khi commit. Phải tay `bash scripts/install-hooks.sh`.
- **Root cause:** `sos new` có bước "[+] Git init + arm hooks" (born-wired doctrine); `sos_adopt` dừng ở copy + report. Có thể là thận trọng có chủ ý (foreign repo có hook setup riêng) — nhưng `install-hooks.sh` đã có F09 guard (detect hijack → confirm/abort) nên arm tự động giờ an toàn.
- **Đề xuất:** adopt gọi `install-hooks.sh` ở cuối (guard F09 lo phần safety). Non-TTY abort của guard = đúng hành vi cho CI.

## JA-04 🟡 — adopt không chạy `sos init security`

- **Triệu chứng:** Sau adopt không có `.sos-stack.toml` → `/advisory-scan` + `/security-review` thiếu foundation. Phải tay `sos init security` (detect requirements.txt → python stack, 5 giây).
- **Root cause:** `sos new` step [3] có gọi `sos_init_security`; `sos_adopt` không.
- **Đề xuất:** adopt gọi `sos_init_security` (đã idempotent — tự skip nếu file tồn tại).

## JA-05 🟡 — JSON merge tay ×2 (.mcp.json + settings.local.json)

- **Triệu chứng:** adopt flag `~ .mcp.json (exists — add the "doctor" server entry by hand)`. Tương tự `settings.local.json` cần thêm 5 marker permissions (INSTALL.md §2.5). Cả 2 đều là JSON-edit tay, dễ sai cú pháp, new user dễ bỏ qua → per-spawn permission prompt + doctor MCP vắng.
- **Root cause:** adopt là bash thuần, không có JSON merge. Non-clobber đúng doctrine nhưng "đúng block cần dán" thì adopt cũng không in ra.
- **Đề xuất (installer):** auto-merge bằng `jq` khi có (thêm key vắng mặt, không đụng key tồn tại); fallback in chính xác JSON block để dán. Rust `sos` port thì serde làm sạch chuyện này.

## JA-06 🟡 — `sos` không có trên PATH

- **Triệu chứng:** `command -v sos` → not found, ngay trên máy chính chủ kit. Phải gọi `bash ~/sos-kit/bin/sos.sh adopt ...` — new user không thể biết.
- **Đề xuất:** curl|sh installer step 1 = đặt `sos` binary/script lên PATH (`~/.local/bin` symlink hoặc cargo install khi Rust port xong). Đây là tiền đề của mọi lệnh khác.

## JA-07 🟡 — 2 đường install song song chưa thống nhất (INSTALL.md vs `sos adopt`)

- **Triệu chứng:** INSTALL.md hướng dẫn copy tay từng file (viết trước khi `sos adopt` ra đời, "5 phút"); `sos adopt` làm 90% việc đó trong 1 lệnh + validator. INSTALL.md không nhắc tới `sos adopt`/`sos new`/`sos sync`. New user đọc INSTALL.md sẽ đi đường tay dài hơn và dễ sót (chính INSTALL.md từng ghi chú "skipping the security pair là root cause media collapse" — adopt tự động thì không sót được).
- **Đề xuất:** INSTALL.md viết lại quanh 3 lệnh (`sos new` / `sos adopt` / `sos sync`) + giữ phụ lục manual cho người muốn hiểu từng mảnh. DOCS-GATE row: `bin/sos.sh` subcommand surface đổi → INSTALL.md + docs/SETUP.md.

## JA-08 🟡 — adopt không warn dirty target tree

- **Triệu chứng:** jarvis lúc adopt có 7 modified + 9 untracked (WIP của Chủ nhà). Adopt thêm ~60 file mới → `git status` thành nồi lẩu WIP-lẫn-adopt, khó review "adopt đã làm gì" để commit riêng.
- **Đề xuất:** adopt mở đầu bằng check `git status --porcelain` → nếu dirty, warn + gợi ý "commit/stash WIP trước, hoặc tiếp tục (adopt chỉ THÊM file, không sửa)". Không block — chỉ 1 dòng warn là đủ 80%.

## JA-09 🟡 — first-commit-after-adopt bị docs-gate block khô

- **Triệu chứng:** Smoke-run `hooks/pre-commit` trong jarvis → `[2/7] docs-gate FAIL: changelog — Last entry too old: 2026-04-24` (+ INV-010 fail, xem JA-12) → rc=1. Commit đầu sau adopt CHẮC CHẮN bị block ở repo có changelog cũ — đúng doctrine nhưng new user chưa biết phải làm gì.
- **Đề xuất:** adopt report cuối in sẵn: "Commit đầu sẽ cần 1 entry CHANGELOG mới (docs-gate freshness) — thêm `## <ver> — sos-kit adopted — <date>`". Biến gate-fail đầu đời thành guided step.

## JA-10 🟢 — F09 install-hooks guard worked-as-intended (n=1 foreign)

- jarvis có sẵn `.git/hooks/pre-commit` (không phải core.hooksPath). Guard move ra `.git/hooks/pre-commit.pre-hookspath.bak` + báo rõ escape hatch, rồi set `core.hooksPath → hooks/`. Đúng thiết kế P070/F09 — lần đầu fire trên foreign repo thật. Giữ.

## JA-11 🟢 — verify-setup 3-way triage dẫn thẳng root cause

- `J6 BROKEN — agent does not emit Verdict:` nghe như contract bug, thực ra là *file vắng mặt* (JA-01). Nhờ chạy verify-setup ngay trong adopt nên agents-mất-tích lộ TRƯỚC lần spawn đầu thay vì fail im lặng giữa workflow. Đây là giá trị của validator-trong-installer — installer 1-lệnh PHẢI giữ verify-setup làm bước chốt.

## JA-12 🔴 → JARVIS — INV-010 bắt GitHub PAT thật (kit-side 🟢 worked-as-intended)

- **Triệu chứng:** Security gate `[4/7]` → `scripts/setup_obsidian_server.sh:8,36: INV-010 violated — ghp_...1lha (github-pat-classic + token-in-url)`. File tracked, committed (17804e7), pushed lên `github.com/aspelldenny/jarvis` (PRIVATE).
- **Kit-side:** check-runtime-secrets quét đúng, day-1 trên foreign repo. Bằng chứng sống cho giá trị adopt (cùng class với media DB-password finding P068).
- **JARVIS-side action (Chủ nhà):** (1) **revoke PAT** trên GitHub settings ngay (token sống trong history, private ≠ an toàn); (2) sửa script đọc token từ env var; (3) optional: history scrub (BFG) — sau revoke thì độ khẩn thấp.
