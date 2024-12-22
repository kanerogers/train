use std::ffi::CStr;

use ash::{vk, Instance};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct Core {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub physical_device: vk::PhysicalDevice,
}
impl Core {
    pub(crate) fn new(window: &winit::window::Window) -> Self {
        let entry = unsafe { ash::Entry::load().unwrap() };

        let display_handle = window.display_handle().unwrap().as_raw();

        let instance_extensions =
            ash_window::enumerate_required_extensions(display_handle).unwrap();

        let instance = unsafe {
            entry
                .create_instance(
                    &vk::InstanceCreateInfo::default()
                        .enabled_extension_names(&instance_extensions)
                        .application_info(
                            &vk::ApplicationInfo::default()
                                .api_version(vk::API_VERSION_1_3)
                                .application_name(c"Train Game")
                                .engine_name(c"Yeah Man"),
                        ),
                    None,
                )
                .unwrap()
        };

        let physical_device = unsafe { instance.enumerate_physical_devices() }
            .unwrap()
            .first()
            .copied()
            .unwrap();

        Self {
            entry,
            instance,
            physical_device,
        }
    }
}
