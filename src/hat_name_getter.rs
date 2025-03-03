use eframe::egui;
use log::error;

use crate::{hats::Hat, ui_text::UiText};

#[derive(Debug, Clone, Copy)]
pub enum HatNameGetterAction {
    SetNameAndSave,
    SetName,
}

#[derive(Debug, Default)]
pub struct HatNameGetter {
    state: State,
}

type State = HatNameGetterState;
type Action = HatNameGetterAction;

#[derive(Debug, Default, Clone)]
enum HatNameGetterState {
    #[default]
    Closed,
    Opened(Action),
}

impl HatNameGetter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, action: Action) {
        if matches!(self.state, State::Closed) {
            self.state = State::Opened(action);
        }
    }

    pub fn update(&mut self, hat: Option<&mut Hat>, ctx: &egui::Context, text: &UiText) {
        if matches!(self.state, State::Closed) {
            return;
        }
        let Some(hat) = hat else {
            return;
        };
        let modal = egui_modal::Modal::new(ctx, "Hat name modal");

        modal.show(|ui| {
            modal.title(ui, text.get("14"));
            ui.vertical_centered(|ui| {
                ui.add(egui::TextEdit::singleline(hat.name_mut()));
            });

            modal.buttons(ui, |ui| {
                if modal.button(ui, text.get("15")).clicked() {
                    self.state = HatNameGetterState::Closed;
                    return;
                }
                if modal.button(ui, text.get("16")).clicked() {
                    let State::Opened(action) = self.state else {
                        unreachable!()
                    };
                    if matches!(action, Action::SetNameAndSave) {
                        panic!("idk what to do really");
                        // if let Err(err) = hat.save_as() {
                        //     error!("while saving hat: {}", err.to_string());
                        // }
                    }
                    self.state = HatNameGetterState::Closed;
                }
            });
        });
        modal.open();
    }
}
