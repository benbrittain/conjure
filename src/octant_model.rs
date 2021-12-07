use {
    crate::{model::ModelVertex, octree::Octant, util},
    wgpu::util::DeviceExt,
};

pub struct OctantMesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

impl OctantMesh {
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue, octants: &[Octant]) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for octant in octants {
            let color = match octant.feature {
                Some(p) => util::color_from_point(&p),
                None => [0.9, 0.9, 0.9],
            };
            let x_axis = octant.x_axis;
            let y_axis = octant.y_axis;
            let z_axis = octant.z_axis;

            for face in &[
                (
                    [x_axis.lower, y_axis.upper, z_axis.lower],
                    [x_axis.upper, y_axis.upper, z_axis.lower],
                    [x_axis.lower, y_axis.lower, z_axis.lower],
                    [x_axis.upper, y_axis.lower, z_axis.lower],
                ),
                (
                    [x_axis.lower, y_axis.upper, z_axis.upper],
                    [x_axis.upper, y_axis.upper, z_axis.upper],
                    [x_axis.lower, y_axis.lower, z_axis.upper],
                    [x_axis.upper, y_axis.lower, z_axis.upper],
                ),
                (
                    [x_axis.lower, y_axis.upper, z_axis.lower],
                    [x_axis.lower, y_axis.upper, z_axis.upper],
                    [x_axis.lower, y_axis.lower, z_axis.lower],
                    [x_axis.lower, y_axis.lower, z_axis.upper],
                ),
                (
                    [x_axis.upper, y_axis.upper, z_axis.lower],
                    [x_axis.upper, y_axis.upper, z_axis.upper],
                    [x_axis.upper, y_axis.lower, z_axis.lower],
                    [x_axis.upper, y_axis.lower, z_axis.upper],
                ),
                (
                    [x_axis.lower, y_axis.lower, z_axis.upper],
                    [x_axis.upper, y_axis.lower, z_axis.upper],
                    [x_axis.lower, y_axis.lower, z_axis.lower],
                    [x_axis.upper, y_axis.lower, z_axis.lower],
                ),
                (
                    [x_axis.lower, y_axis.upper, z_axis.upper],
                    [x_axis.upper, y_axis.upper, z_axis.upper],
                    [x_axis.lower, y_axis.upper, z_axis.lower],
                    [x_axis.upper, y_axis.upper, z_axis.lower],
                ),
            ] {
                for edge in
                    &[(face.0, face.1), (face.0, face.2), (face.2, face.3), (face.1, face.3)]
                {
                    let size = 0.01;
                    let ulf = [edge.0[0] - size, edge.0[1] - size, edge.0[2] - size];
                    let lrr = [edge.1[0] + size, edge.1[1] + size, edge.1[2] + size];

                    for face in &[
                        // front face
                        [
                            [ulf[0], ulf[1], ulf[2]],
                            [lrr[0], ulf[1], ulf[2]],
                            [ulf[0], lrr[1], ulf[2]],
                            [lrr[0], lrr[1], ulf[2]],
                        ],
                        // back face
                        [
                            [ulf[0], ulf[1], lrr[2]],
                            [lrr[0], ulf[1], lrr[2]],
                            [ulf[0], lrr[1], lrr[2]],
                            [lrr[0], lrr[1], lrr[2]],
                        ],
                        // right face
                        [
                            [lrr[0], ulf[1], ulf[2]],
                            [lrr[0], ulf[1], lrr[2]],
                            [lrr[0], lrr[1], ulf[2]],
                            [lrr[0], lrr[1], lrr[2]],
                        ],
                        // left face
                        [
                            [ulf[0], ulf[1], ulf[2]],
                            [ulf[0], ulf[1], lrr[2]],
                            [ulf[0], lrr[1], ulf[2]],
                            [ulf[0], lrr[1], lrr[2]],
                        ],
                        // top face
                        [
                            [ulf[0], ulf[1], lrr[2]],
                            [lrr[0], ulf[1], lrr[2]],
                            [ulf[0], ulf[1], ulf[2]],
                            [lrr[0], ulf[1], ulf[2]],
                        ],
                        // bottom face
                        [
                            [ulf[0], lrr[1], lrr[2]],
                            [lrr[0], lrr[1], lrr[2]],
                            [ulf[0], lrr[1], ulf[2]],
                            [lrr[0], lrr[1], ulf[2]],
                        ],
                    ] {
                        let ul = face[0];
                        let ur = face[1];
                        let ll = face[2];
                        let lr = face[3];
                        vertices.push(ModelVertex { position: ul, color });
                        indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));
                        vertices.push(ModelVertex { position: lr, color });
                        indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));
                        vertices.push(ModelVertex { position: ll, color });

                        indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));
                        vertices.push(ModelVertex { position: ll, color });
                        indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));
                        vertices.push(ModelVertex { position: ur, color });
                        indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));
                        vertices.push(ModelVertex { position: lr, color });
                        indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));
                    }
                }
            }
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        OctantMesh {
            name: String::from("test"),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
        }
    }
}

pub trait DrawOctant<'a> {
    fn draw_octants(&mut self, octants: &'a OctantMesh);
}

impl<'a, 'b> DrawOctant<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_octants(&mut self, octants: &'a OctantMesh) {
        self.set_vertex_buffer(0, octants.vertex_buffer.slice(..));
        self.set_index_buffer(octants.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..octants.num_elements, 0, 0..1);
    }
}
