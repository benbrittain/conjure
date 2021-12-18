use {
    crate::{
        octree::{OctAxis, OctantIdx, Octree},
        types::{Face, Point},
        CsgFunc,
    },
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
fn find_point_on_edge(p1: Point, p2: Point, func: &CsgFunc) -> Option<Point> {
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
    shape_func: CsgFunc,
) -> Option<Point> {

    let points: smallvec::SmallVec<[Point; 12]> = [
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
    ].iter().filter_map(|(p0, p1)| {
        find_point_on_edge(Point::new(p0.0, p0.1, p0.2), Point::new(p1.0, p1.1, p1.2), &shape_func)
    }).collect();


    if points.len() >= 2 {
        let normals: Vec<Vector3<f32>> =
            points.iter().map(|p| shape_func.normal(p.x, p.y, p.z)).collect();

        // TODO consider adjusting the bias again around here
        //
        // I'm not usually a big fan of leaving code commented out in a section, but ya know
        // it's not a bad reminder and I'm not using a bug tracker :)
        //
        // ... yet
        //
        // let mean_x = points.iter().fold(0.0, |accum, p| p.x + accum) / points.len() as f32;
        // let mean_y = points.iter().fold(0.0, |accum, p| p.y + accum) / points.len() as f32;
        // let mean_z = points.iter().fold(0.0, |accum, p| p.z + accum) / points.len() as f32;
        // Add some weak bias to keeping the point within the cell
        // let strength = 0.12;
        // points.push(Point::new(mean_x, mean_y, mean_z));
        // points.push(Point::new(mean_x, mean_y, mean_z));
        // points.push(Point::new(mean_x, mean_y, mean_z));
        // normals.push(Vector3::new(strength, 0.0, 0.0));
        // normals.push(Vector3::new(0.0, strength, 0.0));
        // normals.push(Vector3::new(0.0, 0.0, strength));

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

#[derive(PartialEq, Copy, Clone)]
enum TreeAxis {
    X = 0,
    Y = 1,
    Z = 2,
}

static CELL_FACE_MAP: [(usize, usize, TreeAxis); 12] = [
    (4, 5, TreeAxis::X),
    (0, 1, TreeAxis::X),
    (2, 3, TreeAxis::X),
    (6, 7, TreeAxis::X),
    (0, 2, TreeAxis::Y),
    (1, 3, TreeAxis::Y),
    (5, 7, TreeAxis::Y),
    (4, 6, TreeAxis::Y),
    (0, 4, TreeAxis::Z),
    (1, 5, TreeAxis::Z),
    (3, 7, TreeAxis::Z),
    (2, 6, TreeAxis::Z),
];

static CELL_EDGE_MAP: [(usize, usize, usize, usize, TreeAxis); 6] = [
    (1, 5, 3, 7, TreeAxis::X),
    (0, 4, 2, 6, TreeAxis::X),
    (4, 5, 0, 1, TreeAxis::Y),
    (6, 7, 2, 3, TreeAxis::Y),
    (0, 1, 2, 3, TreeAxis::Z),
    (4, 5, 6, 7, TreeAxis::Z),
];

static CELL_MAP: [[usize; 4]; 3] = [[0, 1, 0, 1], [0, 0, 1, 1], [1, 1, 0, 0]];

type FaceInternal = (usize, usize, usize, usize, TreeAxis, usize);
static FACE_EDGE_MAP: [[FaceInternal; 4]; 3] = [
    // TreeAxis::X
    [
        (5, 4, 1, 0, TreeAxis::Y, 0),
        (7, 6, 3, 2, TreeAxis::Y, 0),
        (1, 0, 3, 2, TreeAxis::Z, 0),
        (5, 4, 7, 6, TreeAxis::Z, 0),
    ],
    // TreeAxis::Y
    [
        (2, 3, 0, 1, TreeAxis::Z, 1),
        (6, 7, 4, 5, TreeAxis::Z, 1),
        (3, 7, 1, 5, TreeAxis::X, 1),
        (2, 6, 0, 4, TreeAxis::X, 1),
    ],
    // TreeAxis::Z
    [
        (0, 1, 4, 5, TreeAxis::Y, 2),
        (2, 3, 6, 7, TreeAxis::Y, 2),
        (5, 1, 7, 3, TreeAxis::X, 0),
        (4, 0, 6, 2, TreeAxis::X, 0),
    ],
];

static EDGE_EDGE_MAP: [[[usize; 4]; 2]; 3] = [
    // TreeAxis::X
    [[7, 3, 5, 1], [6, 2, 4, 0]],
    // TreeAxis::Y
    [[1, 0, 5, 4], [3, 2, 7, 6]],
    // TreeAxis::Z
    [[3, 2, 1, 0], [7, 6, 5, 4]],
];

static FACE_FACE_MAP: [[(usize, usize); 4]; 3] = [
    // TreeAxis::X
    [(5, 4), (7, 6), (1, 0), (3, 2)],
    // TreeAxis::Y
    [(7, 5), (6, 4), (3, 1), (2, 0)],
    // TreeAxis::Z
    [(5, 1), (4, 0), (7, 3), (6, 2)],
];

/// Entry point to the dual contour face extraction
pub fn cell_proc(tree: &Octree, idx: OctantIdx) -> Vec<Face> {
    if tree.get_octant(idx).is_leaf() {
        return vec![];
    }

    let mut faces = vec![];
    // Since it has children, spawn 8 calls to cell_proc
    for child_idx in &tree.get_octant(idx).children.unwrap() {
        faces.extend(cell_proc(tree, *child_idx));
    }
    let children = tree.get_octant(idx).children.unwrap();

    // call face_proc on every set of two cells that share a face
    for (c0, c1, dir) in &CELL_FACE_MAP {
        faces.extend(face_proc(tree, *dir, [children[*c0], children[*c1]]));
    }

    // call edge_proc on every set of four subcells that share an edge
    for (c0, c1, c2, c3, dir) in &CELL_EDGE_MAP {
        faces.extend(edge_proc(
            tree,
            *dir,
            [children[*c0], children[*c1], children[*c2], children[*c3]],
        ));
    }

    faces
}

/// Recursive function that extracts faces from two octants sharing a common face.
///
/// Calls itself 4 times with every pair of cells in direction `dir` that share a face.
fn face_proc(tree: &Octree, dir: TreeAxis, cells: [OctantIdx; 2]) -> Vec<Face> {
    let mut faces = vec![];
    if tree.get_octant(cells[0]).children.is_some() || tree.get_octant(cells[1]).children.is_some()
    {
        for (c0, c1) in FACE_FACE_MAP[dir as usize].iter() {
            let o0 = match tree.get_octant(cells[0]).children {
                Some(children) => children[*c0],
                None => cells[0],
            };
            let o1 = match tree.get_octant(cells[1]).children {
                Some(children) => children[*c1],
                None => cells[1],
            };
            faces.extend(face_proc(tree, dir, [o0, o1]));
        }

        for (c0, c1, c2, c3, edge_dir, order) in FACE_EDGE_MAP[dir as usize].iter() {
            let o0 = match tree.get_octant(cells[CELL_MAP[*order][0]]).children {
                Some(child) => child[*c0],
                None => cells[CELL_MAP[*order][0]],
            };
            let o1 = match tree.get_octant(cells[CELL_MAP[*order][1]]).children {
                Some(child) => child[*c1],
                None => cells[CELL_MAP[*order][1]],
            };
            let o2 = match tree.get_octant(cells[CELL_MAP[*order][2]]).children {
                Some(child) => child[*c2],
                None => cells[CELL_MAP[*order][2]],
            };
            let o3 = match tree.get_octant(cells[CELL_MAP[*order][3]]).children {
                Some(child) => child[*c3],
                None => cells[CELL_MAP[*order][3]],
            };

            faces.extend(edge_proc(tree, *edge_dir, [o0, o1, o2, o3]));
        }
    }

    faces
}

/// Recursive function that extracts faces from four edge-adjacent octants.
///
/// Calls itself twice in the direction `dir` for all four sub-cells
/// that share a half-edge contained in the edge.
fn edge_proc(tree: &Octree, dir: TreeAxis, cells: [OctantIdx; 4]) -> Vec<Face> {
    let mut faces = vec![];
    match (
        tree.get_octant(cells[0]).children,
        tree.get_octant(cells[1]).children,
        tree.get_octant(cells[2]).children,
        tree.get_octant(cells[3]).children,
    ) {
        (None, None, None, None) => {
            if let Some(face) = make_face(tree, cells) {
                faces.push(face);
            }
        }
        (o0, o1, o2, o3) => {
            for [idx0, idx1, idx2, idx3] in &EDGE_EDGE_MAP[dir as usize] {
                let c0 = match o0 {
                    Some(o0) => o0[*idx0],
                    None => cells[0],
                };
                let c1 = match o1 {
                    Some(o1) => o1[*idx1],
                    None => cells[1],
                };
                let c2 = match o2 {
                    Some(o2) => o2[*idx2],
                    None => cells[2],
                };
                let c3 = match o3 {
                    Some(o3) => o3[*idx3],
                    None => cells[3],
                };
                faces.extend(edge_proc(tree, dir, [c0, c1, c2, c3]));
            }
        }
    }
    faces
}

/// Creates a face of the polygon if all leaf cells have a feature.
fn make_face(tree: &Octree, cells: [OctantIdx; 4]) -> Option<Face> {
    // Cells can have duplicated
    let mut dedup_cells = vec![];
    for x in &cells {
        if dedup_cells.contains(x) {
            continue;
        } else {
            dedup_cells.push(*x);
        }
    }
    let dedup_feature_cells: Vec<Point> =
        dedup_cells.iter().filter_map(|c| tree.get_octant(*c).feature).collect();

    // If there aren't three points, a face cannot be constructed
    if dedup_feature_cells.len() < 3 {
        return None;
    }

    if dedup_cells.len() == 4 && dedup_feature_cells.len() == 4 {
        Some(Face::Plane {
            ul: dedup_feature_cells[0],
            ur: dedup_feature_cells[1],
            ll: dedup_feature_cells[2],
            lr: dedup_feature_cells[3],
        })
    } else if dedup_cells.len() == 3 && dedup_feature_cells.len() == 3 {
        Some(Face::Triangle {
            ul: dedup_feature_cells[0],
            lr: dedup_feature_cells[1],
            ll: dedup_feature_cells[2],
        })
    } else {
        None
    }
}
