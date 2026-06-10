#!/usr/bin/env bash
# lexgate: keeps the durable tree in project vocabulary.
# Patterns: .agent/lexgate.d/list -- local-only, Read-denied, user-maintained;
# sessions interact with this gate as pass/fail only. Reports cite file:line,
# matched text is never echoed. docs/ and corpus/fixtures/ are sanctioned containers.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
list="$root/.agent/lexgate.d/list"
msg="reword to project vocabulary (lexgate; patterns are user-maintained)"

skip() {
  case "$1" in
    docs/*|corpus/fixtures/*|.agent/lexgate.d/*|.git/*|target/*) return 0 ;;
  esac
  return 1
}

scan() { # $1 = root-relative path; nonzero on hit, stderr cites file:lines only
  local f="$1" hits
  skip "$f" && return 0
  [ -f "$root/$f" ] || return 0
  hits="$(grep -inE -f "$list" -- "$root/$f" 2>/dev/null | cut -d: -f1 | paste -sd, -)" || true
  if [ -n "$hits" ]; then
    echo "lexgate: $f:$hits $msg" >&2
    return 1
  fi
  return 0
}

need_list() {
  [ -s "$list" ] || {
    echo "lexgate: pattern list missing; stop and ask the user to restore .agent/lexgate.d/list" >&2
    exit 2
  }
}

case "${1:-}" in
  hook) # PostToolUse: JSON on stdin
    need_list
    f="$(jq -r '.tool_input.file_path // empty' 2>/dev/null || true)"
    [ -n "$f" ] || exit 0
    scan "${f#"$root"/}" || exit 2
    ;;
  pre-commit)
    need_list
    ok=0
    while IFS= read -r f; do scan "$f" || ok=1; done \
      < <(git -C "$root" diff --cached --name-only --diff-filter=ACM)
    exit "$ok"
    ;;
  sweep)
    need_list
    ok=0
    while IFS= read -r f; do scan "$f" || ok=1; done < <(git -C "$root" ls-files)
    [ "$ok" -eq 0 ] && echo "lexgate: sweep clean"
    exit "$ok"
    ;;
  scan)
    need_list
    scan "${2:?usage: lexgate.sh scan <root-relative-path>}"
    ;;
  check)
    miss=""
    [ -s "$list" ] || miss="$miss list"
    grep -q lexgate "$root/.git/hooks/pre-commit" 2>/dev/null || miss="$miss pre-commit-hook"
    grep -q lexgate "$root/.claude/settings.json" 2>/dev/null || miss="$miss settings-hook"
    if [ -n "$miss" ]; then
      echo "lexgate: missing:$miss -- stop and ask the user" >&2
      exit 2
    fi
    echo "lexgate: ok"
    ;;
  install)
    printf '#!/bin/sh\nexec bash "%s/.agent/lexgate.sh" pre-commit\n' "$root" \
      > "$root/.git/hooks/pre-commit"
    chmod +x "$root/.git/hooks/pre-commit"
    echo "lexgate: pre-commit installed"
    ;;
  *)
    echo "usage: lexgate.sh hook|pre-commit|sweep|scan <file>|check|install" >&2
    exit 64
    ;;
esac
