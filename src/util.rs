use {
    crate::types::Point,
    std::{
        collections::hash_map::DefaultHasher,
        f32::{self, consts::PI},
        hash::Hasher,
    },
};

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0
    } else if t > 1.0 {
        t -= 1.0
    }

    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [u8; 3] {
    assert!(h <= 360.0 && h >= 0.0);
    if s == 0.0 {
        let l = (l * 255.0).round() as u8;
        return [l, l, l];
    }

    let h = h / (2.0 * PI);
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - (l * s) };
    let p = 2.0 * l - q;

    [
        (hue_to_rgb(p, q, h + 1.0 / 3.0) * 255.0).round() as u8,
        (hue_to_rgb(p, q, h) * 255.0).round() as u8,
        (hue_to_rgb(p, q, h - 1.0 / 3.0) * 255.0).round() as u8,
    ]
}

/// Deterministic RGB from point location
///
/// (r, g, b) 0.0 - 1.
pub fn color_from_point(point: &Point) -> [f32; 3] {
    // Hash the x/y/z so we get something consistent but unique
    let mut hasher = DefaultHasher::new();
    for i in [point.x, point.y, point.z] {
        hasher.write_i32(i as i32);
    }
    let hash = hasher.finish();

    // squash it into 0 - 2pi
    let h = (hash as f32 / std::u64::MAX as f32) * 2.0 * PI;

    // use the truncated hue, convert to rgb color space
    hsl_to_rgb(h, 0.8, 0.48).map(|x| x as f32 / 255.0)
}
