use super::*;

#[test]
fn qr_has_a_white_quiet_zone_and_dark_finder_corner() {
    let image = render("http://192.168.1.20:8080").unwrap();
    let (width, height) = (image.width(), image.height());
    assert_eq!(width, height);
    let pixels = image.as_slice();
    let at = |x: u32, y: u32| pixels[(y * width + x) as usize];
    assert_eq!(at(0, 0), Rgba8Pixel::new(255, 255, 255, 255));
    // Ô đầu tiên của mẫu định vị nằm ngay sau viền.
    let first = QUIET_ZONE * SCALE;
    assert_eq!(at(first, first), Rgba8Pixel::new(0, 0, 0, 255));
}

#[test]
fn empty_text_has_no_qr() {
    assert!(render("").is_none());
}
