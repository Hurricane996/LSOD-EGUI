use egui_glow::egui_winit::egui::{ComboBox, DragValue, Ui};
use enum_map::EnumMap;
use livesplit_core::{hotkey::KeyCode, HotkeyConfig};
use rfd::{MessageDialog, MessageLevel};

use crate::{
    hotkey::{Hotkey, KEY_CODES},
    SharedState, UserEvent,
};

pub struct SettingsState {
    hotkey_state: EnumMap<Hotkey, Option<KeyCode>>,
}

impl SettingsState {
    pub fn new(hotkey_config: HotkeyConfig) -> Self {
        let mut hotkey_state: EnumMap<Hotkey, Option<KeyCode>> = Default::default();

        for (hotkey, keycode) in hotkey_state.iter_mut() {
            *keycode = hotkey.get_keycode(&hotkey_config);
        }

        Self { hotkey_state }
    }

    pub fn on_destroy(&self, _shared_state: &SharedState) -> bool {
        // nop
        true
    }
}

fn display_hotkey<'a>(keycode: Option<KeyCode>) -> &'a str {
    match keycode {
        Some(keycode) => keycode.as_str(),
        None => "None",
    }
}

fn row(ui: &mut Ui, hotkey: Hotkey, _shared_state: &mut SharedState, state: &mut SettingsState) {
    ComboBox::from_label(hotkey.get_name())
        .selected_text(display_hotkey(state.hotkey_state[hotkey]))
        .show_ui(ui, |ui| {
            for key_code in KEY_CODES {
                ui.selectable_value(
                    &mut state.hotkey_state[hotkey],
                    Some(key_code),
                    display_hotkey(Some(key_code)),
                );
            }
        });
}

fn update_hotkeys(shared_state: &mut SharedState, state: &mut SettingsState) {
    let mut hotkey_config = shared_state.hotkey_system.config();

    let mut dirty = false;

    for (hotkey, keycode) in state.hotkey_state {
        if keycode != hotkey.get_keycode(&hotkey_config) {
            hotkey.set_keycode(&mut hotkey_config, keycode);
            dirty = true;
        }
    }

    if dirty {
        match shared_state.hotkey_system.set_config(hotkey_config) {
            Ok(()) => {
                //update config
                shared_state.config.hotkey_config = hotkey_config;
            }
            Err(e) => {
                // reset our state back to the hotkey state
                for (hotkey, keycode) in state.hotkey_state.iter_mut() {
                    *keycode = hotkey.get_keycode(&hotkey_config);
                }

                // then show an error
                MessageDialog::new()
                    .set_title("Failed to Update Hotkeys")
                    .set_level(MessageLevel::Error)
                    .set_description(format!("Could nto update hotkeys, got error {e}").as_str())
                    .show();
            }
        }
    }
}

pub(super) fn hotkey_component(
    ui: &mut Ui,
    shared_state: &mut SharedState,
    state: &mut SettingsState,
) {
    ui.heading("Hotkeys");

    row(ui, Hotkey::Split, shared_state, state);
    row(ui, Hotkey::Reset, shared_state, state);
    row(ui, Hotkey::Undo, shared_state, state);
    row(ui, Hotkey::Skip, shared_state, state);
    row(ui, Hotkey::Pause, shared_state, state);
    row(ui, Hotkey::UndoAllPauses, shared_state, state);
    row(ui, Hotkey::PreviousComparison, shared_state, state);
    row(ui, Hotkey::NextComparison, shared_state, state);
    row(ui, Hotkey::ToggleTimingMethod, shared_state, state);

    update_hotkeys(shared_state, state);

    ui.heading("Size");

    ui.horizontal(|ui| {
        ui.label("Width: ");
        ui.add(DragValue::from_get_set(|x| width_get_set(x, shared_state)));
    });

    ui.horizontal(|ui| {
        ui.label("Height: ");
        ui.add(DragValue::from_get_set(|x| height_get_set(x, shared_state)));
    });
}

fn height_get_set(x: Option<f64>, shared_state: &mut SharedState) -> f64 {
    match x {
        Some(height) => {
            shared_state.config.size.1 = height.round() as u32;

            if shared_state.config.size.1 == 0 {
                shared_state.config.size.1 = 1;
            }

            shared_state.send_event.send_event(UserEvent::Resize).ok();
            height
        }
        None => shared_state.config.size.1.into(),
    }
}

fn width_get_set(x: Option<f64>, shared_state: &mut SharedState) -> f64 {
    match x {
        Some(width) => {
            shared_state.config.size.0 = width.round() as u32;

            if shared_state.config.size.0 == 0 {
                shared_state.config.size.0 = 1;
            }

            shared_state.send_event.send_event(UserEvent::Resize).ok();
            width
        }
        None => shared_state.config.size.0.into(),
    }
}
