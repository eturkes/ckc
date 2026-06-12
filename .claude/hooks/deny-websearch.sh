#!/usr/bin/env bash
# PreToolUse deny for WebSearch (wired in .claude/settings.json): the upstream API
# 400s the tool's forced tool_choice on this model line, so every call fails inline.
# This hook converts an attempt into an instant redirect to the working channels.
# Healed-retest procedure and removal: .agent/memory.md Lessons.
cat <<'EOF'
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"WebSearch is environment-blocked: it 400s upstream on this model line. Run web searches with WebFetch on https://lite.duckduckgo.com/lite/?q=<query>. Targeted lookups: crates.io API via curl -A (403 without UA), GitHub /search/repositories?q=, Wikipedia opensearch. Details + re-test procedure: .agent/memory.md Lessons."}}
EOF
