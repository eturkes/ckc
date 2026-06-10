# dolmen-lsp

Dolmen LSP via opam-distributed `dolmenls`. Covers SMT-LIB 2.6 (.smt2),
TPTP (.p, .tptp), DIMACS (.cnf, .icnf), and Zipperposition (.zf).

Install:
1. `sudo apt-get install -y opam`
2. `opam init --bare --disable-sandboxing --no-setup -y`
3. `opam switch create dolmen-lsp 5.2.0 --no-install -y`
4. `eval $(opam env --switch=dolmen-lsp) && opam install dolmen_lsp -y`
   (opam package name uses an underscore; `dolmen-lsp` with a hyphen
   resolves to nothing.)
5. `install -m755 ~/.opam/dolmen-lsp/bin/dolmenls ~/.local/bin/dolmen-lsp`
   — copy, never symlink: the binary is standalone (libc/libm only), so
   after the copy the ~1.5 GB build tree is removable with
   `rm -rf ~/.opam`. A symlink would dangle on that cleanup.

Notes: tested with `ocaml 5.2.0` + `dolmen_lsp 0.10` on Debian 13 trixie.
The deployed `~/.local/bin/dolmen-lsp` is a standalone copy and the opam
tree is removed; rebuilds start from step 2. `dolmen-lsp --version` exits
with `error: End_of_file` — that is success, not failure: `dolmenls` is
LSP-only and reads stdin.
