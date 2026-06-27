build:
	cargo fmt --check
	cargo clippy --fix --allow-dirty
	cargo build
.PHONY: build

test:
	cargo test
	$(MAKE) smoke
.PHONY: test

smoke:
	sudo -E "$$(which cargo)" test --test smoke
.PHONY: smoke
