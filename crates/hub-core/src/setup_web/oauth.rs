use super::{AppState, CalendarSettings, OAuthEndpoints, Update, save};
use axum::{
    Form,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use url::Url;

const SESSION_TTL: Duration = Duration::from_secs(600);

#[derive(Clone)]
pub(super) struct Flow {
    endpoints: OAuthEndpoints,
    redirect: String,
    pending: Arc<Mutex<Option<Pending>>>,
}
struct Pending {
    state: String,
    code_verifier: String,
    expires_at: Instant,
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
pub(super) async fn start(State(app): State<AppState>) -> impl IntoResponse {
    let endpoints = &app.oauth.endpoints;
    if endpoints.client_id.is_empty() || endpoints.client_secret.is_empty() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Server chưa cấu hình Google client",
        ));
    }
    let state = uuid::Uuid::new_v4().to_string();
    let code_verifier = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let code_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
    let mut url = Url::parse(&app.oauth.endpoints.authorize)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "OAuth không hợp lệ"))?;
    url.query_pairs_mut()
        .append_pair("client_id", &endpoints.client_id)
        .append_pair("redirect_uri", &app.oauth.redirect)
        .append_pair("response_type", "code")
        .append_pair("scope", "https://www.googleapis.com/auth/calendar.events")
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent")
        .append_pair("code_challenge", &code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", &state);
    *app.oauth.pending.lock().await = Some(Pending {
        state,
        code_verifier,
        expires_at: Instant::now() + SESSION_TTL,
    });
    Ok(Redirect::to(url.as_str()))
}
#[derive(Deserialize)]
pub(super) struct Callback {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}
#[derive(Deserialize)]
pub(super) struct Paste {
    url: String,
}
pub(super) async fn paste(
    State(app): State<AppState>,
    Form(form): Form<Paste>,
) -> impl IntoResponse {
    let raw = form.url.trim();
    if !raw.starts_with("http://") && !raw.starts_with("https://") {
        // Chỉ dán mã `code`: state lấy từ phiên đang chờ.
        if raw.is_empty() || raw.len() > 4096 || raw.bytes().any(|b| b.is_ascii_whitespace()) {
            return Err((StatusCode::BAD_REQUEST, "Mã không hợp lệ"));
        }
        let state = app
            .oauth
            .pending
            .lock()
            .await
            .as_ref()
            .map(|p| p.state.clone());
        return finish(
            app,
            Callback {
                code: Some(raw.into()),
                state,
                error: None,
            },
        )
        .await;
    }
    let url = Url::parse(raw).map_err(|_| (StatusCode::BAD_REQUEST, "URL không hợp lệ"))?;
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
        error: None,
    };
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => callback.code = Some(value.into()),
            "state" => callback.state = Some(value.into()),
            "error" => callback.error = Some(value.into()),
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
    if query.error.as_deref().is_some_and(|e| !e.is_empty()) {
        *pending = None;
        return Err((StatusCode::BAD_REQUEST, "Google từ chối quyền truy cập"));
    }
    if pending
        .as_ref()
        .is_some_and(|p| Instant::now() >= p.expires_at)
    {
        *pending = None;
        return Err((StatusCode::BAD_REQUEST, "Phiên đăng nhập đã hết hạn"));
    }
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
            ("client_id", app.oauth.endpoints.client_id.clone()),
            ("client_secret", app.oauth.endpoints.client_secret.clone()),
            ("redirect_uri", app.oauth.redirect.clone()),
            ("code_verifier", p.code_verifier.clone()),
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
            client_id: app.oauth.endpoints.client_id.clone(),
            client_secret: app.oauth.endpoints.client_secret.clone(),
            refresh_token: token.refresh_token,
        }),
    )
    .await
}
pub(super) async fn disconnect(State(app): State<AppState>) -> impl IntoResponse {
    *app.oauth.pending.lock().await = None;
    save(&app, Update::Calendar(CalendarSettings::default())).await
}
