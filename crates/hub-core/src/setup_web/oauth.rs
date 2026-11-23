use super::{AppState, CalendarSettings, OAuthEndpoints, Update, save};
use axum::{
    Form,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;

#[derive(Clone)]
pub(super) struct Flow {
    endpoints: OAuthEndpoints,
    redirect: String,
    pending: Arc<Mutex<Option<Pending>>>,
}
struct Pending {
    state: String,
    client_id: String,
    client_secret: String,
}
impl Flow {
    pub fn new(endpoints: OAuthEndpoints, redirect: String) -> Self {
        Self {
            endpoints,
            redirect,
            pending: Arc::new(Mutex::new(None)),
        }
    }
}
#[derive(Deserialize)]
pub(super) struct Start {
    client_id: String,
    client_secret: String,
}
pub(super) async fn start(
    State(app): State<AppState>,
    Form(form): Form<Start>,
) -> impl IntoResponse {
    if form.client_id.is_empty() || form.client_secret.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Thiếu thông tin Google"));
    }
    let state = uuid::Uuid::new_v4().to_string();
    let mut url = Url::parse(&app.oauth.endpoints.authorize)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "OAuth không hợp lệ"))?;
    url.query_pairs_mut()
        .append_pair("client_id", &form.client_id)
        .append_pair("redirect_uri", &app.oauth.redirect)
        .append_pair("response_type", "code")
        .append_pair("scope", "https://www.googleapis.com/auth/calendar.events")
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent")
        .append_pair("state", &state);
    *app.oauth.pending.lock().await = Some(Pending {
        state,
        client_id: form.client_id,
        client_secret: form.client_secret,
    });
    Ok(Html(format!(
        "<p>Mở <a href=\"{}\">Google</a>. Trên điện thoại, sao chép URL sau khi Google chuyển hướng và dán vào trang setup.</p>",
        url
    )))
}
#[derive(Deserialize)]
pub(super) struct Callback {
    code: Option<String>,
    state: Option<String>,
}
#[derive(Deserialize)]
pub(super) struct Paste {
    url: String,
}
pub(super) async fn paste(
    State(app): State<AppState>,
    Form(form): Form<Paste>,
) -> impl IntoResponse {
    let url = Url::parse(&form.url).map_err(|_| (StatusCode::BAD_REQUEST, "URL không hợp lệ"))?;
    let redirect = Url::parse(&app.oauth.redirect).unwrap();
    if url.scheme() != redirect.scheme()
        || url.host_str() != redirect.host_str()
        || url.port_or_known_default() != redirect.port_or_known_default()
        || url.path() != redirect.path()
    {
        return Err((StatusCode::BAD_REQUEST, "URL chuyển hướng không hợp lệ"));
    }
    let mut callback = Callback {
        code: None,
        state: None,
    };
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => callback.code = Some(value.into()),
            "state" => callback.state = Some(value.into()),
            _ => {}
        }
    }
    finish(app, callback).await
}
pub(super) async fn callback(
    State(app): State<AppState>,
    Query(query): Query<Callback>,
) -> impl IntoResponse {
    finish(app, query).await
}
async fn finish(
    app: AppState,
    query: Callback,
) -> Result<&'static str, (StatusCode, &'static str)> {
    let mut pending = app.oauth.pending.lock().await;
    if pending.is_none()
        || pending.as_ref().map(|p| p.state.as_str()) != query.state.as_deref()
        || query.code.as_deref().unwrap_or("").is_empty()
    {
        return Err((StatusCode::BAD_REQUEST, "OAuth state hoặc mã không hợp lệ"));
    }
    let p = pending.take().unwrap();
    drop(pending);
    let response = reqwest::Client::new()
        .post(&app.oauth.endpoints.token)
        .form(&[
            ("code", query.code.unwrap()),
            ("client_id", p.client_id.clone()),
            ("client_secret", p.client_secret.clone()),
            ("redirect_uri", app.oauth.redirect.clone()),
            ("grant_type", "authorization_code".into()),
        ])
        .send()
        .await
        .map_err(|_| (StatusCode::BAD_GATEWAY, "Không kết nối được Google"))?;
    #[derive(Deserialize)]
    struct Token {
        refresh_token: String,
    }
    let token = response
        .error_for_status()
        .map_err(|_| (StatusCode::BAD_GATEWAY, "Google từ chối mã"))?
        .json::<Token>()
        .await
        .map_err(|_| (StatusCode::BAD_GATEWAY, "Google không trả refresh token"))?;
    if token.refresh_token.is_empty() {
        return Err((StatusCode::BAD_GATEWAY, "Google không trả refresh token"));
    }
    save(
        &app,
        Update::Calendar(CalendarSettings {
            client_id: p.client_id,
            client_secret: p.client_secret,
            refresh_token: token.refresh_token,
        }),
    )
    .await
}
