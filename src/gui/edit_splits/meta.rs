use egui_glow::egui_winit::egui::{Grid, Ui};
use livesplit_core::run::editor;

use super::SplitsState;

pub(super) fn split_metadata(
    ui: &mut Ui,
    state: &mut SplitsState,
    editor_state: &mut editor::State,
) {
    Grid::new("metadata")
        .min_col_width(ui.available_width() / 2.)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label("Game Name");

                if ui.text_edit_singleline(&mut editor_state.game).changed() {
                    state.editor.set_game_name(&editor_state.game);
                    state.dirty = true;
                }
            });

            ui.vertical(|ui| {
                ui.label("Category Name");

                if ui
                    .text_edit_singleline(&mut editor_state.category)
                    .changed()
                {
                    state.editor.set_category_name(&editor_state.category);
                    state.dirty = true;
                }
            });

            ui.end_row();

            ui.vertical(|ui| {
                ui.label("Offset");

                let text = ui.text_edit_singleline(&mut editor_state.offset);

                if text.changed() {
                    state.editor.parse_and_set_offset(&editor_state.offset).ok();
                    state.dirty = true;
                }
            });

            ui.vertical(|ui| {
                ui.label("Attempts");

                // editor state doesnt store this one as a string so we need our own.
                // TODO is there a better way to do this
                let text = ui.text_edit_singleline(&mut state.attempts_string);
                if text.changed() {
                    state
                        .editor
                        .parse_and_set_attempt_count(&state.attempts_string)
                        .ok();

                    state.dirty = true;
                }

                if text.lost_focus() {
                    state.sync_attempts_string();
                }
            })
        });
}
