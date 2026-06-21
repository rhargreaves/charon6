build:
	cargo fmt --check
	cargo clippy --fix --allow-dirty
	cargo build
.PHONY: build

test:
	cargo test
.PHONY: test
