use cgmath::{InnerSpace, SquareMatrix};
use winit::keyboard::KeyCode;

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    // pub dir: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0)
);

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let dir = self.target - self.eye;
        let view = cgmath::Matrix4::look_to_rh(self.eye, dir ,self.up);

        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::ArrowUp => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::ArrowDown => {
                self.is_down_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        let right = forward_norm.cross(camera.up);

        let backward = camera.eye - camera.target;

        let p = cgmath::Quaternion::from_sv(0., backward);
        let angular_step = self.speed.to_radians();
        let r = cgmath::Quaternion::from_sv(angular_step.cos(), angular_step.sin() * camera.up);
        let r_prime =
            cgmath::Quaternion::from_sv(angular_step.cos(), -angular_step.sin() * camera.up);

        let q = cgmath::Quaternion::from_sv(angular_step.cos(), angular_step.sin() * right);
        let q_prime = cgmath::Quaternion::from_sv(angular_step.cos(), -angular_step.sin() * right);

        if self.is_right_pressed {
            camera.eye = camera.target + (r * p * r_prime).v;
        }

        if self.is_left_pressed {
            camera.eye = camera.target + (r_prime * p * r).v;
        }

        if self.is_up_pressed {
            camera.eye = camera.target + (q * p * q_prime).v;
        }
        if self.is_down_pressed {
            camera.eye = camera.target + (q_prime * p * q).v;
        }

        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye = camera.target - forward_norm * (forward_mag - self.speed);
        }
        if self.is_backward_pressed {
            camera.eye = camera.target - forward_norm * (forward_mag + self.speed);
        }
    }
}
