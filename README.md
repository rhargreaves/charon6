# charon6

[![Build & Test](https://github.com/rhargreaves/charon6/actions/workflows/build.yml/badge.svg)](https://github.com/rhargreaves/charon6/actions/workflows/build.yml)

Abusing the IPv6 address space to transmit data covertly.

## Concept

Data is encoded into IPv6 destination addresses and sent as ICMPv6 echo requests
or UDP datagrams. The payload lives entirely in the destination address — packet
bodies are empty. Optional XTEA encryption hides the sequence number, length,
and payload from observers.

## Usage

### Sender

```
charon6 --send --cidr <IPv6 CIDR>
```

- `--send`, `-s` — send mode: read stdin, encode to IPv6 packets.
- `--cidr` (required) — IPv6 `/64` range used to encode/decode destination addresses.
- `--port <N>` — send via UDP to this port instead of ICMP.
- `--key <passphrase>` — encrypt with XTEA (must match on receiver).

By default, packets are sent as ICMPv6 echo requests. Specify `--port` to use
UDP instead. Specify `--key` to encrypt the host portion of each address.

```
echo -n "hello world" | charon6 --send --cidr 2001:db8::/64
echo -n "hello world" | charon6 --send --cidr 2001:db8::/64 --key secret
echo -n "hello world" | charon6 --send --cidr 2001:db8::/64 --port 9999
```

### Receiver

```
charon6 --recv --cidr <IPv6 CIDR>
```

- `--recv`, `-r` — receive mode: decode packets to stdout.
- `--cidr` (required) — IPv6 `/64` range used to encode/decode destination addresses.
- `--port <N>` — listen for UDP on this port instead of ICMP.
- `--key <passphrase>` — decrypt with XTEA (must match sender).

```
charon6 --recv --cidr 2001:db8::/64
charon6 --recv --cidr 2001:db8::/64 --key secret
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

### Encryption

When `--key` is provided, the entire 8-byte host portion (seq, len, and
payload) is encrypted with the XTEA block cipher before embedding in the
destination address. The receiver decrypts each packet independently before
reassembly. The passphrase must match on both sides.

### Current limitations

- No integrity check or multi-message multiplexing.
- XTEA-ECB: identical plaintext blocks produce identical ciphertext across
  messages (within a message, seq always differs so this does not occur).
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
