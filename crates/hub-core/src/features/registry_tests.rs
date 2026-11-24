use super::*;

#[test]
fn hub_tools_map_to_their_feature_screen() {
    assert_eq!(screen_for_hub_tool("hub_timer_start"), Some(Screen::Timers));
    assert_eq!(screen_for_hub_tool("hub_show_image"), Some(Screen::Photos));
    assert_eq!(screen_for_hub_tool("hub_unknown"), None);
}

#[test]
fn server_tools_map_by_keyword() {
    assert_eq!(
        screen_for_server_tool("google_calendar_list_events"),
        Some(Screen::Calendar)
    );
    assert_eq!(screen_for_server_tool("HassTurnOn"), Some(Screen::Home));
    assert_eq!(screen_for_server_tool("get_weather"), Some(Screen::Weather));
    assert_eq!(screen_for_server_tool("server_search"), None);
}

#[test]
fn jobs_map_to_their_feature_screen() {
    assert_eq!(screen_for_job("weather"), Some(Screen::Weather));
    assert_eq!(screen_for_job("home:light.kitchen"), Some(Screen::Home));
    assert_eq!(screen_for_job("calendar-sync"), Some(Screen::Calendar));
    assert_eq!(screen_for_job("calendar-create-1"), Some(Screen::Calendar));
    assert_eq!(screen_for_job("other"), None);
}

#[test]
fn every_defined_tool_routes_back_to_its_own_feature() {
    for spec in &FEATURES {
        for tool in (spec.definitions)() {
            assert_eq!(
                screen_for_hub_tool(&tool.name),
                Some(spec.screen),
                "{} is not routed to its feature",
                tool.name
            );
        }
    }
}

#[test]
fn each_screen_has_at_most_one_row() {
    for (i, a) in FEATURES.iter().enumerate() {
        assert!(FEATURES[i + 1..].iter().all(|b| b.screen != a.screen));
    }
}
