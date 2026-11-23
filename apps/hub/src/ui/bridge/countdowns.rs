use crate::{App, Countdown as UiCountdown, CountdownUnit as UiCountdownUnit, CountdownsData};
use hub_core::present::{CountdownItemView, CountdownUnit, CountdownsView};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;

#[cfg(test)]
#[path = "countdowns_tests.rs"]
mod tests;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct DisplayItem {
    pub item: CountdownItemView,
    pub fading: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Display {
    pub title: String,
    pub main: DisplayItem,
    pub has_main: bool,
    pub others: Vec<DisplayItem>,
    pub confetti_serial: u64,
    pub detail_open: bool,
}

#[derive(Clone)]
struct Removing {
    item: CountdownItemView,
    was_main: bool,
}

#[derive(Default)]
pub(super) struct Presentation {
    view: Option<CountdownsView>,
    selected_key: Option<String>,
    detail_open: bool,
    removing: Vec<(String, Removing)>,
}

impl Presentation {
    pub(super) fn update(&mut self, view: &CountdownsView) -> Display {
        self.view = Some(view.clone());
        self.reconcile_selection();
        self.display()
    }

    pub(super) fn select(&mut self, key: &str) -> Option<Display> {
        if !self.contains(key) || self.is_removing(key) {
            return None;
        }
        self.selected_key = Some(key.to_owned());
        Some(self.display())
    }

    pub(super) fn open_detail(&mut self) -> Option<Display> {
        let current = self.display();
        if !current.has_main || current.main.fading {
            return None;
        }
        self.selected_key = Some(current.main.item.key);
        self.detail_open = true;
        Some(self.display())
    }

    pub(super) fn close_detail(&mut self) -> Option<Display> {
        if self.view.is_none() {
            return None;
        }
        self.detail_open = false;
        Some(self.display())
    }

    pub(super) fn remove(&mut self, key: &str) -> Option<(Display, i64)> {
        let id = key.parse().ok()?;
        let current = self.display();
        let item = std::iter::once(&current.main)
            .chain(current.others.iter())
            .find(|item| item.item.key == key && !item.fading)?
            .item
            .clone();
        self.removing.push((
            key.to_owned(),
            Removing {
                was_main: current.has_main && current.main.item.key == key,
                item,
            },
        ));
        self.detail_open = false;
        Some((self.display(), id))
    }

    pub(super) fn finish_remove(&mut self, key: &str) -> Display {
        self.removing
            .retain(|(removing_key, _)| removing_key != key);
        if self.selected_key.as_deref() == Some(key) {
            self.selected_key = None;
            self.detail_open = false;
        }
        self.reconcile_selection();
        self.display()
    }

    fn contains(&self, key: &str) -> bool {
        self.view.as_ref().is_some_and(|view| {
            (view.has_main && view.main.key == key)
                || view.others.iter().any(|item| item.key == key)
        }) || self
            .removing
            .iter()
            .any(|(removing_key, _)| removing_key == key)
    }

    fn is_removing(&self, key: &str) -> bool {
        self.removing
            .iter()
            .any(|(removing_key, _)| removing_key == key)
    }

    fn reconcile_selection(&mut self) {
        if self
            .selected_key
            .as_deref()
            .is_some_and(|key| !self.contains(key))
        {
            self.selected_key = None;
            self.detail_open = false;
        }
    }

    fn display(&self) -> Display {
        let Some(view) = &self.view else {
            return Display {
                title: String::new(),
                main: DisplayItem {
                    item: CountdownItemView::default(),
                    fading: false,
                },
                has_main: false,
                others: Vec::new(),
                confetti_serial: 0,
                detail_open: false,
            };
        };

        let mut items = Vec::new();
        if view.has_main {
            items.push(view.main.clone());
        }
        items.extend(view.others.iter().cloned());

        let selected = self
            .selected_key
            .as_deref()
            .and_then(|key| items.iter().find(|item| item.key == key).cloned())
            .or_else(|| {
                self.selected_key.as_deref().and_then(|key| {
                    self.removing
                        .iter()
                        .find(|(removing_key, _)| removing_key == key)
                        .map(|(_, removing)| removing.item.clone())
                })
            });
        let pinned_main = self
            .removing
            .iter()
            .find(|(_, removing)| removing.was_main)
            .map(|(_, removing)| removing.item.clone());
        let main = selected
            .or(pinned_main)
            .or_else(|| view.has_main.then(|| view.main.clone()));
        let has_main = main.is_some();
        let main_key = main.as_ref().map(|item| item.key.as_str());

        for (key, removing) in &self.removing {
            if !items.iter().any(|item| item.key == *key) {
                items.push(removing.item.clone());
            }
        }

        let others = items
            .into_iter()
            .filter(|item| Some(item.key.as_str()) != main_key)
            .map(|item| DisplayItem {
                fading: self.is_removing(&item.key),
                item,
            })
            .collect();
        let main = main.unwrap_or_default();

        Display {
            title: view.title.clone(),
            main: DisplayItem {
                fading: self.is_removing(&main.key),
                item: main,
            },
            has_main,
            others,
            confetti_serial: view.confetti_serial,
            detail_open: self.detail_open,
        }
    }
}

pub(super) fn apply(app: &App, display: &Display) {
    let data = app.global::<CountdownsData>();
    data.set_title(display.title.as_str().into());
    data.set_main(to_ui_countdown(&display.main));
    data.set_has_main(display.has_main);
    data.set_others(ModelRc::from(Rc::new(VecModel::from(
        display
            .others
            .iter()
            .map(to_ui_countdown)
            .collect::<Vec<_>>(),
    ))));
    data.set_detail_open(display.detail_open);
    data.set_confetti_serial(display.confetti_serial as i32);
}

fn to_ui_countdown(item: &DisplayItem) -> UiCountdown {
    UiCountdown {
        key: item.item.key.as_str().into(),
        label: item.item.label.as_str().into(),
        value: item.item.value.as_str().into(),
        unit: match item.item.unit {
            CountdownUnit::Days => UiCountdownUnit::Days,
            CountdownUnit::Live => UiCountdownUnit::Live,
            CountdownUnit::Today => UiCountdownUnit::Today,
            CountdownUnit::Passed => UiCountdownUnit::Passed,
        },
        target: item.item.target.as_str().into(),
        lunar: item.item.lunar.as_str().into(),
        repeat_text: item.item.repeat_text.as_str().into(),
        progress: item.item.progress,
        progress_text: item.item.progress_text.as_str().into(),
        fading: item.fading,
    }
}
