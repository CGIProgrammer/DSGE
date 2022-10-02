use nalgebra::{Vector2, Vector3};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hasher, Hash};
use std::vec::Vec;
use std::fs::{File};
use super::utils::{read_struct};
use std::path::Path;
use super::types::*;
use std::sync::Arc;
use bytemuck::{Pod, Zeroable};
use vulkano::buffer::{BufferUsage, TypedBufferAccess, ImmutableBuffer};
use vulkano::device::{Device, Queue};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CheckDescriptorSetsValidityError, DrawIndirectCommand, DrawIndexedIndirectCommand};

use super::teapot::{INDICES, NORMALS, VERTICES};
use crate::game_object::GOTransformUniform;
pub use crate::references::*;
pub type MeshRef = Arc<dyn MeshView>;
//pub type MeshRef = Arc<Mesh>;

/// Структура вершины
#[derive(Copy, Clone, Default)]
pub struct Vertex {
    pub v_pos: Vec3,  // Координаты вершины
    pub v_nor: Vec3,  // Нормаль вершины
    pub v_bin: Vec3,  // Касательная 1
    pub v_tan: Vec3,  // Касательная 2
    pub v_tex1: Vec2, // Текстурные координаты 1 слой
    pub v_tex2: Vec2, // Текстурные координаты 2 слой
    pub v_grp: UVec3, // Группы першины
}

impl Vertex {
    pub fn to_vk_vertex(&self) -> VkVertex
    {
        VkVertex {
            v_pos   : [self.v_pos.x, self.v_pos.y, self.v_pos.z],
            v_nor   : [self.v_nor.x, self.v_nor.y, self.v_nor.z],
            v_bin   : [self.v_bin.x, self.v_bin.y, self.v_bin.z],
            v_tan   : [self.v_tan.x, self.v_tan.y, self.v_tan.z],
            v_tex1  : [self.v_tex1.x, self.v_tex1.y],
            v_tex2  : [self.v_tex2.x, self.v_tex2.y],
            v_grp   : [self.v_grp.x, self.v_grp.y, self.v_grp.z],
        }
    }
}

/// Представление вершины для vulkano
#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
pub struct VkVertex {
    pub v_pos: [f32; 3],
    pub v_nor: [f32; 3],
    pub v_bin: [f32; 3],
    pub v_tan: [f32; 3],
    pub v_tex1: [f32; 2],
    pub v_tex2: [f32; 2],
    pub v_grp: [u32; 3],
}

impl Hash for VkVertex
{
    fn hash<H: Hasher>(&self, h: &mut H)
    {
        for i in 0..3 {
            h.write_i128((self.v_pos[i]*1000.0) as _);
            h.write_i128((self.v_nor[i]*1000.0) as _);
            h.write_i128((self.v_bin[i]*1000.0) as _);
            h.write_i128((self.v_tan[i]*1000.0) as _);
            h.write_u32(self.v_grp[i] as _);
            if i < 2 {
                h.write_i128((self.v_tex1[i]*1000.0) as _);
                h.write_i128((self.v_tex2[i]*1000.0) as _);
            }
        }
    }
}

#[allow(dead_code)]
impl VkVertex {
    pub fn to_vertex(&self) -> Vertex
    {
        Vertex {
            v_pos   : Vec3::new(self.v_pos[0],  self.v_pos[1], self.v_pos[2]),
            v_nor   : Vec3::new(self.v_nor[0],  self.v_nor[1], self.v_nor[2]),
            v_bin   : Vec3::new(self.v_bin[0],  self.v_bin[1], self.v_bin[2]),
            v_tan   : Vec3::new(self.v_tan[0],  self.v_tan[1], self.v_tan[2]),
            v_tex1  : Vec2::new(self.v_tex1[0], self.v_tex1[1]),
            v_tex2  : Vec2::new(self.v_tex2[0], self.v_tex2[1]),
            v_grp   : UVec3::new(self.v_grp[0], self.v_grp[1], self.v_grp[2])
        }
    }
}

vulkano::impl_vertex!(VkVertex,
    v_pos,
    v_nor,
    v_bin,
    v_tan,
    v_tex1,
    v_tex2,
    v_grp
);

vulkano::impl_vertex!(GOTransformUniform,
    transform,
    transform_prev
);

/// Псевдоним для вершинного буфера
pub type VertexBufferRef = Arc<ImmutableBuffer<[VkVertex]>>;

/// Псевдоним для индексного буфера
pub type IndexBufferRef = Arc<ImmutableBuffer<[u32]>>;

#[allow(dead_code)]
impl Vertex {
    pub fn empty() -> Self
    {
        Self {
            v_pos: Vec3::new(0.0, 0.0, 0.0),
            v_nor: Vec3::new(0.0, 0.0, 0.0),
            v_bin: Vec3::new(0.0, 0.0, 0.0),
            v_tan: Vec3::new(0.0, 0.0, 0.0),
            v_tex1: Vec2::new(0.0, 0.0),
            v_tex2: Vec2::new(0.0, 0.0),
            v_grp: UVec3::new(0, 0, 0)
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct BoundingBox {
    pub begin: Vec3,
    pub end: Vec3
}

#[derive(Default)]
pub struct MeshBuilder
{
    _name: String,
    _indices: Vec<u32>,
    _vertices: Vec<VkVertex>,
    _vertex_buffer: Option<VertexBufferRef>,
    _index_buffer : Option<IndexBufferRef>
}

impl Hash for MeshBuilder
{
    fn hash<H: Hasher>(&self, hasher: &mut H)
    {
        self._vertices.hash(hasher);
        self._indices.hash(hasher);
    }
}

#[allow(dead_code)]
impl MeshBuilder
{
    pub fn push_quad_coords(&mut self, a: &Vec3, b: &Vec3, c: &Vec3, d: &Vec3) -> &mut Self
    {
        self
            .push_triangle_coords(a, b, c)
            .push_triangle_coords(a, c, d)
    }

    pub fn push_triangle_coords(&mut self, a: &Vec3, b: &Vec3, c: &Vec3) -> &mut Self
    {
        let dpos1 = b - a;
        let dpos2 = c - a;
        let normal = dpos1.cross(&dpos2).normalize();
        let tangent = dpos1.normalize();
        let bitan = normal.cross(&tangent);
        let vert_a = Vertex {
            v_pos : a.clone(),
            v_nor : normal,
            v_bin : tangent,
            v_tan : bitan,
            v_tex1 : Vector2::new(0.0, 0.0),
            v_tex2 : Vector2::new(0.0, 0.0),
            v_grp : Vector3::new(0, 0, 0)
        };
        let vert_b = Vertex {
            v_pos : b.clone(),
            v_nor : normal,
            v_bin : tangent,
            v_tan : bitan,
            v_tex1 : Vector2::new(0.0, 0.0),
            v_tex2 : Vector2::new(0.0, 0.0),
            v_grp : Vector3::new(0, 0, 0)
        };
        let vert_c = Vertex {
            v_pos : c.clone(),
            v_nor : normal,
            v_bin : tangent,
            v_tan : bitan,
            v_tex1 : Vector2::new(0.0, 0.0),
            v_tex2 : Vector2::new(0.0, 0.0),
            v_grp : Vector3::new(0, 0, 0)
        };
        self._indices.push(self._indices.len() as u32);
        self._vertices.push(vert_a.to_vk_vertex());
        self._indices.push(self._indices.len() as u32);
        self._vertices.push(vert_b.to_vk_vertex());
        self._indices.push(self._indices.len() as u32);
        self._vertices.push(vert_c.to_vk_vertex());
        self
    }

    pub fn calc_tangent_space(&mut self) -> &mut Self
    {
        let mut smoth_vertices = Vec::<u32>::new();
        for i in 0..self._indices.len()/3
        {
            let vert_a = self._vertices[i*3+0].to_vertex();
            let vert_b = self._vertices[i*3+1].to_vertex();
            let vert_c = self._vertices[i*3+2].to_vertex();
            let edge1 = vert_b.v_pos - vert_a.v_pos;
            let edge2 = vert_c.v_pos - vert_a.v_pos;
            let duv1 = vert_b.v_tex1 - vert_a.v_tex1;
            let duv2 = vert_c.v_tex1 - vert_a.v_tex1;
            let normal = edge1.cross(&edge2).normalize();
            let f = 1.0 / (duv1.x * duv2.y - duv2.x * duv1.y);
            let tangent = Vector3::<f32>::new(
                f * (duv2.y * edge1.x - duv1.y * edge2.x),
                f * (duv2.y * edge1.y - duv1.y * edge2.y),
                f * (duv2.y * edge1.z - duv1.y * edge2.z)
            ).normalize();
            let bitangent = Vector3::<f32>::new(
                f * (-duv2.y * edge1.x + duv1.y * edge2.x),
                f * (-duv2.y * edge1.y + duv1.y * edge2.y),
                f * (-duv2.y * edge1.z + duv1.y * edge2.z)
            ).normalize();
            for j in 0..3 {
                let mut v = self._vertices[i].to_vertex();
                let index = (i*3+j) as usize;
                smoth_vertices[index] += 1;
                v.v_nor += normal;
                v.v_tan += tangent;
                v.v_bin += bitangent;
                self._vertices[index] = v.to_vk_vertex();
            }
        }
        for i in 0..smoth_vertices.len()
        {
            let mut v = self._vertices[i].to_vertex();
            v.v_nor = (v.v_nor / smoth_vertices[i] as f32).normalize();
            v.v_tan = (v.v_tan / smoth_vertices[i] as f32).normalize();
            v.v_bin = (v.v_bin / smoth_vertices[i] as f32).normalize();
            self._vertices[i] = v.to_vk_vertex();
        }
        self
    }

    pub fn push_from_file(&mut self, fname: &str) -> Result<(u32, u32), String>
    {
        let path = Path::new(fname);
        let mut file = match File::open(path)
        {
            Ok(file) => file,
            Err(error) => return Err(format!("Ошибка загрузки файла \"{}\": {:?}", fname, error))
        };
        read_struct::<f64, File>(&mut file).unwrap();
        let deformed = read_struct::<bool, File>(&mut file).unwrap();
        let uv_count = (if read_struct::<bool, File>(&mut file).unwrap() {2} else {1}) as u8;
        let ind_count = read_struct::<u32, File>(&mut file).unwrap() as usize;
        let base_index = self._indices.len();
        let base_vertex = self._vertices.len();;
        for _ in 0..ind_count {
            let index = read_struct::<u32, File>(&mut file).unwrap();
            self._indices.push(base_vertex as u32 + index as u32);
        }
        let vert_count = read_struct::<u32, File>(&mut file).unwrap() as usize;
        let mut bbox_begin = Vector3::<f32>::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut bbox_end = Vector3::<f32>::new(-f32::INFINITY, -f32::INFINITY, -f32::INFINITY);

        for _i in 0..(vert_count as usize) {
            //print!("\r                             \r{}", _i);
            let vertex = Vertex{
                v_pos : read_struct(&mut file).unwrap(),
                v_nor : read_struct(&mut file).unwrap(),
                v_tex1: read_struct(&mut file).unwrap(),
                v_bin : read_struct(&mut file).unwrap(),
                v_tan : read_struct(&mut file).unwrap(),
                v_grp : if deformed {
                    read_struct::<Vector3<u32>, File>(&mut file).unwrap()
                } else {
                    Vector3::<u32>::new(0, 0, 0)
                },
                v_tex2: if uv_count==2 {
                    read_struct::<Vector2<f32>, File>(&mut file).unwrap()
                } else {
                    Vector2::<f32>::new(0.0, 0.0)
                }
            };
            if vertex.v_pos.x < bbox_begin.x { bbox_begin.x = vertex.v_pos.x; }
            if vertex.v_pos.y < bbox_begin.y { bbox_begin.y = vertex.v_pos.y; }
            if vertex.v_pos.z < bbox_begin.z { bbox_begin.z = vertex.v_pos.z; }
            if vertex.v_pos.x > bbox_end.x { bbox_end.x = vertex.v_pos.x; }
            if vertex.v_pos.y > bbox_end.y { bbox_end.y = vertex.v_pos.y; }
            if vertex.v_pos.z > bbox_end.z { bbox_end.z = vertex.v_pos.z; }
            self._vertices.push(vertex.to_vk_vertex());
        };
        Ok((base_index as _, ind_count as _))
    }

    /// Добавить чайник из Юты
    pub fn push_teapot(&mut self) -> &mut Self
    {
        let rot = nalgebra::Rotation3::<f32>::from_euler_angles(0.0, 0.0, std::f32::consts::PI);
        let mat = rot.matrix();
        let self_ind_count = self._indices.len();
        let mut max_x = -9999.0;
        let mut max_y = -9999.0;
        let mut min_x = 9999.0;
        let mut min_y = 9999.0;
        for vert in &VERTICES {
            if vert.position[0] > max_x {
                max_x = vert.position[0];
            }
            if vert.position[1] > max_y {
                max_y = vert.position[1];
            }
            if vert.position[0] < min_x {
                min_x = vert.position[0];
            }
            if vert.position[1] < min_y {
                min_y = vert.position[1];
            }
        }
        min_x /= 100.0;
        min_y /= 100.0;
        max_x /= 100.0;
        max_y /= 100.0;
        println!("min {}, {}", min_x, min_y);
        println!("max {}, {}", max_x, max_y);
        for i in 0..VERTICES.len() {
            let pos = Vec3::new(VERTICES[i].position[0] / 100.0, VERTICES[i].position[1] / 100.0, VERTICES[i].position[2] / 100.0);
            let nor = Vec3::new(NORMALS[i].normal[0], NORMALS[i].normal[1], NORMALS[i].normal[2]);
            let vert = Vertex{
                v_pos: mat * pos,
                v_nor: mat * nor,
                v_tan: Vec3::new(0.0, 0.0, 0.0),
                v_bin: Vec3::new(0.0, 0.0, 0.0),
                v_tex1: Vec2::new((pos.x-min_x) / (max_x - min_x), 1.0 - (pos.y-min_y) / (max_y - min_y)),
                v_tex2: Vec2::new(0.0, 0.0),
                v_grp: UVec3::new(0, 0, 0),
            };
            self._vertices.push(vert.to_vk_vertex());
        }
        for i in INDICES {
            self._indices.push(self_ind_count as u32 + i);
        }
        self
    }

    /// Добавить плоскость
    pub fn push_screen_plane(&mut self) -> &mut Self
    {
        self.push_triangle_coords(
            &Vec3::new(-1.0, -1.0, 0.0),
            &Vec3::new(-1.0,  1.0, 0.0),
            &Vec3::new( 1.0,  1.0, 0.0))
            .push_triangle_coords(
            &Vec3::new(-1.0, -1.0, 0.0),
            &Vec3::new( 1.0,  1.0, 0.0),
            &Vec3::new( 1.0, -1.0, 0.0))
    }

    pub fn build_mutex(self, queue: Arc<Queue>) -> Result<MeshRef, String>
    {
        Ok(Arc::new(self.build(queue)?))
    }

    pub fn build(mut self, queue: Arc<Queue>) -> Result<Mesh, String>
    {
        let mut hasher = DefaultHasher::default();
        self._vertices.hash(&mut hasher);
        self._indices.hash(&mut hasher);
        let hash = hasher.finish();
        self._vertex_buffer = Some(ImmutableBuffer::from_iter(self._vertices.clone(), BufferUsage::vertex_buffer(), queue.clone()).unwrap().0);
        self._index_buffer  = Some(ImmutableBuffer::from_iter(self._indices.clone(), BufferUsage::index_buffer(), queue.clone()).unwrap().0);

        let mesh = Mesh {
            name: self._name.clone(),
            device: queue.device().clone(),
            deformed: false,
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_buffer : self._vertex_buffer.as_ref().unwrap().clone(),
            index_buffer : self._index_buffer.as_ref().unwrap().clone(),
            index_count: self._indices.len() as _,
            uv_count: 1,
            hash: hash,
            bbox: BoundingBox::default()
        };
        Ok(mesh)
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Mesh {
    name: String,
    hash: u64,
    indices: Vec<u32>,
    vertices: Vec<VkVertex>,
    device: Arc<Device>,
    vertex_buffer: VertexBufferRef,
    index_buffer : IndexBufferRef,
    index_count : usize,
    pub bbox: BoundingBox,
    deformed: bool,
    uv_count: u8
}

#[allow(dead_code)]
impl Mesh {
    
    pub fn name(&self) -> &String
    {
        &self.name
    }

    pub fn builder(name: &str) -> MeshBuilder
    {
        let mut res = MeshBuilder::default();
        res._name = name.to_owned();
        res
    }

    pub fn make_screen_plane(queue: Arc<Queue>) -> Result<MeshRef, String>
    {
        let mut plane = Mesh::builder("screen_plane");
        plane
            .push_triangle_coords(
                &Vec3::new(-1.0, -1.0, 0.0),
                &Vec3::new(-1.0,  1.0, 0.0),
                &Vec3::new( 1.0,  1.0, 0.0)
            )
            .push_triangle_coords(
                &Vec3::new(-1.0, -1.0, 0.0),
                &Vec3::new( 1.0,  1.0, 0.0),
                &Vec3::new( 1.0, -1.0, 0.0)
            );
        plane.build_mutex(queue)
    }

    pub fn make_cube(name : &str, queue: Arc<Queue>) -> Result<MeshRef, String>
    {
        let mut cube = Mesh::builder(name);
        cube.push_quad_coords(
            &Vec3::new(-1.0, -1.0,  1.0),
            &Vec3::new(-1.0,  1.0,  1.0),
            &Vec3::new( 1.0,  1.0,  1.0),
            &Vec3::new( 1.0, -1.0,  1.0)
        )
        .push_quad_coords(
            &Vec3::new( 1.0, -1.0, -1.0),
            &Vec3::new( 1.0,  1.0, -1.0),
            &Vec3::new(-1.0,  1.0, -1.0),
            &Vec3::new(-1.0, -1.0, -1.0),
        )
        .push_quad_coords(
            &Vec3::new(-1.0, -1.0, -1.0),
            &Vec3::new(-1.0, -1.0,  1.0),
            &Vec3::new( 1.0, -1.0,  1.0),
            &Vec3::new( 1.0, -1.0, -1.0)
        )
        .push_quad_coords(
            &Vec3::new( 1.0,  1.0, -1.0),
            &Vec3::new( 1.0,  1.0,  1.0),
            &Vec3::new(-1.0,  1.0,  1.0),
            &Vec3::new(-1.0,  1.0, -1.0),
        )
        .push_quad_coords(
            &Vec3::new(1.0, -1.0, -1.0),
            &Vec3::new(1.0, -1.0,  1.0),
            &Vec3::new(1.0,  1.0,  1.0),
            &Vec3::new(1.0,  1.0, -1.0)
        )
        .push_quad_coords(
            &Vec3::new(-1.0,  1.0, -1.0),
            &Vec3::new(-1.0,  1.0,  1.0),
            &Vec3::new(-1.0, -1.0,  1.0),
            &Vec3::new(-1.0, -1.0, -1.0),
        );
        cube.build_mutex(queue)
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }
}

pub struct SubMesh
{
    name: String,
    mesh: Arc<Mesh>,
    base_index: u32,
    index_count: u32,
    index_offset: u32,
}

impl SubMesh { 
    pub fn from_mesh(name : String, mesh : &Mesh, base : u32, count : u32, offset: u32) -> Arc<dyn MeshView> {
        Arc::new(SubMesh {
            name : name,
            mesh : Arc::new(mesh.clone()),
            base_index : base,
            index_count : count,
            index_offset : offset
        })
    }
}

unsafe impl Send for SubMesh {}
unsafe impl Sync for SubMesh {}

pub trait MeshView: Send + Sync + 'static
{
    fn name(&self) -> &String;
    fn indirect_command(&self, first_instance: u32, instance_count: u32) -> DrawIndexedIndirectCommand;
    fn vertex_buffer(&self) -> &VertexBufferRef;
    fn index_buffer(&self) -> &IndexBufferRef;
    fn base_index(&self) -> u32;
    fn vertex_offset(&self) -> u32;
    fn buffer_id(&self) -> u32;
    fn as_buffer(&self) -> Option<&Mesh>;
    #[inline(always)]
    fn ref_id(&self) -> i32
    {
        self as *const Self as *const i32 as _
    }
}

impl MeshView for Mesh
{
    fn as_buffer(&self) -> Option<&Mesh> {
        Some(self)
    }

    #[inline(always)]
    fn name(&self) -> &String {
        &self.name
    }

    #[inline(always)]
    fn indirect_command(&self, first_instance: u32, instance_count: u32) -> DrawIndexedIndirectCommand {
        DrawIndexedIndirectCommand {
            index_count : self.index_buffer.len() as _,
            instance_count : instance_count,
            first_index : 0,
            vertex_offset : 0,
            first_instance : first_instance
        }
    }

    #[inline(always)]
    fn vertex_buffer(&self) -> &VertexBufferRef
    {
        &self.vertex_buffer
    }

    #[inline(always)]
    fn index_buffer(&self) -> &IndexBufferRef
    {
        &self.index_buffer
    }

    #[inline(always)]
    fn base_index(&self) -> u32 {
        0
    }

    #[inline(always)]
    fn vertex_offset(&self) -> u32 {
        0
    }

    #[inline(always)]
    fn buffer_id(&self) -> u32 {
        let ib = self.index_buffer.as_ref() as *const _ as u32;
        let vb = self.vertex_buffer.as_ref() as *const _ as u32;
        vb ^ ib
        
    }
}

impl MeshView for SubMesh
{
    fn as_buffer(&self) -> Option<&Mesh> {
        None
    }

    #[inline(always)]
    fn name(&self) -> &String {
        &self.name
    }

    #[inline(always)]
    fn indirect_command(&self, first_instance: u32, instance_count: u32) -> DrawIndexedIndirectCommand {
        DrawIndexedIndirectCommand {
            index_count : self.index_count,
            instance_count : instance_count,
            first_index : self.base_index,
            vertex_offset : self.index_offset,
            first_instance : first_instance
        }
    }

    #[inline(always)]
    fn vertex_buffer(&self) -> &VertexBufferRef
    {
        &self.mesh.vertex_buffer
    }

    #[inline(always)]
    fn index_buffer(&self) -> &IndexBufferRef
    {
        &self.mesh.index_buffer
    }

    #[inline(always)]
    fn base_index(&self) -> u32 {
        self.base_index
    }

    #[inline(always)]
    fn vertex_offset(&self) -> u32 {
        self.index_offset
    }

    #[inline(always)]
    fn buffer_id(&self) -> u32 {
        self.mesh.buffer_id()
    }
}

pub trait MeshCommandSet
{
    fn bind_mesh(&mut self, mesh: &dyn MeshView) -> Result<&mut Self, String>;
    fn draw_mesh(&mut self, mesh: &dyn MeshView) -> Result<&mut Self, String>;
}

use vulkano::command_buffer::DrawIndexedError;

impl <T>MeshCommandSet for AutoCommandBufferBuilder<T>
{
    #[inline(always)]
    fn bind_mesh(&mut self, mesh: &dyn MeshView) -> Result<&mut Self, String>
    {
        let vbo = mesh.vertex_buffer();
        let ibo = mesh.index_buffer();
        let result = self
            .bind_vertex_buffers(0, vbo.clone())
            .bind_index_buffer(ibo.clone())
            .draw_indexed(ibo.len() as u32, 1, 0, 0, 0, true);

        match result {
            Ok(slf) => Ok(slf),
            Err(derr) => 
                match derr {
                    DrawIndexedError::CheckDescriptorSetsValidityError(err) => 
                    match err {
                        CheckDescriptorSetsValidityError::InvalidDescriptorResource {
                            set_num,
                            binding_num,
                            index,
                            ..
                        } => Err(format!("Uniform-переменная {{set_num: {}, binding_num: {}, index: {}}} не передана в шейдер", set_num, binding_num, index)),
                        
                        CheckDescriptorSetsValidityError::MissingDescriptorSet{set_num} => 
                            Err(format!("Набор uniform-переменных №{} не передан в шейдер", set_num)),
                        
                        _ => Err(format!("Ошибка uniform-переменных: {:?}", err))
                    },
                    _ => Err(format!("Ошибка отображения полигональной сетки: {:?}", derr))
                }
        }
    }

    #[inline(always)]
    fn draw_mesh(&mut self, mesh: &dyn MeshView) -> Result<&mut Self, String>
    {
        //let vbo = mesh.vertex_buffer().unwrap();
        let result = self.draw_indexed(mesh.index_buffer().len() as u32, 1, 0, 0, 0, false);

        match result {
            Ok(slf) => Ok(slf),
            Err(derr) => 
                match derr {
                    DrawIndexedError::CheckDescriptorSetsValidityError(err) => 
                    match err {
                        CheckDescriptorSetsValidityError::InvalidDescriptorResource {
                            set_num,
                            binding_num,
                            index,
                            ..
                        } => Err(format!("Uniform-переменная {{set_num: {}, binding_num: {}, index: {}}} не передана в шейдер", set_num, binding_num, index)),
                        _ => Err(format!("Ошибка uniform-переменных: {:?}", err))
                    },
                    _ => Err(format!("Ошибка отображения полигональной сетки: {:?}", derr))
                }
        }
    }
}