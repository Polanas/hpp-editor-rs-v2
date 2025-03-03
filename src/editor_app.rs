use std::{fs::File, io::Write, path::Path, rc::Rc};

use anyhow::{Context, Result, bail};
use eframe::{
    egui::{self},
    glow,
};
use log::{error, info, warn};

use crate::{
    hat_name_getter::{HatNameGetter, HatNameGetterAction},
    hats::Hat,
    hats_data::HatData,
    tabs::{FrameData, Tab, Tabs},
    ui_text::{Language, UiText},
};

pub struct EditorApp {
    ui_text: Rc<UiText>,
    tabs: Tabs,
    hat_name_getter: HatNameGetter,
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ui_text = Rc::new(UiText::new(
            Language::English,
            include_str!("../translations.json"),
        ));

        let tabs = Tabs::new(&ui_text);
        Self {
            ui_text,
            tabs,
            hat_name_getter: HatNameGetter::default(),
        }
    }

    fn draw_app(&mut self, gl: &glow::Context, ui: &mut egui::Ui) {
        let frame_data = FrameData {
            ui_text: &self.ui_text,
            clicked_rename_hat: false,
        };
        let frame_result = self.tabs.ui(ui, frame_data);
        if frame_result.clicked_rename_hat {
            self.hat_name_getter.open(HatNameGetterAction::SetName);
        }
        if frame_result.added_hat {
            if let Err(err) = self.open_hat_with_dialog(gl) {
                error!("while opening hat: {}", err.to_string());
            }
        }
    }

    fn open_hat_with_dialog(&mut self, gl: &glow::Context) -> Result<()> {
        let path = match rfd::FileDialog::new().pick_folder() {
            Some(path) => path,
            None => bail!("could not pick folder"),
        };
        self.open_hat(gl, &path)
    }

    fn open_hat(&mut self, gl: &glow::Context, path: impl AsRef<Path>) -> Result<()> {
        if self
            .tabs
            .dock_state
            .iter_all_tabs()
            .any(|t| matches!(&t.1, Tab::HatElement { hat, .. } if hat.path() == path.as_ref()))
        {
            bail!("hat with this path is already added: {:?}", path.as_ref());
        }

        let hat = Hat::load(path, gl)?;
        info!("hat {} loaded successfully", hat.name());
        let selected_hat_id = hat.elements().next().map(|e| e.id());
        //add textures to reloader
        let tab = Tab::new_hat_tab(hat, selected_hat_id);
        self.tabs.dock_state.push_to_focused_leaf(tab);
        Ok(())
    }

    // fn save_hat_as(&mut self) -> Result<()> {
    //     let last_tab = self
    //         .last_interacted_tab_mut()
    //         .context("could not find last interacted tab")?;
    //     let Tab::HatElement { hat, .. } = last_tab else {
    //         bail!("expected hat tab");
    //     };
    //     hat.save_as()
    // }

    fn save_hat(&mut self) -> Result<()> {
        let last_tab = self
            .last_interacted_tab()
            .context("could not find last interacted tab")?;
        let Tab::HatElement { hat, .. } = last_tab else {
            bail!("expected hat tab");
        };
        hat.save(hat.path())
    }

    fn export_hat_to_file_as(&mut self) -> Result<()> {
        let last_tab = self
            .last_interacted_tab()
            .context("could not find last interacted tab")?;
        let Tab::HatElement { hat, .. } = last_tab else {
            bail!("expected hat tab");
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter(".hatspp", &["hatspp"])
            .save_file()
        else {
            bail!("could not pick file");
        };
        hat.export_to_file(path)
    }

    fn export_hat_to_file(&mut self) -> Result<()> {
        let last_tab = self
            .last_interacted_tab()
            .context("could not find last interacted tab")?;
        let Tab::HatElement { hat, .. } = last_tab else {
            bail!("expected hat tab");
        };
        hat.export_to_file(hat.path().join("hat.hatspp"))
    }

    fn can_export(&mut self) -> bool {
        matches!(self.last_interacted_tab(), Some(Tab::HatElement { .. }))
    }

    fn can_save(&mut self) -> bool {
        matches!(self.last_interacted_tab(), Some(Tab::HatElement { .. }))
    }

    fn draw_menu(&mut self, gl: &glow::Context, ui: &mut egui::Ui) {
        let text = self.ui_text.clone();
        egui::menu::bar(ui, |ui| {
            ui.menu_button(text.get("Hat"), |ui| {
                if ui.button(text.get("New")).clicked() {
                    if let Err(err) = self.add_new_hat_template() {
                        error!("while adding new hat: {}", err.to_string());
                    }
                    ui.close_menu();
                }
                if ui.button(text.get("Open")).clicked() {
                    if let Err(err) = self.open_hat_with_dialog(gl) {
                        error!("while opening hat: {}", err.to_string());
                    }
                    ui.close_menu();
                }
                if ui
                    .add_enabled(self.can_save(), egui::Button::new(text.get("Save")))
                    .clicked()
                {
                    if let Err(err) = self.save_hat() {
                        error!("while saving hat: {}", err.to_string());
                    }
                    ui.close_menu();
                }
                if ui
                    .add_enabled(self.can_export(), egui::Button::new(text.get("Export")))
                    .clicked()
                {
                    if let Err(err) = self.export_hat_to_file() {
                        error!("while exporting hat to file: {}", err.to_string())
                    }
                    ui.close_menu();
                }
                if ui
                    .add_enabled(self.can_export(), egui::Button::new(text.get("Export as")))
                    .clicked()
                {
                    if let Err(err) = self.export_hat_to_file_as() {
                        error!("while exporting hat to file: {}", err.to_string())
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

    fn last_interacted_tab_hat(&mut self) -> Option<&Hat> {
        self.last_interacted_tab().and_then(|t| {
            if let Tab::HatElement { hat, .. } = t {
                Some(hat)
            } else {
                None
            }
        })
    }

    fn last_interacted_tab_hat_mut(&mut self) -> Option<&mut Hat> {
        self.last_interacted_tab_mut().and_then(|t| {
            if let Tab::HatElement { hat, .. } = t {
                Some(hat)
            } else {
                None
            }
        })
    }

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

    fn add_new_hat_template(&mut self) -> Result<()> {
        //TODO: remove the absolute path
        let Some(path) = rfd::FileDialog::new()
            .set_directory("/home/palas/Documents/projects/rust-projects/hpp-editor-v2/")
            .pick_folder()
        else {
            bail!("could not pick folder")
        };
        let data_path = path.join("data.json");
        std::fs::create_dir(path.join("images"))
            .context(format!("could not create images directory at {:?}", &path))?;
        std::fs::create_dir(path.join("src"))
            .context(format!("could not create src directory at {:?}", &path))?;

        let name = path
            .with_extension("")
            .file_stem()
            .and_then(|stem| stem.to_str().map(|s| s.to_string()))
            .unwrap_or(self.tabs.new_hat_tab_name(&self.ui_text));

        let hat_data_string =
            serde_json::to_string(&HatData::new(name.clone())).expect("should always succeed");

        File::create(&data_path)
            .context(format!("could not open {:?}", &data_path))
            .and_then(|mut file| {
                write!(&mut file, "{}", hat_data_string)
                    .context(format!("could not read to {:?}", &data_path))
            })?;

        let hat = Hat::new(&path, &name);
        let tab = Tab::new_hat_tab(hat, None);

        info!("hat template created at {:?} created successfully", &path);

        self.tabs.dock_state.push_to_focused_leaf(tab);
        Ok(())
    }

    // fn update_hat_getter(&mut self, ctx: &egui::Context) {
    //     if let Some(HatNameGetterResult::Name(name)) =
    //         self.hat_name_getter.update(ctx, &self.ui_text)
    //         && let Some(last_tab) = self.tabs.last_interacted_tab_mut()
    //         && let TabVariant::HatElement { hat, .. } = &mut last_tab.variant
    //     {
    //         last_tab.title = name.clone();
    //         *hat.name_mut() = Some(name);
    //         if let Err(err) = self.save_hat_as() {
    //             error!("while saving hat: {}", err.to_string());
    //         }
    //     }
    // }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);
        let gl = &frame.gl().cloned().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_menu(gl, ui);
            self.draw_app(gl, ui);
        });
        let text = &self.ui_text.clone();
        let hat = self.tabs.last_interacted_tab_hat_mut();
        self.hat_name_getter.update(hat, ctx, text);
    }
}
