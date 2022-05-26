use std::{fmt::Write, fs::File, io::BufWriter, mem};

use egui_glow::egui_winit::egui::{Color32, Frame, Grid, Label, ScrollArea, Sense, Ui};
use livesplit_core::{
    run::editor::{Editor, SegmentState},
    run::saver::livesplit::save_run,
    timing::formatter::timer::{Fraction, Time},
    Run,
};

use lazy_static::lazy_static;

use crate::SharedState;

lazy_static! {
    static ref TIME_FORMATTER: Time = Time::default();
    static ref FRACTION_FORMATTER: Fraction = Fraction::default();
}

fn segments(
    ui: &mut Ui,
    state: &mut SplitsState,
    editor_state: &mut livesplit_core::run::editor::State,
) {
    //let mut editor_state = state.editor.state();

    ScrollArea::both()
        .max_height(ui.available_height() / 3.)
        .show(ui, |ui| {
            Grid::new("splits").show(ui, |ui| {
                ui.heading("Segment Name");
                ui.heading("Split time");
                ui.heading("Segment Time");
                ui.heading("Best Segment");

                ui.end_row();

                for (index, segment) in editor_state.segments.iter_mut().enumerate() {
                    if segment.selected.is_selected_or_active() {
                        ui.visuals_mut().widgets.noninteractive.bg_fill = Color32::DARK_BLUE;
                    }


                    let local_segment = &mut state.segments[index];

                    if !local_segment.editing_split {
                        local_segment.split = mem::take(&mut segment.split_time);
                    }

                    if !local_segment.editing_segment {
                        local_segment.segment = mem::take(&mut segment.segment_time);
                    }

                    if !local_segment.editing_best_segment {
                        local_segment.best_segment = mem::take(&mut segment.best_segment_time);
                    }


                    let frame = || {
                        Frame::none().fill(if segment.selected.is_selected_or_active() {
                            Color32::DARK_BLUE
                        } else {
                            Color32::TRANSPARENT
                        })
                    };

                    frame().show(ui, |ui| {
                        let name_editor = ui.text_edit_singleline(&mut segment.name);

                        if name_editor.clicked() {
                            state.editor.select_only(index);
                        }
                        if name_editor.changed() {
                            state.editor.select_only(index);
                            state.editor.active_segment().set_name(&segment.name);
                            state.dirty = true;
                        }
                    });

                    frame().show(ui, |ui| {
                        let split_editor = ui.text_edit_singleline(&mut local_segment.split);

                        if split_editor.clicked() || split_editor.changed() {
                            state.editor.select_only(index);
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

                            println!("storing splits");

                            state.dirty = true;
                        }
                    });

                    frame().show(ui, |ui| {
                        let segment_editor = ui.text_edit_singleline(&mut local_segment.segment);

                        if segment_editor.clicked() || segment_editor.changed() {
                            local_segment.editing_segment = true;
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
                    });
                    
                    frame().show(ui, |ui| {

                        let best_segment_editor =
                            ui.text_edit_singleline(&mut local_segment.best_segment);

                        if best_segment_editor.changed() || best_segment_editor.clicked() {
                            state.editor.select_only(index);
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
                    });

                    ui.end_row();
                }
            });
        });
}

pub(super) fn edit_splits(ui: &mut Ui, shared_state: &mut SharedState, state: &mut SplitsState) {
    // we edit this local copy and then update the actual editor with any changes egui makes
    let mut editor_state = state.editor.state();

    ui.label("This function is not finished yet. In the meantime, generate splits in the web version, then export them.");

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
                }

                if text.lost_focus() {
                    state.attempts_string.clear();
                    write!(state.attempts_string, "{}", state.editor.attempt_count()).ok();
                }
            })
        });

    segments(ui, state, &mut editor_state);

    ui.horizontal(|ui| {
        if ui.button("Save").clicked() && state.dirty {
            if let Some(path) = state.editor.run().path() {
                if let Ok(file) = File::create(path) {
                    save_run(&state.editor.run(), BufWriter::new(file)).ok();
                }
            }

            let mut timer = shared_state.timer.write();
            timer.replace_run(state.editor.run().clone(), false).ok();
        }

        if ui.button("Discard Changes").clicked() {
            *state = SplitsState::new(shared_state.timer.read().run().clone());
        }
    });
}

struct SegmentLocal {
    split: String,
    editing_split: bool,
    segment: String,
    editing_segment: bool,
    best_segment: String,
    editing_best_segment: bool,
}
impl SegmentLocal {
    fn new(seg: SegmentState) -> Self {
        Self {
            split: seg.split_time,
            best_segment: seg.best_segment_time,
            segment: seg.segment_time,
            editing_split: false,
            editing_segment: false,
            editing_best_segment: false,
        }
    }
}

pub(super) struct SplitsState {
    editor: Editor,
    attempts_string: String,
    dirty: bool,
    segments: Vec<SegmentLocal>,
}

impl SplitsState {
    pub fn new(run: Run) -> Self {
        let mut editor = Editor::new(run).expect("Could not eddit run");

        Self {
            segments: editor
                .state()
                .segments
                .into_iter()
                .map(SegmentLocal::new)
                .collect(),
            attempts_string: editor.attempt_count().to_string(),
            editor,
            dirty: false,
        }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub(crate) fn run(&self) -> &Run {
        &self.editor.run()
    }
}
