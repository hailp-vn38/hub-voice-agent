use super::modules;

#[test]
fn has_square_grid_with_finder_pattern() {
    let (n, dark) = modules("WIFI:T:nopass;S:AI-Hub-Setup;;");
    assert_eq!(dark.len(), n * n);
    // Ô tìm vị trí 7x7 ở góc trên trái: hàng đầu đặc, hàng hai viền đặc rồi trắng.
    assert!(dark[..7].iter().all(|d| *d));
    assert!(dark[n] && !dark[n + 1] && dark[n + 6]);
}
