use std::{thread, time};

mod helpers;

#[test]
fn test_ascii() {
    let mcrouter = helpers::connect("memcache://localhost:11311?protocol=ascii").unwrap();
    let memcached = helpers::connect("memcache://localhost:11211?protocol=ascii").unwrap();

    memcached.flush_with_delay(1).unwrap();
    thread::sleep(time::Duration::from_secs(1));
    memcached.flush().unwrap();

    mcrouter.set("ascii_foo", "bar", time::Duration::from_secs(1)).unwrap();
    let value: Option<String> = mcrouter.get("ascii_foo").unwrap();
    assert_eq!(value, Some("bar".into()));

    mcrouter.touch("ascii_foo", time::Duration::from_secs(1000)).unwrap();

    let value: Option<String> = mcrouter.get("not_exists_key").unwrap();
    assert_eq!(value, None);

    mcrouter.delete("ascii_pend").unwrap();
    let value: Option<String> = mcrouter.get("ascii_pend").unwrap();
    assert_eq!(value, None);

    mcrouter.stats().unwrap();
}
