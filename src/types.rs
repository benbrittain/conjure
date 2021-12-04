use {nalgebra::Vector3, std::ops::Sub};

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Point { x, y, z }
    }

    #[allow(dead_code)]
    pub fn as_vector(&self) -> Vector3<f32> {
        Vector3::new(self.x, self.y, self.z)
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
}
