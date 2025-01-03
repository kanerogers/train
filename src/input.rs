use winit::{
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Debug, Default, Clone)]
pub struct Input {
    left: f32,
    right: f32,
    up: f32,
    down: f32,
    forward: f32,
    back: f32,
    pub pitch_degrees: f32,
    pub yaw_degrees: f32,
}

impl Input {
    pub fn reset(&mut self) {
        *self = Input::default();
    }

    pub fn handle_keyboard_event(&mut self, event: winit::event::KeyEvent) {
        let KeyEvent {
            physical_key: PhysicalKey::Code(key_code),
            ..
        } = event
        else {
            return;
        };

        match key_code {
            KeyCode::KeyW => self.forward += 1.,
            KeyCode::KeyS => self.back += 1.,
            KeyCode::KeyA => self.left += 1.,
            KeyCode::KeyD => self.right += 1.,
            KeyCode::Space => self.up += 1.,
            KeyCode::ControlLeft => self.down += 1.,
            _ => {}
        }
    }
    pub fn handle_mouse_motion(&mut self, x: f64, y: f64) {
        if x == 0. && y == 0. {
            return;
        }

        println!("yaw: {x}, pitch: {y}");

        self.pitch_degrees -= y as f32;
        self.yaw_degrees -= x as f32;
    }

    // Returns a normalised (or zero) vector of the movement for this frame
    pub fn get_movement(&self) -> glam::Vec3 {
        glam::Vec3::new(
            self.right - self.left,
            self.up - self.down,
            self.back - self.forward,
        )
        .normalize_or_zero()
    }
}
