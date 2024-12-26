extern crate tobj;
use std::{convert::TryInto, mem, sync::Arc};

use erupt::{vk, DeviceLoader};
use gltf::{
    buffer::Data,
    mesh::util::{tex_coords, ReadTexCoords},
    Gltf,
};

use erupt::vk1_0::{
    Buffer, BufferCopyBuilder, BufferUsageFlags, CommandBuffer, CommandBufferAllocateInfoBuilder,
    CommandBufferBeginInfoBuilder, CommandBufferLevel, CommandBufferUsageFlags, CommandPool,
    DeviceSize, Fence, Format, IndexType, MemoryPropertyFlags, Queue, SubmitInfoBuilder,
    VertexInputAttributeDescriptionBuilder, VertexInputBindingDescriptionBuilder, VertexInputRate,
};
use glam::{Vec2, Vec3};

use super::{buffer::VRTBuffer, device::VRTDevice, result::VkResult};

macro_rules! size_of {
    ($ty:ty) => {
        std::mem::size_of::<$ty>() as u32
    };
}

macro_rules! offset_of {
    ($ty:ty, $field:ident) => {{
        let base = std::mem::MaybeUninit::<$ty>::uninit();
        let base_ptr = base.as_ptr();
        let field_ptr = unsafe { std::ptr::addr_of!((*base_ptr).$field) };
        unsafe { field_ptr.cast::<u8>().offset_from(base_ptr.cast::<u8>()) as u32 }
    }};
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ModelVertex {
    pub position: glam::Vec3,
    pub tex_coords: glam::Vec2,
    pub normal: glam::Vec3,
}

impl ModelVertex {
    pub fn binding_description() -> VertexInputBindingDescriptionBuilder<'static> {
        VertexInputBindingDescriptionBuilder::new()
            .binding(0)
            .stride(size_of!(Self))
            .input_rate(VertexInputRate::VERTEX)
    }

    pub fn attribute_descriptions() -> [VertexInputAttributeDescriptionBuilder<'static>; 3] {
        [
            VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(0)
                .format(Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, position)),
            VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(1)
                .format(Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, tex_coords)),
            VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(2)
                .format(Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, normal)),
        ]
    }
}

pub struct MeshObject {
    pub first_index: u32, // LOL name; first index in index buffer
    pub indices_count: u32,
    //pub vertex_offset: u32,
}

// Structure to store processed node data
struct GltfModel {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    nodes: Vec<Node>,
}

// Vertex structure (adjust according to your vertex format)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coords: [f32; 2],
}

// Node structure to store mesh data
struct Node {
    first_index: u32,
    index_count: u32,
    children: Vec<Node>,
}

impl GltfModel {
    // Function to load and process glTF model
    fn from_gltf(gltf_path: &str) -> Self {
        let (doc, buffers, images) = gltf::import(&gltf_path).unwrap();

        let mut model = GltfModel {
            vertices: Vec::new(),
            indices: Vec::new(),
            nodes: Vec::new(),
        };

        // Process each scene in the glTF file
        for scene in doc.scenes() {
            for node in scene.nodes() {
                model.process_node(node, &buffers, &mut 0);
            }
        }

        model
    }

    // Recursive function to process nodes and extract mesh data
    fn process_node(
        &mut self,
        node: gltf::Node,
        buffers: &Vec<Data>,
        index_offset: &mut u32,
    ) -> Result<(), gltf::Error> {
        let mut mesh_indices: Vec<u32> = Vec::new();

        // Process meshes in the node
        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let mut position_iter = reader.read_positions().unwrap();
                let mut normal_iter = reader.read_normals().unwrap();
                let mut tex_coords_iter = reader.read_tex_coords(0).into_iter();

                while let (Some(pos), Some(normal), Some(tex_coords)) = (
                    position_iter.next(),
                    normal_iter.next(),
                    tex_coords_iter.next(),
                ) {
                    let tex_coords: Vec<[f32; 2]> = tex_coords
                        .into_f32()
                        .into_iter()
                        .map(|item| [item[0], item[1]])
                        .collect();
                    self.vertices.push(Vertex {
                        position: pos,
                        normal,
                        tex_coords: [tex_coords.get(0).unwrap()[0], tex_coords.get(0).unwrap()[1]],
                    });
                }

                let indices: Option<Vec<u32>> = reader
                    .read_indices()
                    .map(|indices| indices.into_u32().collect());
                mesh_indices.extend(indices.unwrap());
            }
        }

        // Calculate index count and store node data
        let index_count = mesh_indices.len() as u32;
        if index_count > 0 {
            self.indices.extend(mesh_indices);
            let node_data = Node {
                first_index: *index_offset,
                index_count,
                children: Vec::new(), // Process children recursively if needed
            };
            self.nodes.push(node_data);
            *index_offset += index_count;
        }

        // Process child nodes recursively
        for child in node.children() {
            self.process_node(child, buffers, index_offset)?;
        }

        Ok(())
    }
}

pub struct Model {
    vertex_buffer: VRTBuffer,
    index_buffer: VRTBuffer,
    meshes: Vec<MeshObject>,
}

impl Model {
    pub fn new(device: Arc<VRTDevice>, path: &str) -> Self {
        let gltf_model = GltfModel::from_gltf(path);

        let model_vertices = gltf_model
            .vertices
            .into_iter()
            .map(|vertex| ModelVertex {
                position: vertex.position.into(),
                normal: vertex.normal.into(),
                tex_coords: vertex.tex_coords.into(),
            })
            .collect();

        let vertex_buffer = Self::create_vertex_buffer(device.clone(), model_vertices).unwrap();
        let index_buffer = Self::create_index_buffer(
            device.clone(),
            gltf_model.indices.iter().map(|&x| x as u16).collect(),
        )
        .unwrap();

        // println!("Mesh count: {}", meshes.len());
        // println!("Indices count: {}", meshes.get(0).unwrap().indices_count);

        Self {
            vertex_buffer,
            index_buffer,
            meshes,
        }
    }

    pub fn bind(&self, device: Arc<VRTDevice>, command_buffer: CommandBuffer) {
        unsafe {
            device.get_device_ptr().cmd_bind_vertex_buffers(
                command_buffer,
                0,
                std::slice::from_ref(&self.vertex_buffer.get_buffer()),
                &[0],
            );

            device.get_device_ptr().cmd_bind_index_buffer(
                command_buffer,
                self.index_buffer.get_buffer(),
                0,
                IndexType::UINT16,
            );
        }
    }

    pub fn draw(&self, device: Arc<VRTDevice>, command_buffer: CommandBuffer) {
        unsafe {
            // let mesh = self.meshes.get(0).unwrap();
            // // println!(
            // //     "offset {}, instance {}",
            // //     mesh.vertex_offset, mesh.first_index
            // // );
            // device.get_device_ptr().cmd_draw_indexed(
            //     command_buffer,
            //     mesh.indices_count as u32,
            //     1,
            //     mesh.first_index,
            //     0,
            //     0,
            // );

            for mesh in self.meshes.iter() {
                device.get_device_ptr().cmd_draw_indexed(
                    command_buffer,
                    mesh.indices_count as u32,
                    1,
                    mesh.first_index,
                    0,
                    0,
                );
            }
        }
    }

    fn create_index_buffer(device: Arc<VRTDevice>, indices: Vec<u16>) -> VkResult<VRTBuffer> {
        let buffer_size = (mem::size_of::<u16>() * indices.len()) as u64;
        let mut staging_buffer = VRTBuffer::new(
            device.clone(),
            mem::size_of::<u16>().try_into().unwrap(),
            indices.len().try_into().unwrap(),
            BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            None,
        );

        staging_buffer.map(Some(buffer_size), Some(0));

        staging_buffer.write_to_buffer(
            indices.as_ptr(),
            staging_buffer.get_mapped_memory().unwrap(),
            indices.len() as DeviceSize,
            0,
        );
        staging_buffer.unmap();

        let index_buffer = VRTBuffer::new(
            device.clone(),
            mem::size_of::<u16>().try_into().unwrap(),
            indices.len().try_into().unwrap(),
            BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::INDEX_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL,
            None,
        );

        Self::copy_buffer(
            &device.get_device_ptr(),
            device.get_queues().graphics,
            device.get_command_pool(),
            staging_buffer.get_buffer(),
            index_buffer.get_buffer(),
            buffer_size,
        )?;

        Ok(index_buffer)
    }

    fn create_vertex_buffer(
        device: Arc<VRTDevice>,
        vertices: Vec<ModelVertex>,
    ) -> VkResult<VRTBuffer> {
        let buffer_size = (mem::size_of::<ModelVertex>() * vertices.len()) as DeviceSize;

        let mut staging_buffer = VRTBuffer::new(
            device.clone(),
            mem::size_of::<ModelVertex>().try_into().unwrap(),
            vertices.len().try_into().unwrap(),
            BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            None,
        );

        staging_buffer.map(Some(buffer_size), Some(0));

        staging_buffer.write_to_buffer(
            vertices.as_ptr(),
            staging_buffer.get_mapped_memory().unwrap(),
            vertices.len() as DeviceSize,
            0,
        );
        staging_buffer.unmap();

        let vertex_buffer = VRTBuffer::new(
            device.clone(),
            mem::size_of::<ModelVertex>().try_into().unwrap(),
            vertices.len().try_into().unwrap(),
            BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL,
            None,
        );

        Self::copy_buffer(
            &device.get_device_ptr(),
            device.get_queues().graphics,
            device.get_command_pool(),
            staging_buffer.get_buffer(),
            vertex_buffer.get_buffer(),
            buffer_size,
        )?;

        Ok(vertex_buffer)
    }

    fn copy_buffer(
        device: &DeviceLoader,
        graphics_queue: Queue,
        command_pool: CommandPool,
        src: Buffer,
        dst: Buffer,
        size: DeviceSize,
    ) -> VkResult<()> {
        let alloc_info = CommandBufferAllocateInfoBuilder::new()
            .level(CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1);

        let command_buffer = unsafe { device.allocate_command_buffers(&alloc_info) }.result()?[0];

        let begin_info =
            CommandBufferBeginInfoBuilder::new().flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { device.begin_command_buffer(command_buffer, &begin_info) }.result()?;

        let copy_region = BufferCopyBuilder::new().size(size);
        unsafe {
            device.cmd_copy_buffer(command_buffer, src, dst, std::slice::from_ref(&copy_region))
        };

        unsafe { device.end_command_buffer(command_buffer) }.result()?;

        let submit_info =
            SubmitInfoBuilder::new().command_buffers(std::slice::from_ref(&command_buffer));

        unsafe {
            device.queue_submit(
                graphics_queue,
                std::slice::from_ref(&submit_info),
                Fence::null(),
            )
        }
        .result()?;
        unsafe { device.queue_wait_idle(graphics_queue) }.result()?;

        unsafe { device.free_command_buffers(command_pool, std::slice::from_ref(&command_buffer)) };

        Ok(())
    }
}

impl PartialEq for ModelVertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.normal == other.normal
            && self.tex_coords == other.tex_coords
    }
}

impl Eq for ModelVertex {}

impl std::hash::Hash for ModelVertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position[0].to_bits().hash(state);
        self.position[1].to_bits().hash(state);
        self.position[2].to_bits().hash(state);
        self.normal[0].to_bits().hash(state);
        self.normal[1].to_bits().hash(state);
        self.normal[2].to_bits().hash(state);
        self.tex_coords[0].to_bits().hash(state);
        self.tex_coords[1].to_bits().hash(state);
    }
}

// let obj_file = obj::Obj::load(path).unwrap();
// let positions = obj_file.data.position;
// let texcoords = obj_file.data.texture;
// let normals = obj_file.data.normal;

// let mut model_vertices: Vec<ModelVertex> = vec![];
// let mut indices: Vec<u16> = vec![];
// let mut meshes: Vec<MeshObject> = vec![];
// let mut unique_vertices = std::collections::HashMap::new();

// println!("number of objects: {}", obj_file.data.objects.len());
// for object in obj_file.data.objects {
//     println!("number of groups in object: {}", object.groups.len());
//     let first_index = indices.len() as u32;
//     let mut indices_count = 0;
//     let vertex_offset = model_vertices.len();
//     for group in object.groups {
//         let mut mesh_indices = Vec::new();
//         println!("number of polys in group: {}", group.polys.len());
//         for obj::SimplePolygon(poly) in group.polys {
//             for vertex in poly {
//                 let obj::IndexTuple(v, vt, vn) = vertex;
//                 let v_index = v;
//                 let vt_index = vt.unwrap();
//                 let vn_index = vn.unwrap();

//                 let vertex = ModelVertex {
//                     position: glam::Vec3::from_array(positions[v_index]),
//                     tex_coords: glam::Vec2::from_array(texcoords[vt_index]),
//                     normal: glam::Vec3::from_array(normals[vn_index]),
//                 };

//                 if let Some(on_index) = unique_vertices.get(&vertex) {
//                     mesh_indices.push(*on_index as u16);
//                 } else {
//                     let index = model_vertices.len();
//                     unique_vertices.insert(vertex, index);
//                     model_vertices.push(vertex);
//                     mesh_indices.push(index as u16);
//                 }
//                 indices_count += 1;
//             }
//         }
//         indices.extend_from_slice(&mesh_indices);
//         println!(
//             "First: {}, offset: {}, count: {}",
//             first_index, vertex_offset, indices_count
//         );
//         println!("count: {}", mesh_indices.len());
//         println!(
//             "first index: {}",
//             indices.len() as u32 - mesh_indices.len() as u32
//         );
//         println!("end");
//         meshes.push(MeshObject {
//             indices_count: mesh_indices.len() as u32,
//             first_index: indices.len() as u32 - mesh_indices.len() as u32,
//         });
//     }
// }

// let (models, materials) = tobj::load_obj(
//     &path,
//     &tobj::LoadOptions {
//         triangulate: true,
//         single_index: true,
//         ..Default::default()
//     },
// )
// .expect("Failed to OBJ load file");

// println!("Model meshes count: {}", models.len());

// let mut model_vertices: Vec<ModelVertex> = vec![];
// let mut indices: Vec<u16> = vec![];
// let mut meshes: Vec<MeshObject> = vec![];
// let mut unique_vertices = std::collections::HashMap::new();

// for model in &models {
//     let first_index = indices.len() as u32;
//     let mut indices_count = 0;
//     println!("Indices in current mesh: {}", &model.mesh.indices.len());

//     for index in &model.mesh.indices {
//         let pos_offset = (3 * index) as usize;
//         let tex_coord_offset = (2 * index) as usize;

//         let vertex = ModelVertex {
//             position: glam::vec3(
//                 model.mesh.positions[pos_offset],
//                 model.mesh.positions[pos_offset + 1],
//                 model.mesh.positions[pos_offset + 2],
//             ),
//             tex_coords: glam::vec2(
//                 model.mesh.texcoords[tex_coord_offset],
//                 model.mesh.texcoords[tex_coord_offset + 1],
//             ),
//             normal: glam::vec3(
//                 model.mesh.normals[pos_offset],
//                 model.mesh.normals[pos_offset + 1],
//                 model.mesh.normals[pos_offset + 2],
//             ),
//         };

//         // model_vertices.push(vertex);
//         // indices.push(indices.len() as u16);

//         if let Some(on_index) = unique_vertices.get(&vertex) {
//             indices.push(*on_index as u16);
//         } else {
//             let index: u16 = model_vertices.len() as u16;
//             unique_vertices.insert(vertex, index);
//             model_vertices.push(vertex);
//             indices.push(index as u16);
//         }
//         indices_count += 1;
//     }
//     meshes.push(MeshObject {
//         first_index,
//         indices_count: indices_count,
//     });
// }

// let mut meshes: Vec<MeshObject> = vec![];

// let (models, materials) = tobj::load_obj(
//     &path,
//     &tobj::LoadOptions {
//         triangulate: true,
//         single_index: true,
//         ..Default::default()
//     },
// )
// .expect("Failed to OBJ load file");

// // Prepare vertex and index data
// let mut vertices = Vec::new();
// let mut indices: Vec<u16> = Vec::new();
// let mut current_index = 0;

// for model in models.iter() {
//     let mesh = &model.mesh;

//     let first_index = current_index;
//     for i in 0..mesh.positions.len() / 3 {
//         let pos = Vec3::new(
//             mesh.positions[3 * i],
//             mesh.positions[3 * i + 1],
//             mesh.positions[3 * i + 2],
//         );
//         let normal = if !mesh.normals.is_empty() {
//             Vec3::new(
//                 mesh.normals[3 * i],
//                 mesh.normals[3 * i + 1],
//                 mesh.normals[3 * i + 2],
//             )
//         } else {
//             Vec3::ZERO
//         };
//         let tex_coord = if !mesh.texcoords.is_empty() {
//             Vec2::new(mesh.texcoords[2 * i], mesh.texcoords[2 * i + 1])
//         } else {
//             Vec2::ZERO
//         };
//         vertices.push(ModelVertex {
//             position: pos,
//             normal,
//             tex_coords: tex_coord,
//         });
//     }

//     for idx in &mesh.indices {
//         indices.push(*idx as u16);
//         current_index += 1;
//     }
//     meshes.push(MeshObject {
//         first_index,
//         indices_count: (mesh.indices.len() as u32),
//     });
// }
