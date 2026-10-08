#!/usr/bin/env bash
# UserPromptSubmit hook — idea-smell detector (skills dogfood 2026-06-11).
#
# Problem this kills: the /idea skill's trigger is FUZZY ("when Sếp has a new idea" —
# undefinable), so the model invokes it coin-flip (tarot: 13 skills registered, 0 calls;
# model writes BACKLOG directly instead). Per enforce-via-mechanism: convert the fuzzy
# trigger into a deterministic one — regex the USER MESSAGE for idea-smell phrases and
# inject a one-line reminder into context. stdout of a UserPromptSubmit hook (exit 0)
# is added to the model's context.
#
# Tight pattern set ON PURPOSE (false-positive = noise = Sếp tunes it out). Add phrases
# here when a real miss is observed — don't speculate.

INPUT=""
if [ ! -t 0 ]; then INPUT=$(cat || true); fi
[ -z "$INPUT" ] && exit 0

# Pure-shell match on the raw JSON payload (no jq/python dep — mirror block-unsafe-merge).
case "$INPUT" in
  *"ghi vào backlog"*|*"thêm vào backlog"*|*"ghi backlog"*|*"anh nghĩ ra"*|*"anh vừa nghĩ"*|*"tự nhiên anh nghĩ"*|*"ý tưởng mới"*|*"new idea"*|*"add to backlog"*|*"log this idea"*)
    echo "💡 Idea-smell trong message của Sếp → invoke /idea skill (dedup BACKLOG + Sếp click section/tag + date stamp). KHÔNG ghi BACKLOG thẳng tay."
    ;;
esac
exit 0
