use std::{path::Path, rc::Rc};

use anyhow::{Context, Result, bail};
use eframe::{
    egui::{self},
    glow,
};
use log::{error, info, warn};

use crate::{
    hats::Hat,
    tabs::{FrameData, Tab, TabVariant, Tabs},
    ui_text::{Language, UiText},
};

pub struct EditorApp {
    ui_text: Rc<UiText>,
    tabs: Tabs,
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ui_text = Rc::new(UiText::new(
            Language::English,
            include_str!("../translations.json"),
        ));

        let tabs = Tabs::new(&ui_text);
        Self { ui_text, tabs }
    }

    fn draw_app(&mut self, gl: &glow::Context, ui: &mut egui::Ui) {
        self.tabs.ui(ui, FrameData {
            ui_text: &self.ui_text,
        });
    }

    fn add_new_hat(&mut self) {
        let name = self.tabs.new_hat_tab_name(&self.ui_text);
        info!("new hat added: {}", name);
        let tab = Tab::new_hat_tab(name, Hat::default(), None);

        self.tabs.dock_state.push_to_focused_leaf(tab);
    }

    fn open_hat_with_dialog(&mut self, gl: &glow::Context) -> Result<()> {
        let path = match rfd::FileDialog::new().pick_file() {
            Some(path) => path,
            None => bail!("could not pick file"),
        };
        self.open_hat(gl, &path)
    }

    fn open_hat(&mut self, gl: &glow::Context, path: impl AsRef<Path>) -> Result<()> {
        if self.tabs.dock_state.iter_all_tabs().any(|t| {
            if let TabVariant::HatElement { hat, .. } = &t.1.variant {
                hat.path()
                    .as_ref()
                    .map(|p| *p == path.as_ref())
                    .unwrap_or_default()
            } else {
                false
            }
        }) {
            bail!("hat with this path is already added: {:?}", path.as_ref());
        }

        let hat = Hat::load(path, gl)?;
        info!("hat {} loaded successfully", hat.name().as_ref().unwrap());
        let selected_hat_id = hat.elements().next().map(|e| e.id());
        //add textures to reloader
        let tab = Tab::new_hat_tab(hat.name().cloned().unwrap(), hat, selected_hat_id);
        self.tabs.dock_state.push_to_focused_leaf(tab);
        Ok(())
    }

    fn save_hat_as(&mut self) -> Result<()> {
        let path = rfd::FileDialog::new()
            .pick_folder()
            .context("could not pick file")?;
        let last_tab = self
            .last_interacted_tab_mut()
            .context("could not find last interacted tab")?;
        let TabVariant::HatElement { hat, .. } = &mut last_tab.variant else {
            bail!("expected hat tab");
        };

        hat.save(&path)?;
        *hat.path_mut() = Some(path);

        Ok(())
    }

    fn save_hat(&mut self) -> Result<()> {
        let last_tab = self
            .last_interacted_tab()
            .context("could not find last interacted tab")?;
        let TabVariant::HatElement { hat, .. } = &last_tab.variant else {
            bail!("expected hat tab");
        };
        hat.save(hat.path().unwrap())
    }

    fn draw_menu(&mut self, gl: &glow::Context, ui: &mut egui::Ui) {
        let text = self.ui_text.clone();
        egui::menu::bar(ui, |ui| {
            ui.menu_button(text.get("Hat"), |ui| {
                if ui.button(text.get("New")).clicked() {
                    self.add_new_hat();
                    ui.close_menu();
                }
                if ui.button(text.get("Open")).clicked() {
                    if let Err(err) = self.open_hat_with_dialog(gl) {
                        error!("while opening hat: {}", err.to_string());
                    }
                    ui.close_menu();
                }
                if ui.button(text.get("Save")).clicked() {
                    if let Err(err) = self.save_hat() {
                        error!("while saving hat: {}", err.to_string());
                    }
                    ui.close_menu();
                }
                if ui.button(text.get("Save as")).clicked() {
                    if let Err(err) = self.save_hat() {
                        error!("while saving hat: {}", err.to_string());
                    }
                    ui.close_menu();
                }
                ui.collapsing(text.get("Recent"), |ui| {});
            });

            ui.menu_button(text.get("Elements"), |ui| {
                self.draw_elements_menu(gl, ui);
            });
            ui.menu_button(text.get("Help"), |ui| {
                if ui.button(text.get("Open help tab")).clicked() {
                    ui.close_menu();
                }
            });
            ui.menu_button(text.get("Settings"), |ui| {
                self.draw_settings_menu(ui);
            });
            ui.menu_button(text.get("Other"), |ui| {
                if ui.button(text.get("Open home tab")).clicked() {
                    ui.close_menu();
                }
            });
        });
    }

    fn draw_settings_menu(&mut self, ui: &mut egui::Ui) {}

    fn draw_elements_menu(&mut self, gl: &glow::Context, ui: &mut egui::Ui) {}

    fn draw_elements_add_menu(&mut self, gl: &glow::Context, ui: &mut egui::Ui) {}

    fn draw_elements_select_menu(&mut self, gl: &glow::Context, ui: &mut egui::Ui) {}

    fn last_interacted_tab(&mut self) -> Option<&Tab> {
        self.tabs
            .dock_state
            .find_active_focused()
            .map(|(_, tab)| &*tab)
    }

    fn last_interacted_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs
            .dock_state
            .find_active_focused()
            .map(|(_, tab)| &mut *tab)
    }
}
impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);
        let gl = &frame.gl().cloned().unwrap();
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_menu(gl, ui);
            self.draw_app(gl, ui);
        });
    }
}
