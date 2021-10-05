use std::collections::HashMap;
use std::time;

use vmemcached::{ErrorKind, MemcacheError, Status};

mod helpers;

#[tokio::test]
async fn test_ascii() {
    // Testing mcrouter
    let client = helpers::connect("memcache://localhost:11311?protocol=ascii")
        .await
        .unwrap();

    let got = client
        .set("ascii_foo", "bar", time::Duration::from_secs(1))
        .await
        .unwrap();

    assert_eq!(got, Status::Stored);

    let got: Option<String> = client.get("ascii_foo").await.unwrap();
    assert_eq!(got.unwrap(), "bar");

    let got: Option<String> = client.get("ascii_foo_none").await.unwrap();
    assert!(got.is_none());

    let got = client.delete("ascii_foo").await.unwrap();
    assert_eq!(got, Status::Deleted);

    let got: Option<String> = client.get("ascii_foo").await.unwrap();
    assert!(got.is_none());

    client
        .set("ascii_foo", "bar", time::Duration::from_secs(1))
        .await
        .unwrap();
    let value: HashMap<String, String> = client.gets(&["ascii_foo"]).await.unwrap().unwrap();
    assert_eq!(value["ascii_foo"], "bar".to_string());

    let got = client
        .touch("ascii_foo", time::Duration::from_secs(1000))
        .await
        .unwrap();

    assert_eq!(got, Status::Touched);

    let got = client
        .touch("ascii_foo_none", time::Duration::from_secs(1000))
        .await
        .unwrap();

    assert_eq!(got, Status::NotFound);
}

#[tokio::test]
async fn test_set_too_large_value() {
    // Testing mcrouter
    let client = helpers::connect("memcache://localhost:11311?protocol=ascii")
        .await
        .unwrap();

    let value = vec![0u8; 1024 * 512];

    let got = client
        .set("large_value", value.clone(), time::Duration::from_secs(1))
        .await
        .unwrap_err();

    assert_eq!(
        got.to_string(),
        MemcacheError::Memcache(ErrorKind::Server("object too large for cache".into())).to_string()
    );
}
