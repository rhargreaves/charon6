# charon6

[![Build & Test](https://github.com/rhargreaves/charon6/actions/workflows/build.yml/badge.svg)](https://github.com/rhargreaves/charon6/actions/workflows/build.yml)

Abusing the IPv6 address space to covertly transmit data

## Usage

```
charon6 [device] --cidr <IPv6 CIDR>
```

- `device` — interface to capture on (defaults to `lo`).
- `--cidr` (required) — IPv6 `/64` range used to decode destination addresses.

```
charon6 lo --cidr 2001:db8::/64
```

Decoded message bytes are written to stdout.

## Wire format

Each captured IPv6 packet carries a chunk of the message in the host portion
of its destination address. Locked to `/64`, so the host portion is 8 bytes:

```
+-------------------+-------+-------+--------------------+
| network prefix /64| seq u8| len u8| payload (len bytes)|
| (8 bytes)         |       |       | (0..=6 bytes)      |
+-------------------+-------+-------+--------------------+
```

- `seq` — reserved for future ordering; ignored for now.
- `len` — 0..=6. A packet with `len < 6` is the terminator of a message;
  the receiver emits the accumulated payload followed by a newline and flushes.
- Frames with `len > 6` or destinations outside the configured CIDR are dropped
  and logged to stderr.

### Current limitations

- No sender/encoder yet — pair with any tool that can target arbitrary IPv6
  destinations (see the e2e test for an example using `UdpSocket`).
- `seq` is ignored; messages must arrive in order.
- No integrity check, no encryption, no multi-message multiplexing.
- Prefix is fixed at `/64`.

## Build

```
make build
```

## Test

Integration tests run with `sudo` to give `CAP_NET_RAW`.

```
make test
```

## CI

The full pipeline (lint + build + tests, including the e2e decode) runs inside
a Docker container with `CAP_NET_RAW`, `CAP_NET_ADMIN`, and a local `/64`
route on `lo`. Reproduce locally with:

```
sudo make ci
```
