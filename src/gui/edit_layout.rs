use egui_glow::egui_winit::egui::Ui;

use crate::SharedState;

pub fn edit_layout(ui: &mut Ui) {
    ui.label("This function is not finished yet. In the meantime, generate a layout in the web version, then export it.");
}
#[derive(Default)]
pub(super) struct LayoutState {
    dirty: bool,
}

impl LayoutState {
    pub fn on_destroy(&mut self, _shared_state: &mut SharedState) -> bool {
        if self.dirty {
            println!("asking about layout")
        }

        true
    }
}
