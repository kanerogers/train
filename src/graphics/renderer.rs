use std::{sync::Arc, u64};

use ash::vk::{self};

use super::{
    context::Context,
    pipeline::Pipeline,
    swapchain::{Drawable, Swapchain},
};

pub struct Renderer {
    pub pipeline: Pipeline,
    pub context: Arc<Context>,
    pub fence: vk::Fence,
    pub rendering_complete: vk::Semaphore,
    pub frame_index: u64,
    pub swapchain: Swapchain,
}

impl Renderer {
    pub(crate) fn new(context: Arc<Context>, swapchain: Swapchain) -> Self {
        let pipeline = Pipeline::new(context.clone(), swapchain.format);
        let device = &context.device;

        let rendering_complete =
            unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) }.unwrap();

        let fence = unsafe {
            device.create_fence(
                &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                None,
            )
        }
        .unwrap();

        Self {
            pipeline,
            context,
            rendering_complete,
            frame_index: 0,
            fence,
            swapchain,
        }
    }

    pub(crate) fn draw(&self) {
        let drawable = self.begin_rendering();
        self.pipeline.draw(drawable);
        self.end_rendering(drawable);
    }

    fn begin_rendering(&self) -> Drawable {
        let device = &self.context.device;
        unsafe {
            device.begin_command_buffer(
                self.context.draw_command_buffer,
                &vk::CommandBufferBeginInfo::default(),
            )
        }
        .unwrap();

        // Block the CPU until we're done rendering the previous frame
        unsafe {
            device
                .wait_for_fences(&[self.fence], true, u64::MAX)
                .unwrap();
            device.reset_fences(&[self.fence]).unwrap();
        }

        // Get a `Drawable` from the swapchain
        self.swapchain.get_drawable()
    }

    fn end_rendering(&self, drawable: Drawable) {
        let device = &self.context.device;
        let queue = self.context.graphics_queue;
        let command_buffer = self.context.draw_command_buffer;
        let swapchain_image = drawable.image;

        unsafe {
            // First, transition the color attachment into the present state

            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&[
                    vk::ImageMemoryBarrier2::default().image(swapchain_image),
                ]),
            );

            device.end_command_buffer(command_buffer).unwrap();
            device
                .queue_submit2(
                    queue,
                    &[vk::SubmitInfo2::default()
                        .command_buffer_infos(&[
                            vk::CommandBufferSubmitInfo::default().command_buffer(command_buffer)
                        ])
                        .wait_semaphore_infos(&[vk::SemaphoreSubmitInfo::default()
                            .semaphore(drawable.ready)
                            .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)])
                        .signal_semaphore_infos(&[vk::SemaphoreSubmitInfo::default()
                            .semaphore(self.rendering_complete)
                            .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)])],
                    self.fence,
                )
                .unwrap();
        }
    }
}
