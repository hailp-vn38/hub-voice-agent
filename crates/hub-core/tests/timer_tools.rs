use hub_core::tool_definitions;

#[test]
fn timer_tools_have_object_schemas() {
    let tools = tool_definitions();
    assert!(tools.iter().any(|tool| tool.name == "hub_calendar_get"));
    assert!(tools.iter().any(|tool| tool.name == "hub_image_list"));
    assert!(tools.iter().any(|tool| tool.name == "hub_image_control"));
    assert!(tools.iter().any(|tool| tool.name == "hub_calendar_create"));
    assert!(tools.iter().any(|tool| tool.name.starts_with("hub_home_")));
    for tool in tools {
        assert_eq!(tool.input_schema["type"], "object", "{}", tool.name);
    }
}
