# Genesis — 0→1 Pipeline

> **Mục đích:** Empty folder → launchable product. Mở rộng SOS Kit cho phase trước phiếu workflow.
> **Triết lý:** 3 vai như nhau xuyên project. Tool là scaffolder, không đóng vai.

## Pipeline mở rộng

```
[0] VISION   → [1] BLUEPRINT → [2] CONTRACT → [3] SCAFFOLD → [4..N] PHIẾU LOOP → [N+1] LAUNCH
   /init        /blueprint     sos contract    /apply ×N     phiếu workflow      sos launch
   (Chủ nhà)    (Kiến trúc sư) (Kiến trúc sư) (Thợ)         (như cũ)            (gate)
```

| Phase | Skill / command | Vai | Output |
|-------|-----------------|-----|--------|
| 0. Vision | `/init` (skill) | Chủ nhà | `docs/PROJECT.md`, `docs/SOUL.md`, `docs/CHARACTER*.md` (nếu persona, e.g. `docs/CHARACTER_CHI_HA.md`) |
| 1. Blueprint | `sos blueprint` (CLI) | Chủ nhà → Kiến trúc sư | `docs/BLUEPRINT.md` (stack, recipes list) |
| 2. Contract | `sos contract` (CLI) | Kiến trúc sư | `docs/ticket/P000-genesis.md` + spec_hash lock |
| 3. Scaffold | `/apply` ×N (skill) | Thợ | Code chạy được, mỗi recipe = 1 commit + sub-phiếu P000.N |
| 4..N. Iterate | phiếu workflow (như cũ) | Kiến trúc sư + Thợ | Từng phiếu nhỏ tiến tới launch |
| N+1. Launch | `sos launch` (CLI) | Chủ nhà | Production live, retro entry |

## Khái niệm cốt lõi

### Recipe
Đơn vị nhỏ nhất, atomic, composable. 1 file = 1 recipe trong `recipes/<category>/<name>.md`. Xem `recipes/README.md`.

**Không có scaffold cứng.** Mỗi project mix recipes theo blueprint. Combo lạ → forge recipe mới.

### Genesis phiếu (P000)
Phiếu master duy nhất per project. Khác phiếu thường:
- Lock toàn MVP scope bằng `spec_hash` (SHA256)
- Recipe list ordered (Thợ apply tuần tự)
- Launch checklist 20 mục
- Audit trail mọi lần re-lock

### Spec hash
Hash SHA256 của các section "frozen" trong P000 (Vision Anchor, MVP Scope, Tech Commitments). Mục đích:
- Phiếu P001+ verify spec_hash trước khi start → không scope creep ngầm
- Đổi spec_hash → audit trail trong `.sos/state.toml > history`
- 2 phiếu có cùng spec_hash = build cùng "version" của vision

### State machine
File `.sos/state.toml` track phase + applied recipes + history.

```toml
[state]
phase = "VISION_CAPTURED" | "BLUEPRINT_DRAFTED" | "LOCKED" | "SCAFFOLDED" | "ITERATING" | "LAUNCHED"
spec_hash = "sha256:..."
last_updated = "2026-04-25T10:30:00Z"

[vision]
project_name = "..."
has_persona = true

[[applied_recipes]]
name = "infra/docker-compose-postgres"
phieu = "P000.1"
applied_at = "..."
verified = true

[[history]]
event = "contract.lock"
spec_hash = "sha256:..."
timestamp = "..."
by = "Chủ nhà"
reason = "Genesis"
```

## Walkthrough — habit tracker từ 0

```bash
mkdir habit-tracker && cd habit-tracker
sos init
# /init skill chạy:
# - Q1: Project type → utility
# - Q2: Voice/persona → no
# - Q3: Pitch → "Track 1 habit per day, friction-free"
# Generates: docs/PROJECT.md, docs/PRINCIPLES.md, .phieu-counter, P000-genesis.md (draft)
# State: VISION_CAPTURED

sos blueprint
# Wizard:
# - Stack: SvelteKit + SQLite + Cloudflare Workers (combo lạ!)
# - Recipes available: framework-starter/sveltekit (chưa có)
# - sos detects missing recipes → prompts: forge or skip?
# → forge sveltekit (invoke /forge skill — Kiến trúc sư mode)
# - After forge done, blueprint completes
# Generates: docs/BLUEPRINT.md with recipe list
# State: BLUEPRINT_DRAFTED

sos contract
# Reads PROJECT.md, BLUEPRINT.md → generates P000-genesis.md filled
# Asks Chủ nhà to review + approve
# On approve: hash + lock + .sos/state.toml updated
# State: LOCKED

# Now scaffold:
sos apply --all
# OR per recipe:
sos apply framework-starter/sveltekit
sos apply infra/cloudflare-workers   # forge nếu chưa có
sos apply observability/sentry-sveltekit  # forge nếu chưa có
# State after all: SCAFFOLDED

# Project bây giờ có code chạy được. Bắt đầu phiếu loop:
phieu feat habit-create
# /verify → code → /review → /qa → ship → vps stats
# ... lặp đến khi launch checklist 100%

sos launch
# Hard block: check launch checklist 100%, guard pre-deploy, ship canary
# State: LAUNCHED
```

## Tarot — case study (đã đi 0→1, hồi tố)

Tarot không dùng `sos init` (vì SOS Kit ra đời sau). Để retrofit audit trail:

1. Document P000-genesis.md hồi tố từ git log (ZERO-1→5, phase-1→3A, launch-1→11, SOUL-PIVOT, SESSION-MODEL-V1)
2. Vision đã có sẵn: `docs/SOUL.md`, `docs/CHARACTER_CHI_HA.md`, `docs/PROJECT.md` (PRD)
3. Recipes đã extract DNA: `payment/payos-vn`, `ai/multi-model-fallback` (sẽ là DNA cho recipe library)
4. Project tarot tiếp tục dùng phiếu workflow như cũ — không refactor.

## Khi nào dùng Genesis vs phiếu thường

| Tình huống | Pipeline |
|-----------|----------|
| Empty folder, project mới | `sos init` → … → `sos launch` |
| Project đã có code, đã launch, thêm feature | `phieu feat <slug>` (như cũ) — KHÔNG cần Genesis |
| Project đã có code, chưa launch, đang xây MVP | Có thể skip Genesis nếu < 5 phiếu nữa là xong; lập P000 hồi tố nếu sếp muốn audit trail |
| Refactor lớn / đổi stack giữa chừng | Không phải Genesis — viết phiếu refactor thường |

## Khi nào sửa P000 sau khi LOCKED

- **Sửa Core features** (mục 2 của P000): scope creep — phải re-hash + audit. Lý do phải document.
- **Move feature từ Core → Can ship without**: cũng re-hash (scope thay đổi), nhưng OK về mặt discipline.
- **Sửa Tech Commitments** (mục 3): re-hash. Recipe list thay đổi = phải `/apply` recipe mới (không drop recipe đã apply).
- **Sửa Verification Anchors / Launch Checklist** (mục 4, 5): KHÔNG re-hash (working area).

## File overview

```
sos-kit/
├── docs/GENESIS.md              ← bạn đang đọc
├── phieu/
│   ├── GENESIS_TEMPLATE.md      Master phiếu P000 template
│   ├── LAUNCH_CHECKLIST.md      20-mục launch gate
│   └── TICKET_TEMPLATE.md       Phiếu thường (như cũ)
├── recipes/
│   ├── README.md                Library structure + categories
│   ├── _TEMPLATE.md             Template cho recipe mới
│   ├── infra/, auth/, payment/, ai/, observability/, framework-starter/
├── skills/
│   ├── init/SKILL.md            Phase 0 — Chủ nhà vision capture
│   ├── apply/SKILL.md           Phase 3 — Thợ apply 1 recipe
│   ├── forge/SKILL.md           Forge recipe mới — Kiến trúc sư
│   └── plan/SKILL.md            (cập nhật) — plan mode interop
└── bootstrap/
    └── sos-rs/                  Rust binary (skeleton, MVP shell-script trước)
```

## Quy tắc gold

1. **Vision không có ngày hết hạn — recipe có.** Vision viết 1 lần, recipe cập nhật theo dep version.
2. **Mỗi project mới đẻ ra recipe mới.** Save lại — project sau dùng → library tăng tự nhiên.
3. **Spec_hash là máy phát hiện scope creep.** Đổi mục 1-2-3 của P000 → spec_hash đổi → bắt buộc audit + Chủ nhà sign-off.
4. **Hard block ở `sos launch`.** Tốn 1 ngày fix là rẻ; fix sau launch là expensive.
5. **3 vai không bị blur.** Tool không "đóng vai" — Chủ nhà nói, Kiến trúc sư viết, Thợ làm. Tool chỉ scaffolder.
