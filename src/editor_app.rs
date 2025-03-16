use std::{fs::File, io::Write, path::Path, rc::Rc, time::Duration};

use anyhow::{Context, Result, bail};
use eframe::{
    egui::{self},
    glow::{self},
};
use log::{error, info};

use crate::{
    catppuccin_egui,
    console::Console,
    hats::{Hat, LoadHatElement, WearableHat},
    hats_data::{HatData, HatType},
    name_getter::{NameGetter, NameGetterResult},
    tabs::{FrameData, Tab, Tabs},
    ui_text::{Language, Translatable, UiText},
};

use borrow::partial as p;
use borrow::traits::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NameGetterVariant {
    Hat,
    Script,
}

#[derive(borrow::Partial)]
#[module(crate::editor_app)]
pub struct EditorApp {
    ui_text: UiText,
    tabs: Tabs,
    hat_name_getter: NameGetter<NameGetterVariant>,
    toasts: egui_notify::Toasts,
    toasts_storage: Vec<(ToastType, String)>,
    console: Option<Console>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Info,
    Warn,
    Error,
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ui_text = UiText::new(Language::English, include_str!("../translations.json"));

        let tabs = Tabs::new(&ui_text);
        catppuccin_egui::set_theme(&cc.egui_ctx, catppuccin_egui::MOCHA);
        Self::set_font(&cc.egui_ctx);
        Self {
            ui_text,
            tabs,
            hat_name_getter: NameGetter::default(),
            toasts: egui_notify::Toasts::default(),
            toasts_storage: Default::default(),
            console: Some(Console::new()),
        }
    }

    fn set_font(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "Caskaydia".to_owned(),
            egui::FontData::from_static(include_bytes!("../font.ttf")).into(),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Caskaydia".to_owned());
        ctx.set_fonts(fonts);
    }

    fn set_min_width(ui: &mut egui::Ui, text: &str) {
        let galley = ui.painter().layout_no_wrap(
            text.to_string(),
            egui::FontId::default(),
            egui::Color32::default(),
        );
        ui.set_min_width(galley.size().x);
    }
}

impl p!(<mut toasts_storage> EditorApp) {
    fn add_toast(&mut self, toast_type: ToastType, message: String) {
        self.toasts_storage.push((toast_type, message));
    }
}
impl p!(<mut toasts_storage, mut toasts> EditorApp) {
    fn display_toasts(&mut self, ctx: &egui::Context) {
        for (toast_type, message) in self.toasts_storage.drain(..) {
            match toast_type {
                ToastType::Success => {
                    self.toasts
                        .success(message)
                        .closable(false)
                        .duration(Some(Duration::from_secs(3)));
                }
                ToastType::Info => {
                    self.toasts
                        .info(message)
                        .closable(false)
                        .duration(Some(Duration::from_secs(3)));
                }
                ToastType::Warn => {
                    self.toasts
                        .warning(message)
                        .closable(false)
                        .duration(Some(Duration::from_secs(3)));
                }
                ToastType::Error => {
                    self.toasts
                        .error(message)
                        .closable(false)
                        .duration(Some(Duration::from_secs(3)));
                }
            }
        }
        self.toasts.show(ctx);
    }
}

impl p!(<mut tabs> EditorApp) {
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

    fn open_hat_with_dialog(&mut self, gl: &glow::Context) -> Result<()> {
        let Some(path) = rfd::FileDialog::new()
            .set_directory("/home/palas/Documents/projects/rust-projects/hpp-editor-v2/")
            .pick_folder()
        else {
            return Ok(());
        };
        self.open_hat(gl, &path)
    }

    fn save_hat(&mut self) -> Result<()> {
        let last_tab = self
            .tabs
            .last_interacted_tab()
            .context("could not find last interacted tab")?;
        let Tab::HatElement { hat, .. } = last_tab else {
            bail!("expected hat tab");
        };
        hat.save(hat.path())
    }

    fn export_hat_to_file_as(&mut self) -> Result<()> {
        let last_tab = self
            .tabs
            .last_interacted_tab()
            .context("could not find last interacted tab")?;
        let Tab::HatElement { hat, .. } = last_tab else {
            bail!("expected hat tab");
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter(".hatspp", &["hatspp"])
            .save_file()
        else {
            return Ok(());
        };
        hat.export_to_file(path)
    }

    fn export_hat_to_file(&mut self) -> Result<()> {
        let last_tab = self
            .tabs
            .last_interacted_tab()
            .context("could not find last interacted tab")?;
        let Tab::HatElement { hat, .. } = last_tab else {
            bail!("expected hat tab");
        };
        hat.export_to_file(hat.path().join("hat.hatspp"))
    }

    fn can_export(&mut self) -> bool {
        matches!(
            self.tabs.last_interacted_tab(),
            Some(Tab::HatElement { .. })
        )
    }

    fn can_save(&mut self) -> bool {
        matches!(
            self.tabs.last_interacted_tab(),
            Some(Tab::HatElement { .. })
        )
    }

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
    fn add_script_template_to_hat(&mut self) {
        if let Some(Tab::HatElement {
            hat,
            selected_hat_id: Some(id),
            ..
        }) = self.tabs.last_interacted_tab_mut()
            && let Some(element) = hat.element(*id)
        {}
    }

    fn rename_hat(&mut self, name: String) {
        if let Some(hat) = self.tabs.last_interacted_tab_hat_mut() {
            *hat.name_mut() = name;
        }
    }
}

impl p!(<mut tabs, ui_text, mut hat_name_getter> EditorApp) {
    fn update_hat_getter(&mut self, ctx: &egui::Context) {
        let text = self.ui_text;
        let result = self.hat_name_getter.update(ctx, text);
        if let Some(NameGetterResult::Confirmed(name, variant)) = result {
            match variant {
                NameGetterVariant::Hat => self.partial_borrow().rename_hat(name),
                NameGetterVariant::Script => self.partial_borrow().add_script_template_to_hat(),
            }
        }
    }
}

impl p!(<mut tabs, ui_text, mut console, mut hat_name_getter> EditorApp) {
    fn draw_app(&mut self, gl: &glow::Context, ui: &mut egui::Ui) {
        let frame_data = FrameData {
            ui_text: self.ui_text,
            clicked_rename_hat: false,
            clicked_open_hat: false,
            clicked_new_hat: false,
            clicked_help_tab: false,
            console: None,
            gl,
        };
        let frame_result = self.tabs.ui(ui, frame_data);

        if frame_result.console.is_some() {
            *self.console = frame_result.console;
        }
        if frame_result.clicked_rename_hat {
            self.hat_name_getter
                .open(self.ui_text.get("14").to_string(), NameGetterVariant::Hat);
        }
        if frame_result.clicked_open_hat {
            if let Err(err) = self.partial_borrow().open_hat_with_dialog(gl) {
                error!("while opening hat: {}", err.to_string());
            }
        }
        if frame_result.cliked_new_hat {
            if let Err(err) = self.partial_borrow().add_new_hat_template() {
                error!("while adding hat: {}", err.to_string());
            }
        }
    }
}

impl p!(<mut tabs, ui_text, mut toasts, mut toasts_storage, mut console> EditorApp) {
    fn draw_menu(&mut self, gl: &glow::Context, ui: &mut egui::Ui) {
        let (text, self2) = self.extract_ui_text();
        egui::menu::bar(ui, |ui| {
            ui.menu_button(text.get("Hat"), |ui| {
                let hat_name = self2
                    .tabs
                    .last_interacted_hat_name()
                    .unwrap_or("")
                    .to_string();
                if ui.button(text.get("New")).clicked() {
                    if let Err(err) = self2.partial_borrow().add_new_hat_template() {
                        error!("while adding new hat: {}", err.to_string());
                    }
                    ui.close_menu();
                }
                if ui.button(text.get("Open")).clicked() {
                    if let Err(err) = self2.partial_borrow().open_hat_with_dialog(gl) {
                        error!("while opening hat: {}", err.to_string());
                    }
                    ui.close_menu();
                }
                if ui
                    .add_enabled(
                        self2.partial_borrow().can_save(),
                        egui::Button::new(text.get("Save")),
                    )
                    .clicked()
                {
                    if let Err(err) = self2.partial_borrow().save_hat() {
                        error!("while saving hat: {}", err.to_string());
                        self2.partial_borrow().add_toast(
                            ToastType::Error,
                            format!(r#"could not save hat "{}""#, &hat_name),
                        );
                    } else {
                        self2.partial_borrow().add_toast(
                            ToastType::Success,
                            format!(r#"hat "{}" saved successfully"#, &hat_name),
                        );
                    }
                    ui.close_menu();
                }
                if ui
                    .add_enabled(
                        self2.partial_borrow().can_export(),
                        egui::Button::new(text.get("Export")),
                    )
                    .clicked()
                {
                    if let Err(err) = self2.partial_borrow().export_hat_to_file() {
                        error!("while exporting hat to file: {}", err.to_string());
                        self2.partial_borrow().add_toast(
                            ToastType::Error,
                            format!(r#"could not export hat "{}""#, &hat_name),
                        );
                    } else {
                        self2.partial_borrow().add_toast(
                            ToastType::Success,
                            format!(r#"hat "{}" was exported successfully"#, &hat_name),
                        );
                    }

                    ui.close_menu();
                }
                if ui
                    .add_enabled(
                        self2.partial_borrow().can_export(),
                        egui::Button::new(text.get("Export as")),
                    )
                    .clicked()
                {
                    if let Err(err) = self2.partial_borrow().export_hat_to_file_as() {
                        error!("while exporting hat to file: {}", err.to_string())
                    }
                    ui.close_menu();
                }
                ui.collapsing(text.get("Recent"), |ui| {});
            });

            ui.menu_button(text.get("Elements"), |ui| {
                if let Err(err) = self2.partial_borrow().draw_elements_menu(gl, ui) {
                    error!("{}", err.to_string());
                }
            });
            ui.menu_button(text.get("Help"), |ui| {
                if ui.button(text.get("Open help tab")).clicked() {
                    ui.close_menu();
                }
            });
            ui.menu_button(text.get("Settings"), |ui| {
                self2.partial_borrow().draw_settings_menu(ui);
            });
            ui.menu_button(text.get("Other"), |ui| {
                if ui.button(text.get("Open home tab")).clicked() {
                    ui.close_menu();
                }
                if ui.button(text.get("39")).clicked()
                    && let Some(console) = self2.console.take()
                {
                    self2.tabs.open_console_tab(console, text);
                    ui.close_menu();
                }
            });
        });
    }
}

impl p!(<mut tabs, ui_text> EditorApp) {
    fn draw_elements_menu(&mut self, gl: &glow::Context, ui: &mut egui::Ui) -> Result<()> {
        let text = &self.ui_text;
        let Some(Tab::HatElement { hat, .. }) = self.tabs.last_interacted_tab_mut() else {
            return Ok(());
        };
        ui.collapsing(text.get("Add"), |ui| -> Result<()> {
            let wereable_key = text.get(HatType::Wearable.translate_key());
            EditorApp::set_min_width(ui, wereable_key);
            if hat.wereable().is_none() && ui.button(wereable_key).clicked() {
                let Some(path) = rfd::FileDialog::new()
                    .add_filter("Image", &["png", "aseprite"])
                    .pick_file()
                else {
                    return Ok(());
                };
                let wereable = WearableHat::load_from_path(&path, gl)
                    .context("could not load wereable hat")?;
                hat.add_element(wereable);
                ui.close_menu();
            }
            Ok(())
        })
        .body_returned
        .unwrap_or(Ok(()))?;

        ui.collapsing(text.get("Select"), |ui| {
            hat.elements().for_each(|e| {
                let translate_key = text.get(e.base().hat_type.translate_key());
                EditorApp::set_min_width(ui, translate_key);
            });
            for element in hat.elements() {
                let translate_key = text.get(element.base().hat_type.translate_key());
                let response = ui.button(translate_key);
                // if response.clicked() {
                //     *selected_hat_id = Some(element.id());
                //     ui.close_menu();
                //     break;
                // }
                if response.clicked_by(egui::PointerButton::Secondary) {
                    let pos = ui.input(|i| i.pointer.latest_pos().unwrap());
                    egui::Window::new("hi")
                        .current_pos(pos)
                        .collapsible(false)
                        .title_bar(false)
                        .show(ui.ctx(), |ui| if ui.button("Add script").clicked() {});
                }
                // response.context_menu(|ui| {
                //     if ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary)) {
                //         dbg!("hi3");
                //     }
                //     let script_attached = element.base().local_script_path.is_some();
                //     let response =
                //         ui.add_enabled(!script_attached, egui::Button::new(text.get("33")));
                //     // if response.clicked()
                //     //     || response.clicked_elsewhere() && response.contains_pointer()
                //     // {
                //     //     dbg!("hi");
                //     // }
                //     // if
                //     //     .clicked()
                //     // {
                //     //     self.hat_name_getter
                //     //         .open(text.get("36").to_string(), NameGetterVariant::Script);
                //     //     should_exit = true;
                //     //     ui.close_menu();
                //     // }
                //     if ui
                //         .add_enabled(!script_attached, egui::Button::new(text.get("34")))
                //         .clicked()
                //     {
                //         should_exit = true;
                //         ui.close_menu();
                //     }
                //     if ui
                //         .add_enabled(script_attached, egui::Button::new(text.get("35")))
                //         .clicked()
                //     {
                //         ui.close_menu();
                //     }
                // });
            }
            //     .context_menu(|ui| {
            //         ui.label("haha");
            //     })
            //     .clicked()
            // {
            // };
            //     .context_menu(|ui| {
            //         ui.label("im here");
            //     })
            // {
            //     if response.response.clicked() {
            //         *selected_hat_id = Some(element.id());
            //         ui.close_menu();
            //         break;
            //     }
            // }
        });
        Ok(())
    }
    fn add_new_hat_template(&mut self) -> Result<()> {
        //TODO: remove the absolute path
        let Some(path) = rfd::FileDialog::new()
            .set_directory("/home/palas/Documents/projects/rust-projects/hpp-editor-v2/")
            .pick_folder()
        else {
            return Ok(());
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
}

impl p!(<> EditorApp) {
    // fn save_hat_as(&mut self) -> Result<()> {
    //     let last_tab = self
    //         .last_interacted_tab_mut()
    //         .context("could not find last interacted tab")?;
    //     let Tab::HatElement { hat, .. } = last_tab else {
    //         bail!("expected hat tab");
    //     };
    //     hat.save_as()
    // }

    fn draw_settings_menu(&mut self, ui: &mut egui::Ui) {}

    // fn add_script_template()

    fn draw_elements_add_menu(&mut self, gl: &glow::Context, ui: &mut egui::Ui) {}

    fn draw_elements_select_menu(&mut self, gl: &glow::Context, ui: &mut egui::Ui) {}
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let gl = &frame.gl().cloned().unwrap();
        ctx.set_pixels_per_point(1.5);

        egui::CentralPanel::default().show(ctx, |ui| {
            self.as_refs_mut().partial_borrow().draw_menu(gl, ui);
            self.as_refs_mut().partial_borrow().draw_app(gl, ui);
        });
        self.as_refs_mut().partial_borrow().update_hat_getter(ctx);
        self.as_refs_mut().partial_borrow().display_toasts(ctx);
    }
}
