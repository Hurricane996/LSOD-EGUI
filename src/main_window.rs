use std::{fs::File, io::BufWriter};

use glutin::{
    dpi::PhysicalSize,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder, WindowId},
};
use livesplit_core::{
    layout::LayoutState, rendering::software::BorrowedRenderer, run::saver::livesplit::save_run,
    TimerPhase,
};
use pixels::{Pixels, SurfaceTexture};
use rfd::{MessageButtons, MessageDialog};

use crate::{window::ApplicationWindow, SharedState, UserEvent};

pub struct MainWindow {
    window: Window,
    pixels: Pixels,
    renderer: BorrowedRenderer,
    layout_state: LayoutState,
    window_size: PhysicalSize<u32>,
}

impl ApplicationWindow for MainWindow {
    fn window_event(&mut self, event: WindowEvent, shared_state: &mut SharedState) {
        match event {
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                ..
            } => {
                if !shared_state.has_configuration_window {
                    shared_state.has_configuration_window = true;
                    shared_state
                        .send_event
                        .send_event(UserEvent::SpawnConfigurationWindow)
                        .ok();
                }
            }
            WindowEvent::Resized(size) => {
                self.pixels.resize_surface(size.width, size.height);
                self.pixels.resize_buffer(size.width, size.height);
                self.window_size = size;

                self.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, vert) => {
                    if vert > 0.0 {
                        shared_state.layout.scroll_up();
                    } else if vert < 0.0 {
                        shared_state.layout.scroll_down();
                    }
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    if pos.y > 0.0 {
                        shared_state.layout.scroll_up();
                    } else if pos.y < 0.0 {
                        shared_state.layout.scroll_down();
                    }
                }
            },
            _ => self.request_redraw(),
        }
    }

    fn redraw(&mut self, shared_state: &mut crate::SharedState) {
        if self.window_size.width != 0 && self.window_size.height != 0 {
            let timer = shared_state.timer.read();

            shared_state
                .layout
                .update_state(&mut self.layout_state, &timer.snapshot());

            self.renderer.render(
                &self.layout_state,
                self.pixels.get_frame(),
                [self.window_size.width, self.window_size.height],
                self.window_size.width,
                false,
            );

            self.pixels
                .render()
                .unwrap_or_else(|e| panic!("Pixels failed to redraw, got error {e}"));
        }
    }

    fn id(&self) -> WindowId {
        self.window.id()
    }

    fn request_redraw(&mut self) {
        self.window.request_redraw();
    }

    fn on_destroy(&mut self, shared_state: &mut SharedState) -> bool {
        shared_state.config.save().ok();

        let mut timer = shared_state.timer.write();

        if let TimerPhase::Running | TimerPhase::Paused = timer.current_phase() {
            let quit = MessageDialog::new()
                .set_buttons(MessageButtons::YesNo)
                .set_title("Really quit?")
                .set_description("The timer is still running. Would you really like to quit?")
                .show();

            if !quit {
                return false;
            }
        }

        let should_save_splits = if timer.run().has_been_modified() {
            MessageDialog::new()
                .set_buttons(MessageButtons::YesNo)
                .set_title("Save Splits?")
                .set_description("Your splits have been edited, would you like to save them now?")
                .show()
        } else {
            false
        };

        if should_save_splits {
            // for some reason when the timer is in the "ended" state, the new pb doesn't save
            if let TimerPhase::Ended = timer.current_phase() {
                timer.reset(true);
            }

            let splits = timer.run();
            if let Some(path) = splits.path() {
                if let Ok(file) = File::create(path) {
                    save_run(splits, BufWriter::new(file)).ok();
                }
            }
        }

        true
    }
}
impl MainWindow {
    pub(crate) fn new(event_loop: &EventLoop<UserEvent>, size: (u32, u32)) -> Self {
        println!("{}, {}", size.0, size.1);
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(size.0, size.1))
            .build(event_loop)
            .unwrap_or_else(|e| panic!("Could not create main window, got error {e}"));

        let window_size = window.inner_size();

        let pixels = {
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(window_size.width, window_size.height, surface_texture).unwrap()
        };

        let renderer = BorrowedRenderer::new();

        Self {
            window,
            window_size,
            pixels,
            renderer,
            layout_state: LayoutState::default(),
        }
    }

    pub fn resize(&mut self, size: (u32, u32)) {
        self.window
            .set_inner_size(PhysicalSize::new(size.0, size.1));
    }
}
