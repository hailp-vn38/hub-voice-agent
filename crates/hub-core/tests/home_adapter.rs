use futures_util::{SinkExt, StreamExt};
use hub_core::{HomeAssistant, HomeEvent, HomeSettings, HomeSource};
use serde_json::json;
use tokio::{net::TcpListener, sync::watch};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[tokio::test]
async fn authenticates_and_delivers_initial_and_changed_states() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        ws.send(Message::Text(
            json!({"type":"auth_required"}).to_string().into(),
        ))
        .await
        .unwrap();
        let auth: serde_json::Value =
            serde_json::from_str(ws.next().await.unwrap().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(auth["access_token"], "secret");
        ws.send(Message::Text(json!({"type":"auth_ok"}).to_string().into()))
            .await
            .unwrap();
        let a: serde_json::Value =
            serde_json::from_str(ws.next().await.unwrap().unwrap().to_text().unwrap()).unwrap();
        let b: serde_json::Value =
            serde_json::from_str(ws.next().await.unwrap().unwrap().to_text().unwrap()).unwrap();
        for message in [a, b] {
            let result = if message["type"] == "get_states" {
                json!([
                    {"entity_id":"light.desk","state":"off","attributes":{"friendly_name":"Desk"}},
                    {"entity_id":"sensor.temp","state":"23.5","attributes":{"friendly_name":"Temperature","unit_of_measurement":"°C"}}
                ])
            } else {
                json!(null)
            };
            ws.send(Message::Text(
                json!({"id":message["id"],"type":"result","success":true,"result":result})
                    .to_string()
                    .into(),
            ))
            .await
            .unwrap();
        }
        ws.send(Message::Text(json!({"id":1,"type":"event","event":{"data":{"entity_id":"light.desk","new_state":{"entity_id":"light.desk","state":"on","attributes":{"friendly_name":"Desk"}}}}}).to_string().into())).await.unwrap();
        let call: serde_json::Value =
            serde_json::from_str(ws.next().await.unwrap().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(call["type"], "call_service");
        assert_eq!(call["domain"], "light");
        assert_eq!(call["service"], "turn_on");
        assert_eq!(call["target"]["entity_id"], "light.desk");
        ws.send(Message::Text(
            json!({"id":call["id"],"type":"result","success":true})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    });
    let source = HomeAssistant::default();
    let (_settings_tx, settings_rx) = watch::channel(HomeSettings {
        url: format!("http://{address}"),
        token: "secret".into(),
    });
    let mut events = source.subscribe(settings_rx);
    assert!(matches!(events.recv().await, Some(HomeEvent::Connected)));
    let first = events.recv().await.unwrap();
    assert!(
        matches!(first, HomeEvent::State(ref entity) if entity.state == "off" && entity.name == "Desk")
    );
    let sensor = events.recv().await.unwrap();
    assert!(
        matches!(sensor, HomeEvent::State(ref entity) if entity.id == "sensor.temp" && entity.unit.as_deref() == Some("°C"))
    );
    let second = events.recv().await.unwrap();
    assert!(matches!(second, HomeEvent::State(ref entity) if entity.state == "on"));
    source
        .call_service("light.desk".into(), "turn_on".into())
        .await
        .unwrap();
    server.await.unwrap();
}
