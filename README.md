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

## Example

First bind the documentation prefix `2001:db8::/64` to the loopback interface:
```sh
$ ip -6 route add local 2001:db8::/64 dev lo
```

Terminal 1: Start the receiver:
```
$ charon6 lo --cidr 2001:db8::/64
charon6 started
Opening AF_PACKET socket for device: lo
Listening for IPv6 packets on lo decoding 2001:db8::/64...
...
```

Terminal 2: Send the ping (`2001:db8::9903:6869:2100:0` decodes to `hi!`):
```
$ ping6 2001:db8::9903:6869:2100:0
PING 2001:db8::9903:6869:2100:0 (2001:db8::9903:6869:2100:0) 56 data bytes
64 bytes from 2001:db8::9903:6869:2100:0: icmp_seq=1 ttl=64 time=3.74 ms
64 bytes from 2001:db8::9903:6869:2100:0: icmp_seq=2 ttl=64 time=0.190 ms
...
```

Output (Terminal 1):

```
src=::1 -> dst=2001:db8::9903:6869:2100:0
hi!
src=::1 -> dst=2001:db8::9903:6869:2100:0
hi!
...
```

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
  destinations.
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
