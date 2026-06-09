# Handoff: kit-level items từ tarot adoption (P343 + P344) — 2026-06-09

> Nguồn: tarot adopt doc-rotate (P343, PR #627 merged) + claude-hooks (P344, PR #628 merged, giám sát APPROVE).
> tarot-side **XONG** — 2 tool chạy OK trong tarot. Tool-level findings đã vứt sang repo tool
> (`~/doc-rotate/docs/TAROT_DOGFOOD_FEEDBACK.md`, `~/claude-hooks/docs/SOS_KIT_FEEDBACK.md`).
> Dưới đây CHỈ các item thuộc **kit** (cross-project / distribution / convention) — không phải việc của tool, không phải việc của tarot.

---

## K-1 — Installer "1 lệnh" cross-OS + cho người không-Rust  🔴 (bước cuối Sếp đặt)

**Evidence từ tarot:** Adopt 2 tool đòi `cargo install` per-tool, mà `cargo install` **giả định Rust đã có sẵn** (giống `npm install` giả định Node — Claude Code cũng cần Node trước). Trên 3 máy Sếp (Mac/Win/Linux) có Rust thì OK; **share cho bạn-không-code thì không** — bắt cài cả Rust toolchain là rào.

**Việc của kit:** cách phân phối "phôi dao" tới nhiều máy/người:
- Publish tool lên crates.io → `cargo install <tool>` mọi OS (vẫn cần Rust).
- **Prebuilt binary** (GitHub Releases, 3 target mac-arm64/win-x64/linux-x64) + `curl … | sh` bootstrap → người-không-Rust chỉ tải về chạy.
- Cross-ref draft sẵn có: `docs/INSTALL.md`, `docs/BOOTSTRAP_AUTOMATION_DRAFT.md`.

## K-2 — `setup-dev.sh` → golden template kit (đừng bespoke mỗi repo)

**Evidence từ tarot:** P344 đẻ ra `scripts/setup-dev.sh` (check Rust→cảnh báo nếu thiếu, `cargo install` cả bộ binary, reinstall doc-rotate cap-check vào `.git/hooks/pre-commit`). Đây là pattern "bootstrap mỗi máy" mà **mọi repo adopt sẽ cần lặp lại**.

**Việc của kit:** nâng setup-dev.sh thành **golden template** ở sos-kit (mỗi repo lắp vào, không tự viết lại). Liên quan K-1 (installer là phần lõi của bootstrap).

## K-3 — Chuẩn hoá tên file handoff dogfood (convention)  ⚪ minor

**Evidence từ tarot:** 2 repo tool đang lệch tên file feedback — doc-rotate nhận `TAROT_DOGFOOD_FEEDBACK.md`, claude-hooks dùng `SOS_KIT_FEEDBACK.md`. Kênh dogfood cross-repo (project → tool → kit) nên có **1 tên + format thống nhất**.

**Việc của kit:** chốt convention 1 tên file + section format cho dogfood feedback, port vào golden template.

---

**Phân vai (không đa nhiệm):**
- tarot: XONG (chạy OK với tool mới).
- doc-rotate / claude-hooks (tool): xử F-* trong file feedback repo mình → blade sắc hơn.
- sos-kit (kit): K-1/K-2/K-3 trên — phân phối + golden template + convention. Thứ tự: **làm SAU khi tool đã ổn ở nhiều dự án** (Sếp: tarot tốt → tool generic → sos-kit cuối).
