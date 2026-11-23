use super::{Screen, screen_for_hub_tool, screen_for_server_tool};

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
