mod oauth;

use crate::{CalendarSettings, HomeSettings, UserInput, WeatherSettings};
use axum::{
    Form, Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use serde::Deserialize;
use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot},
};

pub enum Update {
    Voice(String),
    Home(HomeSettings),
    Weather(WeatherSettings),
    Calendar(CalendarSettings),
}
#[derive(Clone)]
pub struct OAuthEndpoints {
    pub authorize: String,
    pub token: String,
}
impl Default for OAuthEndpoints {
    fn default() -> Self {
        Self {
            authorize: "https://accounts.google.com/o/oauth2/v2/auth".into(),
            token: "https://oauth2.googleapis.com/token".into(),
        }
    }
}
#[derive(Clone)]
struct AppState {
    tx: mpsc::Sender<UserInput>,
    oauth: oauth::Flow,
}

pub async fn serve(listener: TcpListener, tx: mpsc::Sender<UserInput>) -> std::io::Result<()> {
    serve_with_oauth(listener, tx, OAuthEndpoints::default()).await
}
pub async fn serve_with_oauth(
    listener: TcpListener,
    tx: mpsc::Sender<UserInput>,
    endpoints: OAuthEndpoints,
) -> std::io::Result<()> {
    let redirect = format!(
        "http://127.0.0.1:{}/oauth/callback",
        listener.local_addr()?.port()
    );
    let state = AppState {
        tx,
        oauth: oauth::Flow::new(endpoints, redirect),
    };
    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("index.html")) }))
        .route("/voice", post(voice))
        .route("/home", post(home))
        .route("/weather", post(weather))
        .route("/oauth/start", post(oauth::start))
        .route("/oauth/callback", get(oauth::callback))
        .route("/oauth/paste", post(oauth::paste))
        .with_state(state);
    axum::serve(listener, app).await
}
async fn save(
    state: &AppState,
    update: Update,
) -> Result<&'static str, (StatusCode, &'static str)> {
    let (tx, rx) = oneshot::channel();
    state
        .tx
        .send(UserInput::Setup(update, tx))
        .await
        .map_err(|_| (StatusCode::SERVICE_UNAVAILABLE, "Hub không hoạt động"))?;
    match rx.await {
        Ok(Ok(())) => Ok("Đã lưu"),
        Ok(Err(_)) => Err((StatusCode::BAD_REQUEST, "Cấu hình không hợp lệ")),
        Err(_) => Err((StatusCode::SERVICE_UNAVAILABLE, "Hub không hoạt động")),
    }
}
#[derive(Deserialize)]
struct VoiceForm {
    ota_url: String,
}
async fn voice(State(state): State<AppState>, Form(form): Form<VoiceForm>) -> impl IntoResponse {
    save(&state, Update::Voice(form.ota_url)).await
}
#[derive(Deserialize)]
struct HomeForm {
    url: String,
    token: String,
}
async fn home(State(state): State<AppState>, Form(form): Form<HomeForm>) -> impl IntoResponse {
    save(
        &state,
        Update::Home(HomeSettings {
            url: form.url,
            token: form.token,
        }),
    )
    .await
}
#[derive(Deserialize)]
struct WeatherForm {
    latitude: f64,
    longitude: f64,
}
async fn weather(
    State(state): State<AppState>,
    Form(form): Form<WeatherForm>,
) -> impl IntoResponse {
    if !(-90.0..=90.0).contains(&form.latitude) || !(-180.0..=180.0).contains(&form.longitude) {
        return Err((StatusCode::BAD_REQUEST, "Tọa độ không hợp lệ"));
    }
    save(
        &state,
        Update::Weather(WeatherSettings {
            latitude: Some(form.latitude),
            longitude: Some(form.longitude),
        }),
    )
    .await
}
