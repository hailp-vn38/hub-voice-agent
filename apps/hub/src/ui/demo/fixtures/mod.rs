//! Fixture mỗi tính năng một file; ticket sau thêm `mod <feature>;` và gọi vào đây.
use crate::App;

pub fn init(app: &App) {
    super::settings::init(app);
}

pub fn tick(_app: &App) {}
