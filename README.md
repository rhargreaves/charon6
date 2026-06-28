# charon6

[![Build & Test](https://github.com/rhargreaves/charon6/actions/workflows/build.yml/badge.svg)](https://github.com/rhargreaves/charon6/actions/workflows/build.yml)

Abusing the IPv6 address space to transmit data covertly.

<p align="center">
    <img src="https://github.com/rhargreaves/charon6/raw/main/docs/charon.jpg" />
</p>

> This stream an old man tends, clad in foul garb and to the sight abhorrent, and ferries over the quaking shades.
> &mdash; *Virgil's Aeneid (Book VI)*

## Concept

Data is encoded into IPv6 destination addresses and sent as ICMPv6 echo requests
or UDP datagrams. The payload lives entirely in the destination addresses of packets.
There is optional encryption & message integrity checking.

## Usage

### Sender

```
charon6 --send --cidr <IPv6 CIDR>
```

- `--send`, `-s` — send mode: encode stdin to packets.
- `--cidr` (required) — IPv6 `/64` range used to encode/decode destination addresses.
- `--port <N>` — send via UDP to this port instead of ICMP.
- `--key <passphrase>` — encrypt with passphrase.

By default, packets are sent as ICMPv6 echo requests. Specify `--port` to use
UDP instead. Specify `--key` to encrypt the payload.

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
- `--key <passphrase>` — decrypt with passphrase.

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
payload) is encrypted with [XTEA](https://en.wikipedia.org/wiki/XTEA) in ECB mode
before embedding in thedestination address.

- **Cipher:** XTEA: 64-bit block cipher with 128-bit key, 64 Feistel rounds.
- **Key derivation:** The passphrase is hashed with SHA-256 and truncated to
  128 bits (16 bytes) for the XTEA key.
- **Per-packet:** Each packet is encrypted and decrypted independently,
  allowing out-of-order reassembly. The receiver decrypts first, then uses
  the recovered `seq` field to reorder.
- **What is hidden:** An observer cannot see the sequence number, payload
  length, or payload content.
- **Integrity:** A 16-byte HMAC-SHA256 tag is appended to the message before
  encryption. The receiver verifies the tag after decryption and reassembly;
  tampered or wrong-key messages are dropped (but logged to stderr). The HMAC adds ~3
  packets of overhead per message.

### Limitations

- **ECB mode:** Identical plaintext blocks produce identical ciphertext.
  Within a single message this does not occur (seq always differs), but
  across messages it is theoretically possible.
- Prefix is fixed at `/64`.
- Max 256 packets per message.

## Build

```
make build
```

## Test

Integration tests run with `sudo` to give `CAP_NET_RAW` capability.

```
make test
```
