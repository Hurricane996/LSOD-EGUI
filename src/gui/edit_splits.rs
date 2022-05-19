use std::{fmt::Write, fs::File, io::BufWriter};

use egui_glow::egui_winit::egui::{Grid, ScrollArea, Ui};
use livesplit_core::{
    run::saver::livesplit::save_run,
    timing::formatter::{
        timer::{Fraction, Time},
        TimeFormatter,
    },
    Run, TimeSpan,
};

use crate::SharedState;

pub(super) fn edit_splits(ui: &mut Ui, shared_state: &mut SharedState, state: &mut SplitsState) {
    ui.label("This function is not finished yet. In the meantime, generate splits in the web version, then export them.");

    Grid::new("metadata")
        .min_col_width(ui.available_width() / 2.)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label("Game Name");

                if ui.text_edit_singleline(&mut state.game_name).changed() {
                    state.run.set_game_name(&state.game_name);
                }
            });

            ui.vertical(|ui| {
                ui.label("Category Name");

                if ui.text_edit_singleline(&mut state.category_name).changed() {
                    state.run.set_category_name(&state.category_name);
                }
            });

            ui.end_row();

            ui.vertical(|ui| {
                ui.label("Offset");

                let text = ui.text_edit_singleline(&mut state.offset_string);

                if text.changed() {
                    if state.offset_string.is_empty() {
                        state.run.set_offset(TimeSpan::zero())
                    } else {
                        match state.attempts_string.parse::<f64>() {
                            Ok(new_offset) => {
                                state.run.set_offset(TimeSpan::from_seconds(new_offset))
                            }
                            Err(_) => {}
                        }
                    }
                }

                if text.lost_focus() {
                    state.offset_string = state.run.offset().to_duration().to_string();
                }
            });

            ui.vertical(|ui| {
                ui.label("Attempts");

                let text = ui.text_edit_singleline(&mut state.attempts_string);
                if text.changed() {
                    if state.attempts_string.is_empty() {
                        state.run.set_attempt_count(0);
                    } else {
                        match state.attempts_string.parse::<u32>() {
                            Err(_) => {
                                state.attempts_string = state.run.attempt_count().to_string();
                            }
                            Ok(new_count) => state.run.set_attempt_count(new_count),
                        }
                    }
                }

                if text.lost_focus() && state.attempts_string.is_empty() {
                    state.attempts_string.push_str("0")
                }
            })
        });

    ScrollArea::both()
        .max_height(ui.available_height() / 3.)
        .show(ui, |ui| {
            Grid::new("splits").striped(true).show(ui, |ui| {
                ui.heading("Segment name");

                for comparison in state.run.comparisons() {
                    if comparison == "None" {
                        continue;
                    }
                    ui.heading(comparison);
                }

                ui.end_row();

                for segment in state.run.segments_mut() {
                    // todo move this to state to avoid allocation every frame.`
                    let mut text = segment.name().to_string();

                    if ui.text_edit_singleline(&mut text).changed() {
                        segment.set_name(text);
                    }

                    for (name, comparison) in segment.comparisons_mut().iter_mut() {
                        if name.as_ref() == "None" {
                            continue;
                        }
                        // todo move this to state to avoid allocation every frame
                        let mut label = String::new();
                        if let Some(time) = comparison.real_time {
                            write!(label, "{}", state.time_formatter.format(time)).unwrap();
                            write!(label, "{}", state.fraction_formatter.format(time)).unwrap();
                        }

                        if ui.text_edit_singleline(&mut label).changed() {
                            // todo actually update it
                        }
                    }

                    ui.end_row();
                }
            });
        });

    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            if let Some(path) = state.run.path() {
                if let Ok(file) = File::create(path) {
                    save_run(&state.run, BufWriter::new(file)).ok();
                }
            }

            let mut timer = shared_state.timer.write();
            timer.replace_run(state.run.clone(), false).ok();
        }

        if ui.button("Discard Changes").clicked() {
            *state = SplitsState::new(shared_state.timer.read().run().clone())
        }
    });
}

pub(super) struct SplitsState {
    run: Run,
    time_formatter: Time,
    fraction_formatter: Fraction,
    game_name: String,
    category_name: String,
    attempts_string: String,
    offset_string: String,
}

impl SplitsState {
    pub fn new(run: Run) -> Self {
        Self {
            game_name: run.game_name().to_string(),
            category_name: run.category_name().to_string(),
            attempts_string: run.attempt_count().to_string(),
            offset_string: run.offset().to_duration().to_string(),
            run,
            time_formatter: Default::default(),
            fraction_formatter: Default::default(),
        }
    }
}
