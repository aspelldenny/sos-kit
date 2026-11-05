#!/usr/bin/env bash
# worker-guard.sh — PreToolUse hook: chặn cứng các vai không cần canon đọc tài liệu tầm nhìn (MKT-058, 29/09/2026)
#
# How it works:
#   - Hook fires on Read / Glob tool call (xem .claude/settings.json)
#   - Quản đốc ghi tên vai đang chạy vào `.mkt-state/role` trước khi gọi agent, xoá sau khi agent trả việc
#     vd: `mkdir -p .mkt-state && echo tho > .mkt-state/role` … `rm -f .mkt-state/role`
#   - Vai bị chặn canon: `tho`, `data-analyst` (làm theo storyboard / đọc số, không cần giọng nhân vật)
#   - Marker cũ `.mkt-state/worker-active` (mkt-worker, đã bỏ ở MKT-058) vẫn bị chặn như trước để khỏi hở
#   - Các vai còn lại (marketing, bien-kich, dao-dien, nguoi-soat) và Quản đốc: đọc tự do
#
# Marker lives outside .claude/ để YOLO mode không prompt. No external deps (no jq) — pure shell + sed/grep.

set -euo pipefail

ROLE_FILE=".mkt-state/role"
LEGACY_MARKER=".mkt-state/worker-active"

ROLE=""
if [ -f "$ROLE_FILE" ]; then
    ROLE=$(tr -d '[:space:]' < "$ROLE_FILE")
elif [ -f "$LEGACY_MARKER" ]; then
    ROLE="mkt-worker"
fi

case "$ROLE" in
    tho|data-analyst|mkt-worker) ;;   # bị chặn canon
    *) exit 0 ;;                        # không có marker hoặc vai được đọc canon
esac

INPUT_JSON=$(cat)

# Extract path from Read.file_path or Glob.pattern
PATH_ARG=$(echo "$INPUT_JSON" | sed -n 's/.*"file_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
if [ -z "$PATH_ARG" ]; then
    PATH_ARG=$(echo "$INPUT_JSON" | sed -n 's/.*"pattern"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
fi

[ -z "$PATH_ARG" ] && exit 0

NORMALIZED_PATH="${PATH_ARG#./}"

# Canon: bản distill ở root marketing + gốc /tarot/docs (SOUL, NARRATIVE, CHARACTER_CHI_HA, VOICE_SACH_CO, PRODUCT)
case "$NORMALIZED_PATH" in
    PRODUCT.md|*/PRODUCT.md)
        BLOCKED=1; REASON="PRODUCT.md — tài liệu tầm nhìn" ;;
    SOUL.md|*/SOUL.md)
        BLOCKED=1; REASON="SOUL.md — triết lý + 3 hard lines" ;;
    NARRATIVE.md|*/NARRATIVE.md)
        BLOCKED=1; REASON="NARRATIVE.md — triết lý nền" ;;
    CHARACTER.md|*/CHARACTER.md|CHARACTER_*.md|*/CHARACTER_*.md)
        BLOCKED=1; REASON="CHARACTER — giọng nhân vật (đọc sẽ ngấm tone)" ;;
    VOICE_SACH_CO.md|*/VOICE_SACH_CO.md)
        BLOCKED=1; REASON="VOICE_SACH_CO.md — giọng Sách Cổ" ;;
    tickets/TICKET_TEMPLATE.md|*/tickets/TICKET_TEMPLATE.md)
        BLOCKED=1; REASON="ticket template — phiếu là hợp đồng, đừng sửa khuôn" ;;
    *)
        BLOCKED=0 ;;
esac

if [ "${BLOCKED:-0}" = "1" ]; then
    cat >&2 <<EOF
🚫 Vai «$ROLE» không đọc canon

Không đọc được: $PATH_ARG
Lý do: $REASON

Làm thay vậy:
  - Thợ: làm theo STORYBOARD.md + skills/clip/SO_PHONG_CACH.md + skills/clip/MAU.md
  - Data analyst: đọc số ở launch-tracking/ + research/; không cần giọng nhân vật
  - Cần thông tin từ canon → ghi vào báo cáo cho Quản đốc

Nếu mày là Quản đốc (main session) mà bị chặn: marker còn sót, chạy \`rm -f .mkt-state/role .mkt-state/worker-active\`.
EOF
    exit 2
fi

exit 0
