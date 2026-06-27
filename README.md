# charon6

[![Build & Test](https://github.com/rhargreaves/charon6/actions/workflows/build.yml/badge.svg)](https://github.com/rhargreaves/charon6/actions/workflows/build.yml)

Abusing the IPv6 address space to covertly transmit data

## Usage

```
charon6 [device] [--cidr <IPv6 CIDR>]
```

- `device` — interface to capture on (defaults to `lo`).
- `--cidr` — only report packets whose source or destination falls within the
  given IPv6 CIDR range. When omitted, all IPv6 packets are reported.

```
charon6 eth0 --cidr 2001:db8::/32
```

## Build

```
make build
```

## Test

Integration tests run with `sudo` to give `CAP_NET_RAW`.

```
make test
```
