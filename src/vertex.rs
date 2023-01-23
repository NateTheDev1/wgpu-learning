use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use wgpu::{vertex_attr_array, BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

// POD = Plain Old Data
// Zeroable = Allows us to use the zeroed() method

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    // The position of the vertex in 3d space. (xyz)
    position: [f32; 3],
    // RGB values of the vertex
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        // VertexBufferLayout {
        //     // defines how wide the vertex is in bytes
        //     array_stride: size_of::<Vertex>() as BufferAddress,
        //     // Tells the pipeline wether each element of the array in this buffer represents per-vertex data or per-instance data
        //     step_mode: VertexStepMode::Vertex,
        //     // Describe the individual parts of the vertex. Generally 1:1 mapping with the fields of the struct.
        //     attributes: &[
        //         // position of the vertex struct
        //         VertexAttribute {
        //             offset: 0,
        //             shader_location: 0,
        //             format: wgpu::VertexFormat::Float32x3,
        //         },
        //         // color of the vertex struct
        //         VertexAttribute {
        //             offset: size_of::<[f32; 3]>() as BufferAddress,
        //             shader_location: 1,
        //             // Specifies the shape of the attribute
        //             format: wgpu::VertexFormat::Float32x3,
        //         },
        //     ],

        // Quicker way to do the above

        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    // Changed
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.00759614],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.43041354],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.949397],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.84732914],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.2652641],
    }, // E
];

pub const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
