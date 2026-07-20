# Next-session plan — P076 Claude Code adapter parity

> **Status:** PLAN ONLY — chưa mở phiếu, chưa tạo branch, chưa thay runtime behavior.
> **Dependency:** P075 merged at `462a1e8` on 2026-07-20.

## Mục tiêu

Tách toàn bộ wiring riêng của Claude Code thành một adapter có mapping rõ về `SOS.md` + `core/**`, trong khi hành vi người dùng và enforcement trước/sau không đổi.

## Thứ tự phiên sau

1. **Freeze golden oracle**
   - Chụp tracked file/mode/symlink inventory cho `CLAUDE.md`, `.claude/**`, `agents/**`, `skills/**` và lifecycle scripts.
   - Ghi capability matrix: role → tools/model/background; event → matcher/script; skill/command registration; env/payload assumptions.
   - Chạy oracle hiện hữu cho hook exit codes, CLI help/new/adopt/sync và doctor connectivity.

2. **Viết phiếu P076 và CHALLENGE trước code**
   - Chốt exact edit envelope và adapter target layout dưới `adapters/claude/`.
   - Mọi mapping phải trỏ về stable role/policy/workflow ID trong portable core.
   - Chọn cách giữ installed paths tương thích mà không tạo doctrine copy thứ hai.

3. **Tách adapter sources**
   - Runtime entry instructions, registration, frontmatter, lifecycle event/tool/env mapping thuộc adapter.
   - Semantic role/workflow/policy chỉ thuộc portable core; nếu golden handbook có semantics còn thiếu, bổ sung core trước rồi adapter chỉ tham chiếu/render.
   - Không viết temporary renderer mà P077 sẽ phải vứt bỏ.

4. **Parity verification**
   - So sánh normalized content/permissions/settings và symlink topology.
   - Fire-test lifecycle hooks với payload hợp lệ/sai; giữ nguyên allow/block/exit behavior.
   - Verify agents, skills, commands và MCP/tool registration vẫn discoverable.
   - `sos new`, `sos adopt`, `sos sync` và `doctor` giữ golden outcome.

5. **Ship riêng P076**
   - Gate xanh → discovery/changelog/backlog → commit → push/merge → verify remote → xóa branch.
   - Chỉ sau đó mới mở P077 Rust adapter framework.

## Ngoài scope

- Không viết Codex adapter.
- Không dựng Rust workspace/installer manifest/rollback engine (P077).
- Không đóng gói npm/pnpm hoặc release binary (P081).
- Không sửa doctrine lịch sử để làm grep đẹp.

## Điểm phải dừng hỏi founder

- Golden behavior và portable core mâu thuẫn về semantics, không chỉ serialization.
- Có hai cách installed layout khác nhau làm thay đổi UX hoặc compatibility.
- Parity đòi giữ một behavior không an toàn, hoặc bỏ một behavior người dùng đang dựa vào.
- Cần thay đổi order/scope P076→P077 đã chốt trong architecture.

## Definition of done dự kiến

- Claude-specific source/wiring có owner duy nhất dưới adapter boundary.
- Portable core vẫn zero runtime-token.
- Golden parity suite xanh; không có user-visible regression.
- Existing project fresh/adopt/sync đều hoạt động.
- P076 được merge và remote-main hash xác nhận trước khi bắt đầu P077.
