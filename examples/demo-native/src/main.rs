use core_api::{run, Application};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

struct Demo {
    runtime: core_api::Runtime,
    window: Option<Window>,
}

impl Application for Demo {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.window = Some(
                event_loop
                    .create_window(Window::default_attributes().with_title("Rust program"))
                    .expect("failed to create application window"),
            );
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if matches!(event, WindowEvent::CloseRequested) {
            event_loop.exit();
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.runtime.update(1.0 / 60.0);
    }
}

fn main() -> Result<(), winit::error::EventLoopError> {
    run(Demo { runtime: core_api::Runtime::new(), window: None })
}
