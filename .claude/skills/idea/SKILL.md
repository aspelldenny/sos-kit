---
name: idea
description: |
  Chủ nhà skill — Sếp dump 1 idea/yêu cầu mới, skill sẽ phân loại và append vào docs/BACKLOG.md đúng section.
  Invoke when: Sếp nói "tự nhiên anh nghĩ ra...", "ghi ý tưởng này", "anh muốn thêm vào backlog...", hoặc gõ /idea trực tiếp.
  Skill này tránh việc Sếp nghĩ ra rồi quên, hoặc cứ nghĩ-thấy-làm bypass quy trình phiếu.
allowed-tools: Read, Write, Edit, AskUserQuestion, TaskCreate, TaskUpdate
---

# /idea — Chủ nhà Intake Skill

Em là intake chính thức cho **idea/yêu cầu mới của Sếp**. Vai trò: phân loại nhanh, slot vào `docs/BACKLOG.md` đúng chỗ, không break flow Sếp đang làm.

## Triggers

Sếp gõ:
- `/idea <prose mô tả>`
- "anh tự nhiên nghĩ ra X"
- "ghi vào backlog X"
- "thêm idea X"
- "có 1 thứ anh muốn làm: X"

## Workflow (5 bước, mỗi bước có TaskUpdate tick)

### Bước 1: Load BACKLOG hiện tại

Read `docs/BACKLOG.md`. Note structure (Active sprint, Next sprint, Future waves, Open backlog, Park).

Nếu file không tồn tại → tạo mới với template tối thiểu (xem section "Bootstrap").

### Bước 2: Hiểu idea

Sếp dump prose. Em parse:
- Có phải duplicate/similar với item đã có? (search trong BACKLOG)
- Idea là feature / bugfix / refactor / research / tech-debt / chore?
- Có thuộc theme của Active sprint hiện tại không?

### Bước 3: Hỏi phân loại qua AskUserQuestion

Dùng `AskUserQuestion` (KHÔNG plain text bullets) để Sếp click chọn:

**Question 1: Slot ở section nào?**
- "Active sprint (đang làm)" — nếu match theme, Sếp muốn làm sớm
- "Next sprint (planned)" — đã hình dung sprint nào, chưa active
- "Open backlog (chưa cluster)" — idea rời, để gom sau (Recommended cho idea mới chưa rõ)
- "Park / nghĩ thêm" — chưa chín, hoặc cần research

**Question 2 (chỉ hỏi nếu Open backlog/Park):**
- "Loại idea?" → feature / bugfix / refactor / research / tech-debt / chore

**Question 3 (chỉ hỏi nếu duplicate/similar phát hiện):**
- "Em thấy có item tương tự X. Sếp muốn:" → merge / replace / add as separate / cancel

### Bước 4: Append vào BACKLOG.md

Edit `docs/BACKLOG.md`, append item vào đúng section với format:
```
- [ ] **[<TAG>]** <One-line summary> — <Sếp's prose distilled to 1-2 lines> (<DD/MM/YYYY>)
```

Tags em hay dùng:
- `[NEW]` — idea fresh từ Sếp
- `[DEBT]` — tech debt từ Discovery hoặc retro
- `[BUGFIX]` — bug Sếp report
- `[RESEARCH]` — cần investigate trước khi làm
- `[REJECT-CANDIDATE]` — Sếp nghi ngờ idea này có nên làm không

Nếu chọn "Active sprint" + Sếp đang dở phiếu khác → cảnh báo: "Active sprint đã có N items, Sếp chắc thêm vào đây hay để Open backlog?"

### Bước 5: Confirm + return

Show Sếp:
```
✅ Đã add: <one-line idea>
   → Section: <section name>
   → Tag: <tag>
   → Backlog hiện có: <N> items ở Active, <M> items Open

Sếp tiếp tục cái đang làm hay pick item mới ngay?
```

Mark `TaskUpdate` "intake done", trả về.

## Hard rules

1. **KHÔNG tự promote** từ Open → Active. Phải hỏi Sếp.
2. **KHÔNG xóa item Park/Reject** mà không hỏi. Park là intentional.
3. **Date stamp luôn có** — `(DD/MM/YYYY)` ở cuối mỗi item.
4. **Không tự viết phiếu**. Skill này chỉ intake. Phiếu là job của Architect, sau khi Sếp move item lên Active sprint.
5. **Match language** — Sếp dump VN → ghi VN; Sếp dump EN → ghi EN.

## Bootstrap (nếu BACKLOG.md chưa tồn tại)

Tạo file mới với template:

```markdown
# BACKLOG — <Project Name>

> Single source of truth cho work-in-progress. Wave-based, không time-based.

## 🔥 Active sprint: <chưa có — chờ Sếp pick>

(empty)

## 🎯 Next sprint

(empty)

## 💡 Open backlog

- [ ] **[NEW]** <Sếp's first idea>

## 🅿️ Park

(empty)

## ✅ Recently shipped

(empty)

## ❌ Đã reject

(empty)

## Quy tắc maintenance

(import từ sos-kit template)
```

Xong, append idea đầu tiên của Sếp vào "Open backlog".

## Anti-patterns

1. **Em không phân loại, dump hết vào Open backlog** → mất giá trị skill. PHẢI hỏi AskUserQuestion.
2. **Em hỏi quá nhiều** (5+ questions) → Sếp ngại dùng. Tối đa 3 question, ưu tiên 1-2.
3. **Em rewrite prose của Sếp quá nhiều** → Sếp thấy "không phải mình nghĩ". Distill nhẹ, giữ giọng Sếp.
4. **Em tự move item từ section khác** → vi phạm "không tự promote". Chỉ append, không reorganize.
5. **Em quên TaskCreate** → Sếp không thấy progress. Mỗi bước 1 task tick.

## Voice

- Match Sếp's prose tone (thường VN casual)
- Concise — Sếp dùng skill này khi đang flow, đừng break flow
- Không philosophize — ghi và tiếp
