use vmemcached::Settings;

mod helpers;

#[tokio::test]
async fn test_default_settings() {
    let client = helpers::connect("memcache://localhost:11311")
        .await
        .unwrap();

    let got = client.get_settings();
    let expected = Settings::default();

    assert_eq!(got.buffer_size, expected.buffer_size);
}

#[tokio::test]
async fn test_custom_settings() {
    let settings = Settings::new().buffer_size(256);
    let client = helpers::connect_with_custom_settings("memcache://localhost:11311", settings)
        .await
        .unwrap();

    let got = client.get_settings();
    let expected_buffer_size = 256;

    assert_eq!(got.buffer_size, expected_buffer_size);
}
