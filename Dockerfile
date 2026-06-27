FROM rust:1.96.0-trixie

RUN rustup component add clippy rustfmt

RUN apt-get update && apt-get install -y --no-install-recommends \
    make \
    iproute2 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
COPY . .
