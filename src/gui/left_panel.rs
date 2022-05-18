use std::{fs::File, io::BufWriter};

use egui_glow::egui_winit::egui::Ui;
use livesplit_core::run::saver::livesplit::save_run;
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
            if let Ok(layout) = load_layout_from_file(&path) {
                println!("Layout loading successful");
                shared_state.layout = layout;
                shared_state.config.layout_path = Some(path);
            }
        }
        // todo error handling
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

    if ui.button("Edit Splits").clicked() {
        *menu = Menu::EditSplits(SplitsState::new(shared_state.timer.read().run().clone()))
    }

    if ui.button("Edit Layout").clicked() {
        *menu = Menu::EditLayout(Default::default())
    }

    if ui.button("Settings").clicked() {
        *menu = Menu::Settings(SettingsState::new(shared_state.hotkey_system.config()))
    }
}
