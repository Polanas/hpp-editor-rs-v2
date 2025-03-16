use std::{
    sync::mpsc::{Receiver, channel},
    thread::sleep,
    u32,
};

use eframe::{
    egui::{self, SelectableLabel},
    epaint::text::layout,
    glow,
};
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex, TabViewer};

use crate::{
    console::Console,
    hats::{Hat, HatElement, HatElementId, HatId, LoadHatElement, WearableHat, WingsHat, hat_id},
    hats_data::HatType,
    ui_text::{self, Translatable, UiText},
};

#[derive(Debug, Default, Clone)]
pub struct HatTabState {
    element_to_remove: Option<(String, HatElementId)>,
}

pub enum Tab {
    Home {
        title: String,
    },
    Help {
        title: String,
    },
    HatElement {
        hat: Hat,
        selected_hat_id: Option<HatElementId>,
        state: HatTabState,
    },
    Console {
        console: Option<Console>,
        title: String,
    },
}

pub struct FrameData<'a> {
    pub ui_text: &'a UiText,
    pub clicked_rename_hat: bool,
    pub clicked_open_hat: bool,
    pub clicked_new_hat: bool,
    pub clicked_help_tab: bool,
    pub console: Option<Console>,
    pub gl: &'a glow::Context,
}

#[derive(Debug)]
pub struct FrameResult {
    pub clicked_rename_hat: bool,
    pub cliked_new_hat: bool,
    pub clicked_open_hat: bool,
    pub clicked_help_tab: bool,
    pub console: Option<Console>,
}
impl Tab {
    pub fn new_home_tab(title: String) -> Self {
        Tab::Home { title }
    }

    pub fn new_hat_tab(hat: Hat, selected_hat_id: Option<HatElementId>) -> Self {
        Tab::HatElement {
            hat,
            selected_hat_id,
            state: Default::default(),
        }
    }

    pub fn new_console_tab(title: String, console: Console) -> Self {
        Self::Console {
            console: Some(console),
            title,
        }
    }
}

pub struct MyTabViewer<'a, 'b, 'c> {
    #[allow(dead_code)]
    added_nodes: &'b mut Vec<(SurfaceIndex, NodeIndex)>,
    frame_data: &'a mut FrameData<'c>,
}

impl MyTabViewer<'_, '_, '_> {}

impl TabViewer for MyTabViewer<'_, '_, '_> {
    type Tab = Tab;

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        if let Tab::Console { console, .. } = tab {
            self.frame_data.console = console.take();
        }
        true
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::Home { title } | Tab::Help { title } | Tab::Console { title, .. } => {
                title.as_str().into()
            }
            Tab::HatElement { hat, .. } => hat.name().into(),
        }
    }

    fn on_tab_button(&mut self, tab: &mut Self::Tab, response: &egui::Response) {
        response
            .clone()
            .on_hover_text(self.frame_data.ui_text.get(match tab {
                Tab::Home { .. } => "19",
                Tab::Help { .. } => "20",
                Tab::HatElement { .. } => "21",
                Tab::Console { .. } => "40",
            }));
    }

    fn context_menu(
        &mut self,
        ui: &mut egui::Ui,
        tab: &mut Self::Tab,
        _surface: SurfaceIndex,
        _node: NodeIndex,
    ) {
        if matches!(tab, Tab::HatElement { .. })
            && ui.button(self.frame_data.ui_text.get("17")).clicked()
        {
            self.frame_data.clicked_rename_hat = true;
        }
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Home { .. } => self.draw_home_ui(ui),
            Tab::Help { .. } => {}
            Tab::HatElement {
                hat,
                selected_hat_id,
                state,
            } => self.draw_hat_element_tab(ui, hat, selected_hat_id, state),
            Tab::Console {
                console: Some(console),
                ..
            } => {
                console.update(ui);
            }
            _ => {}
        }
    }
}

impl MyTabViewer<'_, '_, '_> {
    fn draw_home_ui(&mut self, ui: &mut egui::Ui) {
        // egui::SidePanel::left(egui::Id::new("left"))
        //     .resizable(true)
        //     .max_width(300.0)
        //     .show_inside(ui, |ui| {
        //         ui.style_mut().text_styles.insert(
        //             egui::TextStyle::Button,
        //             egui::FontId::new(18.0, eframe::epaint::FontFamily::Proportional),
        //         );
        //         ui.add_sized([20., 20.], egui::Button::new("+"));
        //         ui.separator();
        //         let id = ui.make_persistent_id("tree view");
        //         egui_ltreeview::TreeView::new(id).show(ui, |builder| {
        //             builder.node(
        //                 egui_ltreeview::NodeBuilder::leaf(4)
        //                     .label("test node")
        //                     .context_menu(|m| if m.button("hi").clicked() {}),
        //             );
        //             builder.dir(0, "Root");
        //             builder.leaf(1, "Ava");
        //             builder.leaf(2, "Benjamin");
        //             builder.leaf(3, "Charlotte");
        //             builder.close_dir();
        //         });
        //     });
        // egui::CentralPanel::default().show_inside(ui, |ui| {
        let ui_text = &self.frame_data.ui_text;
        let text = |text: &str| ui_text.get(text);
        let label = |ui: &mut egui::Ui, text: &str| ui.label(ui_text.get(text));
        ui.heading(text("22"));
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            label(ui, "23");
            ui.hyperlink_to(text("24"), "https://youtube.com")
                .on_hover_ui(|ui| {
                    label(ui, "25");
                });
            label(ui, "26");
        });

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            label(ui, "27");
            self.frame_data.clicked_open_hat = ui.link(text("Open")).clicked();
            ui.label(",");
        });
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            label(ui, "28");
            self.frame_data.clicked_new_hat = ui.link(text("New")).clicked();
            ui.label(".");
        });

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            label(ui, "29");
            self.frame_data.clicked_help_tab = ui.link(text("30")).clicked();
        });
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            label(ui, "31");
            ui.label(egui::RichText::new("♥").color(egui::Color32::from_rgb(242, 56, 56)));
        });
    }
    fn draw_hat_element_tab(
        &mut self,
        ui: &mut egui::Ui,
        hat: &mut Hat,
        selected_hat_id: &mut Option<HatElementId>,
        state: &mut HatTabState,
    ) {
        let left_panel_response = self.draw_hat_left_panel(ui, hat, state);
        if let Some(id) = left_panel_response.selected_hat_id {
            *selected_hat_id = Some(id);
        }
        if let Some(remove_id) = left_panel_response.removed_hat_id {
            hat.remove_element(remove_id);
            if let Some(selected_id) = selected_hat_id
                && *selected_id == remove_id
            {
                *selected_hat_id = None;
            }
        }
        if let Some(HatType::Wearable) = left_panel_response.added_hat_type
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("Image", &["png", "aseprite"])
                .pick_file()
        {
            let wearable = WearableHat::load_from_path(&path, self.frame_data.gl).unwrap();
            hat.add_element(wearable);
        } else if let Some(HatType::Wings) = left_panel_response.added_hat_type
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("Image", &["png", "aseprite"])
                .pick_file()
        {
            let wings = WingsHat::load_from_path(&path, self.frame_data.gl).unwrap();
            hat.add_element(wings);
        }

        self.draw_hat_ui(ui, hat, selected_hat_id);
    }

    fn set_width(ui: &mut egui::Ui, text: &str) {
        let galley = ui.painter().layout_no_wrap(
            text.to_string(),
            egui::FontId::default(),
            egui::Color32::default(),
        );
        ui.set_width(galley.size().x);
    }


    fn draw_hat_left_panel(
        &mut self,
        ui: &mut egui::Ui,
        hat: &mut Hat,
        state: &mut HatTabState,
    ) -> HatLeftPanelResponse {
        let text = self.frame_data.ui_text;
        egui::SidePanel::left(egui::Id::new(format!("left_panel{}", hat.id().0)))
            .max_width(300.0)
            .show_inside(ui, |ui| {
                let mut response = HatLeftPanelResponse::default();
                let add_modal = egui_modal::Modal::new(ui.ctx(), "add_model");
                add_modal.show(|ui| {
                    add_modal.frame(ui, |ui| {
                        ui.label(text.get("42"));
                        if !hat.has_element(HatType::Wearable)
                            && ui.button(text.get("Wearable")).clicked()
                        {
                            response.added_hat_type = Some(HatType::Wearable);
                            add_modal.close();
                        }
                        if !hat.has_element(HatType::Wings)
                            && ui.button(text.get("Wings")).clicked()
                        {
                            response.added_hat_type = Some(HatType::Wings);
                            add_modal.close();
                        }
                    });
                    add_modal.buttons(ui, |ui| {
                        if ui.button(text.get("43")).clicked() {
                            add_modal.close();
                        }
                    });
                });
                let remove_modal = egui_modal::Modal::new(ui.ctx(), "remove_model");
                remove_modal.show(|ui| {
                    let elem_to_remove = state.element_to_remove.as_ref().unwrap().clone();
                    remove_modal.frame(ui, |ui| {
                        let remove_label = format!(r#"{} "{}"?"#, text.get("46"), elem_to_remove.0);
                        ui.label(remove_label);
                    });
                    remove_modal.buttons(ui, |ui| {
                        if remove_modal.caution_button(ui, text.get("47")).clicked() {
                            remove_modal.close();
                            state.element_to_remove = None;
                            response.removed_hat_id = Some(elem_to_remove.1);
                        } else if remove_modal.button(ui, text.get("48")).clicked() {
                            remove_modal.close();
                            state.element_to_remove = None;
                        }
                    });
                });

                let id = ui.make_persistent_id(egui::Id::new(format!("tree_view{}", hat.id().0)));
                let tree_response = egui_ltreeview::TreeView::new(id).show(ui, |builder| {
                    builder.node(
                        egui_ltreeview::NodeBuilder::dir(0)
                            .label(text.get("Elements"))
                            .default_open(false)
                            .context_menu(|ui| {
                                Self::set_width(ui, text.get("44"));
                                if ui.button(text.get("44")).clicked() {
                                    add_modal.open();
                                }
                            }),
                    );
                    for elem in hat.elements() {
                        builder.node(
                            egui_ltreeview::NodeBuilder::leaf(elem.id().0)
                                .label(text.get(elem.base().hat_type.translate_key()).to_string())
                                .context_menu(|ui| {
                                Self::set_width(ui, text.get("45"));
                                    if ui.button(text.get("45")).clicked() {
                                        remove_modal.open();
                                        state.element_to_remove = Some((
                                            text.get(elem.base().hat_type.translate_key())
                                                .to_string(),
                                            elem.id(),
                                        ));
                                    }
                                }),
                        );
                    }
                    builder.close_dir();
                });

                for action in tree_response.1 {
                    if let egui_ltreeview::Action::SetSelected(ids) = action
                        && !ids.is_empty()
                        && !hat.is_empty()
                    {
                        let id = HatElementId(ids[0]);
                        if hat.has_element_with_id(id) {
                            response.selected_hat_id = Some(id);
                            break;
                        }
                    }
                }

                response
            })
            .inner
    }

    fn draw_hat_ui(
        &self,
        ui: &mut egui::Ui,
        hat: &mut Hat,
        selected_hat_id: &mut Option<HatElementId>,
    ) {
        let text = self.frame_data.ui_text;
        let hat_element_id = match selected_hat_id {
            Some(id) => *id,
            None => {
                if hat.is_empty() {
                    ui.label(text.get("37"));
                    return;
                }

                let selected_hat = hat.elements().next().map(|h| h.id()).unwrap();
                *selected_hat_id = Some(selected_hat);
                selected_hat
            }
        };
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let selected_hat = hat.element_mut(hat_element_id).unwrap();
            ui.label(selected_hat.base().hat_type.translate_key().to_string());
        });
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct HatLeftPanelResponse {
    selected_hat_id: Option<HatElementId>,
    added_hat_type: Option<HatType>,
    removed_hat_id: Option<HatElementId>,
}

pub struct Tabs {
    pub dock_state: DockState<Tab>,
    pub hat_tabs_count: usize,
    pub home_tabs_count: usize,
}

impl Tabs {
    pub fn new(ui_text: &UiText) -> Self {
        let mut dock_state =
            DockState::new(vec![Tab::new_home_tab(ui_text.get("Home tab").to_string())]);
        dock_state
            .set_focused_node_and_surface((egui_dock::SurfaceIndex(0), egui_dock::NodeIndex(0)));
        dock_state.translations.tab_context_menu.close_button = ui_text.get("15").to_string();
        Self {
            dock_state,
            hat_tabs_count: 1,
            home_tabs_count: 1,
        }
    }

    pub fn open_home_tab(&mut self, ui_text: &UiText) {
        self.dock_state
            .push_to_focused_leaf(Tab::new_home_tab(ui_text.get("Home tab").to_string()));
    }

    pub fn open_console_tab(&mut self, console: Console, ui_text: &UiText) {
        self.dock_state
            .push_to_focused_leaf(Tab::new_console_tab(ui_text.get("38").to_string(), console));
    }

    pub fn last_interacted_tab(&mut self) -> Option<&Tab> {
        self.dock_state.find_active_focused().map(|(_, tab)| &*tab)
    }

    pub fn last_interacted_tab_mut(&mut self) -> Option<&mut Tab> {
        self.dock_state
            .find_active_focused()
            .map(|(_, tab)| &mut *tab)
    }

    pub fn last_interacted_tab_hat(&mut self) -> Option<&Hat> {
        self.last_interacted_tab().and_then(|t| {
            if let Tab::HatElement { hat, .. } = t {
                Some(hat)
            } else {
                None
            }
        })
    }

    pub fn last_interacted_tab_hat_mut(&mut self) -> Option<&mut Hat> {
        self.last_interacted_tab_mut().and_then(|t| {
            if let Tab::HatElement { hat, .. } = t {
                Some(hat)
            } else {
                None
            }
        })
    }

    pub fn last_interacted_hat_name(&mut self) -> Option<&str> {
        self.last_interacted_tab_hat().map(|h| h.name())
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, mut frame_data: FrameData) -> FrameResult {
        let mut added_nodes = vec![];
        let mut tab_viewer = MyTabViewer {
            added_nodes: &mut added_nodes,
            frame_data: &mut frame_data,
        };
        let mut style = Style::from_egui(ui.style().as_ref());
        style.buttons.add_tab_align = egui_dock::TabAddAlign::Left;
        DockArea::new(&mut self.dock_state)
            .show_leaf_collapse_buttons(false)
            .show_leaf_close_all_buttons(false)
            .show_add_buttons(true)
            .style(style)
            .show_inside(ui, &mut tab_viewer);
        // if tab_viewer.frame_data.new_help_tab {
        //     self.open_help_tab(&tab_viewer.frame_data.ui_text);
        // }
        // for (surface, node) in added_nodes {
        //     let name = self.new_hat_tab_name(text);
        //     let hat = Hat::new(&name);
        //     let tab = Tab::new_hat_tab(hat, None);
        //     self.dock_state
        //         .set_focused_node_and_surface((surface, node));
        //     self.dock_state.push_to_focused_leaf(tab);
        // }
        FrameResult {
            clicked_rename_hat: frame_data.clicked_rename_hat,
            cliked_new_hat: frame_data.clicked_new_hat,
            clicked_open_hat: frame_data.clicked_open_hat,
            clicked_help_tab: frame_data.clicked_help_tab,
            console: frame_data.console,
        }
    }

    pub fn new_hat_tab_name(&mut self, ui_text: &UiText) -> String {
        let name = format!("{} {}", ui_text.get("Hat"), self.hat_tabs_count);
        self.hat_tabs_count += 1;
        name
    }
}
