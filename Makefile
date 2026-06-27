DOCKER_IMAGE := charon6-ci
CARGO_TEST ?= sudo -E "$(shell which cargo)" test

all: lint build test
.PHONY: all

build:
	cargo build
.PHONY: build

lint:
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged --all-targets
.PHONY: lint

test:
	$(CARGO_TEST)
.PHONY: test

ci:
	docker build -t $(DOCKER_IMAGE) .
	docker run \
		-e CARGO_TEST="cargo test" \
		--rm --cap-add=NET_RAW $(DOCKER_IMAGE) \
		make lint-ci build test
.PHONY: ci

lint-ci:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
.PHONY: lint-ci
