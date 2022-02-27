use nalgebra::{Vector2, Vector3};
use std::vec::Vec;
use std::fs::{File};
use super::utils::{read_struct};
use std::path::Path;
use super::types::*;
use std::sync::Arc;

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::device::Device;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

use super::teapot::{INDICES, NORMALS, VERTICES};
pub use crate::references::*;
pub type MeshRef = RcBox<Mesh>;

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
#[derive(Copy, Clone, Default)]
pub struct VkVertex {
    pub v_pos: [f32; 3],
    pub v_nor: [f32; 3],
    pub v_bin: [f32; 3],
    pub v_tan: [f32; 3],
    pub v_tex1: [f32; 2],
    pub v_tex2: [f32; 2],
    pub v_grp: [u32; 3],
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
    v_grp);

/// Псевдоним для вершинного буфера
type VertexBufferRef = Arc<CpuAccessibleBuffer<[VkVertex]>>;

/// Псевдоним для индексного буфера
type IndexBufferRef = Arc<CpuAccessibleBuffer<[u32]>>;

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

    pub fn push_from_file(&mut self, fname: &str) -> &mut Self
    {
        let path = Path::new(fname);
        let mut file = File::open(path).unwrap();
        read_struct::<f64, File>(&mut file).unwrap();
        let deformed = read_struct::<bool, File>(&mut file).unwrap();
        let uv_count = (if read_struct::<bool, File>(&mut file).unwrap() {2} else {1}) as u8;
        let ind_count = read_struct::<u32, File>(&mut file).unwrap() as usize;
        let self_ind_count = self._indices.len();
        for _ in 0..ind_count {
            let index = read_struct::<u32, File>(&mut file).unwrap();
            self._indices.push(self_ind_count as u32 + index as u32);
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
        self
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
            if vert.position.0 > max_x {
                max_x = vert.position.0;
            }
            if vert.position.1 > max_y {
                max_y = vert.position.1;
            }
            if vert.position.0 < min_x {
                min_x = vert.position.0;
            }
            if vert.position.1 < min_y {
                min_y = vert.position.1;
            }
        }
        min_x /= 100.0;
        min_y /= 100.0;
        max_x /= 100.0;
        max_y /= 100.0;
        println!("min {}, {}", min_x, min_y);
        println!("max {}, {}", max_x, max_y);
        for i in 0..VERTICES.len() {
            let pos = Vec3::new(VERTICES[i].position.0 / 100.0, VERTICES[i].position.1 / 100.0, VERTICES[i].position.2 / 100.0);
            let nor = Vec3::new(NORMALS[i].normal.0, NORMALS[i].normal.1, NORMALS[i].normal.2);
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
            &Vec3::new( 1.0,  1.0, 0.0)
        )
            .push_triangle_coords(
                &Vec3::new(-1.0, -1.0, 0.0),
                &Vec3::new( 1.0,  1.0, 0.0),
                &Vec3::new( 1.0, -1.0, 0.0)
            )
    }

    pub fn build_mutex(self, device: Arc<Device>) -> Result<MeshRef, String>
    {
        Ok(MeshRef::construct(self.build(device)?))
    }

    pub fn build(mut self, device: Arc<Device>) -> Result<Mesh, String>
    {
        self._vertex_buffer = Some(CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, self._vertices.clone()).unwrap());
        self._index_buffer  = Some(CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, self._indices.clone()).unwrap());

        let mesh = Mesh {
            name: self._name.clone(),
            device: device,
            deformed: false,
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_buffer : self._vertex_buffer.clone(),
            index_buffer : self._index_buffer.clone(),
            uv_count: 1,
            bbox: BoundingBox::default()
        };
        Ok(mesh)
    }
}

#[allow(dead_code)]
pub struct Mesh {
    name: String,
    indices: Vec<u32>,
    vertices: Vec<VkVertex>,
    device: Arc<Device>,
    vertex_buffer: Option<VertexBufferRef>,
    index_buffer : Option<IndexBufferRef>,
    pub bbox: BoundingBox,
    deformed: bool,
    uv_count: u8
}

#[allow(dead_code)]
impl Mesh {
    
    pub fn builder(name: &str) -> MeshBuilder
    {
        let mut res = MeshBuilder::default();
        res._name = name.to_string();
        res
    }

    pub fn make_screen_plane(device: Arc<Device>) -> Result<MeshRef, String>
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
        plane.build_mutex(device)
    }

    pub fn make_cube(name : &str, device: Arc<Device>) -> Result<MeshRef, String>
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
        cube.build_mutex(device)
    }

    pub fn vertex_buffer(&self) -> Option<VertexBufferRef>
    {
        self.vertex_buffer.clone()
    }

    pub fn index_buffer(&self) -> Option<IndexBufferRef>
    {
        self.index_buffer.clone()
    }
}

pub trait MeshBinder
{
    fn bind_mesh(&mut self, mesh: &Mesh) -> Result<&mut Self, String>;
}

use vulkano::command_buffer::validity::*;
use vulkano::command_buffer::DrawIndexedError;

impl MeshBinder for AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>
{
    fn bind_mesh(&mut self, mesh: &Mesh) -> Result<&mut Self, String>
    {
        let vbo = mesh.vertex_buffer().unwrap();
        let ibo = mesh.index_buffer().unwrap();
        let result = self
            .bind_vertex_buffers(0, vbo.clone())
            .bind_index_buffer(ibo.clone())
            .draw_indexed(ibo.len() as u32, 1, 0, 0, 0);

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