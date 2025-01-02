use core::Core;
use std::sync::Arc;

use ash::vk;
use context::Context;
use renderer::Renderer;
use swapchain::Swapchain;

mod context;
mod core;
mod pipeline;
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
        let swapchain = Swapchain::new(&context.device, &core, &window, vk::SwapchainKHR::null());
        let renderer = Renderer::new(context.clone(), swapchain);

        Graphics {
            core,
            context,
            renderer,
            window,
        }
    }

    pub(crate) fn draw(&self) {
        self.renderer.draw();
    }
}

const FULL_IMAGE: vk::ImageSubresourceRange = vk::ImageSubresourceRange {
    aspect_mask: vk::ImageAspectFlags::COLOR,
    base_mip_level: 0,
    level_count: vk::REMAINING_MIP_LEVELS,
    base_array_layer: 0,
    layer_count: vk::REMAINING_ARRAY_LAYERS,
};
