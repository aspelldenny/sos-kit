# BACKLOG — <Project Name>

> **Purpose:** Single source of truth for "what should Chủ nhà do next".
> New ideas → enter here first (via /idea skill). Phiếu → only written for items in Active sprint.
> Wave-based, NOT time-based. Sprint ends when done OR when Chủ nhà changes direction.
>
> **Architect rule (Rule 0):** Architect only writes phiếu for items in "Active sprint" or items Chủ nhà has explicitly moved up from "Next sprint". No phiếu for "Open backlog" / "Park" items until Chủ nhà picks them.

---

## 🔥 Active sprint: <Sprint name / number>

> **Goal:** <1-2 sentences describing the sprint's goal.>
> **Done when:** <Exit condition — NOT a time deadline.>
> **Started:** <DD/MM/YYYY when promoted to Active>

<!-- 3-7 items Chủ nhà has committed to in this sprint.
     Tags for classification:
       [NEW]      = fresh idea from Chủ nhà
       [DEBT]     = tech debt from Discovery or retro
       [BUGFIX]   = bug Chủ nhà reported
       [RESEARCH] = needs investigation first
-->

- [ ] **[NEW]** <Item 1 — short summary>
- [ ] **[NEW]** <Item 2>

---

## 🎯 Next sprint: <Sprint name / theme>

> **Trigger:** <When to promote to Active — e.g. "after current sprint done + feedback X".>
> **Theme:** <One sentence describing the sprint's theme.>

<!-- Idea cluster already shaped but not active yet. Can change. -->

- [ ] <Item planned for next sprint>

---

## 🌊 Future waves (low commitment)

> Bigger idea clusters — far-future Phases / Sprints. Subject to change.

- [ ] **<Future Sprint name>** — <high-level description>
  - <sub-bullet>
  - <sub-bullet>

---

## 💡 Open backlog (uncategorized)

> Loose ideas, not yet clustered into a sprint. Cluster when 2-3 items share a theme.

<!-- Chủ nhà dumps ideas here via /idea skill, or edits manually. -->

- [ ] <Idea 1 — what + why + rough estimate>

---

## 🅿️ Park / think more

> Ideas not ripe yet, or already debated without resolution, or soft-rejected (not hard no).

- [ ] <Idea needing research, or one previously debated without conclusion>

---

## ✅ Recently shipped (last 3 sprints)

> Quick reference. Full history → CHANGELOG.md.

<!-- When a sprint finishes, move 1-line summary here. Keep at most 3 recent sprints. -->

- ✅ **<Sprint name>** (DD/MM/YYYY) — <one-line summary>

---

## ❌ Rejected (logged so we don't reconsider)

> Ideas already considered and decided NOT to pursue. Log the reason clearly so 6 months later you don't reconsider.

- **<Idea name>** — rejected DD/MM/YYYY, reason: <brief>

---

## 📌 Maintenance rules

1. **New ideas** → `/idea` skill → auto-appends to "Open backlog" or "Active sprint" depending on classification.
2. **Phiếu shipped** → move item from Active sprint down to "Recently shipped".
3. **Sprint done** → summarize in CHANGELOG.md, BACKLOG keeps only 3 most recent sprints in "Recently shipped".
4. **Discovery debt** → from DISCOVERIES.md → append to "Open backlog" with `[DEBT]` prefix.
5. **Architect rule** (hard): no phiếu for items outside "Active sprint". Chủ nhà must promote first → then Architect writes.
6. **Monthly review** — Chủ nhà reads Park, decides: promote to Open backlog, or move to Rejected with reason.

---

*This file is LIVE. Chủ nhà can edit directly. Architect/Worker only READ — they never edit while writing a phiếu.*
