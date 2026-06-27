FROM rust:1.96.0-bookworm

RUN rustup component add clippy rustfmt

RUN apt-get update && apt-get install -y --no-install-recommends \
    make \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
COPY . .
