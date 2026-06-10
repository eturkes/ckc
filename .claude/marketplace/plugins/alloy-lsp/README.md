# alloy-lsp

Alloy 6 LSP via the `lsp` subcommand of the Alloy distribution jar.

Install:
1. `sudo apt-get install -y openjdk-21-jre-headless`
2. Download the dist jar:
   `curl -sSL -o ~/.local/share/alloy/alloy.jar https://github.com/AlloyTools/org.alloytools.alloy/releases/download/v6.2.0/org.alloytools.alloy.dist.jar`
3. Drop an `alloy-lsp` wrapper into `~/.local/bin/` that runs
   `java -jar ~/.local/share/alloy/alloy.jar lsp "$@"`.
