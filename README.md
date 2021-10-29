# vmemcached

[![CI](https://github.com/vinted/vmemcached-rs/actions/workflows/ci.yaml/badge.svg?branch=master)](https://github.com/vinted/vmemcached-rs/actions/workflows/ci.yaml)

vmemcached is a [memcached](https://memcached.org/) client written in pure Rust.

## Install

The crate is called `vmemcached` and you can depend on it via cargo:

```ini
[dependencies]
vmemcached = "0.1.0"
```

## Features

 - ASCII protocol
 - Key interpreted as slice of u8 (bytes)
 - Value is accepted as implementing Serialize and is stored as JSON using serde_json crate
 - Not supported: increment/decrement/append/prepend/gets operations due to JSON and compression
 - Feature: "compress" enable Brotli encoding/decoding
 - Tokio
 - [bb8](https://github.com/djc/bb8) async connection pool
 - [Nom](https://github.com/Geal/nom) for parsing memcached ASCII protocol

## Development

To start:

```shell
make test
```

# License

MIT
