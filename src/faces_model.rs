use {
    crate::{model::ModelVertex, types::Face},
    wgpu::util::DeviceExt,
};

pub struct FaceMesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

impl FaceMesh {
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue, faces: &[Face]) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for face in faces {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let r: f32 = rng.gen_range(0.0..1.0);
            let g: f32 = rng.gen_range(0.0..1.0);
            let b: f32 = rng.gen_range(0.0..1.0);
            let color = [r as f32, g as f32, b as f32];
            match face {
                Face::Triangle { ul, lr, ll } => {
                    vertices.push(ModelVertex {
                        position: [ul.x as f32, ul.y as f32, ul.z as f32],
                        color,
                    });
                    indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));

                    vertices.push(ModelVertex {
                        position: [lr.x as f32, lr.y as f32, lr.z as f32],
                        color,
                    });
                    indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));

                    vertices.push(ModelVertex {
                        position: [ll.x as f32, ll.y as f32, ll.z as f32],
                        color,
                    });
                    indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));
                }
                Face::Plane { ul, ur, ll, lr } => {
                    for tri in [(ul, ur, lr), (lr, ll, ul)].iter() {
                        vertices.push(ModelVertex {
                            position: [tri.0.x as f32, tri.0.y as f32, tri.0.z as f32],
                            color,
                        });
                        indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));

                        vertices.push(ModelVertex {
                            position: [tri.1.x as f32, tri.1.y as f32, tri.1.z as f32],
                            color,
                        });
                        indices.push(indices.last().map(|&x| x + 1).unwrap_or(0));

                        vertices.push(ModelVertex {
                            position: [tri.2.x as f32, tri.2.y as f32, tri.2.z as f32],
                            color,
                        });
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
        FaceMesh {
            name: String::from("Face Mesh"),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
        }
    }
}

pub trait DrawFaces<'a> {
    fn draw_faces(&mut self, faces: &'a FaceMesh);
}

impl<'a, 'b> DrawFaces<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_faces(&mut self, faces: &'a FaceMesh) {
        self.set_vertex_buffer(0, faces.vertex_buffer.slice(..));
        self.set_index_buffer(faces.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..faces.num_elements, 0, 0..1);
    }
}
