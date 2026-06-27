# charon6

[![Build](https://github.com/rhargreaves/charon6/actions/workflows/build.yml/badge.svg)](https://github.com/rhargreaves/charon6/actions/workflows/build.yml)

Abusing the IPv6 address space to covertly transmit data

## Build

```
make build
```

## Test

```
make test
```

`make test` runs the unit tests followed by the end-to-end smoke test. The
smoke test opens an `AF_PACKET` socket and therefore needs `CAP_NET_RAW`, so it
is invoked via `sudo`. Run it on its own with:

```
make smoke
```
