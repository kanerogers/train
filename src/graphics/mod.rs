use core::Core;
use std::sync::Arc;

use context::Context;
use renderer::Renderer;

mod context;
mod core;
mod renderer;
mod swapchain;

pub struct Graphics {
    core: Core,
    context: Arc<Context>,
    renderer: Renderer,
    window: winit::window::Window,
}

impl Graphics {
    pub fn new(window: winit::window::Window) -> Self {
        let core = Core::new(&window);
        let context = Context::new(&core, &window);
        let context = Arc::new(context);
        let renderer = Renderer::new(context.clone());

        Graphics {
            core,
            context,
            renderer,
            window,
        }
    }

    pub(crate) fn draw(&self) {
        // println!("Drawing!");
    }
}
