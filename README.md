# charon6

[![Build & Test](https://github.com/rhargreaves/charon6/actions/workflows/build.yml/badge.svg)](https://github.com/rhargreaves/charon6/actions/workflows/build.yml)

Abusing the IPv6 address space to covertly transmit data

## Usage

### Receiving (decoding packets)

```
charon6 [device] --recv --cidr <IPv6 CIDR>
```

- `device` — interface to capture on (defaults to `lo`).
- `--recv`, `-r` — receive mode: decode packets to stdout.
- `--cidr` (required) — IPv6 `/64` range used to encode/decode destination addresses.

Packets may arrive out of order; the receiver reassembles them using the `seq` field.

### Sending (encoding stdin to packets)

```
charon6 --send --cidr <IPv6 CIDR>
```

- `--send`, `-s` — send mode: read stdin, encode to IPv6 packets.

Reads all of stdin, splits into 6-byte chunks, and sends each as a UDP packet
to an address within the CIDR range.

## Example

First bind the documentation prefix `2001:db8::/64` to the loopback interface:
```
$ ip -6 route add local 2001:db8::/64 dev lo
```

Terminal 1: Start the receiver:
```
$ charon6 lo --recv --cidr 2001:db8::/64
```

Terminal 2: Send a message:
```
$ echo -n "hello world" | charon6 --send --cidr 2001:db8::/64
```

Output (Terminal 1):
```
hello world
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

- `seq` — sequence number (0-255) for packet reordering.
- `len` — 0..=6. A packet with `len < 6` is the terminator of a message;
  the receiver emits the accumulated payload followed by a newline and flushes.
- Frames with `len > 6` or destinations outside the configured CIDR are dropped
  and logged to stderr.

### Current limitations

- No integrity check, no encryption, no multi-message multiplexing.
- Prefix is fixed at `/64`.
- Max 256 packets per message (seq is u8).

## Build

```
make build
```

## Test

Integration tests run with `sudo` to give `CAP_NET_RAW`.

```
make test
```
