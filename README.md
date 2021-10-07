# vmemcached

[![Build Status](https://app.travis-ci.com/vinted/vmemcached-rs.svg?branch=master)](https://app.travis-ci.com/vinted/vmemcached-rs)

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
 - Value is accepted as implementing Serialize and is stored as JSON using simd-json crate
 - Not supported: increment/decrement/append/prepend/gets operations due to JSON and compression
 - Feature: "compress" enable Brotli encoding/decoding
 - Feature: "tls" enables OpenSSL support

## Basic usage

```rust
let pool = vmemcached::Pool::builder()
    .connection_timeout(std::time::Duration::from_secs(1))
    .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    .expect("Memcache is available at localhost port 11211");

let client = vmemcached::Client::with_pool(pool);
client.set("sample", "bar", 10).unwrap();
let value: Option<String> = client.get("sample").unwrap();
assert_eq!(value, Some(String::from("bar")));
```

## Development

To start:

```shell
make test
```

# License

MIT
