//! The one table that says which features exist and how they are reached.
//! Adding a feature adds one row here; routing for tools, server tools, jobs and
//! tool schemas is derived from it.

use super::{calendar, countdowns, home, photos, reminders, timers, weather};
use crate::Screen;
use voice_client::ToolDef;

pub(crate) struct Spec {
    pub screen: Screen,
    /// Device MCP tool names starting with one of these belong to the feature.
    pub tool_prefixes: &'static [&'static str],
    /// Server-side tool names containing one of these (lowercase) open the screen.
    pub server_keywords: &'static [&'static str],
    /// Job ids starting with one of these are handled by the feature.
    pub job_prefixes: &'static [&'static str],
    pub definitions: fn() -> Vec<ToolDef>,
}

/// Order is the order of `tool_definitions()` and of server keyword matching.
pub(crate) const FEATURES: [Spec; 7] = [
    Spec {
        screen: Screen::Timers,
        tool_prefixes: &["hub_timer_"],
        server_keywords: &[],
        job_prefixes: &[],
        definitions: timers::tools::definitions,
    },
    Spec {
        screen: Screen::Countdowns,
        tool_prefixes: &["hub_countdown_"],
        server_keywords: &[],
        job_prefixes: &[],
        definitions: countdowns::tools::definitions,
    },
    Spec {
        screen: Screen::Reminders,
        tool_prefixes: &["hub_reminder_"],
        server_keywords: &[],
        job_prefixes: &[],
        definitions: reminders::tools::definitions,
    },
    Spec {
        screen: Screen::Calendar,
        tool_prefixes: &["hub_calendar_"],
        server_keywords: &["calendar", "event"],
        job_prefixes: calendar::JOB_PREFIXES,
        definitions: calendar::tools::definitions,
    },
    Spec {
        screen: Screen::Weather,
        tool_prefixes: &["hub_weather_"],
        server_keywords: &["weather", "forecast"],
        job_prefixes: &[weather::JOB],
        definitions: weather::tools::definitions,
    },
    Spec {
        screen: Screen::Home,
        tool_prefixes: &["hub_home_"],
        server_keywords: &["hass", "home", "light", "switch"],
        job_prefixes: &[home::JOB_PREFIX],
        definitions: home::tools::definitions,
    },
    Spec {
        screen: Screen::Photos,
        tool_prefixes: &["hub_image_", "hub_show_image"],
        server_keywords: &["image", "photo"],
        job_prefixes: &[photos::JOB],
        definitions: photos::tools::definitions,
    },
];

fn find(matches: impl Fn(&Spec) -> bool) -> Option<Screen> {
    FEATURES.iter().find(|spec| matches(spec)).map(|s| s.screen)
}

/// The screen a Device MCP tool drives (ADR 0008).
pub(crate) fn screen_for_hub_tool(name: &str) -> Option<Screen> {
    find(|spec| spec.tool_prefixes.iter().any(|p| name.starts_with(p)))
}

/// The screen a server-side tool opens. The hub only learns the tool's name, and
/// names come from the server's configuration, so they are matched by keyword;
/// an unknown name leaves the view as it is.
pub(crate) fn screen_for_server_tool(name: &str) -> Option<Screen> {
    let name = name.to_ascii_lowercase();
    find(|spec| spec.server_keywords.iter().any(|k| name.contains(k)))
}

/// The feature that owns a background job id.
pub(crate) fn screen_for_job(id: &str) -> Option<Screen> {
    find(|spec| spec.job_prefixes.iter().any(|p| id.starts_with(p)))
}

pub(crate) fn tool_definitions() -> Vec<ToolDef> {
    FEATURES.iter().flat_map(|spec| (spec.definitions)()).collect()
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
