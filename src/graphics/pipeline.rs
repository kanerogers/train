use std::{path::Path, sync::Arc};

use ash::vk;

use super::context::Context;

pub struct Pipeline {
    handle: vk::Pipeline,
    layout: vk::PipelineLayout,
    context: Arc<Context>,
}

impl Pipeline {
    pub fn new(context: Arc<Context>) -> Self {
        let device = &context.device;

        let layout = unsafe {
            device.create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default(), None)
        }
        .unwrap();

        let handle = unsafe {
            device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[vk::GraphicsPipelineCreateInfo::default()
                    .stages(&[
                        vk::PipelineShaderStageCreateInfo::default()
                            .name(c"main")
                            .module(load_module("triangle.vertex.spv", &context))
                            .stage(vk::ShaderStageFlags::VERTEX),
                        vk::PipelineShaderStageCreateInfo::default()
                            .name(c"main")
                            .module(load_module("triangle.fragment.spv", &context))
                            .stage(vk::ShaderStageFlags::FRAGMENT),
                    ])
                    .vertex_input_state(&vk::PipelineVertexInputStateCreateInfo::default())
                    .dynamic_state(
                        &vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&[
                            vk::DynamicState::SCISSOR,
                            vk::DynamicState::VIEWPORT,
                        ]),
                    )
                    .rasterization_state(
                        &vk::PipelineRasterizationStateCreateInfo::default()
                            .cull_mode(vk::CullModeFlags::BACK)
                            .polygon_mode(vk::PolygonMode::FILL)
                            .line_width(1.0),
                    )
                    .input_assembly_state(
                        &vk::PipelineInputAssemblyStateCreateInfo::default()
                            .topology(vk::PrimitiveTopology::TRIANGLE_LIST),
                    )
                    .color_blend_state(
                        &vk::PipelineColorBlendStateCreateInfo::default().attachments(&[
                            vk::PipelineColorBlendAttachmentState::default()
                                .blend_enable(false)
                                .color_write_mask(vk::ColorComponentFlags::RGBA),
                        ]),
                    )
                    .multisample_state(
                        &vk::PipelineMultisampleStateCreateInfo::default()
                            .rasterization_samples(vk::SampleCountFlags::TYPE_1),
                    )
                    .layout(layout)
                    .render_pass(vk::RenderPass::null())
                    .push_next(
                        &mut vk::PipelineRenderingCreateInfo::default()
                            .color_attachment_formats(&[context.swapchain.format]),
                    )],
                None,
            )
        }
        .unwrap()[0];

        Self {
            context,
            layout,
            handle,
        }
    }

    pub(crate) fn draw(&self) {}
}

fn load_module(path: &str, context: &Context) -> vk::ShaderModule {
    let path = Path::new("assets/shaders/").join(path);
    let mut file = std::fs::File::open(path).unwrap();
    let words = ash::util::read_spv(&mut file).unwrap();

    unsafe {
        context
            .device
            .create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&words), None)
    }
    .unwrap()
}
