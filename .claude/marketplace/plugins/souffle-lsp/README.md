# souffle-lsp

Soufflé Datalog LSP via [jdaridis/souffle-lsp-plugin](https://github.com/jdaridis/souffle-lsp-plugin).

Install:
1. `sudo apt-get install -y openjdk-21-jre-headless`
2. Install Soufflé (Debian 13 needs libffi7 from Debian snapshot, since the
   official package targets Ubuntu 20.04 with libffi7):
   ```
   sudo wget -q https://souffle-lang.github.io/ppa/souffle-key.public \
     -O /usr/share/keyrings/souffle-archive-keyring.gpg
   echo "deb [signed-by=/usr/share/keyrings/souffle-archive-keyring.gpg] \
     https://souffle-lang.github.io/ppa/ubuntu/ stable main" \
     | sudo tee /etc/apt/sources.list.d/souffle.list
   sudo apt-get update
   curl -sL -o /tmp/libffi7.deb \
     http://snapshot.debian.org/archive/debian/20210602T144247Z/pool/main/libf/libffi/libffi7_3.3-6_amd64.deb
   sudo dpkg -i /tmp/libffi7.deb
   sudo apt-get install -y souffle
   ```
3. Build the LSP jar:
   ```
   git clone https://github.com/jdaridis/souffle-lsp-plugin.git
   cd souffle-lsp-plugin
   ./gradlew jar
   install -m 644 build/libs/Souffle_Ide_Plugin-1.0-SNAPSHOT.jar \
     ~/.local/share/souffle-lsp/souffle-lsp.jar
   ```
4. Drop a `souffle-lsp` wrapper into `~/.local/bin/` that runs
   `java -jar ~/.local/share/souffle-lsp/souffle-lsp.jar "$@"`.

Notes: tested with `souffle 2.4` + Soufflé LSP `1.0-SNAPSHOT` (commit at
clone time) on Debian 13 trixie. The LSP uses an in-process ANTLR parser
for most features; it also shells out to `souffle-lint` for diagnostics
(separately installable; basic LSP features work without it).
