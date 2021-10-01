use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::iter;
use std::thread;
use std::thread::JoinHandle;
use std::time;

mod helpers;

fn gen_random_key() -> String {
    let bs = iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .take(10)
        .collect::<Vec<u8>>();
    String::from_utf8(bs).unwrap()
}

#[test]
fn tcp_test() {
    let mcrouter = helpers::connect("memcache://localhost:11311").unwrap();
    let memcached = helpers::connect("memcache://localhost:11211").unwrap();
    let expiration = time::Duration::from_secs(0);

    mcrouter.version().unwrap();

    mcrouter.set("foo", "bar", expiration).unwrap();
    memcached.flush().unwrap();
    let value: Option<String> = mcrouter.get("foo").unwrap();
    assert_eq!(value, None);

    mcrouter.set("foo", "bar", expiration).unwrap();
    memcached.flush_with_delay(3).unwrap();
    let value: Option<String> = mcrouter.get("foo").unwrap();
    assert_eq!(value, Some(String::from("bar")));
    thread::sleep(time::Duration::from_secs(4));
    let value: Option<String> = mcrouter.get("foo").unwrap();
    assert_eq!(value, None);

    mcrouter.set("foo", "bar", expiration).unwrap();
    let value = mcrouter.add("foo", "baz", expiration);
    assert!(value.is_ok());

    mcrouter.delete("foo").unwrap();
    let value: Option<String> = mcrouter.get("foo").unwrap();
    assert_eq!(value, None);

    mcrouter.add("foo", "bar", expiration).unwrap();
    let value: Option<String> = mcrouter.get("foo").unwrap();
    assert_eq!(value, Some(String::from("bar")));

    mcrouter.replace("foo", "baz", expiration).unwrap();
    let value: Option<String> = mcrouter.get("foo").unwrap();
    assert_eq!(value, Some(String::from("baz")));

    assert_eq!(mcrouter.touch("foooo", time::Duration::from_secs(123)).unwrap(), false);
    mcrouter.set("fooo", 0, expiration).unwrap();
    assert_eq!(mcrouter.touch("fooo", time::Duration::from_secs(12345)).unwrap(), true);

    let mut keys: Vec<String> = Vec::new();
    for _ in 0..1000 {
        let key = gen_random_key();
        keys.push(key.clone());
        mcrouter.set(key.as_str(), "xxx", expiration).unwrap();
    }

    for key in keys {
        let value: String = mcrouter.get(key.as_str()).unwrap().unwrap();

        assert_eq!(value, "xxx");
    }

    // test with multiple TCP connections
    let mut handles: Vec<Option<JoinHandle<_>>> = Vec::new();
    for i in 0..10 {
        handles.push(Some(thread::spawn(move || {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            let mcrouter = helpers::connect("memcache://localhost:11311").unwrap();
            for j in 0..50 {
                let value = format!("{}{}", value, j);
                mcrouter.set(key.as_str(), &value, expiration).unwrap();
                let result: Option<String> = mcrouter.get(key.as_str()).unwrap();
                assert_eq!(result.as_ref(), Some(&value));

                let result = mcrouter.add(key.as_str(), &value, expiration);
                assert!(result.is_ok());

                mcrouter.delete(key.as_str()).unwrap();
                let result: Option<String> = mcrouter.get(key.as_str()).unwrap();
                assert_eq!(result, None);

                mcrouter.add(key.as_str(), &value, expiration).unwrap();
                let result: Option<String> = mcrouter.get(key.as_str()).unwrap();
                assert_eq!(result.as_ref(), Some(&value));

                mcrouter.replace(key.as_str(), &value, expiration).unwrap();
                let result: Option<String> = mcrouter.get(key.as_str()).unwrap();
                assert_eq!(result.as_ref(), Some(&value));
            }
        })));
    }

    for i in 0..10 {
        handles[i].take().unwrap().join().unwrap();
    }
}
