# Discoveries — sos-kit

> **Index file.** Per-phiếu Discovery Reports live at `docs/discoveries/P<NNN>.md` (one file per phiếu). This file = chronological index, newest on top.
>
> **Why per-file:** monolithic file phình to 100k+ bytes after ~30 phiếu. Architect's auto-Read of full file wastes tokens on irrelevant content. Per-phiếu = Architect Read selective (only when current phiếu references the same component).
>
> **Pre-2026-05 entries:** archived to `docs/archive/DISCOVERIES_pre-2026-05.md`. Migration date: 2026-05-02 (P038).

## 2026-07-22 — P083: doc-drift cleanup (README install + LAYERS v1-residue + tagline)
External kit review found 3 real drift, orchestrator verified, Worker CHALLENGE accepted V1 (anchors #2/#4/#8 confirmed 5-living/8-attic split + architect.md envelope + LAYERS skill table location). Fixed: F3 README+SETUP.md skill-install blocks (12→5 living, fresh-clone was FAIL); F2 LAYERS.md v1 residue (`:48,76,84-85,91-92,104,115,166,195-198`) re-framed to v2 in-session subagent model, kept Tầng-def + Chủ nhà's Web/Code row untouched; RELAY_PROTOCOL.md gained v2-mode banner; HANDOFF.md Handoff 3 key-constraint line (`:146`) updated to point at in-session Debate Log path, 7 other v1-framed lines (`:31,35,47,70,163,208,258`) deferred with written reason (partial-fix, not oversight — see report). F1 README tagline → Sếp-approved Option A ("from code-ready to production health"). Docs-only, no runtime touched. Full report: `docs/discoveries/P083.md`.

## 2026-07-22 — P077b: Crate boundary carve + dependency-direction gate
Second sub-phiếu of P077 decomposition. Additive/reversible, `bin/sos.sh` untouched. Carved flat `bootstrap/sos-rs` crate into virtual workspace (`members=["crates/*"]`) with 5 crates: `sos-core` (lifted `state.rs`, confirmed host-neutral — no CLAUDE_*/`.claude`/clap tokens), `sos-cli` (composition root, binary `sos`, lifted `main.rs`+`commands/`, rewired `use crate::state`→`use sos_core::state`), and 3 empty skeletons (`sos-install`/`sos-adapter-claude`/`sos-hooks`, logic in P077d). Dependency-direction gate: compiler graph (sos-core zero adapter dep) + guard test (`crates/sos-core/tests/dep_direction.rs`); negative-test proof ran (`use sos_adapter_claude;` in core → `cargo build` FAIL with E0432 → reverted) — confirmed one-way enforcement. Parity harness + golden fixtures moved together into `crates/sos-cli/tests/` (byte-identical, pure rename) — resolved Anchor #7 with zero path-string edits. `cargo build --workspace` + `cargo test --workspace` both green. 3 known target deviations confirmed, no new ones. Full report: `docs/discoveries/P077b.md`.

## 2026-07-22 — P077a: Rust workspace scaffold + freeze Bash golden oracle + parity-harness skeleton
First sub-phiếu of P077 decomposition (`docs/plans/P077-decomposition.md`). Additive-only, `bin/sos.sh` untouched (`git diff` empty, remains canonical through P077a-d). Added transitional `[workspace]` shell to `bootstrap/sos-rs/Cargo.toml` (root at `bootstrap/sos-rs/`, not repo-root); froze Bash golden oracle for `new/adopt/map/sync` (`tests/golden/*.golden` + `capture.sh` reproduction script) with normalization for absolute paths, dates, and a found-during-execute filesystem-enumeration-order flap in `new`'s TODO list (`bin/sos.sh:583` unsorted `grep -rl`) — 2 independent capture runs verified byte-identical; parity-harness skeleton `tests/parity.rs` reports "not yet parity" per command, stays green (`HARD_FAIL` const flips at P077c). CHALLENGE-surfaced refinement: `sos sync`'s classification depends on sos-kit's OWN git history (`_blob_in_history`), not just fixture state — documented HEAD-pin requirement in `tests/README.md`. Full report: `docs/discoveries/P077a.md`.

## 2026-07-22 — P076: Claude adapter parity — declarative boundary
Sếp confirmed (B) declarative at ESCALATE GATE (physical move infeasible without breaking golden parity or a plan-forbidden temp renderer). Shipped `adapters/claude/README.md` + `MAPPING.md` (artifact → `core/ROLES.md#<role_id>` mapping, covers golden sections 1-6,8; added `.mcp.json` row per Worker CHALLENGE completeness note) + body-only provenance markers on 11 files (5 agents + README + 5 skills, zero frontmatter touch). Golden parity re-run post-execute: sections 1,2,3,4,6,8 diff = 0. One Tầng-2 self-adapt: reworded `core/ASSETS.md` note to drop literal `adapters/` path string — literal wording would have failed the phiếu's own regression check (constraint 7, no core→adapter reference). Zero physical file move; render deferred P077. Full report: `docs/discoveries/P076.md`.

## 2026-07-22 — P082: Lane-field template drift fix (OA-01)
`phieu/TICKET_TEMPLATE.md` had `**Tầng:**` but no `**Lane:**` field, so every phiếu born from it exited 2 (missing field) on `doctor lane-check` — the pre-CHALLENGE lane budget gate was blind on all canonical phiếu. Added the field (bare `Normal` token) + `scripts/lane-check-contract.sh` (degraded warn-skip) wired as sub-check `3f` in pre-commit `[3/8]` (no new phase). Oracle-verified pre-fix exit 2 → post-fix exit 0 (181 lines/3 anchors/2 constraints, well under Normal budget — no F1/F2 needed). Full report: `docs/discoveries/P082.md`.

## 2026-07-20 — P075: Portable SOS Core extracted
Six neutral contract files now own roles, workflow, policy and asset classes. Capability vocabulary replaces host tool names; one ticket must reach verified remote delivery before the next executes. Zero runtime-token oracle passes. Full report: `docs/discoveries/P075.md`.

## 2026-07-20 — P074: Runtime boundary + monorepo ownership locked
Chủ nhà chọn A: một `sos-kit` monorepo, một version và một `sos` entrypoint; portable core + Claude/Codex adapters là module riêng. Inventory xác nhận Claude coupling lan ngoài `.claude/**`; one-command tool install sẽ do manifest quản lý. Full report: `docs/discoveries/P074.md`.

## 2026-06-15 — P073: Trust gate shipped (baseline-diff + hidden-unicode + SECURITY.md, 20 surfaces)
Unicode pre-fix: 2 raw U+FEFF escaped (DISCOVERY_PROTOCOL.md:196 + P073-trust-gate.md:45); re-scan 0. grep-P BSD gap: switched to byte-pattern `LC_ALL=C grep -e $'\x...'` fallback. Baseline: 20 surfaces (5 non-script + 11 scripts incl trust-gate.sh + install.sh + phieu.sh + setup-dev.sh). All 4 discrimination tests PASS. Phase count bumped to [8/8] everywhere.
Full report: `docs/discoveries/P073.md`

## 2026-06-15 — P071 Stage 2: install.sh sha256 fetch-verify shipped (mac PASS; Linux+Win pending)
Full report: `docs/discoveries/P071.md`. macOS has `/sbin/sha256sum` (Darwin 1.0) — probe selects sha256sum (not shasum) on mac; shasum is the fallback. 3-case discrimination: good binary PASS, corrupted ABORT, missing-.sha256-required ABORT, missing-.sha256-optional WARN. INVARIANTS.md INV-LOCAL candidate → INV-LOCAL-1 active. SETUP.md N/A.

## 2026-06-11 — Hot-swap subagent model giữa chừng (IG-02, inv-gate Sprint 1)

Đổi model agent đang chạy nền (opus → fable): `TaskStop` agent cũ → sửa frontmatter → respawn prompt y hệt. Sạch — agent cũ kịp trả partial findings, không mất state. Pattern chuẩn cho hot-swap: stop-edit-respawn. (Nguồn: inv-gate `docs/SOS_KIT_FEEDBACK.md` IG-02.)

## Index

| Phiếu | Date | 1-line summary |
|---|---|---|
| [P075](discoveries/P075.md) | 2026-07-20 | Portable core: six neutral contract files, capability-based roles, lifecycle/policy/asset ownership, one-ticket/one-remote-delivery rule; zero runtime-token oracle passes |
| [P074](discoveries/P074.md) | 2026-07-20 | Runtime boundary inventory; ownership A locked: one monorepo/version/`sos`, modular core + Claude/Codex adapters; managed tool manifest preserves one-command UX without big-bang source merge |
| [P073](discoveries/P073.md) | 2026-06-15 | Trust gate: 20-surface baseline-diff + hidden-unicode gate; 2 U+FEFF escaped pre-enable; BSD grep-P gap → byte-pattern fallback; 4 discrimination tests PASS; [8/8] phase wired; SECURITY.md written |
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
