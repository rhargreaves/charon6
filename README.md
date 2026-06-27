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

`make test` runs the unit tests and the end-to-end smoke test. The smoke test
opens an `AF_PACKET` socket, which needs `CAP_NET_RAW`; without that capability
it skips so the suite stays green. Run it on its own with:

```
make smoke
```

## CI

CI runs inside a container so the smoke test gets the `CAP_NET_RAW` capability
it needs. The exact same pipeline that runs in GitHub Actions can be reproduced
locally:

```
make ci
```

This builds the image from `Dockerfile` and runs `make build test` inside a
container started with `--cap-add=NET_RAW`. Inside the dev container, run it
with `sudo` (the Docker socket is owned by root): `sudo make ci`.
