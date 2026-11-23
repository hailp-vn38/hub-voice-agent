use crate::{HomeEntity, HomeEvent, HomeSettings, HomeSource};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Mutex, time::Duration};
use tokio::sync::{mpsc, oneshot, watch};
use tokio_tungstenite::{connect_async, tungstenite::Message};

type Command = (String, String, oneshot::Sender<anyhow::Result<()>>);
#[derive(Default)]
pub struct HomeAssistant {
    commands: Mutex<Option<mpsc::Sender<Command>>>,
}
impl HomeSource for HomeAssistant {
    fn subscribe(&self, mut settings: watch::Receiver<HomeSettings>) -> mpsc::Receiver<HomeEvent> {
        let (events, rx) = mpsc::channel(64);
        let (commands, mut command_rx) = mpsc::channel(32);
        *self.commands.lock().unwrap() = Some(commands);
        tokio::spawn(async move {
            loop {
                let config = settings.borrow_and_update().clone();
                if !config.url.is_empty() && !config.token.is_empty() {
                    let _ = session(&config, &events, &mut command_rx, &mut settings).await;
                    if events.send(HomeEvent::Disconnected).await.is_err() {
                        break;
                    }
                }
                tokio::select! {
                    _ = events.closed() => break,
                    result = settings.changed() => if result.is_err() { break; },
                    _ = tokio::time::sleep(Duration::from_secs(5)) => {},
                    Some((_, _, reply)) = command_rx.recv() => { let _ = reply.send(Err(anyhow::anyhow!("Home Assistant disconnected"))); }
                }
            }
        });
        rx
    }
    fn call_service(
        &self,
        entity_id: String,
        service: String,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        let commands = self.commands.lock().unwrap().clone();
        Box::pin(async move {
            let commands =
                commands.ok_or_else(|| anyhow::anyhow!("Home Assistant disconnected"))?;
            let (tx, rx) = oneshot::channel();
            commands
                .send((entity_id, service, tx))
                .await
                .map_err(|_| anyhow::anyhow!("Home Assistant disconnected"))?;
            rx.await
                .map_err(|_| anyhow::anyhow!("Home Assistant disconnected"))?
        })
    }
}

async fn session(
    config: &HomeSettings,
    events: &mpsc::Sender<HomeEvent>,
    commands: &mut mpsc::Receiver<Command>,
    settings: &mut watch::Receiver<HomeSettings>,
) -> anyhow::Result<()> {
    let mut url = url::Url::parse(&config.url)?;
    let scheme = match url.scheme() {
        "http" => "ws",
        "https" => "wss",
        "ws" => "ws",
        "wss" => "wss",
        _ => anyhow::bail!("Invalid Home Assistant URL"),
    };
    url.set_scheme(scheme)
        .map_err(|_| anyhow::anyhow!("Invalid URL"))?;
    url.set_path("/api/websocket");
    let (mut socket, _) = connect_async(url.as_str()).await?;
    let required = socket
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("Disconnected"))??;
    let required: Value = serde_json::from_str(required.to_text()?)?;
    anyhow::ensure!(
        required["type"] == "auth_required",
        "Expected auth_required"
    );
    socket
        .send(Message::Text(
            json!({"type":"auth","access_token":config.token})
                .to_string()
                .into(),
        ))
        .await?;
    let auth = socket
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("Disconnected"))??;
    let auth: Value = serde_json::from_str(auth.to_text()?)?;
    anyhow::ensure!(
        auth["type"] == "auth_ok",
        "Home Assistant authentication failed"
    );
    socket
        .send(Message::Text(
            json!({"id":1,"type":"subscribe_events","event_type":"state_changed"})
                .to_string()
                .into(),
        ))
        .await?;
    socket
        .send(Message::Text(
            json!({"id":2,"type":"get_states"}).to_string().into(),
        ))
        .await?;
    events.send(HomeEvent::Connected).await?;
    let mut next_id = 3u64;
    let mut pending = HashMap::new();
    loop {
        tokio::select! {
            _ = events.closed() => break,
            _ = settings.changed() => break,
            Some((entity, service, reply)) = commands.recv() => {
                let domain = entity.split('.').next().unwrap_or("");
                socket.send(Message::Text(json!({"id":next_id,"type":"call_service","domain":domain,"service":service,"target":{"entity_id":entity}}).to_string().into())).await?;
                pending.insert(next_id, reply); next_id += 1;
            }
            frame = socket.next() => {
                let Some(frame) = frame else { break }; let frame = frame?;
                if !frame.is_text() { continue; }
                let value: Value = serde_json::from_str(frame.to_text()?)?;
                if value["type"] == "result" {
                    let id = value["id"].as_u64().unwrap_or(0);
                    if id == 2 {
                        anyhow::ensure!(value["success"] == true, "get_states failed");
                        if let Some(states) = value["result"].as_array() { for state in states { if let Some(entity) = parse_state(state) { events.send(HomeEvent::State(entity)).await?; } } }
                    } else if id == 1 { anyhow::ensure!(value["success"] == true, "subscribe_events failed"); }
                    else if let Some(reply) = pending.remove(&id) { let result = if value["success"] == true { Ok(()) } else { Err(anyhow::anyhow!("Home Assistant service failed")) }; let _ = reply.send(result); }
                } else if value["type"] == "event" && let Some(entity) = parse_state(&value["event"]["data"]["new_state"]) { events.send(HomeEvent::State(entity)).await?; }
            }
        }
    }
    Ok(())
}
fn parse_state(value: &Value) -> Option<HomeEntity> {
    let id = value["entity_id"].as_str()?.to_owned();
    Some(HomeEntity {
        name: value["attributes"]["friendly_name"]
            .as_str()
            .unwrap_or(&id)
            .to_owned(),
        id,
        state: value["state"].as_str()?.to_owned(),
        stale: false,
    })
}
