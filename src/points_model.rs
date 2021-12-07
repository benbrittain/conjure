use {
    crate::{model::ModelVertex, types::Point, util},
    wgpu::util::DeviceExt,
};

pub struct PointMesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

impl PointMesh {
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue, points: &[Point]) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        use rand::Rng;
        let mut rng = rand::thread_rng();
        for point in points {
            let x_center = point.x;
            let y_center = point.y;
            let z_center = point.z;
            let x_min = point.x - 0.05;
            let x_max = point.x + 0.05;
            let y_min = point.y - 0.08;
            let y_max = point.y + 0.08;
            let z_min = point.z - 0.05;
            let z_max = point.z + 0.05;
            // light green
            let color = [0.19, 0.8, 0.19];
            let bottom_vertex = [x_center, y_center, z_min];
            let top_vertex = [x_center, y_center, z_max];
            let one = [x_max, y_center, z_center];
            let two = [x_center, y_min, z_center];
            let three = [x_min, y_center, z_center];
            let four = [x_center, y_max, z_center];

            // Draw a 6 sided diamond
            // looks like the Sims!
            for face in &[
                (one, bottom_vertex, two),
                (two, bottom_vertex, three),
                (three, bottom_vertex, four),
                (four, bottom_vertex, one),
                (one, top_vertex, two),
                (two, top_vertex, three),
                (three, top_vertex, four),
                (four, top_vertex, one),
            ] {
                vertices.push(ModelVertex { position: face.0, color });
                indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));
                vertices.push(ModelVertex { position: face.1, color });
                indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));
                vertices.push(ModelVertex { position: face.2, color });
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
        PointMesh {
            name: String::from("Point Mesh"),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
        }
    }
}

pub trait DrawPoints<'a> {
    fn draw_points(&mut self, octants: &'a PointMesh);
}

impl<'a, 'b> DrawPoints<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_points(&mut self, octants: &'a PointMesh) {
        self.set_vertex_buffer(0, octants.vertex_buffer.slice(..));
        self.set_index_buffer(octants.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..octants.num_elements, 0, 0..1);
    }
}
