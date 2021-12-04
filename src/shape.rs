use {crate::types::Point, nalgebra::Vector3};

#[derive(Copy, Clone)]
pub struct ShapeFunc {
    func: fn(f32, f32, f32) -> f32,
}

impl ShapeFunc {
    pub fn new(func: fn(f32, f32, f32) -> f32) -> Self {
        ShapeFunc {
            func,
            // TODO normalized function
        }
    }

    pub fn call(&self, x: f32, y: f32, z: f32) -> f32 {
        (self.func)(x, y, z)
    }

    pub fn call_point(&self, p: Point) -> f32 {
        (self.func)(p.x, p.y, p.z)
    }

    pub fn normal(&self, x: f32, y: f32, z: f32) -> Vector3<f32> {
        (normal(self.func))(x, y, z)
    }
}

fn normal(func: fn(f32, f32, f32) -> f32) -> (impl Fn(f32, f32, f32) -> Vector3<f32> + Copy) {
    move |x, y, z| {
        Vector3::new(
            func(x + 0.001, y, z) - func(x - 0.001, y, z),
            func(x, y + 0.001, z) - func(x, y - 0.001, z),
            func(x, y, z + 0.001) - func(x, y, z - 0.001),
        )
        .normalize()
    }
}
