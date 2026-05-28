#!/usr/bin/env python3
"""Print Claude Code's current context-window usage as `(NNNK/200K)`.

Reads the active session transcript (newest JSONL under ~/.claude/projects),
takes the last turn's token usage, and sums input + cache-read + cache-creation
tokens — the figure that fills the window. Prints `(?/200K)` when usage is
unreadable (before the first turn, or just after /compact). Read-only, stdlib
only, no args; edit MAX for a non-200K window.
"""
import glob, json, os

MAX = 200_000
FIELDS = ("input_tokens", "cache_creation_input_tokens", "cache_read_input_tokens")


def fmt(n):
    return f"{round(n / 1000)}K"


def used():
    paths = glob.glob(os.path.expanduser("~/.claude/projects/**/*.jsonl"), recursive=True)
    if not paths:
        return None
    with open(max(paths, key=os.path.getmtime), encoding="utf-8", errors="ignore") as f:
        lines = f.read().splitlines()
    for line in reversed(lines):  # most recent usage first
        if '"usage"' not in line:
            continue
        try:
            u = (json.loads(line).get("message") or {}).get("usage")
        except Exception:
            continue
        if u:
            return sum(u.get(k, 0) for k in FIELDS)
    return None


if __name__ == "__main__":
    n = used()
    print(f"(?/{fmt(MAX)})" if n is None else f"({fmt(n)}/{fmt(MAX)})")
