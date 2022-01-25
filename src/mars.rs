use {
    crate::{lang::Ty, CsgFunc},
    nalgebra::DMatrix,
    std::f32::consts::PI,
};

const PIXELS_PER_DEGREE: usize = 16;
const DEGREE_PER_POINT: f32 = 1.0 / PIXELS_PER_DEGREE as f32; // 0.0625 degrees
const LINES: usize = 2880;
const LINE_SAMPLES: usize = 5760;

#[derive(Debug)]
pub struct Polar {
    latitude: f32,
    longitude: f32,
    height: f32,
}

#[derive(Debug)]
pub struct Cartesian {
    x: f32,
    y: f32,
    z: f32,
}

impl Cartesian {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Cartesian { x, y, z }
    }
}

impl Polar {
    pub fn new(latitude: f32, longitude: f32, height: f32) -> Self {
        // Safety checks to ensure our coordinate system is set up properly.
        // The standard trigonometry functions assume radians.
        assert!(latitude >= -90.0);
        assert!(latitude <= 90.0);
        assert!(longitude >= 0.0);
        assert!(longitude <= 360.0);
        Polar { latitude, longitude, height }
    }
}

impl From<Cartesian> for Polar {
    fn from(coord: Cartesian) -> Polar {
        let r = ((coord.x * coord.x) + (coord.y * coord.y) + (coord.z * coord.z)).sqrt();
        let phi = (coord.z / r).acos();
        let mut theta: f32 = (coord.y / coord.x).atan();
        if coord.x < 0.0 && coord.y >= 0.0 && theta == 0.0 {
            theta = PI;
        } else if coord.x < 0.0 && coord.y < 0.0 && theta.signum() > 0.0 {
            theta += PI;
        } else if coord.x > 0.0 && coord.y < 0.0 && theta.signum() < 0.0 {
            theta += 2.0 * PI;
        } else if coord.x < 0.0 && coord.y > 0.0 && theta.signum() < 0.0 {
            theta += PI;
        }
        Polar::new(phi.to_degrees() - 90.0, theta.to_degrees(), r)
    }
}

impl From<Polar> for Cartesian {
    fn from(coord: Polar) -> Cartesian {
        let phi = (coord.latitude + 90.0).to_radians();
        //let phi = (coord.latitude).to_radians();
        let theta = coord.longitude.to_radians();
        let h = coord.height; // + MARS_RADIUS as f64;

        let x = h * theta.cos() * phi.sin();
        let y = h * theta.sin() * phi.sin();
        let z = h * phi.cos();
        Cartesian { x, y, z }
    }
}

type TerrainData = DMatrix<i16>;
pub fn mars_func() -> CsgFunc {
    // Read the file directly into a buffer.
    let fin = std::fs::read("megt90n000eb.img").unwrap();

    // Read the file two bytes at a time, converting the big endian values into host endian.
    let raw_data: Vec<i16> =
        fin.chunks_exact(2).into_iter().map(|x| i16::from_be_bytes([x[0], x[1]])).collect();

    let mars_data: TerrainData = TerrainData::from_row_slice(LINES, LINE_SAMPLES, &raw_data);
    CsgFunc::new(Box::new(move |x: f32, y: f32, z: f32| {
        let latlng: Polar = Cartesian::new(x, y, z).into();
        0.0
        //let r = ((x * x) + (y * y) + (z * z)).sqrt();
        //let theta = y.atan2(x);
        //let phi = (z / r).acos();
        //let radius = 33960.0;
        //let lng_idx = (theta / DEGREE_PER_POINT) as usize;
        //let lat_idx = (phi / DEGREE_PER_POINT) as usize;
        //let r: f32 =
        //    (((0.0 - z) * (0.0 - z)) + ((0.0 - x) * (0.0 - x)) + ((0.0 - y) * (0.0 - y))).sqrt();
        //let m = mars_data[(LINES - lat_idx - 1, lng_idx)] as f32;
        //r - ((m * 2.0) + radius)
    }))
}
