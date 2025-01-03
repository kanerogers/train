use core::Core;
use std::sync::Arc;

use ash::vk;
use camera::Camera;
use context::Context;
use renderer::Renderer;
use swapchain::Swapchain;

use crate::input::Input;

mod camera;
mod context;
mod core;
mod depth_buffer;
mod pipeline;
mod renderer;
mod swapchain;

pub struct Graphics {
    #[allow(unused)]
    core: Core,
    #[allow(unused)]
    context: Arc<Context>,
    renderer: Renderer,
    #[allow(unused)]
    window: winit::window::Window,
    pub camera: Camera,
}

impl Graphics {
    pub fn new(window: winit::window::Window) -> Self {
        let core = Core::new(&window);
        let context = Context::new(&core);
        let context = Arc::new(context);
        let swapchain = Swapchain::new(&context.device, &core, &window, vk::SwapchainKHR::null());
        let camera = Camera::new(swapchain.extent);
        let renderer = Renderer::new(context.clone(), swapchain);

        Graphics {
            core,
            context,
            renderer,
            window,
            camera,
        }
    }

    pub(crate) fn draw(&mut self, input: &Input) {
        self.camera.update(1.0 / 60.0, input);
        self.renderer.draw(&self.camera);
    }
}

const FULL_IMAGE: vk::ImageSubresourceRange = vk::ImageSubresourceRange {
    aspect_mask: vk::ImageAspectFlags::COLOR,
    base_mip_level: 0,
    level_count: vk::REMAINING_MIP_LEVELS,
    base_array_layer: 0,
    layer_count: vk::REMAINING_ARRAY_LAYERS,
};
