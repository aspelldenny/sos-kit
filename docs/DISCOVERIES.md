# Discoveries — sos-kit

> **Index file.** Per-phiếu Discovery Reports live at `docs/discoveries/P<NNN>.md` (one file per phiếu). This file = chronological index, newest on top.
>
> **Why per-file:** monolithic file phình to 100k+ bytes after ~30 phiếu. Architect's auto-Read of full file wastes tokens on irrelevant content. Per-phiếu = Architect Read selective (only when current phiếu references the same component).
>
> **Pre-2026-05 entries:** archived to `docs/archive/DISCOVERIES_pre-2026-05.md`. Migration date: 2026-05-02 (P038).

## 2026-06-11 — Hot-swap subagent model giữa chừng (IG-02, inv-gate Sprint 1)

Đổi model agent đang chạy nền (opus → fable): `TaskStop` agent cũ → sửa frontmatter → respawn prompt y hệt. Sạch — agent cũ kịp trả partial findings, không mất state. Pattern chuẩn cho hot-swap: stop-edit-respawn. (Nguồn: inv-gate `docs/SOS_KIT_FEEDBACK.md` IG-02.)

## Index

| Phiếu | Date | 1-line summary |
|---|---|---|
| [P072](discoveries/P072.md) | 2026-06-15 | Fleet Node20 bump: all 10 repos rc-oracle PASS (3/3 green, 0 annotations, prerelease=true, latest unchanged); guard/vps/doc-rotate/advisory-cron upgraded to full draft/publish+sha256 template (P071+P072 combined for optionals); P009 oracle gap documented (github-script@v7 invisible to P009 because P071 added it after P009) |
| [P071-stage1](discoveries/P071-stage1.md) | 2026-06-15 | Stage 1 fan-out: .sha256 publish pattern applied to 5 REQUIRED repos (claude-hooks 0.9.2, docs-gate/ship/advisory-inbox/inv-gate 0.1.1); all CI GREEN; all 3 targets carry .sha256; inv-gate @v3 draft/publish confirmed working; Stage 2 gate condition met |
| [P052](discoveries/P052.md) | 2026-06-08 | git-level .env* block gate — `scripts/block-env-commit.sh` + pre-commit [7/7]; 9/9 fire-test cases PASS; .envrc deliberately excluded (O1.1 locked as test); regex verbatim with block-env-edit.sh; Tầng 1 docs updated |
| [P053](discoveries/P053.md) | 2026-06-06 | sentinel deadlock fix — PR mode always-post sentinel incl. clean APPROVE; dry-run grep simulation PASS; stale-sentinel documented + [P055] filed; Tầng 1 docs updated (boundary-check.md, README.md) |
| [P050](discoveries/P050.md) | 2026-06-06 | no-code-on-default gate — `scripts/no-code-on-default.sh` + pre-commit [6/6]; 17/17 test cases PASS; .gitignore dir-pattern vs glob-form self-adapted (Tầng 2); all 6 ket-harvest constraints present; Tầng 1 docs updated |
| [P042](discoveries/P042.md) | 2026-05-25 | Giám sát (boundary-check) — 5 generic INV port from tarot (drops nginx+credits), sentinel rename, ADVISORY-only; Anchor #18 Bash allowlist confirmed non-blocking; 0 tarot refs; Wave 1 sprint COMPLETE |
| [P041](discoveries/P041.md) | 2026-05-25 | Trinh sát (advisory-watch) — generic port from tarot; pnpm+npm parsers shipped; CHALLENGE caught 3 real V1→V2 issues (PyYAML, parser location, Bash scope); 0 tarot refs in delivered files |
| [P043](discoveries/P043.md) | 2026-05-25 | Doc drift consolidate — Quản đốc persona codify + alignment engineering + deferred-tool loading; tarot skeletons self-contained; CHALLENGE caught concrete Thay bằng gap V1→V2 |
| [P040](discoveries/P040.md) | 2026-05-25 | Bootstrap stack detection — `sos init security` + 6 parser stubs; .gitignore __pycache__ expansion; CHALLENGE caught dash/underscore Tầng 1 issue pre-EXECUTE |
| [P005](discoveries/P005.md) | 2026-05-10 | Skills are Orchestrator-only — option B locked (frozen-artifact pattern in phiếu Context) |
| [P006](discoveries/P006.md) | 2026-05-10 | docs-gate bootstrap: missing-config guard in hook + dogfood config + template |
| [P039](discoveries/P039.md) | 2026-05-05 | Doc drift + symmetry sweep (10 surgical edits, Tầng 2; orchestrator-fetch lesson) |
| [P038](discoveries/P038.md) | 2026-05-02 | Phiếu lifecycle cleanup + safety rails + DISCOVERIES decoupling |
| [P037](archive/DISCOVERIES_pre-2026-05.md#p037----2026-04-27----pre-approve-marker-bash-ops-permission-prompt-fix----first-skip-challenge-phiếu) | 2026-04-27 | Marker file pre-approve template (skip-CHALLENGE first dogfood) |
| [P036](archive/DISCOVERIES_pre-2026-05.md#p036----2026-04-27----tier-routing--architect-humility-markers--path-drift-fixes-v2) | 2026-04-27 | Tier routing + humility markers + path-drift fixes |
| [P035](archive/DISCOVERIES_pre-2026-05.md#p035----2026-04-27----orchestrator-handbook--bulk-input-rule--install-anti-patterns-v3) | 2026-04-27 | Orchestrator handbook + bulk-input rule |
| [P004](archive/DISCOVERIES_pre-2026-05.md#p004----2026-04-26----vision-doc-naming-flex-charactermd-glob) | 2026-04-26 | Vision doc naming flex (CHARACTER*.md glob) |
| [P003](archive/DISCOVERIES_pre-2026-05.md#p003----2026-04-26----backlog-format-flexibility-banner-fallback--architect-rule-0--orchestrationmd) | 2026-04-26 | BACKLOG format flexibility |
| [v2.1-dogfood](archive/DISCOVERIES_pre-2026-05.md#v21-dogfood----2026-04-26----debate-flow-proven-on-tarot-p029--p030) | 2026-04-26 | Debate flow proven on Tarot (P029 + P030) |
| [P002](archive/DISCOVERIES_pre-2026-05.md#p002----2026-04-26----tarot-voicecharacter-template-harvest) | 2026-04-26 | Tarot voice/character template harvest |
| [P001](archive/DISCOVERIES_pre-2026-05.md#p001----2026-04-26----architect--worker-debate-loop) | 2026-04-26 | Architect ↔ Worker debate loop (initial framework) |

<!-- Future entries: 1 line per phiếu, link to per-file. Worker on EXECUTE adds entry as last step before commit. -->
