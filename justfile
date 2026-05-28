default:
    @just --list

test:
    cargo test --workspace

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

ci: fmt-check clippy test

demo:
    cargo run --bin ckc -- demo toy-research-kernel --replay --out runs/toy
