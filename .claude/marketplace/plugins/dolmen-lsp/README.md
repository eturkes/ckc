# dolmen-lsp

Dolmen LSP via opam-distributed `dolmenls`. Covers SMT-LIB 2.6 (.smt2),
TPTP (.p, .tptp), DIMACS (.cnf, .icnf), and Zipperposition (.zf).

Install:
1. `sudo apt-get install -y opam`
2. `opam init --bare --disable-sandboxing --no-setup -y`
3. `opam switch create dolmen-lsp 5.2.0 --no-install -y`
4. `eval $(opam env --switch=dolmen-lsp) && opam install dolmen-lsp -y`
5. Drop a `dolmen-lsp` wrapper into `~/.local/bin/` that runs
   `opam exec --switch=dolmen-lsp -- dolmenls "$@"` (or symlink the
   `dolmenls` binary out of `~/.opam/dolmen-lsp/bin/`).

Notes: tested with `ocaml 5.2.0` + `dolmen-lsp` (latest release at install
time) on Debian 13 trixie. The opam switch holds an OCaml compiler plus the
Dolmen toolchain (~1.5 GB on disk).
