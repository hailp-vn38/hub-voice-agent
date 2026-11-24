use crate::{
    App, CalBlock, CalDay, CalDetail, CalKind, CalMonthDay, CalUpcoming, CalendarData,
    PendingState as UiPendingState,
};
use hub_core::present::{
    CalendarBlockView, CalendarDayView, CalendarDetailView, CalendarKind, CalendarMonthDayView,
    CalendarUpcomingView, CalendarView, PendingState,
};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

#[derive(Default)]
pub(super) struct CalendarInteraction {
    month_view: bool,
    detail_key: Option<String>,
}

impl CalendarInteraction {
    fn toggle_month(&mut self) {
        self.month_view = !self.month_view;
    }

    fn select_day(&mut self) {
        self.month_view = false;
    }

    fn open_detail(&mut self, key: String) {
        self.detail_key = Some(key);
    }

    fn close_detail(&mut self) {
        self.detail_key = None;
    }

    fn detail(
        &mut self,
        details: &[CalendarDetailView],
        upcoming: &[CalendarUpcomingView],
    ) -> Option<CalendarDetailView> {
        let key = self.detail_key.clone()?;
        let detail = details
            .iter()
            .find(|detail| detail.key == key)
            .cloned()
            .or_else(|| {
                upcoming
                    .iter()
                    .find(|item| item.key == key)
                    .map(|item| CalendarDetailView {
                        key: item.key.clone(),
                        title: item.title.clone(),
                        time: format!(
                            "{} · {}",
                            item.date,
                            if item.time.is_empty() {
                                "Cả ngày"
                            } else {
                                item.time.as_str()
                            }
                        ),
                        location: String::new(),
                        description: String::new(),
                    })
            });
        if detail.is_none() {
            self.detail_key = None;
        }
        detail
    }
}

#[derive(Default)]
struct CalendarBridgeState {
    interaction: CalendarInteraction,
    view: Option<CalendarView>,
}

thread_local! {
    static STATE: RefCell<CalendarBridgeState> = RefCell::default();
}

pub(super) fn apply(app: &App, view: &CalendarView) {
    let (month_view, detail) = STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.view = Some(view.clone());
        let detail = state.interaction.detail(&view.details, &view.upcoming);
        (state.interaction.month_view, detail)
    });

    let calendar = app.global::<CalendarData>();
    calendar.set_month_title(view.month_title.as_str().into());
    calendar.set_synced(view.synced.as_str().into());
    calendar.set_stale(view.stale);
    calendar.set_week(model(view.week.iter().map(day).collect()));
    calendar.set_selected_day(view.selected_day.as_str().into());
    calendar.set_blocks(model(view.blocks.iter().map(block).collect()));
    calendar.set_now_min(view.now_min);
    calendar.set_upcoming(model(view.upcoming.iter().map(upcoming).collect()));
    calendar.set_month_view(month_view);
    calendar.set_month(model(view.month.iter().map(month_day).collect()));
    calendar.set_detail(ui_detail(detail.as_ref()));
    calendar.set_focus(view.focus.as_str().into());
    calendar.set_focus_serial(view.focus_serial as i32);
}

pub(super) fn select_day(app: &App) {
    STATE.with(|state| state.borrow_mut().interaction.select_day());
    app.global::<CalendarData>().set_month_view(false);
}

pub(super) fn toggle_month(app: &App) {
    let month_view = STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.interaction.toggle_month();
        state.interaction.month_view
    });
    app.global::<CalendarData>().set_month_view(month_view);
}

pub(super) fn open_detail(app: &App, key: String) {
    STATE.with(|state| state.borrow_mut().interaction.open_detail(key));
    update_detail(app);
}

pub(super) fn close_detail(app: &App) {
    STATE.with(|state| state.borrow_mut().interaction.close_detail());
    update_detail(app);
}

fn update_detail(app: &App) {
    let detail = STATE.with(|state| {
        let CalendarBridgeState { interaction, view } = &mut *state.borrow_mut();
        let view = view.as_ref()?;
        interaction.detail(&view.details, &view.upcoming)
    });
    app.global::<CalendarData>()
        .set_detail(ui_detail(detail.as_ref()));
}

fn ui_detail(detail: Option<&CalendarDetailView>) -> CalDetail {
    match detail {
        Some(detail) => CalDetail {
            visible: true,
            title: detail.title.as_str().into(),
            time: detail.time.as_str().into(),
            location: detail.location.as_str().into(),
            description: detail.description.as_str().into(),
        },
        None => CalDetail {
            visible: false,
            title: "".into(),
            time: "".into(),
            location: "".into(),
            description: "".into(),
        },
    }
}

fn day(day: &CalendarDayView) -> CalDay {
    CalDay {
        key: day.key.as_str().into(),
        weekday: day.weekday.as_str().into(),
        day: day.day.as_str().into(),
        dots: day.dots,
        today: day.today,
    }
}

fn block(block: &CalendarBlockView) -> CalBlock {
    CalBlock {
        key: block.key.as_str().into(),
        kind: match block.kind {
            CalendarKind::Event => CalKind::Event,
            CalendarKind::Reminder => CalKind::Reminder,
        },
        title: block.title.as_str().into(),
        time: block.time.as_str().into(),
        start_min: block.start_min,
        end_min: block.end_min,
        all_day: block.all_day,
        ongoing: block.ongoing,
        fresh: block.fresh,
        pending: match block.pending {
            PendingState::None => UiPendingState::None,
            PendingState::Saving => UiPendingState::Saving,
            PendingState::Saved => UiPendingState::Saved,
            PendingState::Failed => UiPendingState::Failed,
        },
        col: block.col,
        cols: block.cols,
    }
}

fn upcoming(item: &CalendarUpcomingView) -> CalUpcoming {
    CalUpcoming {
        key: item.key.as_str().into(),
        date: item.date.as_str().into(),
        time: item.time.as_str().into(),
        title: item.title.as_str().into(),
        more: item.more,
        fresh: item.fresh,
    }
}

fn month_day(day: &CalendarMonthDayView) -> CalMonthDay {
    CalMonthDay {
        key: day.key.as_str().into(),
        day: day.day.as_str().into(),
        dots: day.dots,
        in_month: day.in_month,
        today: day.today,
    }
}

fn model<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}

#[cfg(test)]
#[path = "calendar_tests.rs"]
mod tests;
