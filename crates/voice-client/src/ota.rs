//! OTA discovery: which WebSocket to open, and with which token.

use serde::Deserialize;
use url::Url;

use crate::{DeviceIdentity, connection::Failure};

#[derive(Deserialize)]
struct OtaResponse {
    websocket: Option<OtaWebsocket>,
    activation: Option<OtaActivation>,
}

#[derive(Deserialize)]
struct OtaActivation {
    code: String,
}

pub(crate) enum Discovery {
    Websocket(Url, String),
    Activation(String),
}

#[derive(Deserialize)]
struct OtaWebsocket {
    url: String,
    token: String,
}

pub(crate) async fn discover(ota_url: Url, device: DeviceIdentity) -> Result<Discovery, Failure> {
    let response = reqwest::Client::new()
        .get(ota_url)
        .header("device-id", &device.device_id)
        .header("client-id", &device.client_id)
        .send()
        .await
        .map_err(|_| Failure::OtaRequest)?
        .error_for_status()
        .map_err(|_| Failure::OtaResponse)?;
    let body = response.bytes().await.map_err(|_| Failure::OtaRequest)?;
    let OtaResponse {
        websocket,
        activation,
    } = serde_json::from_slice(&body).map_err(|_| Failure::OtaResponse)?;
    if let Some(websocket) = websocket {
        let url = Url::parse(&websocket.url).map_err(|_| Failure::OtaResponse)?;
        Ok(Discovery::Websocket(url, websocket.token))
    } else if let Some(activation) = activation {
        Ok(Discovery::Activation(activation.code))
    } else {
        Err(Failure::OtaResponse)
    }
}
