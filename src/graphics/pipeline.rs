use std::{path::Path, sync::Arc};

use ash::vk;

use super::{
    camera::Camera,
    context::Context,
    depth_buffer::{DepthBuffer, DEPTH_FORMAT},
    swapchain::Drawable,
    FULL_IMAGE,
};

pub struct Pipeline {
    handle: vk::Pipeline,
    layout: vk::PipelineLayout,
    context: Arc<Context>,
}

impl Pipeline {
    pub fn new(context: Arc<Context>, format: vk::Format) -> Self {
        let device = &context.device;

        let layout = unsafe {
            device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default().push_constant_ranges(&[
                    vk::PushConstantRange::default()
                        .size(std::mem::size_of::<Registers>() as u32)
                        .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT),
                ]),
                None,
            )
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
                    .input_assembly_state(
                        &vk::PipelineInputAssemblyStateCreateInfo::default()
                            .topology(vk::PrimitiveTopology::TRIANGLE_LIST),
                    )
                    .viewport_state(
                        &vk::PipelineViewportStateCreateInfo::default()
                            .scissor_count(1)
                            .viewport_count(1),
                    )
                    .dynamic_state(
                        &vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&[
                            vk::DynamicState::SCISSOR,
                            vk::DynamicState::VIEWPORT,
                        ]),
                    )
                    .rasterization_state(
                        &vk::PipelineRasterizationStateCreateInfo::default()
                            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                            .cull_mode(vk::CullModeFlags::NONE)
                            .polygon_mode(vk::PolygonMode::FILL)
                            .line_width(1.0),
                    )
                    .depth_stencil_state(
                        &vk::PipelineDepthStencilStateCreateInfo::default()
                            .depth_write_enable(true)
                            .depth_test_enable(true)
                            .depth_compare_op(vk::CompareOp::GREATER_OR_EQUAL)
                            .stencil_test_enable(false)
                            .depth_bounds_test_enable(false),
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
                    .push_next(
                        &mut vk::PipelineRenderingCreateInfo::default()
                            .depth_attachment_format(DEPTH_FORMAT)
                            .color_attachment_formats(&[format]),
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

    pub(crate) fn draw(&self, drawable: Drawable, depth_buffer: DepthBuffer, camera: &Camera) {
        let device = &self.context.device;
        let command_buffer = self.context.draw_command_buffer;
        let render_area = drawable.extent;
        unsafe {
            // First, transition the color attachment into the present state
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&[
                    vk::ImageMemoryBarrier2::default()
                        .subresource_range(FULL_IMAGE)
                        .image(drawable.image)
                        .src_access_mask(vk::AccessFlags2::NONE)
                        .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
                        .dst_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
                        .dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
                        .old_layout(vk::ImageLayout::UNDEFINED)
                        .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
                ]),
            );

            // Next, bind the pipeline and set the dynamic state
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.handle);
            device.cmd_set_scissor(command_buffer, 0, &[render_area.into()]);
            device.cmd_set_viewport(
                command_buffer,
                0,
                &[vk::Viewport::default()
                    .width(render_area.width as _)
                    .height(render_area.height as _)],
            );

            // Begin rendering
            device.cmd_begin_rendering(
                command_buffer,
                &vk::RenderingInfo::default()
                    .render_area(render_area.into())
                    .layer_count(1)
                    .depth_attachment(
                        &vk::RenderingAttachmentInfo::default()
                            .image_view(depth_buffer.view)
                            .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                            .load_op(vk::AttachmentLoadOp::CLEAR)
                            .store_op(vk::AttachmentStoreOp::DONT_CARE)
                            .clear_value(vk::ClearValue {
                                depth_stencil: vk::ClearDepthStencilValue {
                                    depth: 0.0,
                                    stencil: 0,
                                },
                            }),
                    )
                    .color_attachments(&[vk::RenderingAttachmentInfo::default()
                        .image_view(drawable.view)
                        .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .load_op(vk::AttachmentLoadOp::CLEAR)
                        .store_op(vk::AttachmentStoreOp::STORE)
                        .clear_value(vk::ClearValue {
                            color: vk::ClearColorValue {
                                float32: [0.1, 0.2, 1.0, 1.0],
                            },
                        })]),
            );

            let transform = glam::Affine3A::from_rotation_translation(
                glam::Quat::from_rotation_x(90_f32.to_radians()),
                Default::default(),
            );

            let registers = Registers {
                ndc_from_local: camera.ndc_from_world() * transform,
                colour: [0.1, 1.0, 0.1, 1.0].into(),
            };

            // Push constant!
            device.cmd_push_constants(
                command_buffer,
                self.layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                &std::slice::from_raw_parts(
                    &registers as *const _ as *const u8,
                    std::mem::size_of::<Registers>(),
                ),
            );

            // Issue our draw commands
            device.cmd_draw(command_buffer, 6, 1, 0, 0);

            // End rendering
            device.cmd_end_rendering(command_buffer);
        }
    }
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

#[repr(C)]
#[derive(Debug, Clone)]
struct Registers {
    ndc_from_local: glam::Mat4,
    colour: glam::Vec4,
}
