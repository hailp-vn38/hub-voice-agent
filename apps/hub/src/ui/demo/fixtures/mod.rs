//! Fixture mỗi tính năng một file; ticket sau thêm `mod <feature>;` và gọi vào đây.
use crate::App;

pub mod home;

pub fn init(app: &App) {
    home::init(app);
}

pub fn tick(_app: &App) {}
