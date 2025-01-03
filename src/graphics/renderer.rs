use std::{sync::Arc, u64};

use ash::vk::{self};

use super::{
    camera::Camera,
    context::Context,
    depth_buffer::{DepthBuffer, DEPTH_RANGE},
    pipeline::Pipeline,
    swapchain::{Drawable, Swapchain},
    FULL_IMAGE,
};

pub struct Renderer {
    pub pipeline: Pipeline,
    pub context: Arc<Context>,
    pub fence: vk::Fence,
    pub rendering_complete: vk::Semaphore,
    pub frame_index: u64,
    pub swapchain: Swapchain,
    pub depth_buffer: DepthBuffer,
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

        let depth_buffer = DepthBuffer::new(&context, &swapchain);

        Self {
            pipeline,
            context,
            rendering_complete,
            frame_index: 0,
            fence,
            swapchain,
            depth_buffer,
        }
    }

    pub(crate) fn draw(&self, camera: &Camera) {
        let drawable = self.begin_rendering();
        self.pipeline.draw(drawable, self.depth_buffer, camera);
        self.end_rendering(drawable);
        self.swapchain.present(
            drawable,
            self.context.graphics_queue,
            self.rendering_complete,
        );
    }

    fn begin_rendering(&self) -> Drawable {
        let device = &self.context.device;

        // Block the CPU until we're done rendering the previous frame
        unsafe {
            device
                .wait_for_fences(&[self.fence], true, u64::MAX)
                .unwrap();
            device.reset_fences(&[self.fence]).unwrap();
        }

        // Begin the command buffer
        let command_buffer = self.context.draw_command_buffer;
        unsafe {
            device
                .begin_command_buffer(command_buffer, &vk::CommandBufferBeginInfo::default())
                .unwrap()
        };

        // Transition the depth buffer into the correct state
        unsafe {
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&[
                    vk::ImageMemoryBarrier2::default()
                        .subresource_range(DEPTH_RANGE)
                        .image(self.depth_buffer.image)
                        .src_access_mask(vk::AccessFlags2::empty())
                        .src_stage_mask(vk::PipelineStageFlags2::empty())
                        .dst_access_mask(
                            vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ
                                | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        )
                        .dst_stage_mask(vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS)
                        .old_layout(vk::ImageLayout::UNDEFINED)
                        .new_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL),
                ]),
            );
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
                    vk::ImageMemoryBarrier2::default()
                        .subresource_range(FULL_IMAGE)
                        .image(swapchain_image)
                        .src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
                        .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
                        .dst_access_mask(vk::AccessFlags2::NONE)
                        .dst_stage_mask(vk::PipelineStageFlags2::BOTTOM_OF_PIPE)
                        .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .new_layout(vk::ImageLayout::PRESENT_SRC_KHR),
                ]),
            );

            // End the command buffer
            device.end_command_buffer(command_buffer).unwrap();

            // Submit the work to the queue
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
