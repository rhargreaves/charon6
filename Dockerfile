FROM rust:1.96.0-trixie
RUN rustup component add clippy rustfmt
RUN apt-get update && apt-get install -y --no-install-recommends \
    make \
    iproute2 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo fetch && rm -rf src
COPY . .
