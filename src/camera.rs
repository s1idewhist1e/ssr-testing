use cgmath::{Deg, InnerSpace, Quaternion, Rotation3, SquareMatrix};
use winit::keyboard::KeyCode;

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub view: cgmath::Vector3<f32>,
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
        let view = cgmath::Matrix4::look_to_rh(self.eye, self.view, self.up);

        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
    fn build_inv_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        self.build_view_projection_matrix().invert().unwrap()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view_proj_inv: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            view_proj_inv: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        let view_proj = camera.build_view_projection_matrix();
        self.view_proj_inv = view_proj
            .invert()
            .unwrap_or(cgmath::Matrix4::identity())
            .into();
        self.view_proj = view_proj.into();
    }
}

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_arrowup_pressed: bool,
    is_arrowdown_pressed: bool,
    is_right_pressed: bool,
    is_arrowleft_pressed: bool,
    is_arrowright_pressed: bool,
    is_down_pressed: bool,
    is_up_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_arrowup_pressed: false,
            is_arrowdown_pressed: false,
            is_arrowleft_pressed: false,
            is_arrowright_pressed: false,
            is_down_pressed: false,
            is_up_pressed: false,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::ArrowUp => {
                self.is_arrowup_pressed = is_pressed;
                true
            }
            KeyCode::ArrowDown => {
                self.is_arrowdown_pressed = is_pressed;
                true
            }
            KeyCode::ArrowLeft => {
                self.is_arrowleft_pressed = is_pressed;
                true
            }
            KeyCode::ArrowRight => {
                self.is_arrowright_pressed = is_pressed;
                true
            }
            KeyCode::KeyR => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::KeyF => {
                self.is_down_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.view.normalize();

        let up = camera.up.normalize();

        let right = forward.cross(up).normalize();

        if self.is_forward_pressed {
            camera.eye += forward * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward * self.speed;
        }
        if self.is_left_pressed {
            camera.eye -= right * self.speed;
        }
        if self.is_right_pressed {
            camera.eye += right * self.speed;
        }
        if self.is_up_pressed {
            camera.eye += up * self.speed;
        }
        if self.is_down_pressed {
            camera.eye -= up * self.speed;
        }

        if self.is_arrowdown_pressed {
            let q = cgmath::Quaternion::from_axis_angle(right, Deg(self.speed));
            camera.view = (q * Quaternion::from_sv(0., camera.view) * q.conjugate()).v;
        }
        if self.is_arrowup_pressed {
            let q = cgmath::Quaternion::from_axis_angle(right, -Deg(self.speed));
            camera.view = (q * Quaternion::from_sv(0., camera.view) * q.conjugate()).v;
        }
        if self.is_arrowleft_pressed {
            let q = cgmath::Quaternion::from_axis_angle(up, Deg(self.speed));
            camera.view = (q * Quaternion::from_sv(0., camera.view) * q.conjugate()).v;
        }
        if self.is_arrowright_pressed {
            let q = cgmath::Quaternion::from_axis_angle(up, -Deg(self.speed));
            camera.view = (q * Quaternion::from_sv(0., camera.view) * q.conjugate()).v;
        }
    }
}
