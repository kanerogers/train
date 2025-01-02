use ash::vk;

pub struct Context {
    pub device: ash::Device,
    pub command_pool: vk::CommandPool,
    pub draw_command_buffer: vk::CommandBuffer,
    pub graphics_queue: vk::Queue,
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
                        &mut vk::PhysicalDeviceVulkan13Features::default()
                            .dynamic_rendering(true)
                            .synchronization2(true),
                    ),
                None,
            )
        }
        .unwrap();

        let command_pool = unsafe {
            device.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(0)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )
        }
        .unwrap();

        let draw_command_buffer = unsafe {
            device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(command_pool)
                    .command_buffer_count(1),
            )
        }
        .unwrap()[0];

        let graphics_queue = unsafe { device.get_device_queue(0, 0) };

        Self {
            device,
            command_pool,
            draw_command_buffer,
            graphics_queue,
        }
    }
}
