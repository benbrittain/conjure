use {
    crate::{
        dual_contour,
        shape::ShapeFunc,
        types::{Face, Point},
    },
    log::{info, warn},
};

/// Index into the Octree for a unique `Octant`
pub type OctantIdx = usize;

/// Handle to an object (created from a `ShapeFunc`) in the Octree.
pub type ShapeHandle = usize;

/// Axis (range) of the Octree
// TODO make private?
#[derive(Debug, Clone, Copy)]
pub struct OctAxis {
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
#[derive(Debug, Clone)]
pub struct Octant {
    pub x_axis: OctAxis,
    pub y_axis: OctAxis,
    pub z_axis: OctAxis,
    pub children: Option<[OctantIdx; 8]>,
    pub feature: Option<Point>,
}

#[allow(dead_code)]
impl Octant {
    /// Creates a new leaf node octant.
    fn new(x_axis: OctAxis, y_axis: OctAxis, z_axis: OctAxis, feature: Option<Point>) -> Octant {
        Self { x_axis, y_axis, z_axis, children: None, feature }
    }

    /// Returns a bool based on if the `Octant` contains a feature point.
    pub fn has_feature(&self) -> bool {
        self.feature.is_some()
    }

    /// Returns a bool based on if the Octant is a leaf node or not.
    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }
}

/// Stores a 3d representation of the shape functions at arbitrary resolutions.
#[derive(Clone, Debug)]
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
        self.subdivide(self.range, self.range, self.range, depth, function);
        warn!("Rendering a shape, ShapeHandle not yet implemented");
        0
    }

    pub fn extract_faces(&self) -> Vec<Face> {
        match self.root_idx {
            Some(idx) => dual_contour::cell_proc(self, idx),
            None => vec![],
        }
    }

    /// Adds an Octant to the Octree returning an `OctantIdx` to represent it's place in the tree.
    fn add_octant(&mut self, oct: Octant) -> OctantIdx {
        self.octants.push(oct);
        self.octants.len() - 1
    }

    /// Returns a refrence to an `Octant` from an `OctantIdx`
    pub fn get_octant(&self, idx: OctantIdx) -> &Octant {
        &self.octants[idx]
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

        let subdivides = [
            [left_x, top_y, front_z],
            [left_x, top_y, back_z],
            [left_x, bottom_y, front_z],
            [left_x, bottom_y, back_z],
            [right_x, top_y, front_z],
            [right_x, top_y, back_z],
            [right_x, bottom_y, front_z],
            [right_x, bottom_y, back_z],
        ];

        let octant_children =
            subdivides.map(|[x, y, z]| self.subdivide(x, y, z, new_depth, shape_func));

        // Merge octants if possible
        if let Some(merged_region) = Self::merge_octants(octant_children) {
            return merged_region;
        }

        // Create the new octant and connect it's children below
        let root = self.add_octant(Octant::new(x_axis, y_axis, z_axis, None));

        // If the subdivide region already exists, return it's index,
        // otherwise create a new octant.
        let octant_children =
            octant_children.zip(subdivides).map(|(child, [x, y, z])| match child {
                Subdivided::Idx(idx) => idx,
                Subdivided::Value(_) => {
                    let feature = dual_contour::new_feature(x, y, z, shape_func);
                    self.add_octant(Octant::new(x, y, z, feature))
                }
            });
        self.octants[root].children = Some(octant_children);

        // This region is now the rootiest root, unless it's deep in the subdivide
        // graph, then it'll be replaced by the parent caller.
        self.root_idx = Some(root);

        Subdivided::Idx(root)
    }
}

pub struct OctreeIter {
    nodes: Vec<Octant>,
    queue: Vec<OctantIdx>,
}

impl Iterator for OctreeIter {
    type Item = Octant;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // As long as there are nodes in the queue, visit them.
            let idx = match self.queue.pop() {
                Some(idx) => idx,
                None => return None,
            };

            // If a leaf node, return the octant.
            // Otherwise, add the children to the queue for visiting.
            match self.nodes[idx].children {
                Some(children) => self.queue.extend_from_slice(&children),
                None => return Some(self.nodes[idx].clone()),
            }
        }
    }
}

impl IntoIterator for Octree {
    type Item = Octant;
    type IntoIter = OctreeIter;

    fn into_iter(self) -> Self::IntoIter {
        match self.root_idx {
            Some(root_idx) => OctreeIter { nodes: self.octants, queue: vec![root_idx] },
            None => {
                eprintln!("Octree has no root index!");
                OctreeIter { nodes: vec![], queue: vec![] }
            }
        }
    }
}
