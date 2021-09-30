extern crate vmemcached;

use std::{thread, time};

mod helpers;

#[test]
fn test_ascii() {
    // Testing mcrouter
    let client = helpers::connect("memcache://localhost:11211?protocol=ascii").unwrap();

    client.flush_with_delay(1).unwrap();
    thread::sleep(time::Duration::from_secs(1));
    client.flush().unwrap();

    client.set("ascii_foo", "bar", time::Duration::from_secs(1)).unwrap();
    let value: Option<String> = client.get("ascii_foo").unwrap();
    assert_eq!(value, Some("bar".into()));

    client.touch("ascii_foo", time::Duration::from_secs(1000)).unwrap();

    let value: Option<String> = client.get("not_exists_key").unwrap();
    assert_eq!(value, None);

    client.delete("ascii_pend").unwrap();
    let value: Option<String> = client.get("ascii_pend").unwrap();
    assert_eq!(value, None);

    client.stats().unwrap();
}
