use super::color_from_hex;
use slint::Color;

#[test]
fn parses_hub_core_aqi_hex_color() {
    assert_eq!(color_from_hex("#FF7E00"), Color::from_rgb_u8(255, 126, 0));
}

#[test]
fn empty_aqi_color_uses_default_color() {
    assert_eq!(color_from_hex(""), Color::default());
}
