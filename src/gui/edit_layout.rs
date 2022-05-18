use egui_glow::egui_winit::egui::Ui;

pub fn edit_layout(ui: &mut Ui) {
    ui.label("This function is not finished yet. In the meantime, generate a layout in the web version, then export it.");
}
pub(super) struct LayoutState {}

impl Default for LayoutState {
    fn default() -> Self {
        Self {}
    }
}
