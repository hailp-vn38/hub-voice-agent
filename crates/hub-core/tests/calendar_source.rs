use axum::{
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use hub_core::{
    CalendarSettings, CalendarSource, NewCalendarEvent, sources::google_calendar::GoogleCalendar,
};
use serde_json::json;

#[tokio::test]
async fn google_adapter_lists_inserts_and_reports_expired_sync_token() {
    let app = Router::new()
        .route("/token", post(|| async { Json(json!({"access_token":"access"})) }))
        .route("/calendar/calendars/primary/events", get(|query: axum::extract::RawQuery| async move {
            if query.0.unwrap_or_default().contains("syncToken=expired") { return (StatusCode::GONE, Json(json!({}))).into_response(); }
            Json(json!({"items":[{"id":"one","summary":"Meeting","start":{"dateTime":"2026-11-23T10:00:00+07:00"},"end":{"dateTime":"2026-11-23T11:00:00+07:00"}},{"id":"gone","status":"cancelled"}],"nextSyncToken":"next"})).into_response()
        }).post(|| async { Json(json!({"id":"new","summary":"Lunch","start":{"dateTime":"2026-11-23T12:00:00+07:00"},"end":{"dateTime":"2026-11-23T13:00:00+07:00"}})) }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let source = GoogleCalendar::new(
        &format!("http://{addr}/calendar"),
        &format!("http://{addr}/token"),
    );
    let settings = CalendarSettings {
        client_id: "id".into(),
        client_secret: "secret".into(),
        refresh_token: "refresh".into(),
    };
    let delta = source.list(settings.clone(), None).await.unwrap();
    assert_eq!(delta.events[0].summary, "Meeting");
    assert_eq!(delta.removed, ["gone"]);
    assert_eq!(delta.sync_token.as_deref(), Some("next"));
    assert!(
        source
            .list(settings.clone(), Some("expired".into()))
            .await
            .unwrap()
            .expired
    );
    let event = source
        .create(
            settings,
            NewCalendarEvent {
                summary: "Lunch".into(),
                start: "2026-11-23T12:00:00+07:00".into(),
                end: "2026-11-23T13:00:00+07:00".into(),
            },
        )
        .await
        .unwrap();
    assert_eq!(event.id, "new");
    server.abort();
}
