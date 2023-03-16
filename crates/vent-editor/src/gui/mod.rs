use egui::epaint::ahash::{HashSet, HashSetExt};
use egui::{CentralPanel, Color32, Frame, RichText, TextBuffer, TopBottomPanel, Ui, WidgetText};
use egui_dock::{DockArea, Node, NodeIndex, TabViewer, Tree};

pub(crate) struct EditorViewer {
    open_tabs: HashSet<String>,
}

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

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        self.open_tabs.remove(tab);
        true
    }
}

impl EditorViewer {
    // TODO:
    fn view_console(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("[ERROR]: UwU Just a Joke, Its Rust :D").color(Color32::RED));
        });
    }

    fn view_tab_file(&mut self, ui: &mut Ui, _tree: &mut Tree<String>) {
        for tab in &["New Project", "Open Project"] {
            if ui
                .selectable_label(self.open_tabs.contains(*tab), *tab)
                .clicked()
            {
                ui.close_menu();
                todo!();
            }
        }
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
        let [_, _] = tree.split_below(a, 0.6, vec!["Console".to_owned()]);

        let mut open_tabs = HashSet::new();

        for node in tree.iter() {
            if let Node::Leaf { tabs, .. } = node {
                for tab in tabs {
                    open_tabs.insert(tab.clone());
                }
            }
        }

        let viewer = EditorViewer { open_tabs };

        Self { tree, viewer }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("vent::MenuBar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    self.viewer.view_tab_file(ui, &mut self.tree);
                })
            })
        });
        CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                DockArea::new(&mut self.tree).show_inside(ui, &mut self.viewer);
            });
    }
}
