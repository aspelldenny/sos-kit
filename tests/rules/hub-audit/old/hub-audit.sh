#!/usr/bin/env bash
# Claude Code PostToolUse hook — chuông báo SỚM khi sửa file .md của Hub: chỉ chặn khi có câu trích BỊA chắc chắn.
# Phạm vi: cây sách/Hub ($BOOK) và gói nghiên cứu Hub chép sang dự án (*/docs/research/hub-*/). File khác bỏ qua.
# Chỉ chặn (exit 2) khi `hub audit` in dòng ✘ FABRICATED. "Chưa kết luận được" (INCONCLUSIVE) thì im: kiểm kỹ là
# việc của cổng giao gói `hub audit --strict` (CLAUDE.md), không phải của chuông mỗi lần sửa — chặn nó chỉ dạy
# agent đổi ngoặc kép sang ngoặc đơn cho im (2026-09-24).
HUB="$HOME/.local/bin/hub"; PY="/usr/bin/python3"; BOOK="$HOME/Downloads/Documents/BOOK"
[ -x "$HUB" ] || exit 0
input=$(cat)
fp=$(printf '%s' "$input" | "$PY" -c 'import sys,json
try: print(json.load(sys.stdin).get("tool_input",{}).get("file_path",""))
except Exception: print("")' 2>/dev/null)
[ -n "$fp" ] || exit 0
case "$fp" in *.md|*.markdown) ;; *) exit 0 ;; esac
case "$fp" in "$BOOK/"*|*/docs/research/hub-*/*) ;; *) exit 0 ;; esac   # ngoài Hub (vd ✔/✘ đạt-không-đạt của dự án) → không soát
[ -f "$fp" ] || exit 0
# pre-filter: chỉ soát file CÓ trích kiểu hub (dấu ✔/✘ hoặc [..loc/p./ch./hồi..]); md thường bỏ qua (khỏi tải corpus)
grep -qE '✔|✘|\[[^]]*(loc |p\.[0-9]|pp\.|tr\.|ch\.[0-9]|chapter |hồi |quyển )' "$fp" 2>/dev/null || exit 0
out=$("$HUB" audit "$fp" --quiet 2>/dev/null)
fab=$(printf '%s\n' "$out" | grep -E '^[[:space:]]*✘ FABRICATED')
[ -n "$fab" ] || exit 0   # sạch / chưa kết luận / lỗi lệnh → không chặn
{ echo "🚫 hub audit: câu trích BỊA (không có nguyên văn trong kho) ở $fp — sửa/bỏ trước khi tiếp:";
  printf '%s\n' "$fab";
  echo "(kiểm 1 câu: hub verify \"...\"  ·  trước khi giao: hub audit --strict \"$fp\")"; } >&2
exit 2
