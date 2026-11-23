use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn serve(response: Vec<u8>) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0; 1024];
        let _ = stream.read(&mut request).await;
        let _ = stream.write_all(&response).await;
    });
    format!("http://{addr}/")
}

async fn request_local(url: String) -> anyhow::Result<FetchedImage> {
    download_with("http://8.8.8.8/image".into(), move |_, _| {
        let url = url.clone();
        Box::pin(async move {
            Ok(reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()?
                .get(url)
                .send()
                .await?)
        })
    })
    .await
}

#[tokio::test]
async fn rejects_declared_and_streamed_oversized_images() {
    let url = serve(b"HTTP/1.1 200 OK\r\nContent-Length: 10485761\r\n\r\n".to_vec()).await;
    assert!(
        request_local(url)
            .await
            .unwrap_err()
            .to_string()
            .contains("too large")
    );
    let mut body = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\na00001\r\n".to_vec();
    body.extend(vec![0; MAX_BYTES + 1]);
    body.extend(b"\r\n0\r\n\r\n");
    let url = serve(body).await;
    assert!(
        request_local(url)
            .await
            .unwrap_err()
            .to_string()
            .contains("too large")
    );
}

#[tokio::test]
async fn rejects_redirect_to_localhost_before_second_request() {
    let url = serve(
        b"HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1/secret\r\nContent-Length: 0\r\n\r\n"
            .to_vec(),
    )
    .await;
    assert!(
        request_local(url)
            .await
            .unwrap_err()
            .to_string()
            .contains("non-public")
    );
}

#[test]
fn ipv6_tunnels_embedding_ipv4_are_not_public() {
    assert!(public("2606:4700::1111".parse().unwrap()));
    assert!(!public("2002:7f00:1::1".parse().unwrap()));
    assert!(!public("2001:0:7f00:1::1".parse().unwrap()));
    assert!(!public("2001:db8::1".parse().unwrap()));
}
