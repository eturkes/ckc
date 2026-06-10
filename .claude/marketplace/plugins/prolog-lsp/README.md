# prolog-lsp

SWI-Prolog LSP via the `lsp_server` pack (jamesnvc/lsp_server).

Install:
1. `sudo apt-get install -y swi-prolog-nox swi-prolog-core-packages`
2. `swipl pack install lsp_server -y`
3. Drop a `prolog-lsp` wrapper into `~/.local/bin/` that runs swipl with the
   pack and patches `library(json)` to resolve to `library(http/json)` (the
   Debian package only registers the http-namespaced path). See the project's
   `prolog-lsp` wrapper for the exact invocation.

Notes: tested with `swi-prolog-nox 9.2.9` + `lsp_server 3.16.3` on Debian 13
trixie. The Debian package splits the JSON library into `ext/http/http/`, so a
bare `library(json)` does not resolve; the wrapper adds that directory to the
SWI-Prolog library search path before loading the pack.
