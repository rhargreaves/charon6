# charon6

[![Build & Test](https://github.com/rhargreaves/charon6/actions/workflows/build.yml/badge.svg)](https://github.com/rhargreaves/charon6/actions/workflows/build.yml)

Abusing the IPv6 address space to transmit data covertly.

## Concept

Data is encoded into IPv6 destination addresses and in the form of a series of ICMPv6 echoes or UDP datagrams. The payload lives entirely in the destination address.

## Usage

### Sender

```
charon6 --send --cidr <IPv6 CIDR>
```

- `--send`, `-s` — send mode: read stdin, encode to IPv6 packets.
- `--cidr` (required) — IPv6 `/64` range used to encode/decode destination addresses.
- `--port <N>` — send via UDP to this port instead of ICMP.

By default, packets are sent as ICMPv6 echo requests. Specify `--port` to use UDP instead.

Examples:
```
# send via ICMP
echo -n "hello world" | charon6 --send --cidr 2001:db8::/64

# send via UDP
echo -n "hello world" | charon6 --send --cidr 2001:db8::/64 --port 9999
```

### Receiver

```
charon6 --recv --cidr <IPv6 CIDR>
```

- `--recv`, `-r` — receive mode: decode packets to stdout.
- `--cidr` (required) — IPv6 `/64` range used to encode/decode destination addresses.
- `--port <N>` — listen for UDP on this port instead of ICMP.

By default, the receiver only accepts ICMPv6 echo packets. Specify `--port`
to listen for UDP instead.

Examples:
```
# receive via ICMP
charon6 --recv --cidr 2001:db8::/64

# receive via UDP
charon6 --recv --cidr 2001:db8::/64 --port 9999
```

## Example

First ensure the documentation prefix `2001:db8::/64` is bound to an interface:
```
$ ip -6 route add local 2001:db8::/64 dev lo
```

Terminal 1: Start the receiver:
```
$ charon6 --recv --cidr 2001:db8::/64
```

Terminal 2: Send a message:
```
$ echo -n "hello world" | charon6 --send --cidr 2001:db8::/64
```

Terminal 1 output:
```
hello world
```

## Wire format

Each captured IPv6 packet carries a chunk of the message in the host portion
of its destination address. Locked to `/64`, so the host portion is 8 bytes:

```
+--------------------+--------+--------+---------------------+
| network prefix /64 | seq u8 | len u8 | payload (len bytes) |
| (8 bytes)          |        |        | (0..=6 bytes)       |
+--------------------+--------+--------+---------------------+
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
