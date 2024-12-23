use ash::vk;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use super::swapchain::Swapchain;

pub struct Context {
    pub device: ash::Device,
    pub swapchain: Swapchain,
    pub command_pool: vk::CommandPool,
    pub draw_command_buffer: vk::CommandBuffer,
}

impl Context {
    pub(crate) fn new(core: &super::core::Core, window: &winit::window::Window) -> Self {
        let instance = &core.instance;
        let physical_device = core.physical_device;

        let device = unsafe {
            instance.create_device(
                physical_device,
                &vk::DeviceCreateInfo::default()
                    .enabled_extension_names(&[ash::khr::swapchain::NAME.as_ptr()])
                    .queue_create_infos(&[vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(0)
                        .queue_priorities(&[1.0])])
                    .enabled_features(
                        &vk::PhysicalDeviceFeatures::default().fill_mode_non_solid(true),
                    )
                    .push_next(
                        &mut vk::PhysicalDeviceVulkan13Features::default().dynamic_rendering(true),
                    ),
                None,
            )
        }
        .unwrap();

        let command_pool = unsafe {
            device.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )
        }
        .unwrap();

        let draw_command_buffer = unsafe {
            device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(command_pool)
                    .command_buffer_count(1)
                    .level(vk::CommandBufferLevel::PRIMARY),
            )
        }
        .unwrap()[0];

        let swapchain = Swapchain::new(&device, core, window, vk::SwapchainKHR::null());

        Self {
            device,
            swapchain,
            command_pool,
            draw_command_buffer,
        }
    }
}
