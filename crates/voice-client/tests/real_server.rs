//! Manual check against a running vela-server. Ignored by default.
//!
//! Manual check against a running vela-server. Ignored by default.
//!
//! Uses the same identity as the hub, read from `ai-hub/.data/config.toml`
//! `[voice]`, so the device stays fixed across runs:
//!
//! ```text
//! cargo test -p voice-client --test real_server -- --ignored --nocapture
//! ```
//!
//! Optional overrides: `VOICE_OTA_URL`, `VOICE_DEVICE_ID`, `VOICE_CLIENT_ID`.
//! If the server asks for activation, the code is printed and the test waits
//! until the device is linked from the admin web.

use std::{path::Path, time::Duration};

use tokio::sync::watch;
use url::Url;
use voice_client::{ConnectionState, DeviceIdentity, VoiceConfig, VoiceEvent};

const DEFAULT_OTA_URL: &str = "http://127.0.0.1:8000/voice/ota";

/// Reads `[voice]` from the hub's data config; `None` if the file is missing.
fn hub_identity() -> Option<(String, String)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.data/config.toml");
    let document: toml::Table = toml::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    let voice = document.get("voice")?.as_table()?;
    Some((
        voice.get("device_id")?.as_str()?.to_owned(),
        voice.get("client_id")?.as_str()?.to_owned(),
    ))
}

fn identity() -> (String, String) {
    let (file_device, file_client) = hub_identity().unwrap_or_default();
    let device = std::env::var("VOICE_DEVICE_ID").unwrap_or(file_device);
    let client = std::env::var("VOICE_CLIENT_ID").unwrap_or(file_client);
    assert!(
        !device.is_empty() && !client.is_empty(),
        "no device identity: set VOICE_DEVICE_ID/VOICE_CLIENT_ID or fill .data/config.toml [voice]"
    );
    (device, client)
}

#[tokio::test]
#[ignore = "needs a running vela-server; see module docs"]
async fn connects_to_real_server() {
    let ota_url =
        Url::parse(&std::env::var("VOICE_OTA_URL").unwrap_or_else(|_| DEFAULT_OTA_URL.into()))
            .unwrap();
    let (device_id, client_id) = identity();
    let config = VoiceConfig {
        ota_url,
        device: DeviceIdentity::new(device_id, client_id),
        tools: vec![],
    };
    let (_config_tx, config_rx) = watch::channel(Some(config));
    let (_commands, mut events) = voice_client::spawn(config_rx);

    let outcome = tokio::time::timeout(Duration::from_secs(600), async {
        while let Some(event) = events.recv().await {
            match event {
                VoiceEvent::Activation { code } => {
                    eprintln!("\n==> ACTIVATION CODE: {code}  (nhập mã này trên admin web)\n");
                }
                // Ready only means the WebSocket opened. A websocket-transport
                // server also opens one for an Enrollment Session, so wait
                // briefly for its code prompt before trusting it.
                VoiceEvent::Connection(ConnectionState::Ready) => {
                    eprintln!("connection: Ready (đang xác nhận phiên voice)");
                    let probe = tokio::time::timeout(Duration::from_secs(3), events.recv()).await;
                    if let Ok(Some(VoiceEvent::Stt { text })) = &probe
                        && text.contains("Mã kết nối")
                    {
                        return Err(
                            "server đang ở enrollment transport=websocket, không phải phiên voice",
                        );
                    }
                    return Ok(());
                }
                VoiceEvent::Connection(state) => eprintln!("connection: {state:?}"),
                other => eprintln!("event: {other:?}"),
            }
        }
        Err("event channel closed before Ready")
    })
    .await;

    match outcome {
        Ok(Ok(())) => eprintln!("==> READY: đã kết nối server thật"),
        Ok(Err(reason)) => panic!("{reason}"),
        Err(_) => panic!("timeout 600s: chưa Ready (chưa kích hoạt?)"),
    }
}
