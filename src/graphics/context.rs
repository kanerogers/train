use ash::vk;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use super::swapchain::Swapchain;

pub struct Context {
    device: ash::Device,
    swapchain: Swapchain,
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
                    ),
                None,
            )
        }
        .unwrap();

        let swapchain = Swapchain::new(&device, core, window, vk::SwapchainKHR::null());

        Self { device, swapchain }
    }
}
