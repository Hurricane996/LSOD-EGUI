use std::{fs::File, io::BufWriter};

use egui_glow::egui_winit::egui::Ui;
use livesplit_core::{run::saver::livesplit::save_run, Run};

use crate::SharedState;

pub(super) fn edit_splits(ui: &mut Ui, shared_state: &mut SharedState, state: &mut SplitsState) {
    ui.label("This function is not finished yet. In the meantime, generate splits in the web version, then export them.");

    ui.horizontal(|ui| {
        ui.label("Game Name");

        if ui.text_edit_singleline(&mut state.game_name).changed() {
            state.run.set_game_name(&state.game_name);
        }
    });

    if ui.button("Save").clicked() {
        if let Some(path) = state.run.path() {
            if let Ok(file) = File::create(path) {
                save_run(&state.run, BufWriter::new(file)).ok();
            }
        }

        let mut timer = shared_state.timer.write();
        timer.replace_run(state.run.clone(), false).ok();
    }
}

pub(super) struct SplitsState {
    run: Run,
    game_name: String,
}

impl SplitsState {
    pub fn new(run: Run) -> Self {
        let game_name = String::from(run.game_name());
        Self { run, game_name }
    }
}
