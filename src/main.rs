use configuration::Configuration;
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::WindowId,
};

use livesplit_core::{parking_lot::RwLock, HotkeySystem, Layout, Timer};
use std::{collections::HashMap, sync::Arc};

use crate::{gui::ConfigurationWindow, main_window::MainWindow, window::ApplicationWindow};

mod configuration;
mod gui;
mod hotkey;
mod main_window;
mod utils;
mod window;

pub enum UserEvent {
    SpawnConfigurationWindow,
    Resize,
    DestroyWindow(WindowId),
}

pub struct SharedState {
    layout: Layout,
    timer: Arc<RwLock<Timer>>,
    hotkey_system: HotkeySystem,
    config: Configuration,
    send_event: EventLoopProxy<UserEvent>,
    has_configuration_window: bool,
}

impl SharedState {
    fn new(config: Configuration, event_loop: &EventLoop<UserEvent>) -> SharedState {
        let timer: Arc<RwLock<Timer>> = Timer::new(config.run_or_default()).unwrap().into_shared();
        let hotkey_system = HotkeySystem::with_config(timer.clone(), config.hotkey_config)
            .unwrap_or_else(|e| panic!("Could not initialize hotkey system, got error {e}"));

        SharedState {
            layout: config.layout_or_default(),
            timer,
            config,
            hotkey_system,
            has_configuration_window: false,
            send_event: event_loop.create_proxy(),
        }
    }
}

fn main() {
    let config = Configuration::get_or_default();
    let event_loop = EventLoop::with_user_event();

    let mut main_window = MainWindow::new(&event_loop, config.size);
    let main_window_id = main_window.id();

    let mut other_windows: HashMap<WindowId, Box<dyn ApplicationWindow>> = HashMap::new();

    let mut shared_state = SharedState::new(config, &event_loop);

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested | WindowEvent::Destroyed,
            }
            | Event::UserEvent(UserEvent::DestroyWindow(window_id)) => {
                if window_id == main_window_id {
                    if main_window.on_destroy(&mut shared_state) {
                        other_windows.clear();
                        *control_flow = ControlFlow::Exit;
                    }
                } else if let Some(window) = other_windows.get_mut(&window_id) {
                    if window.on_destroy(&mut shared_state) {
                        other_windows.remove(&window_id);
                    }
                }
            }

            Event::WindowEvent { window_id, event } => {
                let window = if window_id == main_window_id {
                    &mut main_window
                } else if let Some(window) = other_windows.get_mut(&window_id) {
                    window.as_mut()
                } else {
                    return;
                };

                window.window_event(event, &mut shared_state);
            }

            Event::RedrawRequested(window_id) => {
                let window = if window_id == main_window_id {
                    &mut main_window
                } else if let Some(window) = other_windows.get_mut(&window_id) {
                    window.as_mut()
                } else {
                    return;
                };

                window.redraw(&mut shared_state);
            }

            Event::MainEventsCleared => {
                main_window.redraw(&mut shared_state);
            }
            Event::UserEvent(UserEvent::SpawnConfigurationWindow) => {
                let configuration_window = ConfigurationWindow::new(event_loop);
                other_windows.insert(configuration_window.id(), Box::new(configuration_window));
            }

            Event::UserEvent(UserEvent::Resize) => {
                main_window.resize(shared_state.config.size);
            }
            _ => {}
        }
    })
}
