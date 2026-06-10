# lean4-lsp

Lean 4 LSP via `lake serve`.

Runtime install is deferred until the first Lean code lands (Phase 4 per
`.agent/roadmap.md`). At that point install elan and a Lean 4 toolchain:

```
curl https://elan.lean-lang.org/elan-init.sh -sSf | sh -s -- -y
. ~/.elan/env
elan default leanprover/lean4:stable
```

Then add `~/.elan/bin` to PATH (or symlink `lake` into `~/.local/bin`) and the
LSP starts working in any directory containing a `lakefile.lean`.
