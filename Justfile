default:
    @just --list

connect path:
    cargo run --example cli -- {{path}} --query-at-start

dev log_level='trace':
    cargo build && sudo RUST_LOG={{log_level}} ./target/debug/nm-mon --dev
