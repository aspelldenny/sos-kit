# Mechanization Audit — lesson mềm nào còn ép cứng được

> Ghi 2026-05-30. Repo soi: `~/tarot` (Soul Signature).
> Câu hỏi gốc của Sếp: "cơ học hóa thêm được lesson nào đang trượt không?"
> Trạng thái: **CHƯA quyết** — Sếp đọc rồi quyết sau.

---

## Khung tư duy

Mức cải thiện của một vai agent qua các session ≈ tỉ lệ với **mức cơ-học-hóa của lesson**:

- Lesson → **lệnh grep / hook cơ học** (chạy → ✅/❌): tái phạm ≈ 0. Khá lên thật.
- Lesson → **doctrine đọc-hiểu** (đọc file rồi tự nhớ làm theo): giảm tần suất nhưng vẫn trượt.
- Lesson → **chưa được gọi tên** (unknown-unknown): không bắt được, chỉ va vấp mới lòi.

Bằng chứng: catalog Sub-mechanism A→F trong tarot/CLAUDE.md — cùng họ lỗi tái diễn 11 lần. Cái nào đã thành grep/hook thì hết tái, cái nào còn doctrine mềm thì còn trượt.

→ Mục tiêu audit: tìm lesson đang ở dạng **doctrine mềm** mà biến được thành **grep/hook/Task 0 check** SẠCH (deterministic, false-positive thấp). KHÔNG vẽ thêm gác cho lesson không thực sự tái diễn (AI completeness bias).

---

## Đã cơ-học-hóa rồi (không đụng)

`~/tarot` hiện có:

- **pre-commit** (`.git/hooks/pre-commit`, đã install): chạy `pnpm test:gate` (port-bind allowlist) + `security-gate.sh --mechanical-only` (INV-001→007) + docs-stats sync check (chỉ fire khi touch src/tests/prisma).
- **3 PreToolUse hook** (settings.json):
  - `architect-guard.sh` (matcher Read|Glob|Write|Edit)
  - `block-env-edit.sh` (matcher Edit|Write) — chặn sửa `.env*` trừ `.env.example`
  - `block-unsafe-merge.sh` (matcher Bash) — chặn `gh pr merge` khi chưa có APPROVE sentinel
- **SessionStart**: `session-start-banner.sh`
- **package.json gates**: `security:gate`, `security:gate:runtime` (check-runtime-secrets.py — Sub-mech F), `test:gate`.
- **Runtime word ceiling** `CHI_HA_SYSTEM_PROMPT_V3 < 16000` — đã có hard test ceiling (Vitest).

---

## Kết quả chấm từng lesson mềm còn lại

| # | Lesson mềm | Trạng thái hiện tại | Cơ-học-hóa được? | Verdict |
|---|---|---|---|---|
| 1 | **GitHub MCP `create_or_update_file` / `push_files` dùng để commit** | Chỉ có doctrine "⛔ TUYỆT ĐỐI KHÔNG DÙNG" trong CLAUDE.md + memory. **KHÔNG hook nào chặn.** | ✅ SẠCH — PreToolUse matcher trên 2 tool name → block exit 2. Zero false-positive (2 tool này không bao giờ nên dùng để commit code). | **NÊN LÀM** (win sạch duy nhất) |
| 2 | **pre-push security-review reminder** | Script `scripts/pre-push-hook.sh` ĐÃ VIẾT nhưng **chưa wire thành `.git/hooks/pre-push`** (chỉ pre-commit được install) → đúng Sub-mech A trigger gap. | ✅ Được (chạy `install-hooks.sh`), nhưng **REDUNDANT** với orchestrator auto-spawn rule 9 + `block-unsafe-merge.sh` đã gác cửa merge. | Tùy — belt-and-suspenders, urgency thấp |
| 3 | **`console.log` trước commit** | ~10 file thật (app:5, lib:5 — cron/webhook/telegram logging hợp lệ; generated code loại trừ). | ⚠️ Ban cứng → false-positive cao. Chỉ nên warn, không block. | Không đáng |
| 4 | **Prisma gotcha** (`findUnique` by email / `new PrismaClient` trực tiếp) | Grep ra TOÀN generated code (`src/generated/prisma/*`) + token lookup hợp lệ (`emailVerificationToken.findUnique`). Source thật recurrence ≈ 0. | Được về kỹ thuật nhưng… | **REJECT** — giải vấn đề giả định (AI bias: gác cho lỗi không tái diễn) |
| 5 | **Research Gate / CHARACTER-first flow** | `/research` slash + `prompt-reviewer` subagent cover MỀM. | ❌ Khó ép sạch — phải parse intent phiếu + diff để biết "có chạm chị Hạ" + "đã cite 3 research file chưa". False-positive cao. | DEFER — mechanize sạch khó, để doctrine + subagent |

---

## Chốt khuyến nghị

**Chỉ đúng 1 cái là win sạch: hook chặn GitHub MCP commit (#1).**

- Biến một "⛔ luật mồm" thành lệnh chặn cứng.
- Y hệt pattern `block-unsafe-merge.sh` đã có → ~15 dòng, rủi ro thấp.
- Đúng loại "soft ban dễ trượt" mà cơ-học-hóa đáng giá: tốn 30-50K token/lần nếu lỡ tay, deterministic, không false-positive.

Phác implementation (nếu Sếp duyệt):
- Tạo `scripts/block-github-mcp-commit.sh`: đọc tool name từ stdin JSON, nếu khớp `mcp__github__create_or_update_file` hoặc `mcp__github__push_files` → `exit 2` + reason "Dùng git bash commit/push, không dùng GitHub MCP (token-burn). Xem CLAUDE.md §GIT WORKFLOW."
- Thêm vào `.claude/settings.json` PreToolUse, matcher trên 2 tool name đó.
- (Tùy) port lên sos-kit golden template nếu thấy chung cho mọi repo.

Mấy cái #3 #4 #5 **cố tình KHÔNG đề xuất** — chấm theo câu hỏi vàng, không vẽ thêm gác cho lesson không thực sự tái diễn.

---

## Ghi chú cho sos-kit golden template (nếu generalize)

- Hook #1 (block GitHub MCP commit) là **chung cho mọi repo** dùng git-bash-commit doctrine → ứng viên đưa vào golden template hook set.
- pre-push wiring (#2): nếu golden template đã có orchestrator auto-spawn + block-unsafe-merge thì pre-push redundant — cân nhắc bỏ khỏi template để giảm trùng lặp.
