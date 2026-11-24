use qrcode::{Color, QrCode};
use slint::{Rgba8Pixel, SharedPixelBuffer};

#[cfg(test)]
#[path = "qr_tests.rs"]
mod tests;

/// Số điểm ảnh cho mỗi ô QR.
const SCALE: u32 = 6;
/// Viền trắng quanh mã, tính theo ô (chuẩn tối thiểu là 4).
const QUIET_ZONE: u32 = 4;

/// Vẽ mã QR của `text` thành ảnh đen trên nền trắng; `None` nếu `text` rỗng hoặc quá dài.
pub(super) fn render(text: &str) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
    if text.is_empty() {
        return None;
    }
    let code = QrCode::new(text.as_bytes()).ok()?;
    let modules = code.width() as u32;
    let side = (modules + 2 * QUIET_ZONE) * SCALE;
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(side, side);
    let colors = code.to_colors();
    for (index, pixel) in buffer.make_mut_slice().iter_mut().enumerate() {
        let (x, y) = (index as u32 % side / SCALE, index as u32 / side / SCALE);
        let dark = x
            .checked_sub(QUIET_ZONE)
            .zip(y.checked_sub(QUIET_ZONE))
            .filter(|(x, y)| *x < modules && *y < modules)
            .is_some_and(|(x, y)| colors[(y * modules + x) as usize] == Color::Dark);
        let value = if dark { 0 } else { 255 };
        *pixel = Rgba8Pixel::new(value, value, value, 255);
    }
    Some(buffer)
}
