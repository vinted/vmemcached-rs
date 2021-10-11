use std::collections::HashMap;
use std::time;
use std::time::Duration;
use tokio::time::sleep;

use vmemcached::Status;

mod helpers;

#[tokio::test]
async fn test_haproxy() {
    // Testing mcrouter
    let client = helpers::connect("memcache://localhost:21311")
        .await
        .unwrap();

    let mut i = 0;
    while i < 20 {
        let client_clone = client.clone();
        let _ = tokio::spawn(async move {
            let key = "haproxy_fun";

            let got = client_clone
                .set(key, "bar", time::Duration::from_secs(0))
                .await
                .unwrap();

            assert_eq!(got, Status::Stored);

            let value: HashMap<String, String> = client_clone.gets(&[key]).await.unwrap().unwrap();
            assert_eq!(value[key], "bar".to_string());

            sleep(Duration::from_millis(50)).await;

            let got = client_clone
                .touch(key, time::Duration::from_secs(1000))
                .await
                .unwrap();

            assert_eq!(got, Status::Touched);
        });

        let client_clone = client.clone();
        let _ = tokio::spawn(async move {
            let key = "haproxy_fun2";

            client_clone
                .set(key, "bar", time::Duration::from_secs(0))
                .await
                .unwrap();

            let got: Option<String> = client_clone.get(key).await.unwrap();
            assert_eq!(got.unwrap(), "bar");

            let got: Option<String> = client_clone.get("ascii_foo_none").await.unwrap();
            assert!(got.is_none());

            sleep(Duration::from_millis(10)).await;

            let got = client_clone.delete(key).await.unwrap();
            assert_eq!(got, Status::Deleted);

            let got: Option<String> = client_clone.get(key).await.unwrap();
            assert!(got.is_none());
        });

        let got = client
            .touch("ascii_foo_none", time::Duration::from_secs(1000))
            .await;

        assert!(got.is_ok(), "{:?}", got);

        sleep(Duration::from_millis(100)).await;

        i += 1;
    }
}
