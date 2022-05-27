use std::mem::take;

use egui_glow::egui_winit::egui::{Color32, Grid, Rect, ScrollArea, Ui, Vec2};

use super::SplitsState;

pub(super) fn segments(ui: &mut Ui, state: &mut SplitsState) {
    // this active regeneration is necessary because we may have deleted splits this frame
    let mut editor_state = state.editor.state();

    ScrollArea::both()
        .max_height(0.67 * ui.available_height())
        .show(ui, |ui| {
            Grid::new("splits").show(ui, |ui| {
                // empyty label for padding
                ui.heading("Segment Name");
                ui.heading("Split time");
                ui.heading("Segment Time");
                ui.heading("Best Segment");

                state.grid_width = ui.min_rect().width();

                ui.end_row();

                for (index, segment) in editor_state.segments.iter_mut().enumerate() {
                    if segment.selected.is_selected_or_active() {
                        let rect = Rect::from_min_size(
                            ui.max_rect().left_bottom(),
                            Vec2::new(state.grid_width, 24.0),
                        );

                        ui.painter().rect_filled(rect, 0.0, Color32::DARK_BLUE)
                    }

                    // empyty label for padding

                    let local_segment = &mut state.segments[index];

                    if !local_segment.editing_split {
                        local_segment.split = take(&mut segment.split_time);
                    }

                    if !local_segment.editing_segment {
                        local_segment.segment = take(&mut segment.segment_time);
                    }

                    if !local_segment.editing_best_segment {
                        local_segment.best_segment = take(&mut segment.best_segment_time);
                    }

                    let name_editor = ui.text_edit_singleline(&mut segment.name);

                    if name_editor.clicked() {
                        state.editor.select_only(index);
                    }
                    if name_editor.changed() {
                        state.editor.select_only(index);
                        state.editor.active_segment().set_name(&segment.name);
                        state.dirty = true;
                    }

                    let split_editor = ui.text_edit_singleline(&mut local_segment.split);

                    if split_editor.clicked() {
                        state.editor.select_only(index);
                    }

                    if split_editor.changed() {
                        local_segment.editing_split = true;
                    }
                    // dirty hack because egui lost_focus method ~~sucks balls~~ is extremely inconsistent
                    if !split_editor.has_focus() && local_segment.editing_split {
                        state.editor.select_only(index);
                        state
                            .editor
                            .active_segment()
                            .parse_and_set_split_time(&local_segment.split)
                            .ok();
                        local_segment.editing_split = false;
                        state.dirty = true;
                    }

                    let segment_editor = ui.text_edit_singleline(&mut local_segment.segment);

                    if segment_editor.changed() {
                        local_segment.editing_segment = true;
                    }

                    if segment_editor.clicked() {
                        state.editor.select_only(index);
                    }

                    if !segment_editor.has_focus() && local_segment.editing_segment {
                        state.editor.select_only(index);
                        state
                            .editor
                            .active_segment()
                            .parse_and_set_segment_time(&local_segment.segment)
                            .ok();
                        local_segment.editing_segment = false;
                        state.dirty = true;
                    }

                    let best_segment_editor =
                        ui.text_edit_singleline(&mut local_segment.best_segment);

                    if best_segment_editor.clicked() {
                        state.editor.select_only(index);
                    }
                    if best_segment_editor.changed() {
                        local_segment.editing_best_segment = true;
                    }

                    if !best_segment_editor.has_focus() && local_segment.editing_best_segment {
                        state.editor.select_only(index);
                        state
                            .editor
                            .active_segment()
                            .parse_and_set_best_segment_time(&local_segment.best_segment)
                            .ok();
                        local_segment.editing_best_segment = false;
                        state.dirty = true;
                    }

                    ui.end_row();
                }
            });
        });
}
