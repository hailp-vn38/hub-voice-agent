//! QR demo sinh từ chuỗi giả bằng crate `qrcode`.
use qrcode::QrCode;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

const SCALE: usize = 8;
const QUIET: usize = 4;

/// (cạnh, ô tối theo hàng) của mã QR, chưa có viền trắng.
fn modules(text: &str) -> (usize, Vec<bool>) {
    let code = QrCode::new(text.as_bytes()).expect("chuỗi demo vừa một mã QR");
    let n = code.width();
    (n, code.to_colors().iter().map(|c| *c == qrcode::Color::Dark).collect())
}

pub fn image(text: &str) -> Image {
    let (n, dark) = modules(text);
    let side = (n + 2 * QUIET) * SCALE;
    let mut buf = SharedPixelBuffer::<Rgba8Pixel>::new(side as u32, side as u32);
    for (i, px) in buf.make_mut_slice().iter_mut().enumerate() {
        let (x, y) = (i % side / SCALE, i / side / SCALE);
        let on = x >= QUIET && y >= QUIET && x < n + QUIET && y < n + QUIET && dark[(y - QUIET) * n + x - QUIET];
        let v = if on { 0 } else { 255 };
        *px = Rgba8Pixel { r: v, g: v, b: v, a: 255 };
    }
    Image::from_rgba8(buf)
}

#[cfg(test)]
#[path = "qr_tests.rs"]
mod tests;
