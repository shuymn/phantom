.PHONY: test lint format check

test:
	cargo test --all-features

lint:
	cargo clippy -- -D warnings

format:
	cargo fmt --all

check:
	cargo check --all-targets --all-features
