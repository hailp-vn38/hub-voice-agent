//! Kịch bản demo cho Spawn / thẻ nổi (ticket 06).
use crate::{App, FloatingCard, FloatingCards, PendingState, Spawn, SpawnDemo};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;

/// Ghi Spawn: mục có `key` bay từ Orb vào chỗ của nó.
pub(in crate::ui::demo) fn from_orb(app: &App, key: &str) {
    let s = app.global::<Spawn>();
    s.set_key(key.into());
    s.set_serial(s.get_serial() + 1);
}

pub fn spawn_cell3(app: &App) {
    app.global::<SpawnDemo>().set_show(true);
    app.global::<SpawnDemo>().set_pending(PendingState::None);
    from_orb(app, "cell3");
}

pub fn save_ok(app: &App) {
    app.global::<SpawnDemo>().set_pending(PendingState::Saving);
    let weak = app.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(1200), move || {
        if let Some(app) = weak.upgrade() {
            app.global::<SpawnDemo>().set_pending(PendingState::Saved);
        }
    });
}

pub fn save_fail(app: &App) {
    app.global::<SpawnDemo>().set_pending(PendingState::Saving);
    let weak = app.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(1200), move || {
        if let Some(app) = weak.upgrade() {
            app.global::<SpawnDemo>().set_pending(PendingState::Failed);
        }
    });
}

pub fn floating(app: &App) {
    let f = app.global::<FloatingCards>();
    f.set_fading(false);
    let card = FloatingCard {
        key: "ev1".into(),
        title: "Đã tạo sự kiện".into(),
        body: "Họp nhóm · 15:00 ngày mai".into(),
        icon: "calendar".into(),
    };
    f.set_cards(ModelRc::from(Rc::new(VecModel::from(vec![card]))));
    from_orb(app, "ev1");
}

pub fn floating_fade(app: &App) {
    app.global::<FloatingCards>().set_fading(true);
}
