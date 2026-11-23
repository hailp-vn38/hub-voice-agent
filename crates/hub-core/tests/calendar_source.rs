use axum::{
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use hub_core::{CalendarQuery, CalendarSettings, CalendarSource, GoogleCalendar, NewCalendarEvent};
use serde_json::json;

#[tokio::test]
async fn google_adapter_lists_inserts_and_reports_expired_sync_token() {
    let app = Router::new()
        .route("/token", post(|| async { Json(json!({"access_token":"access"})) }))
        .route("/calendar/calendars/primary/events", get(|query: axum::extract::RawQuery| async move {
            let query = query.0.unwrap_or_default();
            if query.contains("syncToken=expired") { return (StatusCode::GONE, Json(json!({}))).into_response(); }
            assert!(query.contains("timeMin=2026-10-24T00") && query.contains("timeMax=2027-01-22T00"), "{query}");
            Json(json!({"items":[{"id":"one","summary":"Meeting","description":"Check-in","start":{"dateTime":"2026-11-23T10:00:00+07:00"},"end":{"dateTime":"2026-11-23T11:00:00+07:00"}},{"id":"gone","status":"cancelled"}],"nextSyncToken":"next"})).into_response()
        }).post(|Json(body): Json<serde_json::Value>| async move { assert_eq!(body["start"]["date"], "2026-11-23"); assert_eq!(body["end"]["date"], "2026-11-24"); assert_eq!(body["location"], "Hanoi"); Json(json!({"id":"new","summary":"Lunch","location":"Hanoi","start":{"date":"2026-11-23"},"end":{"date":"2026-11-24"}})) }));
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
    let full = CalendarQuery::Full {
        time_min: "2026-10-24T00:00:00Z".parse().unwrap(),
        time_max: "2027-01-22T00:00:00Z".parse().unwrap(),
    };
    let delta = source.list(settings.clone(), full).await.unwrap();
    assert_eq!(delta.events[0].summary, "Meeting");
    assert_eq!(delta.events[0].description.as_deref(), Some("Check-in"));
    assert_eq!(delta.removed, ["gone"]);
    assert_eq!(delta.sync_token.as_deref(), Some("next"));
    assert!(
        source
            .list(settings.clone(), CalendarQuery::Since("expired".into()))
            .await
            .unwrap()
            .expired
    );
    let event = source
        .create(
            settings,
            NewCalendarEvent {
                location: Some("Hanoi".into()),
                summary: "Lunch".into(),
                start: "2026-11-23".into(),
                end: "2026-11-24".into(),
            },
        )
        .await
        .unwrap();
    assert_eq!(event.id, "new");
    assert_eq!(event.location.as_deref(), Some("Hanoi"));
    server.abort();
}
