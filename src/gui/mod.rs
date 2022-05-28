mod left_panel;
use left_panel::left_panel;

pub mod settings;
use settings::hotkey_component;

mod edit_layout;
use edit_layout::edit_layout;
mod edit_splits;
use edit_splits::edit_splits;

use egui_glow::{
    egui_winit::egui::{self, FontData, FontDefinitions, FontFamily},
    EguiGlow,
};
use glutin::{
    event::{KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder, WindowId},
    ContextWrapper, PossiblyCurrent,
};
use std::rc::Rc;

use crate::{window::ApplicationWindow, SharedState, UserEvent};

use self::{edit_layout::LayoutState, edit_splits::SplitsState, settings::SettingsState};

enum Menu {
    Main,
    Settings(Box<SettingsState>),
    EditSplits(Box<SplitsState>),
    EditLayout(Box<LayoutState>),
}

impl Menu {
    fn on_destroy(&mut self, shared_state: &mut SharedState) -> bool {
        match self {
            Menu::Main => true,
            Menu::Settings(state) => state.on_destroy(shared_state),
            Menu::EditSplits(state) => state.on_destroy(shared_state),
            Menu::EditLayout(state) => state.on_destroy(shared_state),
        }
    }
}
const ARIAL: &[u8] = include_bytes!("../arial.ttf");
pub struct ConfigurationWindow {
    pub gl_window: ContextWrapper<PossiblyCurrent, Window>,
    pub egui_glow: EguiGlow,
    gl: Rc<glow::Context>,
    clear_color: [f32; 3],

    current_menu: Menu,
}

impl ApplicationWindow for ConfigurationWindow {
    fn window_event(&mut self, event: WindowEvent, shared_state: &mut SharedState) {
        if let WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                },
            ..
        } = event
        {
            shared_state
                .send_event
                .send_event(UserEvent::DestroyWindow(self.id()))
                .ok();
        }

        self.egui_glow.on_event(&event);

        self.gl_window.window().request_redraw();
    }

    fn redraw(&mut self, shared_state: &mut SharedState) {
        let needs_repaint = self.egui_glow.run(self.gl_window.window(), |ctx| {
            ConfigurationWindow::egui(ctx, &mut self.current_menu, shared_state);
        });

        if needs_repaint {
            unsafe {
                use glow::HasContext as _;
                self.gl.clear_color(
                    self.clear_color[0],
                    self.clear_color[1],
                    self.clear_color[2],
                    1.0,
                );

                // idk what this does
                self.gl.clear(glow::COLOR_BUFFER_BIT);
            }

            self.egui_glow.paint(self.gl_window.window());

            self.gl_window.swap_buffers().unwrap();

            self.request_redraw();
        }
    }

    fn id(&self) -> WindowId {
        self.gl_window.window().id()
    }

    fn request_redraw(&mut self) {
        self.gl_window.window().request_redraw();
    }

    fn on_destroy(&mut self, shared_state: &mut SharedState) -> bool {
        let ret = self.current_menu.on_destroy(shared_state);

        if ret {
            shared_state.has_configuration_window = false;
        }

        ret
    }
}

impl ConfigurationWindow {
    pub fn new(event_loop: &EventLoopWindowTarget<UserEvent>) -> Self {
        let wb = WindowBuilder::new().with_title("LiveSplit One Configuration");
        let gl_window = unsafe {
            glutin::ContextBuilder::new()
                .with_depth_buffer(0)
                .with_srgb(true)
                .with_stencil_buffer(0)
                .with_vsync(true)
                .build_windowed(wb, event_loop)
                .unwrap()
                .make_current()
                .unwrap()
        };

        let gl = Rc::new(unsafe {
            glow::Context::from_loader_function(|s| gl_window.get_proc_address(s))
        });

        let mut egui_glow = EguiGlow::new(gl_window.window(), gl.clone());

        ConfigurationWindow::egui_setup(&mut egui_glow.egui_ctx);

        ConfigurationWindow {
            gl_window,
            egui_glow,
            gl,
            current_menu: Menu::Main,
            clear_color: [0., 1., 0.1],
        }
    }

    pub fn id(&self) -> WindowId {
        self.gl_window.window().id()
    }

    fn egui(ctx: &egui::Context, menu: &mut Menu, shared_state: &mut SharedState) {
        egui::SidePanel::left("Left Panel").show(ctx, |ui| left_panel(ui, menu, shared_state));
        egui::CentralPanel::default().show(ctx, |ui| match menu {
            Menu::Main => {}
            Menu::Settings(state) => hotkey_component(ui, shared_state, state),
            Menu::EditLayout(_) => edit_layout(ui),
            Menu::EditSplits(state) => edit_splits(ui, shared_state, state),
        });
    }

    fn egui_setup(ctx: &mut egui::Context) {
        let mut fonts = FontDefinitions::default();

        fonts
            .font_data
            .insert("arial".to_owned(), FontData::from_static(ARIAL));

        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .push("arial".to_owned());

        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .push("arial".to_owned());

        ctx.set_fonts(fonts);
    }
}
