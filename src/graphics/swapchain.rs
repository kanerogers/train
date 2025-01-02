use ash::vk;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct Swapchain {
    pub surface_handle: vk::SurfaceKHR,
    pub surface_fn: ash::khr::surface::Instance,
    pub swapchain_handle: vk::SwapchainKHR,
    pub swapchain_fn: ash::khr::swapchain::Device,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub extent: vk::Extent2D,
    pub format: vk::Format,
    pub current_image_index: u32,
    image_available: vk::Semaphore,
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

        let (images, image_views) = unsafe { swapchain_fn.get_swapchain_images(swapchain_handle) }
            .unwrap()
            .into_iter()
            .map(|image| {
                let view = unsafe {
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
                .unwrap();

                (image, view)
            })
            .unzip();

        let image_available =
            unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) }.unwrap();

        Self {
            surface_handle,
            surface_fn,
            swapchain_handle,
            swapchain_fn,
            images,
            image_views,
            extent,
            format,
            current_image_index: 0,
            image_available,
        }
    }

    pub fn get_drawable(&self) -> Drawable {
        let (index, optimal) = unsafe {
            self.swapchain_fn
                .acquire_next_image(
                    self.swapchain_handle,
                    u64::MAX,
                    self.image_available,
                    vk::Fence::null(),
                )
                .unwrap()
        };

        if !optimal {
            // eprintln!("Swapchain is not optimal! This isn't handled yet");
        }

        Drawable {
            image: self.images[index as usize],
            view: self.image_views[index as usize],
            ready: self.image_available,
            index,
            extent: self.extent,
        }
    }

    pub fn present(&self, drawable: Drawable, queue: vk::Queue, rendering_complete: vk::Semaphore) {
        unsafe {
            self.swapchain_fn
                .queue_present(
                    queue,
                    &vk::PresentInfoKHR::default()
                        .wait_semaphores(&[rendering_complete])
                        .image_indices(&[drawable.index])
                        .swapchains(&[self.swapchain_handle]),
                )
                .unwrap();
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Drawable {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub ready: vk::Semaphore,
    pub index: u32,
    pub extent: vk::Extent2D,
}
