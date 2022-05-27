use std::{fs::File, io::BufWriter};

use egui_glow::egui_winit::egui::Ui;
use livesplit_core::{run::saver::livesplit::save_run, TimerPhase};
use rfd::{FileDialog, MessageButtons, MessageDialog};

use crate::{
    utils::{load_layout_from_file, load_splits_from_file},
    SharedState,
};

use super::{edit_splits::SplitsState, settings::SettingsState, Menu};

pub(super) fn left_panel(ui: &mut Ui, menu: &mut Menu, shared_state: &mut SharedState) {
    if ui.button("Load Layout").clicked() {
        println!("loading layout");
        let path = FileDialog::new()
            .add_filter("LiveSplit Layout File", &["lsl"])
            .pick_file();

        if let Some(path) = path {
            match load_layout_from_file(&path) {
                Ok(layout) => {
                    println!("Layout loading successful");
                    shared_state.layout = layout;
                    shared_state.config.layout_path = Some(path);
                }
                Err(e) => {
                    MessageDialog::new()
                        .set_title("Failed to load layout")
                        .set_description(format!("Failed to load layout, got error {e}").as_str());
                }
            }
        } else {
        }
    }

    if ui.button("Load Splits").clicked() {
        println!("loading splits");
        let path = FileDialog::new()
            .add_filter("LiveSplit Splits File", &["lss"])
            .pick_file();

        if let Some(path) = path {
            if let Ok(splits) = load_splits_from_file(&path) {
                shared_state.config.splits_path = Some(path);

                println!("Split loading successful");

                // when we have new splits, discard the old ones
                if let Menu::EditSplits(state) = menu {
                    if state.dirty() {
                        let should_save_splits = MessageDialog::new()
                            .set_buttons(MessageButtons::YesNo)
                            .set_title("Save Splits?")
                            .set_description(
                                "Your splits have been edited, would you like to save them now?",
                            )
                            .show();

                        if should_save_splits {
                            if let Some(path) = state.run().path() {
                                if let Ok(file) = File::create(path) {
                                    save_run(state.run(), BufWriter::new(file)).ok();
                                }
                            }
                        }
                    }
                    *menu = Menu::EditSplits(SplitsState::new(splits.clone()).into());
                }

                let mut timer = shared_state.timer.write();

                let should_save_splits = if timer.run().has_been_modified() {
                    MessageDialog::new()
                        .set_buttons(MessageButtons::YesNo)
                        .set_title("Save Splits?")
                        .set_description(
                            "Your splits have been edited, would you like to save them now?",
                        )
                        .show()
                } else {
                    false
                };

                let old_splits = timer.replace_run(splits, true);

                if should_save_splits {
                    if let Ok(splits) = old_splits {
                        if let Some(path) = splits.path() {
                            if let Ok(file) = File::create(path) {
                                save_run(&splits, BufWriter::new(file)).ok();
                            }
                        }
                    }
                }
            }
        }
    }

    if ui.button("Edit Splits").clicked() && menu.on_destroy(shared_state) {
        if shared_state.timer.read().current_phase() == TimerPhase::NotRunning {
            *menu =
                Menu::EditSplits(SplitsState::new(shared_state.timer.read().run().clone()).into());

        } else {
            MessageDialog::new()
                .set_title("Can't edit splits")
                .set_description("You can't edit splits while the timer is running!+")
                .show();
        }
    }

    if ui.button("Edit Layout").clicked() && menu.on_destroy(shared_state) {
        *menu = Menu::EditLayout(Default::default());
    }

    if ui.button("Settings").clicked() && menu.on_destroy(shared_state) {
        *menu = Menu::Settings(SettingsState::new(shared_state.hotkey_system.config()).into());
    }
}
