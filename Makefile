DOCKER_IMAGE := charon6-ci

build:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo build
.PHONY: build

test:
	cargo test
.PHONY: test

smoke:
	cargo test --test smoke
.PHONY: smoke

ci:
	docker build -t $(DOCKER_IMAGE) .
	docker run --rm --cap-add=NET_RAW $(DOCKER_IMAGE) make build test
.PHONY: ci
