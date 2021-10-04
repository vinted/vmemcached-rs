use std::{thread, time};

use vmemcached::Status;

mod helpers;

#[tokio::test]
async fn test_ascii() {
    // Testing mcrouter
    let client = helpers::connect("memcache://localhost:11211?protocol=ascii")
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

    // client.set("ascii_foo", "bar", time::Duration::from_secs(1)).unwrap();
    // let value: Option<String> = client.get("ascii_foo").unwrap();
    // assert_eq!(value, Some("bar".into()));
    //
    // client.touch("ascii_foo", time::Duration::from_secs(1000)).unwrap();
    //
    // let value: Option<String> = client.get("not_exists_key").unwrap();
    // assert_eq!(value, None);
    //
    // client.delete("ascii_pend").unwrap();
    // let value: Option<String> = client.get("ascii_pend").unwrap();
    // assert_eq!(value, None);
    //
    // client.stats().unwrap();
}

#[tokio::test]
async fn test_set_too_large_value() {
    // Testing mcrouter
    let client = helpers::connect("memcache://localhost:11211?protocol=ascii")
        .await
        .unwrap();

    let value = vec![0u8; 1024 * 256];

    let got = client
        .set("large_value", value.clone(), time::Duration::from_secs(1))
        .await
        .unwrap();

    assert!(got.is_server_error())
}
