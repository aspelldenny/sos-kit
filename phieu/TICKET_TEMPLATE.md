# PHIẾU P<NNN>: <short title>

> **ID format:** `P` + 3 digits (P001, P042, P123). ID is auto-assigned by the `phieu` shell function from `<project>/.phieu-counter` — do not set manually.
> **Filename:** `docs/ticket/P<NNN>-<slug>.md` (matches branch name, without `<type>/` prefix).
> **Branch:** `<type>/P<NNN>-<slug>` where `<type>` ∈ {feat, fix, chore, docs, infra}.
> **Usually created via `phieu <slug>`** (shell function auto-fills ID + creates branch + worktree + this file).

---

> **Loại:** Feature / Bugfix / Prompt-only / Hotfix
> **Ưu tiên:** P0 / P1 / P2
> **Ảnh hưởng:** [main files affected]
> **Dependency:** [which phiếu must finish first, or "None"]

---

## Context

### Vấn đề hiện tại
[Describe the problem or feature]

### Giải pháp
[Describe the approach]

### Scope
- CHỈ sửa [list files]
- KHÔNG sửa [list files that must not be touched]

---

## Task 0 — Verification Anchors

> **REQUIRED** — Architect must grep/verify real code before writing assumptions.
> Worker reads this table to know which assumptions were verified vs. unverified.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | [Function X exists in file Y] | `grep "function X" src/...` | ✅ Line 123 |
| 2 | [Constant Z = "abc"] | `grep "Z" src/...` | ✅ Line 456 |
| 3 | [Spread list is constant SPREADS_REBUILD] | `grep "SPREADS_REBUILD" src/...` | ❌ NOT FOUND — inline string instead |

**If "Result" column has ❌ → architect acknowledged the wrong assumption and specified how to handle it in the Nhiệm vụ section below.**

---

## Nhiệm vụ

### Task 1: [task name]

**File:** `src/path/to/file.ts`

**Tìm:** [exact text to locate — use content, NOT constant/variable names unless verified]

**Thay bằng / Thêm:**
```
[new content]
```

**Lưu ý:** [edge cases, conditions, cross-module interactions]

### Task 2: [...]

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/path/file.ts` | Task 1: short description |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `src/path/other.ts` | [function X should continue to work with the change] |

---

## Luật chơi (Constraints)

1. [Constraint 1]
2. [Constraint 2]

---

## Nghiệm thu

### Automated
- [ ] Type-check clean (e.g., `pnpm type-check` / `cargo check`)
- [ ] Tests pass (e.g., `pnpm test --run` / `cargo test`)

### Manual Testing
- [ ] [Test case 1]
- [ ] [Test case 2]

### Regression
- [ ] [Feature X still works]

### Docs Gate
- [ ] `CHANGELOG.md` — entry for this phiếu
- [ ] `[GUIDE].md` — [section updated]

### Discovery Report
- [ ] Append entry to `docs/DISCOVERIES.md` (newest on top, like CHANGELOG)
  - Assumptions in phiếu — CORRECT / WRONG
  - Edge cases / limitations found
  - Docs updated to match reality
