extern crate rand;
extern crate vmemcached;

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
    let client = helpers::connect("memcache://localhost:11211").unwrap();
    let expiration = time::Duration::from_secs(0);

    client.version().unwrap();

    client.set("foo", "bar", expiration).unwrap();
    client.flush().unwrap();
    let value: Option<String> = client.get("foo").unwrap();
    assert_eq!(value, None);

    client.set("foo", "bar", expiration).unwrap();
    client.flush_with_delay(3).unwrap();
    let value: Option<String> = client.get("foo").unwrap();
    assert_eq!(value, Some(String::from("bar")));
    thread::sleep(time::Duration::from_secs(4));
    let value: Option<String> = client.get("foo").unwrap();
    assert_eq!(value, None);

    client.set("foo", "bar", expiration).unwrap();
    let value = client.add("foo", "baz", expiration);
    assert!(value.is_ok());

    client.delete("foo").unwrap();
    let value: Option<String> = client.get("foo").unwrap();
    assert_eq!(value, None);

    client.add("foo", "bar", expiration).unwrap();
    let value: Option<String> = client.get("foo").unwrap();
    assert_eq!(value, Some(String::from("bar")));

    client.replace("foo", "baz", expiration).unwrap();
    let value: Option<String> = client.get("foo").unwrap();
    assert_eq!(value, Some(String::from("baz")));

    assert_eq!(client.touch("foooo", time::Duration::from_secs(123)).unwrap(), false);
    client.set("fooo", 0, expiration).unwrap();
    assert_eq!(client.touch("fooo", time::Duration::from_secs(12345)).unwrap(), true);

    let mut keys: Vec<String> = Vec::new();
    for _ in 0..1000 {
        let key = gen_random_key();
        keys.push(key.clone());
        client.set(key.as_str(), "xxx", expiration).unwrap();
    }

    for key in keys {
        let value: String = client.get(key.as_str()).unwrap().unwrap();

        assert_eq!(value, "xxx");
    }

    // test with multiple TCP connections
    let mut handles: Vec<Option<JoinHandle<_>>> = Vec::new();
    for i in 0..10 {
        handles.push(Some(thread::spawn(move || {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            let client = helpers::connect("memcache://localhost:11211").unwrap();
            for j in 0..50 {
                let value = format!("{}{}", value, j);
                client.set(key.as_str(), &value, expiration).unwrap();
                let result: Option<String> = client.get(key.as_str()).unwrap();
                assert_eq!(result.as_ref(), Some(&value));

                let result = client.add(key.as_str(), &value, expiration);
                assert!(result.is_ok());

                client.delete(key.as_str()).unwrap();
                let result: Option<String> = client.get(key.as_str()).unwrap();
                assert_eq!(result, None);

                client.add(key.as_str(), &value, expiration).unwrap();
                let result: Option<String> = client.get(key.as_str()).unwrap();
                assert_eq!(result.as_ref(), Some(&value));

                client.replace(key.as_str(), &value, expiration).unwrap();
                let result: Option<String> = client.get(key.as_str()).unwrap();
                assert_eq!(result.as_ref(), Some(&value));
            }
        })));
    }

    for i in 0..10 {
        handles[i].take().unwrap().join().unwrap();
    }
}
