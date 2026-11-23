use hub_core::{HttpImageFetcher, ImageFetcher};

#[tokio::test]
async fn rejects_local_and_private_destinations() {
    let fetcher = HttpImageFetcher::default();
    for url in [
        "http://127.0.0.1/a.png",
        "http://192.168.1.2/a.png",
        "http://[::1]/a.png",
        "http://[::ffff:127.0.0.1]/a.png",
        "http://localhost/a.png",
        "http://169.254.169.254/a.png",
        "file:///tmp/a.png",
    ] {
        assert!(
            fetcher.fetch(url.to_owned()).await.is_err(),
            "accepted {url}"
        );
    }
}
