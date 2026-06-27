FROM mcr.microsoft.com/devcontainers/rust:latest
RUN apt-get update && apt-get install -y --no-install-recommends \
    iproute2 \
    iputils-ping \
    iptables \
    net-tools \
    && rm -rf /var/lib/apt/lists/*
RUN ip -6 route add local 2001:db8::/64 dev lo
