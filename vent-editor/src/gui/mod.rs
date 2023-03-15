use egui::epaint::ahash::{HashSet, HashSetExt};
use egui::{
    CentralPanel, Color32, Frame, InnerResponse, RichText, TextBuffer, TopBottomPanel, Ui,
    WidgetText,
};
use egui_dock::{DockArea, Node, NodeIndex, Style, TabViewer, Tree};

pub(crate) struct EditorViewer {}

impl TabViewer for EditorViewer {
    type Tab = String;

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "Console" => self.view_console(ui),
            _ => {}
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.as_str().into()
    }
}

impl EditorViewer {
    // TODO:
    fn view_console(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("[ERROR]: UwU Just a Joke, Its Rust :D").color(Color32::RED));
        });
    }
}

pub(crate) struct EditorGUI {
    tree: Tree<String>,
    viewer: EditorViewer,
}

impl EditorGUI {
    pub fn new() -> Self {
        let mut tree = Tree::new(vec!["Vent-Engine Placeholder".to_owned()]);
        let [a, _] = tree.split_left(NodeIndex::root(), 0.3, vec!["Files".to_owned()]);
        let [_, _] = tree.split_below(a, 0.5, vec!["Console".to_owned()]);

        let viewer = EditorViewer {};

        Self { tree, viewer }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("vent::MenuBar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {});
            })
        });
        CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                DockArea::new(&mut self.tree).show_inside(ui, &mut self.viewer);
            });
    }
}
