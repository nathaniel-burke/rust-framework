use core_api::{run, Application};
use graphics_wgpu::GraphicsContext;
use std::sync::Arc;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

struct Demo {
    runtime: core_api::Runtime,
    window: Option<Arc<Window>>,
    graphics: Option<GraphicsContext>,
}

impl Application for Demo {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes().with_title("Rust program"))
                    .expect("failed to create application window"),
            );
            self.graphics = Some(
                pollster::block_on(GraphicsContext::new(Arc::clone(&window)))
                    .expect("failed to initialize graphics"),
            );
            self.window = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if Some(window_id) != self.window.as_ref().map(|window| window.id()) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(graphics) = &mut self.graphics {
                    graphics.resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(graphics) = &self.graphics {
                    let _ = graphics.render_clear(graphics_wgpu::wgpu::Color {
                        r: 0.04,
                        g: 0.08,
                        b: 0.12,
                        a: 1.0,
                    });
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.runtime.update(1.0 / 60.0);
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<(), winit::error::EventLoopError> {
    run(Demo {
        runtime: core_api::Runtime::new(),
        window: None,
        graphics: None,
    })
}
