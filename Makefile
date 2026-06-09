# Makefile — convenience wrappers over the SPEC §6 CLI. Targets become functional
# as their build units land (see .agent/roadmap.md); they are placeholders until
# the underlying crates/commands exist.
.PHONY: test schema registry-check

test:           ## Rust + Python test suites
	cargo test --workspace
	uv run pytest

schema:         ## regenerate committed JSON Schema from Rust types
	uv run ckc schema export --out schemas/

registry-check: ## validate all registries
	uv run ckc registry check
