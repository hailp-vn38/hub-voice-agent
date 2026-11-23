mod support;
use support::Hub;

#[tokio::test]
async fn posting_ota_updates_the_running_voice_configuration() {
    let mut hub = Hub::start(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(hub_core::setup_web::serve(listener, hub.input_tx.clone()));
    let response = reqwest::Client::new()
        .post(format!("http://{address}/voice"))
        .form(&[("ota_url", "http://localhost/voice/ota")])
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    while hub.config_rx.borrow_and_update().is_none() {
        hub.config_rx.changed().await.unwrap();
    }
    assert_eq!(
        hub.config_rx.borrow().as_ref().unwrap().ota_url.as_str(),
        "http://localhost/voice/ota"
    );
}

#[tokio::test]
async fn oauth_pasted_callback_exchanges_code_and_persists_refresh_token() {
    use axum::{Json, Router, routing::post};
    let fake = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let fake_addr = fake.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(fake, Router::new().route("/token", post(|axum::Form(form): axum::Form<std::collections::HashMap<String, String>>| async move {
            assert_eq!(form.get("code").unwrap(), "test-code");
            assert_eq!(form.get("client_id").unwrap(), "client-id");
            Json(serde_json::json!({"refresh_token": "refresh-from-google"}))
        }))).await.unwrap();
    });
    let hub = Hub::start(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(hub_core::setup_web::serve_with_oauth(
        listener,
        hub.input_tx.clone(),
        hub_core::setup_web::OAuthEndpoints {
            authorize: format!("http://{fake_addr}/authorize"),
            token: format!("http://{fake_addr}/token"),
        },
    ));
    let client = reqwest::Client::new();
    let page = client
        .post(format!("http://{address}/oauth/start"))
        .form(&[
            ("client_id", "client-id"),
            ("client_secret", "client-secret"),
        ])
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let link = page
        .split("href=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .replace("&amp;", "&");
    let auth = url::Url::parse(&link).unwrap();
    let state = auth
        .query_pairs()
        .find(|(k, _)| k == "state")
        .unwrap()
        .1
        .into_owned();
    let callback = format!(
        "http://127.0.0.1:{}/oauth/callback?code=test-code&state={state}",
        address.port()
    );
    let response = client
        .post(format!("http://{address}/oauth/paste"))
        .form(&[("url", callback)])
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    let saved = hub_core::ConfigStore::open(hub._dir.path()).unwrap();
    assert_eq!(saved.get().calendar.refresh_token, "refresh-from-google");
}

#[tokio::test]
async fn invalid_settings_and_unsolicited_callback_are_rejected() {
    let hub = Hub::start(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(hub_core::setup_web::serve(listener, hub.input_tx.clone()));
    let client = reqwest::Client::new();
    assert_eq!(
        client
            .post(format!("http://{address}/voice"))
            .form(&[("ota_url", "invalid")])
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    assert_eq!(
        client
            .post(format!("http://{address}/weather"))
            .form(&[("latitude", "91"), ("longitude", "0")])
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    assert_eq!(
        client
            .get(format!("http://{address}/oauth/callback?code=unsolicited"))
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    assert!(
        hub_core::ConfigStore::open(hub._dir.path())
            .unwrap()
            .get()
            .voice
            .ota_url
            .is_none()
    );
}
