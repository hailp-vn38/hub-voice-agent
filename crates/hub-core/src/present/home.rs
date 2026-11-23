use crate::{HomeDomain, HomeEntity, UiState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeView {
    pub tiles: Vec<HomeTileView>,
    pub configured: bool,
    pub connected: bool,
    pub updated: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeTileView {
    pub key: String,
    pub name: String,
    pub kind: HomeKind,
    pub on: bool,
    pub value: String,
    pub pending: bool,
    pub stale: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomeKind {
    Light,
    Switch,
    Sensor,
}

pub(super) fn from_state(state: &UiState) -> HomeView {
    let tiles = state
        .home
        .iter()
        .map(|entity| tile(entity, state))
        .collect();
    let updated = state
        .home_last_updated
        .map(|seconds| super::timers::time_at(&state.now, seconds))
        .unwrap_or_default();
    HomeView {
        tiles,
        configured: state.home_configured,
        connected: state.home_connected,
        updated,
    }
}

fn tile(entity: &HomeEntity, state: &UiState) -> HomeTileView {
    let kind = match entity.domain() {
        HomeDomain::Light => HomeKind::Light,
        HomeDomain::Switch => HomeKind::Switch,
        HomeDomain::Sensor => HomeKind::Sensor,
    };
    let value = if kind == HomeKind::Sensor {
        let localized = if entity.state.parse::<f64>().is_ok() {
            entity.state.replace('.', ",")
        } else {
            entity.state.clone()
        };
        match entity.unit.as_deref() {
            Some(unit) if !unit.is_empty() => format!("{localized} {unit}"),
            _ => localized,
        }
    } else {
        String::new()
    };
    HomeTileView {
        key: entity.id.clone(),
        name: entity.name.clone(),
        kind,
        on: kind != HomeKind::Sensor && entity.state == "on",
        value,
        pending: state.home_pending.iter().any(|id| id == &entity.id),
        stale: entity.stale,
    }
}
