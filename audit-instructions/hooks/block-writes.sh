#!/usr/bin/env bash
# Claude Code PreToolUse hook: audit je READ-ONLY.
# Povolen zápis pouze do lean-audit/out/ (a scratch). Vše ostatní se blokuje.
# Registrace v .claude/settings.json:
# {
#   "hooks": {
#     "PreToolUse": [
#       { "matcher": "Edit|Write|MultiEdit|NotebookEdit|Bash",
#         "hooks": [ { "type": "command", "command": "bash lean-audit/hooks/block-writes.sh" } ] }
#     ]
#   }
# }
set -euo pipefail
INPUT="$(cat)"
TOOL="$(printf '%s' "$INPUT" | jq -r '.tool_name // empty')"
ALLOW_PREFIX="lean-audit/out/"

deny() { printf '{"decision":"block","reason":"%s"}\n' "$1"; exit 0; }

case "$TOOL" in
  Edit|Write|MultiEdit|NotebookEdit)
    FP="$(printf '%s' "$INPUT" | jq -r '.tool_input.file_path // .tool_input.notebook_path // empty')"
    # normalizace na relativní cestu vůči repu
    REL="${FP#"$PWD"/}"
    case "$REL" in
      "$ALLOW_PREFIX"*|/tmp/*|*/scratchpad/*) exit 0 ;;
      *) deny "LEAN AUDIT je read-only. Zapis mimo ${ALLOW_PREFIX} je zakazan: ${REL}" ;;
    esac
    ;;
  Bash)
    CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty')"
    # zakázané vzory: cokoli, co mění strom nebo git historii
    if printf '%s' "$CMD" | grep -Eq '(^|[;&| ])(rm|mv|cp|sed -i|perl -i|tee|truncate|chmod|chown|composer (update|require|remove)|npm (i|install|update)|git (commit|checkout|reset|clean|stash|rebase|merge|push|rm|mv|apply|cherry-pick)|rector(?! .*--dry-run)|php-cs-fixer fix|phpcbf)( |$)'; then
      # výjimka: cílí-li příkaz výhradně do lean-audit/out
      if printf '%s' "$CMD" | grep -Eq 'lean-audit/out' && ! printf '%s' "$CMD" | grep -Eq '(src|app|lib|modules|tests)/'; then
        exit 0
      fi
      deny "LEAN AUDIT je read-only. Prikaz meni repo nebo git: ${CMD}"
    fi
    # redirect zápisu mimo out/
    if printf '%s' "$CMD" | grep -Eq '>>?[[:space:]]*[^ ]+' && ! printf '%s' "$CMD" | grep -Eq '>>?[[:space:]]*(lean-audit/out|/tmp|/dev/null)'; then
      deny "LEAN AUDIT: presmerovani vystupu mimo lean-audit/out je zakazano"
    fi
    exit 0
    ;;
  *) exit 0 ;;
esac
