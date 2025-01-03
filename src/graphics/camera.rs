use ash::vk;
use dolly::prelude::{CameraRig, Position, Smooth, YawPitch};
use glam::{Quat, Vec3};

use crate::input::Input;

#[derive(Debug)]
pub struct Camera {
    rig: CameraRig,
    pub movement_left: f32,
    pub movement_right: f32,
    pub movement_up: f32,
    pub movement_down: f32,
    pub movement_forward: f32,
    pub movement_backward: f32,
    pub boost: f32,
    pub extent: vk::Extent2D,
}

impl Camera {
    pub fn new(extent: vk::Extent2D) -> Camera {
        let initial_position = glam::Vec3::new(0., 10.0, 10.);
        Camera {
            rig: CameraRig::builder()
                .with(Position::new(initial_position))
                .with(YawPitch::new())
                .with(Smooth::new_position_rotation(1.0, 1.0))
                .build(),
            movement_left: 0.,
            movement_right: 0.,
            movement_up: 0.,
            movement_down: 0.,
            movement_forward: 0.,
            movement_backward: 0.,
            boost: 0.,
            extent,
        }
    }

    pub(crate) fn update(&mut self, dt: f32, input: &Input) {
        self.rig
            .driver_mut::<YawPitch>()
            .rotate_yaw_pitch(input.yaw_degrees, input.pitch_degrees);

        // simple fly-cam impl
        let move_vec = Quat::from(self.rig.final_transform.rotation)
            * input.get_movement().normalize_or_zero()
            * 10.0f32.powf(self.boost);

        self.rig
            .driver_mut::<Position>()
            .translate(move_vec * dt * 10.);
        self.rig.update(dt);
    }

    pub(crate) fn ndc_from_world(&self) -> glam::Mat4 {
        // Get the transform of the camera rig
        let (translation, rotation) = self.rig.final_transform.into_position_rotation();

        // Build up the perspective matrix
        let aspect_ratio = self.extent.width as f32 / self.extent.height as f32;
        let mut perspective =
            glam::Mat4::perspective_infinite_reverse_rh(60_f32.to_radians(), aspect_ratio, 0.01);

        // adjust for wulkan
        perspective.y_axis *= -1.0;

        // Get view_from_world
        let world_from_view = glam::Affine3A::from_rotation_translation(rotation, translation);
        let view_from_world = world_from_view.inverse();

        // Combine the matrices
        perspective * view_from_world
    }
}
