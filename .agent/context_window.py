#!/usr/bin/env python3
"""Print Claude Code's current context-window usage as `(NNNK/200K)` and nothing else.

Finds the active session's transcript (the most-recently-written JSONL under
~/.claude/projects), reads the last turn's token usage, and prints e.g. `(126K/200K)`.
Startup/baseline overhead is treated as ~constant and is included in the figure.

Self-contained: stdlib only, read-only, takes no arguments. Just run it:
    python3 context_window.py
Edit MAX_TOKENS below if your model's window isn't 200k (e.g. 1_000_000).
Prints `(?/200K)` when usage can't be read yet (before the first API call, or
right after /compact until the next turn).
"""
import os
import glob
import json

MAX_TOKENS = 200_000          # context window size for the model in use
TAIL_BYTES = 262_144          # scan only the file's tail for speed; fall back to full


def k(n):
    return f"{round(n / 1000)}K"


def _scan(lines):
    """Return the last `message.usage` object found across the given lines."""
    usage = None
    for line in lines:
        if '"usage"' not in line:          # cheap pre-filter before JSON parse
            continue
        try:
            msg = json.loads(line).get("message")
        except Exception:
            continue
        if isinstance(msg, dict) and msg.get("usage"):
            usage = msg["usage"]
    return usage


def _lines(path, tail_only):
    with open(path, "rb") as f:
        size = os.path.getsize(path)
        if tail_only and size > TAIL_BYTES:
            f.seek(size - TAIL_BYTES)
            data = f.read()
            nl = data.find(b"\n")            # drop the partial first line
            if nl != -1:
                data = data[nl + 1:]
        else:
            data = f.read()
    return data.decode("utf-8", errors="ignore").splitlines()


def current_input_tokens():
    files = glob.glob(os.path.expanduser("~/.claude/projects/**/*.jsonl"), recursive=True)
    if not files:
        return None
    path = max(files, key=os.path.getmtime)

    try:
        usage = _scan(_lines(path, tail_only=True))
        if usage is None and os.path.getsize(path) > TAIL_BYTES:
            usage = _scan(_lines(path, tail_only=False))   # tail had none; read it all
    except Exception:
        return None

    if not usage:
        return None
    return (
        usage.get("input_tokens", 0)
        + usage.get("cache_creation_input_tokens", 0)
        + usage.get("cache_read_input_tokens", 0)
    )


if __name__ == "__main__":
    used = current_input_tokens()
    print(f"(?/{k(MAX_TOKENS)})" if used is None else f"({k(used)}/{k(MAX_TOKENS)})")
