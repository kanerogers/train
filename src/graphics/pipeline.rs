use std::{f32::consts::TAU, path::Path, sync::Arc};

use ash::vk;

use super::{
    camera::Camera,
    context::Context,
    depth_buffer::{DepthBuffer, DEPTH_FORMAT},
    swapchain::Drawable,
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
                            .cull_mode(vk::CullModeFlags::BACK)
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

            self.draw_cube(device, command_buffer, camera, glam::Affine3A::from_scale_rotation_translation(
                glam::Vec3::splat(10.),
                glam::Quat::IDENTITY,
                Default::default(),
            ), [0.1, 1.0, 0.1, 1.0].into());

            self.draw_cube(device, command_buffer, camera, glam::Affine3A::from_scale_rotation_translation(
                glam::Vec3::splat(3.),
                glam::Quat::IDENTITY,
                [15.0, 0., 0.].into(),
            ), [0.1, 1.0, 0.1, 1.0].into());

            // End rendering
            device.cmd_end_rendering(command_buffer);
        }
    }

    fn draw_cube(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        camera: &Camera,
        transform: glam::Affine3A,
        colour: glam::Vec4,
    ) {
        #[rustfmt::skip]
        let transforms = [
            // TOP
            (
                [0., 0.5, 0.], 
                glam::Quat::from_rotation_x(-TAU / 4.),
            ),
            // BOTTOM
            (
                [0., -0.5, 0.],
                glam::Quat::from_rotation_x(TAU / 4.),
            ),
            // LEFT
            (
                [-0.5, 0.0, 0.], 
                glam::Quat::from_rotation_y(-TAU / 4.),
            ),
            // RIGHT
            (
                [0.5, 0.0, 0.], 
                glam::Quat::from_rotation_y(TAU / 4.),
            ),
            // FRONT
            (
                [0.0, 0.0, 0.5], 
                glam::Quat::IDENTITY
            ),
            // BACK
            (
                [0.0, 0.0, -0.5], 
                glam::Quat::from_rotation_y(TAU / 2.),
            ),
        ];

        for (translation, rotation) in transforms {
            let registers = Registers {
                ndc_from_local: camera.ndc_from_world()
                    * transform
                    * glam::Affine3A::from_rotation_translation(rotation, translation.into()),
                colour,
            };

            unsafe {
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
                device.cmd_draw(command_buffer, 6, 1, 0, 0);
            }
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
