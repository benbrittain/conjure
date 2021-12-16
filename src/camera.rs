use {
    cgmath::{perspective, InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Vector3},
    std::{f32::consts::FRAC_PI_2, time::Duration},
    winit::{
        dpi::PhysicalPosition,
        event::{ElementState, MouseScrollDelta, VirtualKeyCode},
    },
};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self { view_position: [0.0; 4], view_proj: Matrix4::identity().into() }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into()
    }
}

#[derive(Debug)]
pub struct Camera {
    position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self { position: position.into(), yaw: yaw.into(), pitch: pitch.into() }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self { aspect: width as f32 / height as f32, fovy: fovy.into(), znear, zfar }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        // make left handed perspective matrix?
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[derive(Debug)]
pub struct CameraController {
    x_axis_delta: f32,
    y_axis_delta: f32,
    x_axis_rotation: f32,
    y_axis_rotation: f32,
    scroll: f32,
    zoom_speed: f32,
    speed: f32,
    sensitivity: f32,
    reset: bool,
}

impl CameraController {
    pub fn new(zoom_speed: f32, speed: f32, sensitivity: f32) -> Self {
        Self {
            x_axis_delta: 0.0,
            y_axis_delta: 0.0,
            x_axis_rotation: 0.0,
            y_axis_rotation: 0.0,
            scroll: 0.0,
            zoom_speed,
            speed,
            sensitivity,
            reset: false,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, _state: ElementState) -> bool {
        match key {
            VirtualKeyCode::Space => {
                self.reset = true;
                true
            }
            _ => false,
        }
    }

    pub fn process_left_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.x_axis_rotation = -mouse_dx as f32;
        self.y_axis_rotation = -mouse_dy as f32;
    }

    pub fn process_middle_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.y_axis_delta = mouse_dy as f32;
        self.x_axis_delta = -mouse_dx as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) -> bool {
        self.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => *scroll,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
        true
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();

        // Camera relative vectors
        let x_vector = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        let y_vector = Vector3::new(-pitch_sin, pitch_cos, 0.0).normalize();
        let z_vector =
            Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();

        // Move the position based on camera
        camera.position += z_vector * self.scroll * self.zoom_speed * self.sensitivity * dt;
        camera.position += x_vector * self.x_axis_delta * self.speed * dt;
        camera.position += y_vector * self.y_axis_delta * self.speed * dt;

        // Camera Rotation
        camera.yaw += Rad(self.x_axis_rotation) * self.sensitivity * dt;
        camera.pitch += Rad(-self.y_axis_rotation) * self.sensitivity * dt;

        // zero everything in case update_camera is not called on the next render loop
        self.scroll = 0.0;
        self.x_axis_rotation = 0.0;
        self.y_axis_rotation = 0.0;
        self.x_axis_delta = 0.0;
        self.y_axis_delta = 0.0;

        if camera.pitch < -Rad(FRAC_PI_2) {
            camera.pitch = -Rad(FRAC_PI_2);
        } else if camera.pitch > Rad(FRAC_PI_2) {
            camera.pitch = Rad(FRAC_PI_2);
        }

        if self.reset {
            self.reset = false;
            camera.yaw = cgmath::Deg(-90.0).into();
            camera.pitch = cgmath::Deg(0.0).into();
        }
    }
}
