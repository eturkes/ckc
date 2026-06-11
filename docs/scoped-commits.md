# Scoped Commits

Vendored from scopedcommits.com (fetched 2026-06-11; no LLM-native version published), token-optimized: format spec, examples, and multi-scope rules kept; changelog/ticket/conventional-commits FAQs dropped.

A loose commit-message standard: the scope — the subsystem, area, or module touched — leads, making the log scannable for what areas changed (the question contributors, debuggers, and incident responders all bring to it).

```text
<scope>: <description>

[optional body]

[optional trailer(s)]
```

- `<scope>` — subsystem/area/module the commit touches
- `<description>` — short description of the changes
- body — detailed information; trailers — additional metadata
- Reverts, merges, and other special commits: any format.
- Projects may add rules constraining valid scopes and description/body/trailer formatting.

Examples (Linux, FreeBSD, Git, Go, nixpkgs):

```text
i2c: virtio: mark device ready before registering the adapter
linuxulator: Return EINVAL for invalid inotify flags
gitlab-ci: update macOS image
net/http/cookiejar: add godoc links
xwayland: 24.1.11 -> 24.1.12
```

A commit spanning multiple scopes: use one broader scope that covers them, or comma-separate the scopes, or use `treewide`/`all`/`global` for tree-wide changes; if nothing fits, drop the scope and write a good description.
