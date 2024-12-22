use std::sync::Arc;

use ash::vk;

use super::{context::Context, pipeline::Pipeline};

pub struct Renderer {
    pub pipeline: Pipeline,
    pub context: Arc<Context>,
}

impl Renderer {
    pub(crate) fn new(context: Arc<Context>) -> Self {
        let pipeline = Pipeline::new(context.clone());

        Self { pipeline, context }
    }

    pub(crate) fn draw(&self) {
        self.begin_rendering();
        self.pipeline.draw();
        self.end_rendering();
    }

    fn begin_rendering(&self) {
        let device = &self.context.device;
        unsafe {
            device.begin_command_buffer(
                self.context.draw_command_buffer,
                &vk::CommandBufferBeginInfo::default(),
            )
        }
        .unwrap();
    }

    fn end_rendering(&self) {}
}
