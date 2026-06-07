# xml-lsp

Generic XML LSP via Eclipse LemMinX. Covers DMN, BPMN, SHACL-XML, XSD.

Install:
1. `sudo apt-get install -y openjdk-21-jre-headless`
2. Download the uber jar:
   `curl -sSL -o ~/.local/share/lemminx/lemminx.jar https://repo.eclipse.org/content/repositories/lemminx-releases/org/eclipse/lemminx/org.eclipse.lemminx/0.31.1/org.eclipse.lemminx-0.31.1-uber.jar`
3. Drop a `lemminx` wrapper into `~/.local/bin/` that runs
   `java -jar ~/.local/share/lemminx/lemminx.jar "$@"`.
