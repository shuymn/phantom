.PHONY: test lint format check

test:
	cargo test --all-features

lint:
	cargo clippy --all-features -- -D warnings

format:
	cargo fmt --all

check-format:
	cargo fmt --all --check

check:
	cargo check --all-targets --all-features
