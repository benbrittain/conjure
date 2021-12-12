use {crate::types::Point, nalgebra::Vector3, std::sync::Arc};

#[derive(Clone)]
pub struct CsgFunc {
    func: Arc<Box<dyn Fn(f32, f32, f32) -> f32>>,
}

impl CsgFunc {
    pub fn new(func: Box<(dyn Fn(f32, f32, f32) -> f32 + Send + Sync)>) -> Self {
        CsgFunc { func: Arc::new(func) }
    }

    pub fn call(&self, x: f32, y: f32, z: f32) -> f32 {
        (self.func)(x, y, z)
    }

    pub fn call_point(&self, p: Point) -> f32 {
        (self.func)(p.x, p.y, p.z)
    }

    pub fn normal(&self, x: f32, y: f32, z: f32) -> Vector3<f32> {
        Vector3::new(
            self.call(x + 0.001, y, z) - self.call(x - 0.001, y, z),
            self.call(x, y + 0.001, z) - self.call(x, y - 0.001, z),
            self.call(x, y, z + 0.001) - self.call(x, y, z - 0.001),
        )
        .normalize()
    }
}
