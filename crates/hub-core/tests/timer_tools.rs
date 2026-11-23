use hub_core::tool_definitions;

#[test]
fn timer_tools_have_object_schemas() {
    let tools = tool_definitions();
    assert_eq!(tools.len(), 8);
    for tool in tools {
        assert_eq!(tool.input_schema["type"], "object", "{}", tool.name);
    }
}
