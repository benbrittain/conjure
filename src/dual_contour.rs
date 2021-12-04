use {
    crate::{octree::OctAxis, types::Point, ShapeFunc},
    nalgebra::{linalg::SVD, MatrixXx1, MatrixXx3, RowVector1, RowVector3, Vector3},
};

/*
 *  Our coordinate system
 *
 *        ^        ___________________
 *        |       /|       / |       /|
 *        |      / |  4   /  |  5   / |
 *        |     /________/___|_____/  |
 * y-axis |    /   |    / |  |    /|  |
 *        |   /    |   /  |  |   / |  |
 *        |   |----|--+---|--/--/  |  |
 *        |   |    |_ |___|_/___|__|__|      ^
 *        |   |   /|0 |   |/ 1  |  |/ |     / z-axis
 *        |   |  /----|---+-----|--/|7|    /
 *        |   | /  |  |6 /|     | / | |   /
 *        |   |____/__|_/_|_____|/  |/   /
 *        |   |   /   |   |     |   /   /
 *        |   |  /2   |  /  3   |  /   /
 *        |   | /     | /       | /   /
 *        |   |/______|/________|/   /
 *
 *        -------------------------->
 *                x-axis
 *
 *      0 - 3 in the front set of boxes
 *      4 - 7 in the back
 */

/// Locate where the isosurface intersects the line between `p1` and `p2`.
///
/// To effectivly use this, one axis of the cube should be held constant
fn find_point_on_edge(p1: Point, p2: Point, func: ShapeFunc) -> Option<Point> {
    if func.call_point(p1) > func.call_point(p2) {
        // if p1 is bigger than 0, swap the direction of the points
        return find_point_on_edge(p2, p1, func);
    }
    // // If there isn't a sign change, we don't have any point on this edge
    if func.call_point(p1) < 0.0 && func.call_point(p2) < 0.0 {
        return None;
    }
    if func.call_point(p1) >= 0.0 && func.call_point(p2) >= 0.0 {
        return None;
    }
    // Binary search along the side
    let mut f = 0.5;
    let mut step = 0.25;
    let mut x_val = p1.x;
    let mut y_val = p1.y;
    let mut z_val = p1.z;
    for _ in 0..10 {
        if p1.x != p2.x && p1.y == p2.y && p1.z == p2.z {
            x_val = p1.x + (p2.x - p1.x) * f;
        } else if p1.y != p2.y && p1.x == p2.x && p1.z == p2.z {
            y_val = p1.y + (p2.y - p1.y) * f;
        } else if p1.z != p2.z && p1.x == p2.x && p1.y == p2.y {
            z_val = p1.z + (p2.z - p1.z) * f;
        } else {
            unreachable!();
        }
        if func.call(x_val, y_val, z_val) < 0.0 {
            f += step;
        } else {
            f -= step;
        }
        step /= 2.0;
    }

    Some(Point::new(x_val, y_val, z_val))
}

/// Find a point in the cell that minimizes the error from the normals
pub fn new_feature(
    x_axis: OctAxis,
    y_axis: OctAxis,
    z_axis: OctAxis,
    shape_func: ShapeFunc,
) -> Option<Point> {
    let mut points: Vec<Point> = vec![];

    for edge in [
        // front left vertical
        ((x_axis.upper, y_axis.lower, z_axis.lower), (x_axis.upper, y_axis.upper, z_axis.lower)),
        // front right vertical
        ((x_axis.lower, y_axis.lower, z_axis.lower), (x_axis.lower, y_axis.upper, z_axis.lower)),
        // front bottom horizontal
        ((x_axis.lower, y_axis.lower, z_axis.lower), (x_axis.upper, y_axis.lower, z_axis.lower)),
        // front top horizontal
        ((x_axis.lower, y_axis.upper, z_axis.lower), (x_axis.upper, y_axis.upper, z_axis.lower)),
        // back left vertical
        ((x_axis.upper, y_axis.lower, z_axis.upper), (x_axis.upper, y_axis.upper, z_axis.upper)),
        // back right vertical
        ((x_axis.lower, y_axis.lower, z_axis.upper), (x_axis.lower, y_axis.upper, z_axis.upper)),
        // back bottom horizontal
        ((x_axis.lower, y_axis.lower, z_axis.upper), (x_axis.upper, y_axis.lower, z_axis.upper)),
        // back top horizontal
        ((x_axis.lower, y_axis.upper, z_axis.upper), (x_axis.upper, y_axis.upper, z_axis.upper)),
        // right top side
        ((x_axis.upper, y_axis.upper, z_axis.lower), (x_axis.upper, y_axis.upper, z_axis.upper)),
        // left top side
        ((x_axis.lower, y_axis.upper, z_axis.lower), (x_axis.lower, y_axis.upper, z_axis.upper)),
        // left bottom side
        ((x_axis.lower, y_axis.lower, z_axis.lower), (x_axis.lower, y_axis.lower, z_axis.upper)),
        // right bottom side
        ((x_axis.upper, y_axis.lower, z_axis.lower), (x_axis.upper, y_axis.lower, z_axis.upper)),
    ] {
        if let Some(p) = find_point_on_edge(
            Point::new(edge.0 .0, edge.0 .1, edge.0 .2),
            Point::new(edge.1 .0, edge.1 .1, edge.1 .2),
            shape_func,
        ) {
            points.push(p);
        }
    }

    for p in &points {
        if p.x > x_axis.upper || p.x < x_axis.lower {
            panic!("x: {:?} not within {:?}", p, x_axis);
        }
        if p.z > z_axis.upper || p.z < z_axis.lower {
            panic!("z: {:?} not within {:?}", p, z_axis);
        }
        if p.y > y_axis.upper || p.y < y_axis.lower {
            panic!("y: {:?} not within {:?}", p, y_axis);
        }
    }

    let mean_x = points.iter().fold(0.0, |accum, p| p.x + accum) / points.len() as f32;
    let mean_y = points.iter().fold(0.0, |accum, p| p.y + accum) / points.len() as f32;
    let mean_z = points.iter().fold(0.0, |accum, p| p.z + accum) / points.len() as f32;

    if points.len() >= 2 {
        let mut normals: Vec<Vector3<f32>> =
            points.iter().map(|p| shape_func.normal(p.x, p.y, p.z)).collect();

        // Add some weak bias to keeping the point within the cell
        let strength = 0.12;
        points.push(Point::new(mean_x, mean_y, mean_z));
        points.push(Point::new(mean_x, mean_y, mean_z));
        points.push(Point::new(mean_x, mean_y, mean_z));
        normals.push(Vector3::new(strength, 0.0, 0.0));
        normals.push(Vector3::new(0.0, strength, 0.0));
        normals.push(Vector3::new(0.0, 0.0, strength));

        let mut rows = vec![];
        for n in &normals {
            rows.push(RowVector3::new(n.x, n.y, n.z));
        }
        let a = MatrixXx3::from_rows(&rows);

        let mut b_rows = vec![];
        for (p, norm) in points.iter().zip(normals.iter()) {
            b_rows.push(RowVector1::new((p.x * norm.x) + (p.y * norm.y) + (p.z * norm.z)));
        }

        let b = MatrixXx1::from_rows(&b_rows);
        let svd = SVD::new(a, true, true);
        let solution = svd.solve(&b, 0.0).unwrap();

        let x = solution.data.0[0][0];
        let y = solution.data.0[0][1];
        let z = solution.data.0[0][2];

        Some(Point::new(x, y, z))
    } else {
        None
    }
}
