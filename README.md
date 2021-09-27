# vmemcached

[![Build Status](https://travis-ci.org/vinted/vmemcached.svg?branch=master)](https://travis-ci.org/vinted/vmemcached)

vmemcached is a [memcached](https://memcached.org/) client written in pure rust.

![logo](https://cloudflare-ipfs.com/ipfs/QmY2otmZFbrLfCQZ2JG8bsEsMGegHrh8WgupcyTcyoShiS)

## Install

The crate is called `memcache` and you can depend on it via cargo:

```ini
[dependencies]
vmemcached = "0.1.0"
```
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
