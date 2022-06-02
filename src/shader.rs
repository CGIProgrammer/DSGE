use std::collections::{HashMap, HashSet};
pub use super::glenums::{ShaderType, GLSLVersion, GLSLType, AttribType};
use std::sync::Arc;
use vulkano::pipeline::graphics::rasterization::{RasterizationState, CullMode};
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::descriptor_set::{PersistentDescriptorSet};
use vulkano::pipeline::{GraphicsPipeline, ComputePipeline, PipelineLayout};
use vulkano::render_pass::{RenderPass, Subpass};
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::buffer::{CpuBufferPool, CpuAccessibleBuffer, BufferUsage};
use vulkano::pipeline::PipelineBindPoint;

use crate::references::*;
use crate::texture::{TextureView, TextureViewGlsl, Texture};
use crate::vulkano::pipeline::Pipeline as VkPipeline;
use vulkano::device::Device;
use bytemuck::Pod;

use std::path::Path;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::io::prelude::*;

pub type ShaderProgramRef = RcBox<ShaderProgram>;

/// Trait для единообразной передачи uniform-структур и текстур в GLSL шейдер.
/// Он же используется для сборки GLSL шейдера
pub trait ShaderStructUniform
{
    fn structure() -> String;       // Должна возвращать текстовое представление структуры типа для GLSL
    fn glsl_type_name() -> String;  // Должна возвращать название типа
    fn texture(&self) -> Option<&crate::texture::Texture>; // Позволяет получить текстуру, если структура является таковой
}

#[derive(Clone)]
struct ShaderSourceUniform
{
    name: String,
    type_name: String,
    set: usize,
    binding: usize,
}

/// Структура для построения GLSL шейдера и компиляции его в SPIR-V
pub struct Shader
{
    glsl_version : GLSLVersion,
    sh_type      : ShaderType,
    device       : Arc<vulkano::device::Device>,
    module       : Option<Arc<vulkano::shader::ShaderModule>>,
    source       : String,
    inputs       : HashMap<String, AttribType>,
    outputs      : HashMap<String, AttribType>,
    uniforms     : HashMap<String, ShaderSourceUniform>,
    push_constants : Option<(String, ShaderSourceUniform)>
}

impl std::fmt::Debug for Shader
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut outputs_list = Vec::new();
        let mut inputs_list = Vec::new();
        for i in &self.inputs {
            inputs_list.push(format!("{}: {}", i.0, i.1));
        }
        for i in &self.outputs {
            outputs_list.push(format!("{}: {}", i.0, i.1));
        }
        f.debug_struct("Shader")
         .field("glsl_version", &self.glsl_version)
         .field("inputs", &inputs_list.join(", "))
         .field("outputs", &outputs_list.join(", "))
         .finish()
    }
}

fn _convert_vec_types(mut vec8: Vec<u8>) -> Vec<u32>
{
    // I copy-pasted this code from StackOverflow without reading the answer 
    // surrounding it that told me to write a comment explaining why this code 
    // is actually safe for my own use case.
    unsafe {
        let ratio = std::mem::size_of::<u32>() / std::mem::size_of::<u8>();

        let length = vec8.len() / ratio;
        let capacity = vec8.capacity() / ratio;
        let ptr = vec8.as_mut_ptr() as *mut u32;

        // Don't run the destructor for vec32
        std::mem::forget(vec8);

        // Construct new Vec
        Vec::from_raw_parts(ptr, length, capacity)
    }
}

#[allow(dead_code)]
impl Shader
{
    pub fn get_source(&self) -> String
    {
        self.source.clone()
    }

    /// Построить щейдер указанного типа
    pub fn builder(shader_type: ShaderType, device: Arc<vulkano::device::Device>) -> Self
    {
        let gl_version = GLSLVersion::V450;
        let mut source = gl_version.stringify() + "\n";
        
        if gl_version.need_precision_qualifier() {
            source += "precision mediump float;\n";
        }
        
        Self {
            glsl_version : gl_version,
            sh_type  : shader_type,
            module   : None,
            device   : device,
            source   : source,
            inputs   : HashMap::new(),
            outputs  : HashMap::new(),
            uniforms : HashMap::new(),
            push_constants : None
        }
    }

    /// Атрибуты вершин по умолчанию
    pub fn default_vertex_attributes(&mut self) -> &mut Self
    {
        self
            .input("v_pos", AttribType::FVec3)
            .input("v_nor", AttribType::FVec3)
            .input("v_bin", AttribType::FVec3)
            .input("v_tan", AttribType::FVec3)
            .input("v_tex1", AttribType::FVec2)
            .input("v_tex2", AttribType::FVec2)
            .input("v_grp", AttribType::UVec3)
    }

    fn last_set_index(&self, set: usize) -> usize
    {
        match self.uniforms.values().filter(|val| {val.set == set}).max_by_key(|val1| { val1.binding })
        {
            Some(ShaderSourceUniform{binding,..}) => *binding+1,
            None => 0
        }
    }

    /// Объявляет буфер-хранилище
    pub fn storage_buffer<T: ShaderStructUniform>(&mut self, name: &str, set: usize, binding : usize) -> Result<&mut Self, String>
    {
        let name = name.to_string();
        if self.uniforms.contains_key(&name) {
            return Err(format!("Переменная {} уже объявлена в этом шейдере.", name));
        }
        let uniform_source = ShaderSourceUniform{
            name: name.clone(),
            type_name: T::glsl_type_name(),
            set: set,
            binding: binding
        };
        match self.uniforms.get(&uniform_source.name)
        {
            Some(ShaderSourceUniform{type_name,..}) => return Err(format!("Переменная с именем {} уже есть в этом шейдере.", type_name)),
            None => ()
        };
        self.source += format!("layout (std140, set = {}, binding = {}) readonly buffer {} {} {};\n", set, binding, uniform_source.type_name, T::structure(), name).as_str();
        self.uniforms.insert(uniform_source.name.clone(), uniform_source);
        Ok(self)
    }

    pub fn storage_buffer_autoincrement<T: ShaderStructUniform>(&mut self, name: &str, set: usize) -> Result<&mut Self, String>
    {
        let binding = self.last_set_index(set);
        self.storage_buffer::<T>(name, set, binding)
    }

    /// Объявление uniform-структуры
    pub fn uniform<T: ShaderStructUniform>(&mut self, name: &str, set: usize, binding : usize) -> Result<&mut Self, String>
    {
        let name = name.to_string();
        if self.uniforms.contains_key(&name) {
            return Err(format!("Переменная {} уже объявлена в этом шейдере.", name));
        }
        let uniform_source = ShaderSourceUniform{
            name: name.clone(),
            type_name: T::glsl_type_name(),
            set: set,
            binding: binding
        };
        self.source += format!("layout (std140, set = {}, binding = {}) uniform {} {} {};\n", set, binding, uniform_source.type_name, T::structure(), name).as_str();
        self.uniforms.insert(name.clone(), uniform_source);
        Ok(self)
    }

    /// Объявление постоянной uniform-структуры
    pub fn uniform_constant<T: ShaderStructUniform>(&mut self, name: &str) -> Result<&mut Self, String>
    {
        if self.push_constants.is_some() {
            return Err(format!("Можно объявить только одну константу."));
        }
        let name = name.to_string();
        
        let uniform_source = ShaderSourceUniform{
            name: name.clone(),
            type_name: T::glsl_type_name(),
            set: 0,
            binding: 0
        };
        self.source += format!("layout (push_constant) uniform {} {} {};\n", uniform_source.type_name, T::structure(), name).as_str();
        self.push_constants = Some((name, uniform_source));
        Ok(self)
    }

    /// Объявление uniform-структуры с автоопределением расположения
    pub fn uniform_autoincrement<T: ShaderStructUniform>(&mut self, name: &str, set: usize) -> Result<&mut Self, String>
    {
        let binding = self.last_set_index(set);
        self.uniform::<T>(name, set, binding)
    }

    /// Объявление uniform-структуры с явной передачей кода структуры
    pub fn uniform_structure(&mut self, name: &str, _type: &str, structure: &str, set: usize, binding: usize) -> Result<&mut Self, String>
    {
        let name = name.to_string();
        if self.uniforms.contains_key(&name) {
            return Err(format!("Переменная {} уже объявлена в этом шейдере.", name));
        }
        let uniform_source = ShaderSourceUniform{
            name: name.clone(),
            type_name: _type.to_string(),
            set: set,
            binding: binding
        };
        self.source += format!("layout (std140, set = {}, binding = {}) uniform {} {} {};\n", set, binding, _type, structure, name).as_str();
        self.uniforms.insert(name.to_string(), uniform_source);
        Ok(self)
    }

    /// Объявление uniform-структуры с явной передачей кода структуры и автоопределением расположения
    pub fn uniform_structure_autoincrement(&mut self, name: &str, _type: &str, structure: &str, set: usize) -> Result<&mut Self, String>
    {
        let binding = self.last_set_index(set);
        self.uniform_structure(name, _type, structure, set, binding)
    }

    pub fn uniform_sampler(&mut self, name: &str, set: usize, binding: usize, dims: TextureView) -> Result<&mut Self, String>
    {
        let name = name.to_string();
        if self.uniforms.contains_key(&name) {
            return Err(format!("Переменная {} уже объявлена в этом шейдере.", name));
        }
        
        let utype = dims.glsl_sampler_name().to_string();
        let uniform_source = ShaderSourceUniform{
            name: name.clone(),
            type_name: utype.clone(),
            set: set,
            binding: binding
        };
        self.source += format!("layout (set = {}, binding = {}) uniform {} {};\n", set, binding, utype, name).as_str();
        self.uniforms.insert(name, uniform_source);
        Ok(self)
    }

    pub fn uniform_sampler_autoincrement(&mut self, name: &str, set: usize, dims: TextureView) -> Result<&mut Self, String>
    {
        let binding = self.last_set_index(set);
        self.uniform_sampler(name, set, binding, dims)
    }

    /// Объявляет выход шейдера
    pub fn output(&mut self, name: &str, type_: AttribType) -> &mut Self
    {
        let layout_location = 
            if self.glsl_version.have_explicit_attri_location() {
                format!("layout (location = {}) ", self.outputs.len()).to_string()
            } else {
                String::new()
            };
        self.outputs.insert(name.to_string(), type_);
        self.source += format!("{}out {} {};\n", layout_location, type_.get_glsl_name(), name).as_str();
        self
    }

    /// Объявляет вход шейдера
    pub fn input(&mut self, name: &str, type_: AttribType) -> &mut Self
    {
        let layout_location = 
            if self.glsl_version.have_explicit_attri_location() {
                format!("layout (location = {}) ", self.inputs.len()).to_string()
            } else {
                String::new()
            };
        self.inputs.insert(name.to_string(), type_);
        self.source += format!("{}in {} {};\n", layout_location, type_.get_glsl_name(), name).as_str();
        self
    }

    /// Добавляет код
    pub fn code(&mut self, code: &str) -> &mut Self
    {   
        self.source += code;
        self.source += "\n";
        self
    }

    /// Объявляет макрос-переменную
    pub fn define(&mut self, name: &str, expr: &str) -> &mut Self
    {   
        self.code(format!("#define {} ({})", name, expr).as_str())
    }

    pub fn hash(&self) -> u64
    {
        let mut hasher = DefaultHasher::default();
        self.source.hash(&mut hasher);
        hasher.finish()
    }

    /// Строит шейдер.
    /// Здесь GLSL код компилируется в SPIR-V.
    pub fn build(&mut self) -> Result<&Self, String>
    {
        let hash = self.hash();
        let fname = format!("./shader_cache/{:X}.spv", hash);
        let cache_path = Path::new("./shader_cache/");
        let spv_path = Path::new(fname.as_str());
        
        if !cache_path.is_dir() {
            std::fs::create_dir(cache_path).unwrap();
        }

        if !spv_path.is_file() {
            let sh_type = match self.sh_type {
                ShaderType::Vertex => "vertex",
                ShaderType::Fragment => "fragment",
                ShaderType::TesselationControl => "tess_control",
                ShaderType::TesselationEval => "tess_evaluation",
                ShaderType::Compute => "compute",
                ShaderType::Geometry => "geometry",
            };
            
            let source = self.source.as_str().as_bytes();
            #[cfg(target_os = "windows")]
            let glslc = ".\\vulkan_sdk\\Bin\\glslc.exe";
            #[cfg(target_os = "linux")]
            let glslc = "glslc";
            let mut child_compiler = std::process::Command::new(glslc)
                .args([
                    format!("-fshader-stage={}", sh_type).as_str(),
                    "-fauto-bind-uniforms",
                    "-fauto-map-locations",
                    "--target-env=vulkan1.2",
                    "--target-spv=spv1.5",
                    "-fentry-point=main",
                    "-",
                    "-O0",
                    "-o", spv_path.to_str().unwrap()
                ])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();

            let mut spirv_log = String::new();
            {
                let compiler_stdin = child_compiler.stdin.as_mut().unwrap();
                compiler_stdin.write_all(source).unwrap();
            }
            match child_compiler.wait()
            {
                Ok(status) => {
                    if status.code().unwrap() != 0 {
                        let compiler_stdout = child_compiler.stderr.as_mut().unwrap();
                        compiler_stdout.read_to_string(&mut spirv_log).unwrap();
                        let mut numbered_src = String::new();
                        let mut line_num = 1;
                        for line in self.source.split("\n") {
                            numbered_src += format!("{}: {}\n", line_num, line).as_str();
                            line_num += 1;
                        }
                        panic!("{}\nОшибка шейдера (исходник с нуменованными строками представлен выше)\n{}", numbered_src, spirv_log);
                    }
                },
                Err(_) => return Err(format!("Шейдер не скомпилирован"))
            }
        };

        if spv_path.is_file() {
            let mut file = File::open(spv_path).unwrap();
            let mut spv = Vec::new();
            file.read_to_end(&mut spv).unwrap();
            unsafe {
                self.module = Some(vulkano::shader::ShaderModule::from_bytes(self.device.clone(), spv.as_ref()).unwrap());
            }
            match self.module {
                Some (ref module) => {
                    let ep = module.entry_point("main").unwrap();
                    let module_uniforms = ep.descriptor_requirements().map(
                        |((set, binding), _)| {
                            (set, binding)
                        }
                    ).collect::<HashSet<_>>();

                    let source_uniforms = self.uniforms.clone();
                    for (_, ShaderSourceUniform {name, type_name: _, set, binding} ) in &source_uniforms {
                        let key = (*set as u32, *binding as u32);
                        if !module_uniforms.contains(&key) {
                            self.uniforms.remove(name);
                        }
                    }
                },
                None => ()
            };
            return Ok(self);
        };

        Err(format!("Шейдер не скомпилирован"))
    }
}

/// Строитель шейдера
#[allow(dead_code)]
pub struct ShaderProgramBuilder
{
    vertex_shader_source : String,
    fragment_shader_source : String,
    tess_controll_source : String,
    tess_eval_source : String,
    compute_source : String,

    vertex_shader: Option<Arc<vulkano::shader::ShaderModule>>,
    fragment_shader: Option<Arc<vulkano::shader::ShaderModule>>,
    fragment_outputs: HashMap<String, AttribType>,
    tess_controll: Option<Arc<vulkano::shader::ShaderModule>>,
    tess_eval: Option<Arc<vulkano::shader::ShaderModule>>,
    compute: Option<Arc<vulkano::shader::ShaderModule>>,
    uniforms_locations : HashMap<String, (usize, usize)>,
    uniforms_types  : HashMap<String, String>,
    uniform_constant : Option<(String, String)>,
    hash : u64,
}

#[allow(dead_code)]
impl ShaderProgramBuilder
{

    fn check_uniforms_compatibility<T>(uniforms_s1: &HashMap<String, T>, uniforms_s2: &HashMap<String, T>) -> bool
    where T : Eq + PartialEq
    {
        let keys_a = HashSet::<_>::from_iter(uniforms_s1.keys().map(|element| { element.clone() }).collect::<Vec<String>>());
        let keys_b = HashSet::<_>::from_iter(uniforms_s2.keys().map(|element| { element.clone() }).collect::<Vec<String>>());
        for name in keys_a.intersection(&keys_b) {
            if uniforms_s1[name] != uniforms_s2[name]
            {
                return false;
            }
        };
        true
    }

    /// Фрагментный шейдер
    pub fn fragment(&mut self, shader: &Shader) -> Result<&mut Self, String>
    {
        let mut uniforms_locations = HashMap::new();
        let mut uniforms_types = HashMap::new();
        for (uname, ShaderSourceUniform{set, binding, type_name, ..}) in &shader.uniforms {
            uniforms_locations.insert(uname.clone(), (*set, *binding));
            uniforms_types.insert(uname.clone(), type_name.clone());
        }
        let loc_compat = Self::check_uniforms_compatibility(&self.uniforms_locations, &uniforms_locations);
        let type_compat = Self::check_uniforms_compatibility(&self.uniforms_types, &uniforms_types);
        
        if !loc_compat {
            return Err(format!("Фрагментный шейдер использует те же имена uniform-переменных с разным расположением."));
        }
        if !type_compat {
            return Err(format!("Фрагментный шейдер использует те же имена uniform-переменных с другими типами."));
        }

        self.hash ^= shader.hash();
        self.fragment_shader_source = shader.source.clone();
        self.fragment_shader = shader.module.clone();
        self.fragment_outputs = shader.outputs.clone();
        for (name, location) in &uniforms_locations {
            self.uniforms_locations.insert(name.clone(), *location);
        }
        for (name, _type) in &uniforms_types {
            self.uniforms_types.insert(name.clone(), _type.clone());
        }
        match self.uniform_constant {
            Some(_) => return Err(format!("Uniform-константу можно объявить только в одном шейдере.")),
            None => self.uniform_constant =
            match &shader.push_constants {
                Some(pc) => Some((pc.1.name.to_string(), pc.1.type_name.to_string())),
                None => None
            }
        }
        Ok(self)
    }

    /// Вершинный шейдер
    pub fn vertex(&mut self, shader: &Shader) -> Result<&mut Self, String>
    {
        let mut uniforms_locations = HashMap::new();
        let mut uniforms_types = HashMap::new();
        for (uname, ShaderSourceUniform{set, binding, type_name, ..}) in &shader.uniforms {
            uniforms_locations.insert(uname.clone(), (*set, *binding));
            uniforms_types.insert(uname.clone(), type_name.clone());
        }

        let loc_compat = Self::check_uniforms_compatibility(&self.uniforms_locations, &uniforms_locations);
        let type_compat = Self::check_uniforms_compatibility(&self.uniforms_types, &uniforms_types);
        
        if !loc_compat {
            return Err(format!("Вершинный шейдер использует существующие имена uniform-переменных с разными расположениями."));
        }
        if !type_compat {
            return Err(format!("Вершинный шейдер использует существующие имена uniform-переменных с разными типами."));
        }

        self.hash ^= shader.hash();
        self.vertex_shader_source = shader.source.clone();
        self.vertex_shader = shader.module.clone();
        for (name, location) in &uniforms_locations {
            self.uniforms_locations.insert(name.clone(), *location);
        }
        for (name, _type) in &uniforms_types {
            self.uniforms_types.insert(name.clone(), _type.clone());
        }
        match self.uniform_constant {
            Some(_) => return Err(format!("Uniform-константу можно объявить только в одном шейдере.")),
            None => self.uniform_constant =
            match &shader.push_constants {
                Some(pc) => Some((pc.1.name.to_string(), pc.1.type_name.to_string())),
                None => None
            }
        }
        Ok(self)
    }

    /// Пара тесселяционных шейдеров
    pub fn tesselation(&'static mut self, eval: &Shader, control: &Shader) -> Result<&mut Self, String>
    {
        let mut eval_uniforms_locations = HashMap::new();
        let mut eval_uniforms_types = HashMap::new();
        let mut control_uniforms_locations = HashMap::new();
        let mut control_uniforms_types = HashMap::new();

        for (uname, ShaderSourceUniform{set, binding, type_name, ..}) in &eval.uniforms {
            eval_uniforms_locations.insert(uname.clone(), (*set, *binding));
            eval_uniforms_types.insert(uname.clone(), type_name.clone());
        }
        for (uname, ShaderSourceUniform{set, binding, type_name, ..}) in &control.uniforms {
            control_uniforms_locations.insert(uname.clone(), (*set, *binding));
            control_uniforms_types.insert(uname.clone(), type_name.clone());
        }

        let loc_compat =
            Self::check_uniforms_compatibility(&self.uniforms_locations, &eval_uniforms_locations) && 
            Self::check_uniforms_compatibility(&self.uniforms_locations, &control_uniforms_locations);

        let type_compat =
            Self::check_uniforms_compatibility(&self.uniforms_types, &eval_uniforms_types) &&
            Self::check_uniforms_compatibility(&self.uniforms_types, &control_uniforms_types);
        
        if !loc_compat {
            return Err(format!("Шейдеры тесселяции использует одни и те же имена uniform-переменных с разными расположениями."));
        }
        if !type_compat {
            return Err(format!("Шейдеры тесселяции использует одни и те же имена uniform-переменных с разными типами."));
        }
        self.hash ^= eval.hash();
        self.hash ^= control.hash();

        self.tess_controll_source = control.source.clone();
        self.tess_eval_source = eval.source.clone();

        self.tess_controll = control.module.clone();
        self.tess_eval = eval.module.clone();

        for (name, location) in &eval_uniforms_locations {
            self.uniforms_locations.insert(name.clone(), *location);
        }
        for (name, location) in &control_uniforms_locations {
            self.uniforms_locations.insert(name.clone(), *location);
        }
        for (name, _type) in &eval_uniforms_types {
            self.uniforms_types.insert(name.clone(), _type.clone());
        }
        for (name, _type) in &control_uniforms_types {
            self.uniforms_types.insert(name.clone(), _type.clone());
        }

        match self.uniform_constant {
            Some(_) => return Err(format!("Uniform-константу можно объявить только в одном шейдере.")),
            None => self.uniform_constant =
            match &eval.push_constants {
                Some(pc) => Some((pc.1.name.to_string(), pc.1.type_name.to_string())),
                None => None
            }
        }
        match self.uniform_constant {
            Some(_) => return Err(format!("Uniform-константу можно объявить только в одном шейдере.")),
            None => self.uniform_constant =
            match &control.push_constants {
                Some(pc) => Some((pc.1.name.to_string(), pc.1.type_name.to_string())),
                None => None
            }
        }
        
        Ok(self)
    }

    /// Вычислительный шейдер
    /// TODO: реализовать
    pub fn compute(&mut self) -> &mut Self
    {
        panic!("Не реализовано");
    }

    pub fn build_mutex(self, device: Arc<Device>) -> Result<ShaderProgramRef, String>
    {
        Ok(RcBox::construct(self.build(device)?))
    }

    /// Строит шейдерную программу
    pub fn build(self, device: Arc<Device>) -> Result<ShaderProgram, String>
    {
        Ok(
            ShaderProgram{
                device : device.clone(),
                pipeline : PipelineType::None,
                subpass_id : (0, 0),

                vertex_shader : self.vertex_shader.clone(),
                fragment_shader : self.fragment_shader.clone(),
                tess_controll : self.tess_controll.clone(),
                tess_eval : self.tess_eval.clone(),
                compute : self.compute.clone(),
                fragment_outputs : self.fragment_outputs.clone(),

                uniforms_types : self.uniforms_types,
                uniforms_locations : self.uniforms_locations,

                vertex_shader_source : self.vertex_shader_source,
                fragment_shader_source : self.fragment_shader_source,
                tess_controll_source : self.tess_controll_source,
                tess_eval_source : self.tess_eval_source,
                compute_source : self.compute_source,

                cull_faces : CullMode::None,

                hash : self.hash
            }
        )
    }
}

use vulkano::descriptor_set::WriteDescriptorSet;
pub trait PipelineUniform: ShaderStructUniform + std::marker::Send + std::marker::Sync {}

/// Буфер uniform-переменных для шейдеров
pub struct ShaderProgramUniformBuffer
{
    write_set_descriptors: HashMap<usize, Vec<WriteDescriptorSet>>,
    uniforms_sets: HashMap<usize, Arc<PersistentDescriptorSet>>,
    uniforms_locations : HashMap<String, (usize, usize)>,
    uniform_buffer : CpuBufferPool<f32>,
    device: Arc<Device>,
    pipeline: PipelineType,
}

impl Clone for ShaderProgramUniformBuffer
{
    fn clone(&self) -> Self {
        Self {
            write_set_descriptors: HashMap::new(),
            uniforms_sets: self.uniforms_sets.clone(),
            uniforms_locations: self.uniforms_locations.clone(),
            uniform_buffer: self.uniform_buffer.clone(),
            device: self.device.clone(),
            pipeline: self.pipeline.clone(),
        }
    }
}

#[allow(dead_code)]
impl ShaderProgramUniformBuffer
{
    /// Собирает дескрипторы наборов uniform-переменных (`PersistentDescriptorSet`).
    /// Полезно для формирования неизменяемого списка текстур материала
    pub fn build_uniform_sets(&mut self, sets: &[usize])
    {
        let layouts = self.pipeline.layout().unwrap().set_layouts();
        for set_num in sets {
            let set_desc = self.write_set_descriptors.remove(set_num).unwrap();
            let pers_decc_set = PersistentDescriptorSet::new(layouts.get(*set_num).unwrap().clone(), set_desc).unwrap();
            self.uniforms_sets.insert(*set_num, pers_decc_set);
        }
    }

    /// Проверяет инициализирован ли набор uniform-переменных
    pub fn is_set_initialized(&self, set_num: usize) -> bool
    {
        self.uniforms_sets.contains_key(&set_num)
    }

    /// Очищает заданный набор uniform-переменных
    pub fn clear_uniform_set(&mut self, set_num: usize)
    {
        //self.uniform_set_builders.remove(&set_num);
        self.uniforms_sets.remove(&set_num);
    }

    /// Передаёт в шейдер массив данных неопределённой структуры
    pub fn uniform_structure<I>(&mut self, obj: I, set_num: usize, binding_num: usize)
        where I: IntoIterator<Item = f32>,
        I::IntoIter: ExactSizeIterator
    {
        let uniform_buffer = WriteDescriptorSet::buffer(binding_num as u32, self.uniform_buffer.chunk(obj).unwrap());
        
        match self.write_set_descriptors.get_mut(&set_num)
        {
            Some(set_buffer) => {
                if binding_num >= set_buffer.len() {
                    for binding in set_buffer.len()..binding_num {
                        set_buffer.push(WriteDescriptorSet::none(binding as u32));
                    }
                } else {
                    set_buffer.remove(binding_num);
                }
                set_buffer.insert(binding_num, uniform_buffer);
            },
            None => {
                let mut uniform_set = Vec::new();
                for binding in 0..binding_num {
                    uniform_set.push(WriteDescriptorSet::none(binding as u32));
                }
                uniform_set.push(uniform_buffer);
                self.write_set_descriptors.insert(set_num, uniform_set);
            }
        };
    }
    
    /// Передаёт uniform-переменную в шейдер
    /// Может передавать как `TextureRef`, так и структуры, для которых определён `trait ShaderStructUniform`
    pub fn uniform<T>(&mut self, obj: T, set_num: usize, binding_num: usize)
        where T: ShaderStructUniform + std::marker::Send + std::marker::Sync + Pod + Clone + 'static
    {
        let s = [obj];
        let sas : &[f32] = bytemuck::cast_slice(&s);
        self.uniform_structure(sas.to_vec(), set_num, binding_num);
    }

    pub fn uniform_sampler(&mut self, texture: &Texture, set_num: usize, binding_num: usize)
    {
        let wds = WriteDescriptorSet::image_view_sampler(binding_num as u32, texture.image_view().clone(), texture.sampler().clone());
        match self.write_set_descriptors.get_mut(&set_num)
        {
            Some(set_buffer) => {
                if binding_num >= set_buffer.len() {
                    for binding in set_buffer.len()..binding_num {
                        set_buffer.push(WriteDescriptorSet::none(binding as u32));
                    }
                } else {
                    set_buffer.remove(binding_num);
                }
                set_buffer.insert(binding_num, wds);
            },
            None => {
                let mut uniform_set = Vec::new();
                for binding in 0..binding_num {
                    uniform_set.push(WriteDescriptorSet::none(binding as u32));
                }
                uniform_set.push(wds);
                self.write_set_descriptors.insert(set_num, uniform_set);
            }
        };
    }

    pub fn uniform_by_name<T>(&mut self, obj: T, name: &String) -> Result<(), String>
    where T: ShaderStructUniform + std::marker::Send + std::marker::Sync + Pod + 'static
    {
        //println!("uniform {} to {}", name, self.hash);
        let (set_num, binding_num) = *match self.uniforms_locations.get(name)
        {
            Some(val) => val,
            None => return Err(format!("Uniform-переменная {} не объявлена в этом шейдере.", name))
        };
        self.uniform(obj, set_num, binding_num);
        Ok(())
    }

    pub fn uniform_sampler_by_name(&mut self, texture: &Texture, name: &String) -> Result<(usize, usize), String>
    {
        
        let (set_num, binding_num) = *match self.uniforms_locations.get(name)
        {
            Some(val) => {
                //println!("uniform (set={}, binding={}) {}", val.0, val.1, name);
                val
            },
            None => {
                //println!("Uniform-переменная {} не объявлена в этом шейдере.", name);
                return Err(format!("Uniform-переменная {} не объявлена в этом шейдере.", name));
            }
        };
        //println!("{}", self.fragment_shader_source());
        self.uniform_sampler(texture, set_num, binding_num);
        Ok((set_num, binding_num))
    }
    
    pub fn storage_buffer<T>(&mut self, obj: T, set_num: usize, binding_num: usize)
        where T: std::marker::Send + std::marker::Sync + Pod + 'static
    {
        if self.uniforms_sets.contains_key(&set_num) {
            return;
        }
        let uniform_buffer = CpuAccessibleBuffer::from_data(self.device.clone(), BufferUsage::storage_buffer(), false, obj).unwrap();
        let ub = WriteDescriptorSet::buffer(binding_num as u32, uniform_buffer);

        match self.write_set_descriptors.get_mut(&set_num)
        {
            Some(set_buffer) => {
                if binding_num > set_buffer.len() {
                    for binding in set_buffer.len()..binding_num {
                        set_buffer.push(WriteDescriptorSet::none(binding as u32));
                    }
                }
                set_buffer.insert(binding_num, ub);
            },
            None => {
                let mut uniform_set = vec![];
                for binding in 0..binding_num {
                    uniform_set.push(WriteDescriptorSet::none(binding as u32));
                }
                uniform_set.insert(binding_num, ub);
                self.write_set_descriptors.insert(set_num, uniform_set);
            }
        };
    }
    
    pub fn storage_buffer_by_name<T>(&mut self, obj: T, name: &String) -> Result<(), String>
        where T: std::marker::Send + std::marker::Sync + Pod + 'static
    {
        let (set_num, binding_num) = *match self.uniforms_locations.get(name)
        {
            Some(val) => val,
            None => return Err(format!("Uniform-переменная {} не объявлена в этом шейдере.", name))
        };
        self.storage_buffer(obj, set_num, binding_num);
        Ok(())
    }
}

/// Шейдерная программа
#[allow(dead_code)]
#[derive(Clone)]
pub struct ShaderProgram
{
    device : Arc<Device>,
    pipeline : PipelineType,
    subpass_id : (u32, u32),
    
    uniforms_types  : HashMap<String, String>,
    uniforms_locations : HashMap<String, (usize, usize)>,

    vertex_shader : Option<Arc<vulkano::shader::ShaderModule>>,
    fragment_shader : Option<Arc<vulkano::shader::ShaderModule>>,
    fragment_outputs : HashMap<String, AttribType>,
    tess_controll : Option<Arc<vulkano::shader::ShaderModule>>,
    tess_eval : Option<Arc<vulkano::shader::ShaderModule>>,
    compute : Option<Arc<vulkano::shader::ShaderModule>>,

    vertex_shader_source : String,
    fragment_shader_source : String,
    tess_controll_source : String,
    tess_eval_source : String,
    compute_source : String,

    pub cull_faces : CullMode,

    hash : u64,
}

#[allow(dead_code)]
impl ShaderProgram
{
    pub fn hash(&self) -> u64
    {
        self.hash
    }

    pub fn vertex_shader_source(&self) -> &String
    {
        &self.vertex_shader_source
    }

    pub fn fragment_shader_source(&self) -> &String
    {
        &self.fragment_shader_source
    }

    pub fn tess_controll_source(&self) -> &String
    {
        &self.tess_controll_source
    }

    pub fn tess_eval_source(&self) -> &String
    {
        &self.tess_eval_source
    }

    pub fn compute_source(&self) -> &String
    {
        &self.compute_source
    }

}

#[allow(dead_code)]
impl ShaderProgram
{
    pub fn builder() -> ShaderProgramBuilder
    {
        ShaderProgramBuilder {
            vertex_shader_source : String::new(),
            fragment_shader_source : String::new(),
            tess_controll_source : String::new(),
            tess_eval_source : String::new(),
            compute_source : String::new(),

            vertex_shader : None,
            fragment_shader : None,
            fragment_outputs : HashMap::new(),
            tess_controll : None,
            tess_eval : None,
            compute : None,
            uniforms_locations : HashMap::new(),
            uniforms_types : HashMap::new(),
            uniform_constant : None,
            hash : 0,
        }
    }

    pub fn new_uniform_buffer(&self) -> ShaderProgramUniformBuffer
    {
        ShaderProgramUniformBuffer {
            write_set_descriptors: HashMap::new(),
            uniforms_sets: HashMap::new(),
            uniforms_locations: self.uniforms_locations.clone(),
            uniform_buffer: CpuBufferPool::uniform_buffer(self.device.clone()),
            device: self.device.clone(),
            pipeline: self.pipeline.clone(),
        }
    }

    pub fn device(&self) -> &Arc<Device>
    {
        &self.device
    }

    pub fn pipeline(&self) -> PipelineType
    {
        self.pipeline.clone()
    }

    /// Перестраивает `Pipeline` для использования с заданным `Subpass`'ом.
    /// Возвращает true, если переданный subpass отличается от использованного в прошлый раз
    pub fn use_subpass(&mut self, subpass : Subpass) -> (PipelineType, bool)
    {
        let render_pass_id = subpass.render_pass().as_ref() as *const RenderPass as u32;
        let subpass_full_id = (render_pass_id, subpass.index());
        
        if subpass_full_id == self.subpass_id {
            return (self.pipeline.clone(), false);
        }
        println!("Исользуется новый subpass {:?}", subpass_full_id);
        self.subpass_id = subpass_full_id;
        /*self.uniforms_sets.clear();*/
        let depth_test = subpass.has_depth();
        let mut pipeline = vulkano::pipeline::GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<super::mesh::VkVertex>());
        
        pipeline = pipeline.rasterization_state(RasterizationState{cull_mode: vulkano::pipeline::StateMode::Fixed(self.cull_faces), ..Default::default()});
        
        if self.vertex_shader.is_some() {
            pipeline = pipeline.vertex_shader(self.vertex_shader.as_ref().unwrap().entry_point("main").unwrap(), ());
        }
        if self.fragment_shader.is_some() {
            pipeline = pipeline.fragment_shader(self.fragment_shader.as_ref().unwrap().entry_point("main").unwrap(), ());
        }
        if self.tess_eval.is_some() && self.tess_controll.is_some() {
            pipeline = pipeline.tessellation_shaders(
                self.tess_controll.as_ref().unwrap().entry_point("main").unwrap(), (),
                self.tess_eval.as_ref().unwrap().entry_point("main").unwrap(), ()
            );
        }
        pipeline = pipeline
            .input_assembly_state(vulkano::pipeline::graphics::input_assembly::InputAssemblyState::new())
            .viewport_state(vulkano::pipeline::graphics::viewport::ViewportState::viewport_dynamic_scissor_irrelevant())
            .render_pass(subpass);
        
        if depth_test {
            pipeline = pipeline.depth_stencil_state(DepthStencilState::simple_depth_test());
        }
        //if self.
        self.pipeline = PipelineType::Graphics(pipeline.build(self.device.clone()).unwrap());
        (self.pipeline.clone(), true)
    }

    /// Возвращает имена выходов фрагментного фейдера
    #[allow(dead_code)]
    pub fn outputs(&self) -> &HashMap<String, AttribType>
    {
        &self.fragment_outputs
    }
}

/// trait для удобной передачи шейдеров и uniform-переменные в `AutoCommandBufferBuilder`
pub trait ShaderProgramBinder
{
    /// Присоединение шейдерной программы (`GraphicsPipeline`) к `AutoCommandBufferBuilder`'у
    fn bind_shader_program(&mut self, shader: &ShaderProgram) -> Result<&mut Self, String>;

    /// Присоединение uniform-переменных к `AutoCommandBufferBuilder`'у
    fn bind_shader_uniforms(&mut self, uniform_buffer: &mut ShaderProgramUniformBuffer, only_dynamic: bool) -> Result<&mut Self, String>;

    fn bind_uniform_constant<T>(&mut self, shader: &mut ShaderProgram, data: T) -> Result<&mut Self, String>;
}

impl <BufferType>ShaderProgramBinder for AutoCommandBufferBuilder<BufferType>
{
    fn bind_shader_program(&mut self, shader: &ShaderProgram) -> Result<&mut Self, String>
    {
        match shader.pipeline() {
            PipelineType::Graphics(pipeline) => Ok(self.bind_pipeline_graphics(pipeline)),
            PipelineType::Compute(pipeline) => Ok(self.bind_pipeline_compute(pipeline)),
            PipelineType::None => Err("Не установлен Subpass".to_string())
        }
    }

    fn bind_uniform_constant<T>(&mut self, shader: &mut ShaderProgram, data: T) -> Result<&mut Self, String>
    {
        let pipeline_layout = shader.pipeline.layout().unwrap();
        self.push_constants(pipeline_layout.clone(), 0, data);
        Ok(self)
    }

    fn bind_shader_uniforms(&mut self, uniform_buffer: &mut ShaderProgramUniformBuffer, only_dynamic: bool) -> Result<&mut Self, String>
    {
        let pipeline_layout = uniform_buffer.pipeline.layout().unwrap();
        let layouts = pipeline_layout.set_layouts();
        let mut desc_sets = Vec::new();
        
        let keys = uniform_buffer.write_set_descriptors.keys().map(|elem| {*elem}).collect::<Vec<_>>();
        for set_num in keys {
            let descriptor_writes = uniform_buffer.write_set_descriptors.remove(&set_num).unwrap();
            let layout = 
            match layouts.get(set_num)
            {
                Some(layout) => layout,
                None => return Err(format!("В шейдере есть неиспользуемые uniform-переменные. Набор {} не используется нигде.", set_num))
            };
            
            let set = PersistentDescriptorSet::new(layout.clone(), descriptor_writes);
            match set {
                Ok(set) => desc_sets.push((set_num, set)),
                Err(e)  => {
                    //println!("{}", shader.fragment_shader_source);
                    return Err(format!("Не удалось сформировать набор uniform-переменных №{}: {:?}", set_num, e));
                }
            };
        }
        if !only_dynamic {
            for (set_num, desc_set) in &uniform_buffer.uniforms_sets {
                desc_sets.push((*set_num, desc_set.clone()));
            }
        }
        desc_sets.sort_by(|(set_num_a, _), (set_num_b, _)| {
            if set_num_a < set_num_b {
                return std::cmp::Ordering::Less;
            } else {
                if set_num_a == set_num_b {
                    return std::cmp::Ordering::Equal;
                } else {
                    return std::cmp::Ordering::Greater;
                }
            }
        });
        if desc_sets.len() == 0 {
            return Ok(self);
        }
        //println!("desc_sets {}", desc_sets.len());
        let first_set_num = desc_sets.first().unwrap().0;
        let desc_sets = desc_sets.iter().map(|elem| {elem.1.clone()}).collect::<Vec<_>>();
        self.bind_descriptor_sets(PipelineBindPoint::Graphics, pipeline_layout.clone(), first_set_num as _, desc_sets);
        Ok(self)
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum PipelineType
{
    Compute(Arc<ComputePipeline>),
    Graphics(Arc<GraphicsPipeline>),
    None
}

impl PipelineType
{
    pub fn layout(&self) -> Option<&Arc<PipelineLayout>>
    {
        match self
        {
            Self::Compute(pipeline) => Some(pipeline.layout()),
            Self::Graphics(pipeline) => Some(pipeline.layout()),
            Self::None => None
        }
    }
}