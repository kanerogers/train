use ash::vk;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct Swapchain {
    pub surface_handle: vk::SurfaceKHR,
    pub surface_fn: ash::khr::surface::Instance,
    pub swapchain_handle: vk::SwapchainKHR,
    pub swapchain_fn: ash::khr::swapchain::Device,
    pub image_views: Vec<vk::ImageView>,
    pub extent: vk::Extent2D,
    pub format: vk::Format,
}

impl Swapchain {
    pub(crate) fn new(
        device: &ash::Device,
        core: &super::core::Core,
        window: &winit::window::Window,
        old_swapchain: vk::SwapchainKHR,
    ) -> Self {
        let entry = &core.entry;
        let instance = &core.instance;
        let window_handle = window.window_handle().unwrap().as_raw();
        let display_handle = window.display_handle().unwrap().as_raw();

        let surface_handle = unsafe {
            ash_window::create_surface(entry, instance, display_handle, window_handle, None)
        }
        .unwrap();

        let surface_fn = ash::khr::surface::Instance::new(entry, instance);
        let surface_formats = unsafe {
            surface_fn.get_physical_device_surface_formats(core.physical_device, surface_handle)
        }
        .unwrap();

        let format_preferences = [vk::Format::B8G8R8A8_SRGB, vk::Format::R8G8B8A8_SRGB];

        let format = *format_preferences
            .iter()
            .find(|&&f| surface_formats.iter().any(|sf| sf.format == f))
            .expect("Desired swapchain format unavailable");

        let capabilities = unsafe {
            surface_fn
                .get_physical_device_surface_capabilities(core.physical_device, surface_handle)
        }
        .unwrap();

        let extent = vk::Extent2D {
            width: window.inner_size().width,
            height: window.inner_size().height,
        };

        let swapchain_fn = ash::khr::swapchain::Device::new(instance, device);
        let swapchain_handle = unsafe {
            swapchain_fn.create_swapchain(
                &vk::SwapchainCreateInfoKHR::default()
                    .surface(surface_handle)
                    .min_image_count(capabilities.min_image_count + 1)
                    .image_format(format)
                    .image_extent(extent)
                    .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
                    .image_array_layers(1)
                    .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                    .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .queue_family_indices(&[0])
                    .clipped(true)
                    .present_mode(vk::PresentModeKHR::FIFO)
                    .pre_transform(capabilities.current_transform)
                    .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                    .old_swapchain(old_swapchain),
                None,
            )
        }
        .unwrap();

        let image_views = unsafe { swapchain_fn.get_swapchain_images(swapchain_handle) }
            .unwrap()
            .into_iter()
            .map(|image| {
                unsafe {
                    device.create_image_view(
                        &vk::ImageViewCreateInfo::default()
                            .view_type(vk::ImageViewType::TYPE_2D)
                            .image(image)
                            .format(format)
                            .subresource_range(
                                vk::ImageSubresourceRange::default()
                                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                                    .base_mip_level(0)
                                    .level_count(1)
                                    .base_array_layer(0)
                                    .layer_count(1),
                            ),
                        None,
                    )
                }
                .unwrap()
            })
            .collect::<Vec<_>>();

        Self {
            surface_handle,
            surface_fn,
            swapchain_handle,
            swapchain_fn,
            image_views,
            extent,
            format,
        }
    }
}
