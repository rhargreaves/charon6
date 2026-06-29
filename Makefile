DOCKER_IMAGE := charon6-ci
CARGO_TEST ?= sudo -E "$(shell which cargo)" test

all: lint build test
.PHONY: all

build:
	cargo build
.PHONY: build

release:
	cargo build --release
.PHONY: release

lint:
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged --all-targets
.PHONY: lint

test:
	$(CARGO_TEST) -- --include-ignored
.PHONY: test

docker-build:
	docker build -t $(DOCKER_IMAGE) .
.PHONY: docker-build

ci: docker-build
	docker run \
		-e CARGO_TEST="cargo test" \
		--rm --cap-add=NET_RAW --cap-add=NET_ADMIN $(DOCKER_IMAGE) \
		sh -c "\
			ip -6 route add local 2001:db8::/32 dev lo && \
			make lint-ci build test"
.PHONY: ci

release-ci: docker-build
	mkdir -p dist
	docker run --rm \
		-v "$(CURDIR)/dist:/dist" \
		$(DOCKER_IMAGE) \
		sh -c "cargo build --release && cp target/release/charon6 /dist/charon6"
.PHONY: release-ci

lint-ci:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
.PHONY: lint-ci
