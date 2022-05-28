use std::{fmt::Write, fs::File, io::BufWriter};

use egui_glow::egui_winit::egui::Ui;
use livesplit_core::{
    run::editor::{Editor, SegmentState},
    run::saver::livesplit::save_run,
    timing::formatter::timer::{Fraction, Time},
    Run,
};

use lazy_static::lazy_static;
use rfd::{MessageButtons, MessageDialog};

use crate::SharedState;

lazy_static! {
    static ref TIME_FORMATTER: Time = Time::default();
    static ref FRACTION_FORMATTER: Fraction = Fraction::default();
}

mod meta;
mod segments;

fn split_editing_buttons(ui: &mut Ui, state: &mut SplitsState) {
    ui.vertical(|ui| {
        if ui.button("Insert Above").clicked() {
            state.editor.insert_segment_above();
            state.segments = SegmentLocal::gen_vec(&mut state.editor);
            state.dirty = true;
        };

        if ui.button("Insert Below").clicked() {
            state.editor.insert_segment_below();
            state.segments = SegmentLocal::gen_vec(&mut state.editor);
            state.dirty = true;
        };

        if ui.button("Remove Segment").clicked() {
            state.editor.remove_segments();
            state.segments = SegmentLocal::gen_vec(&mut state.editor);
            state.dirty = true;
        };

        if ui.button("Move Up").clicked() {
            state.editor.move_segments_up();
            state.segments = SegmentLocal::gen_vec(&mut state.editor);
            state.dirty = true;
        };

        if ui.button("Move Down").clicked() {
            state.editor.move_segments_down();
            state.segments = SegmentLocal::gen_vec(&mut state.editor);
            state.dirty = true;
        };

        if ui.button("Clear History").clicked() {
            state.editor.clear_history();
            state.segments = SegmentLocal::gen_vec(&mut state.editor);

            state.sync_attempts_string();

            state.dirty = true;
        };

        if ui.button("Clear Times").clicked() {
            state.editor.clear_times();
            state.segments = SegmentLocal::gen_vec(&mut state.editor);

            state.sync_attempts_string();

            state.dirty = true;
        };
    });
}

pub(super) fn edit_splits(ui: &mut Ui, shared_state: &mut SharedState, state: &mut SplitsState) {
    // we edit this local copy and then update the actual editor with any changes egui makes
    let mut editor_state = state.editor.state();
    
    meta::split_metadata(ui, state, &mut editor_state);

    ui.horizontal_top(|ui| {
        split_editing_buttons(ui, state);
        segments::segments(ui, state);
    });

    ui.horizontal(|ui| {
        if ui.button("Save").clicked() && state.dirty {
            if let Some(path) = state.editor.run().path() {
                if let Ok(file) = File::create(path) {
                    save_run(state.editor.run(), BufWriter::new(file)).ok();
                }
            }

            let mut timer = shared_state.timer.write();
            timer.replace_run(state.editor.run().clone(), false).ok();

            state.dirty = false;
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

    fn gen_vec(editor: &mut Editor) -> Vec<Self> {
        editor
            .state()
            .segments
            .into_iter()
            .map(SegmentLocal::new)
            .collect()
    }
}

pub(super) struct SplitsState {
    editor: Editor,
    attempts_string: String,
    dirty: bool,
    segments: Vec<SegmentLocal>,
    grid_width: f32,
}

impl SplitsState {
    pub fn new(run: Run) -> Self {
        let mut editor = Editor::new(run).expect("Could not eddit run");

        Self {
            segments: SegmentLocal::gen_vec(&mut editor),
            attempts_string: editor.attempt_count().to_string(),
            editor,
            dirty: false,
            grid_width: 0.0,
        }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn on_destroy(&mut self, shared_state: &SharedState) -> bool {
        if self.dirty {
            let should_save_splits = MessageDialog::new()
                .set_buttons(MessageButtons::YesNo)
                .set_title("Save Splits?")
                .set_description("Your splits have been edited, would you like to save them now?")
                .show();

            if should_save_splits {
                if let Some(path) = self.run().path() {
                    if let Ok(file) = File::create(path) {
                        save_run(self.run(), BufWriter::new(file)).ok();
                    }
                }

                shared_state
                    .timer
                    .write()
                    .replace_run(self.editor.run().clone(), false)
                    .ok();
            }
        }
        true
    }

    pub(crate) fn run(&self) -> &Run {
        self.editor.run()
    }

    pub fn sync_attempts_string(&mut self) {
        self.attempts_string.clear();
        write!(self.attempts_string, "{}", self.editor.attempt_count()).ok();
    }
}
