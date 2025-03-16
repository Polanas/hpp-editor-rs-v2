use eframe::egui;

use crate::{hats::Hat, ui_text::UiText};

#[derive(Debug)]
pub struct NameGetter<T> {
    state: State<T>,
}

impl<T> Default for NameGetter<T> {
    fn default() -> Self {
        Self {
            state: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum NameGetterResult<T> {
    Closed,
    Confirmed(String, T),
}

type State<T> = NameGetterState<T>;

#[derive(Debug, Clone)]
enum NameGetterState<T> {
    Closed,
    Opened {
        message: String,
        buffer: String,
        data: T,
    },
}

impl<T> Default for NameGetterState<T> {
    fn default() -> Self {
        Self::Closed
    }
}

impl<T> NameGetter<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, message: String, data: T) {
        if matches!(self.state, State::Closed) {
            self.state = State::Opened {
                message,
                buffer: String::new(),
                data,
            };
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, text: &UiText) -> Option<NameGetterResult<T>> {
        if matches!(self.state, NameGetterState::Closed) {
            return None;
        }
        let modal = egui_modal::Modal::new(ctx, "Hat name modal");

        let mut result = None;
        modal.show(|ui| {
            let State::Opened {
                message, buffer, ..
            } = &mut self.state
            else {
                unreachable!()
            };
            modal.title(ui, message);
            ui.vertical_centered(|ui| {
                ui.add(egui::TextEdit::singleline(buffer));
            });

            modal.buttons(ui, |ui| {
                if modal.button(ui, text.get("15")).clicked() {
                    self.state = NameGetterState::Closed;
                    result = Some(NameGetterResult::Closed);
                    return;
                }
                if modal.button(ui, text.get("16")).clicked() {
                    let state = std::mem::replace(&mut self.state, NameGetterState::Closed);
                    let State::Opened { buffer, data, .. } = state else {
                        unreachable!()
                    };
                    result = Some(NameGetterResult::Confirmed(buffer, data));
                }
            });
        });
        modal.open();
        result
    }
}
