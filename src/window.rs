use glutin::{event::WindowEvent, window::WindowId};

use crate::SharedState;

pub trait ApplicationWindow {
    fn window_event(&mut self, event: WindowEvent, shared_state: &mut SharedState);
    fn redraw(&mut self, shared_state: &mut SharedState);
    fn id(&self) -> WindowId;
    fn request_redraw(&mut self);
    // returns true if the window should be destroyed
    fn on_destroy(&mut self, _shared_state: &mut SharedState) -> bool {
        true
    }
}
