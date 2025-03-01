use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, SurfaceIndex, TabViewer};

use crate::{
    hats::{Hat, HatElementId},
    ui_text::UiText,
};

pub struct Tab {
    pub title: String,
    pub variant: TabVariant,
}

pub enum TabVariant {
    Home,
    Help,
    HatElement {
        hat: Hat,
        selected_hat_id: Option<HatElementId>,
    },
}

pub struct FrameData<'a> {
    pub ui_text: &'a UiText,
}

impl Tab {
    pub fn new_home_tab(title: String) -> Self {
        Self {
            title,
            variant: TabVariant::Home,
        }
    }

    pub fn new_hat_tab(title: String, hat: Hat, selected_hat_id: Option<HatElementId>) -> Self {
        Self {
            title,
            variant: TabVariant::HatElement {
                hat,
                selected_hat_id,
            },
        }
    }
}

pub struct MyTabViewer<'a> {
    added_nodes: &'a mut Vec<(SurfaceIndex, NodeIndex)>,
    frame_data: FrameData<'a>,
}

impl MyTabViewer<'_> {}

impl TabViewer for MyTabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title.as_str().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let text = &self.frame_data.ui_text;
        ui.label(&tab.title);
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
        Self {
            dock_state,
            hat_tabs_count: 1,
            home_tabs_count: 1,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, frame_data: FrameData) {
        let text = frame_data.ui_text;
        let mut added_nodes = vec![];
        let mut tab_viewer = MyTabViewer {
            added_nodes: &mut added_nodes,
            frame_data,
        };
        DockArea::new(&mut self.dock_state)
            .show_add_buttons(true)
            .show_leaf_collapse_buttons(false)
            .show_inside(ui, &mut tab_viewer);
        // if tab_viewer.frame_data.new_help_tab {
        //     self.open_help_tab(&tab_viewer.frame_data.ui_text);
        // }
        for (surface, node) in added_nodes {
            let name = self.new_hat_tab_name(text);
            let tab = Tab::new_hat_tab(name, Hat::default(), None);
            self.dock_state
                .set_focused_node_and_surface((surface, node));
            self.dock_state.push_to_focused_leaf(tab);
        }
    }

    pub fn new_hat_tab_name(&mut self, ui_text: &UiText) -> String {
        let name = format!("{} {}", ui_text.get("Hat"), self.hat_tabs_count);
        self.hat_tabs_count += 1;
        name
    }
}
