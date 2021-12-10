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

        for point in points {
            let x_center = point.x;
            let y_center = point.y;
            let z_center = point.z;
            let x_min = point.x - 0.07;
            let x_max = point.x + 0.07;
            let y_min = point.y - 0.10;
            let y_max = point.y + 0.10;
            let z_min = point.z - 0.07;
            let z_max = point.z + 0.07;
            let color = util::color_from_point(point);
            let bottom_vertex = [x_center, y_center, z_min];
            let top_vertex = [x_center, y_center, z_max];
            let one = [x_max, y_center, z_center];
            let two = [x_center, y_min, z_center];
            let three = [x_min, y_center, z_center];
            let four = [x_center, y_max, z_center];

            // Draw a 6 sided diamond
            for face in &[
                (one, two, bottom_vertex),
                (two, three, bottom_vertex),
                (three, four, bottom_vertex),
                (four, one, bottom_vertex),
                (one, two, top_vertex),
                (two, three, top_vertex),
                (three, four, top_vertex),
                (four, one, top_vertex),
            ] {
                vertices.push(ModelVertex { position: face.0, color });
                indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));

                vertices.push(ModelVertex { position: face.1, color });
                indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));

                vertices.push(ModelVertex { position: face.2, color });
                indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));
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
