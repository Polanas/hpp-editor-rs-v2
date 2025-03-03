use eframe::egui::{self, SelectableLabel};
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex, TabViewer};

use crate::{
    hats::{Hat, HatElementId},
    ui_text::UiText,
};

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
    },
}

pub struct FrameData<'a> {
    pub ui_text: &'a UiText,
    pub clicked_rename_hat: bool,
}

#[derive(Debug, Clone)]
pub struct FrameResult {
    pub clicked_rename_hat: bool,
    pub added_hat: bool,
}
impl Tab {
    pub fn new_home_tab(title: String) -> Self {
        Tab::Home { title }
    }

    pub fn new_hat_tab(hat: Hat, selected_hat_id: Option<HatElementId>) -> Self {
        Tab::HatElement {
            hat,
            selected_hat_id,
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

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::Home { title } | Tab::Help { title } => title.as_str().into(),
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
            }));
    }

    fn context_menu(
        &mut self,
        ui: &mut egui::Ui,
        _tab: &mut Self::Tab,
        _surface: SurfaceIndex,
        _node: NodeIndex,
    ) {
        if ui.button(self.frame_data.ui_text.get("17")).clicked() {
            self.frame_data.clicked_rename_hat = true;
        }
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Home { title } => {}
            Tab::Help { title } => {}
            Tab::HatElement {
                hat,
                selected_hat_id,
            } => {
                self.draw_hat_element_tab(ui, hat, selected_hat_id);
            }
        }
    }
}

impl MyTabViewer<'_, '_, '_> {
    fn draw_hat_element_tab(
        &self,
        ui: &mut egui::Ui,
        hat: &mut Hat,
        selected_hat_id: &mut Option<HatElementId>,
    ) {
        let text = &self.frame_data.ui_text;
    }
}

pub struct Tabs {
    pub dock_state: DockState<Tab>,
    pub hat_tabs_count: usize,
    pub home_tabs_count: usize,
}

impl Tabs {
    pub fn new(ui_text: &UiText) -> Self {
        let mut dock_state = DockState::new(vec![Tab::new_home_tab(ui_text.get("Home tab"))]);
        dock_state
            .set_focused_node_and_surface((egui_dock::SurfaceIndex(0), egui_dock::NodeIndex(0)));
        dock_state.translations.tab_context_menu.close_button = ui_text.get("15").to_string();
        Self {
            dock_state,
            hat_tabs_count: 1,
            home_tabs_count: 1,
        }
    }

    pub fn last_interacted_tab(&mut self) -> Option<&Tab> {
        self.dock_state.find_active_focused().map(|(_, tab)| &*tab)
    }

    pub fn last_interacted_tab_mut(&mut self) -> Option<&mut Tab> {
        self.dock_state
            .find_active_focused()
            .map(|(_, tab)| &mut *tab)
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

    pub fn last_interacted_tab_hat_mut(&mut self) -> Option<&mut Hat> {
        self.last_interacted_tab_mut().and_then(|t| {
            if let Tab::HatElement { hat, .. } = t {
                Some(hat)
            } else {
                None
            }
        })
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
            added_hat: !added_nodes.is_empty(),
        }
    }

    pub fn new_hat_tab_name(&mut self, ui_text: &UiText) -> String {
        let name = format!("{} {}", ui_text.get("Hat"), self.hat_tabs_count);
        self.hat_tabs_count += 1;
        name
    }
}
