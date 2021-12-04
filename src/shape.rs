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
}
