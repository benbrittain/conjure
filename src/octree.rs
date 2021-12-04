use {
    crate::shape::ShapeFunc,
    log::{info, warn},
    std::ops::Range,
};

type OctantIdx = usize;

/// Handle to an object (created from a `ShapeFunc`) in the Octree.
type ShapeHandle = usize;

/// Axis (range) of the Octree
#[derive(Debug, Clone, Copy)]
struct OctAxis {
    pub lower: f32,
    pub upper: f32,
}

impl OctAxis {
    fn new(l: f32, u: f32) -> Self {
        let (lower, upper) = if l <= u { (l, u) } else { (u, l) };
        Self { lower, upper }
    }

    pub fn center(self) -> f32 {
        (self.lower + self.upper) / 2.0
    }

    pub fn split(self) -> (OctAxis, OctAxis) {
        let mid = (self.lower + self.upper) / 2.0;
        (OctAxis::new(self.lower, mid), OctAxis::new(mid, self.upper))
    }

    pub fn length(&self) -> f32 {
        self.upper - self.lower
    }
}

/// Represents each individual octant in the greater octree.
#[derive(Debug)]
pub struct Octant {
    x_axis: OctAxis,
    y_axis: OctAxis,
    z_axis: OctAxis,
    pub children: Option<[OctantIdx; 8]>,
}

impl Octant {
    /// Creates a new leaf node octant.
    fn new(x_axis: OctAxis, y_axis: OctAxis, z_axis: OctAxis) -> Octant {
        Self { x_axis, y_axis, z_axis, children: None }
    }

    /// Octant is a leaf node if there are no children.
    pub fn is_leaf_node(&self) -> bool {
        self.children.is_none()
    }
}

/// Stores a 3d representation of the shape functions at arbitrary resolutions.
#[derive(Debug)]
pub struct Octree {
    // Stores all octants in the octree, child/parent relationships are maintained
    // in the Octant itself.
    //
    // TODO: consider a slab allocation scheme here
    octants: Vec<Octant>,
    range: OctAxis,
    root_idx: Option<OctantIdx>,
}

#[derive(Debug, Copy, Clone)]
enum Subdivided {
    Value(f32),
    Idx(OctantIdx),
}

impl Octree {
    /// Creates a new octree bounding the space of `bounds` in 3 dimensions.
    pub fn new(lower_bound: f32, upper_bound: f32) -> Self {
        Octree { octants: vec![], range: OctAxis::new(lower_bound, upper_bound), root_idx: None }
    }

    /// Adds an object to the Octree rendered from the `function` at a resolution of `resolution`
    pub fn render_shape(&mut self, resolution: f32, function: ShapeFunc) -> ShapeHandle {
        let depth = (self.range.length() / resolution).log2() as u8;
        info!("Rendering a shape at a resolution of {} (depth: {})", resolution, depth);
        self.subdivide(self.range.clone(), self.range.clone(), self.range.clone(), depth, function);
        warn!("Rendering a shape, ShapeHandle not yet implemented");
        0
    }

    /// Adds an Octant to the Octree returning an `OctantIdx` to represent it's place in the tree.
    fn add_octant(&mut self, oct: Octant) -> OctantIdx {
        self.octants.push(oct);
        self.octants.len() - 1
    }

    /// Checks if the `Subdivided` regions can be merged into a single octant.
    /// Examples: all the subregions are contained within the shape.
    ///
    /// TODO: add some fancier logic, bilinear interpolation perhaps?
    fn merge_octants(octant_children: [Subdivided; 8]) -> Option<Subdivided> {
        // If all the values are within the shape, unify the region
        if octant_children.iter().all(|&x| match x {
            Subdivided::Value(x) => x < 0.0,
            _ => false,
        }) {
            return Some(octant_children[0]);
        }

        // If all the values are outside the shape, unify the region
        if octant_children.iter().all(|&x| match x {
            Subdivided::Value(x) => x >= 0.0,
            _ => false,
        }) {
            return Some(octant_children[0]);
        }

        None
    }

    fn subdivide(
        &mut self,
        x_axis: OctAxis,
        y_axis: OctAxis,
        z_axis: OctAxis,
        depth: u8,
        shape_func: ShapeFunc,
    ) -> Subdivided {
        if depth == 0 {
            // We're at the bottom of the octree, generate a leaf node Octant
            return Subdivided::Value(shape_func.call(
                x_axis.center(),
                y_axis.center(),
                z_axis.center(),
            ));
        }

        // Since not at the leaf node, check every child octant in the current octant
        let (left_x, right_x) = x_axis.split();
        let (bottom_y, top_y) = y_axis.split();
        let (front_z, back_z) = z_axis.split();
        let new_depth = depth - 1;

        let octant_children = [
            self.subdivide(left_x, top_y, front_z, new_depth, shape_func),
            self.subdivide(left_x, top_y, back_z, new_depth, shape_func),
            self.subdivide(left_x, bottom_y, front_z, new_depth, shape_func),
            self.subdivide(left_x, bottom_y, back_z, new_depth, shape_func),
            self.subdivide(right_x, top_y, front_z, new_depth, shape_func),
            self.subdivide(right_x, top_y, back_z, new_depth, shape_func),
            self.subdivide(right_x, bottom_y, front_z, new_depth, shape_func),
            self.subdivide(right_x, bottom_y, back_z, new_depth, shape_func),
        ];

        // Merge octants if possible
        if let Some(merged_region) = Self::merge_octants(octant_children) {
            return merged_region;
        }

        // Create the new octant and connect it's children below
        let root = self.add_octant(Octant::new(x_axis, y_axis, z_axis));

        // If the subdivide region already exists, return it's index,
        // otherwise create a new octant.
        let octant_children = octant_children.map(|child| {
            match child {
                Subdivided::Idx(idx) => idx,
                Subdivided::Value(v) => {
                    // TODO: dual contour the feature here
                    self.add_octant(Octant::new(left_x, top_y, front_z))
                }
            }
        });
        self.octants[root].children = Some(octant_children);

        // This region is now the rootiest root, unless it's deep in the subdivide
        // graph, then it'll be replaced by the parent caller.
        self.root_idx = Some(root);

        Subdivided::Idx(root)
    }
}
