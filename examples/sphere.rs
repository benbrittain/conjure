use conjure::{octree::Octree, shape::CsgFunc};

fn main() {
    let resolution = 1.0;
    let radius = 100.0;

    let csg_func = CsgFunc::new(Box::new(move |x, y, z| {
        (((0.0 - z) * (0.0 - z)) + ((0.0 - x) * (0.0 - x)) + ((0.0 - y) * (0.0 - y))).sqrt()
            - radius
    }));
    let mut octree = Octree::new(-128.0, 128.0);
    octree.render_shape(resolution, &csg_func);
}
