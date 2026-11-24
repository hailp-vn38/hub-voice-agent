//! Tool descriptions point the agent at sibling tools; those names must exist.

#[test]
fn every_tool_named_in_a_description_exists() {
    let tools = hub_core::tool_definitions();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    for tool in &tools {
        for word in tool
            .description
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        {
            if word.starts_with("hub_") {
                assert!(
                    names.contains(&word),
                    "{} names unknown tool {word}",
                    tool.name
                );
            }
        }
    }
}

#[test]
fn time_tools_point_at_each_other() {
    let tools = hub_core::tool_definitions();
    let description = |name: &str| {
        tools
            .iter()
            .find(|t| t.name == name)
            .map(|t| t.description.clone())
            .unwrap()
    };
    let timer = description("hub_timer_start");
    let countdown = description("hub_countdown_create");
    let reminder = description("hub_reminder_create");
    assert!(timer.contains("hub_reminder_create") && timer.contains("hub_countdown_create"));
    assert!(countdown.contains("hub_timer_start") && countdown.contains("hub_reminder_create"));
    assert!(reminder.contains("hub_timer_start") && reminder.contains("hub_countdown_create"));
}
