use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::iter;
use std::time;

use vmemcached::Status;

mod helpers;

#[tokio::test]
async fn test_version() {
    // Testing memcached
    let client = helpers::connect("memcache://localhost:11211?protocol=ascii")
        .await
        .unwrap();

    let version = client.version().await.unwrap();

    assert_eq!(version, "1.6.9");

    // Testing mcrouter
    let client = helpers::connect("memcache://localhost:11311?protocol=ascii")
        .await
        .unwrap();

    let version = client.version().await.unwrap();

    assert_eq!(version, "38.0.0 mcrouter");
}

fn gen_random_key() -> String {
    let bs = iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .take(10)
        .collect::<Vec<u8>>();
    String::from_utf8(bs).unwrap()
}

#[tokio::test]
async fn tcp_test() {
    let client = helpers::connect("memcache://localhost:11311")
        .await
        .unwrap();
    let expiration = time::Duration::from_secs(0);

    client.version().await.unwrap();

    client.set("foo", "bar", expiration).await.unwrap();
    let value: Option<String> = client.get("foo").await.unwrap();
    assert_eq!(value, Some("bar".to_string()));

    client.delete("foo").await.unwrap();
    let value: Option<String> = client.get("foo").await.unwrap();
    assert_eq!(value, None);

    client.add("foo", "bar", expiration).await.unwrap();
    let value: Option<String> = client.get("foo").await.unwrap();
    assert_eq!(value, Some(String::from("bar")));

    client.replace("foo", "baz", expiration).await.unwrap();
    let value: Option<String> = client.get("foo").await.unwrap();
    assert_eq!(value, Some(String::from("baz")));

    assert_eq!(
        client
            .touch("foooo", time::Duration::from_secs(123))
            .await
            .unwrap(),
        Status::NotFound
    );
    client.set("fooo", 0, expiration).await.unwrap();
    assert_eq!(
        client
            .touch("fooo", time::Duration::from_secs(12345))
            .await
            .unwrap(),
        Status::Touched
    );

    let mut keys: Vec<String> = Vec::new();
    for _ in 0..1000 {
        let key = gen_random_key();
        keys.push(key.clone());
        client.set(key.as_str(), "xxx", expiration).await.unwrap();
    }

    for key in keys {
        let value: String = client.get(key.as_str()).await.unwrap().unwrap();

        assert_eq!(value, "xxx");
    }

    // Test with multiple TCP connections
    for i in 0..10 {
        for j in 0..50 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);

            tokio::spawn(async move {
                let client = helpers::connect("memcache://localhost:11311")
                    .await
                    .unwrap();

                let value = format!("{}{}", value, j);
                client.set(key.as_str(), &value, expiration).await.unwrap();
                let result: Option<String> = client.get(key.as_str()).await.unwrap();
                assert_eq!(result.as_ref(), Some(&value));

                let result = client.add(key.as_str(), &value, expiration).await;
                assert!(result.is_ok());

                client.delete(key.as_str()).await.unwrap();
                let result: Option<String> = client.get(key.as_str()).await.unwrap();
                assert_eq!(result, None);

                client.add(key.as_str(), &value, expiration).await.unwrap();
                let result: Option<String> = client.get(key.as_str()).await.unwrap();
                assert_eq!(result.as_ref(), Some(&value));

                client
                    .replace(key.as_str(), &value, expiration)
                    .await
                    .unwrap();
                let result: Option<String> = client.get(key.as_str()).await.unwrap();
                assert_eq!(result.as_ref(), Some(&value));
            });
        }
    }
}
