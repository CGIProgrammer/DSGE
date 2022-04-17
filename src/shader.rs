//extern crate spirv_compiler;

use std::collections::{HashMap, HashSet};
use std::process::ExitStatus;
pub use super::glenums::{ShaderType, GLSLVersion, GLSLType, AttribType};
use std::sync::Arc;
use vulkano::pipeline::graphics::rasterization::{RasterizationState, CullMode};
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::pipeline::{GraphicsPipeline, StateMode};
use vulkano::render_pass::{RenderPass, Subpass};
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::buffer::{CpuBufferPool, CpuAccessibleBuffer, BufferUsage};
use vulkano::pipeline::PipelineBindPoint;

use crate::references::*;
use crate::texture::{TextureType, TextureTypeGlsl, TextureRef};
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
    fn texture(&self) -> Option<&crate::texture::TextureRef>; // Позволяет получить текстуру, если структура является таковой
}

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

fn convert_vec_types(mut vec8: Vec<u8>) -> Vec<u32>
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

    pub fn uniform_sampler(&mut self, name: &str, set: usize, binding: usize, dims: TextureType) -> Result<&mut Self, String>
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

    pub fn uniform_sampler_autoincrement(&mut self, name: &str, set: usize, dims: TextureType) -> Result<&mut Self, String>
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
        
        if cache_path.is_dir() && spv_path.is_file() {
            let mut file = File::open(spv_path).unwrap();
            let mut spv = Vec::new();
            file.read_to_end(&mut spv).unwrap();
            unsafe {
                self.module = Some(vulkano::shader::ShaderModule::from_bytes(self.device.clone(), spv.as_ref()).unwrap());
            }
            println!("{} шейдер загружен из кэша", self.sh_type);
            return Ok(self);
        };
        //return Err(format!("Шейдер не найден в кэше. Возможность компиляции шейдеров отключена."));
        
        //let mut compiler = spirv_compiler::CompilerBuilder::new().with_source_language(spirv_compiler::SourceLanguage::GLSL).build().unwrap();
        let sh_type = match self.sh_type {
            ShaderType::Vertex => "vertex",
            ShaderType::Fragment => "fragment",
            ShaderType::TesselationControl => "tess_control",
            ShaderType::TesselationEval => "tess_evaluation",
            ShaderType::Compute => "compute",
            ShaderType::Geometry => "geometry",
        };
        //println!("{}", self.source);
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
            Err(_) => ()
        }

        if cache_path.is_dir() && spv_path.is_file() {
            let mut file = File::open(spv_path).unwrap();
            let mut spv = Vec::new();
            file.read_to_end(&mut spv).unwrap();
            unsafe {
                self.module = Some(vulkano::shader::ShaderModule::from_bytes(self.device.clone(), spv.as_ref()).unwrap());
            }
            println!("{} шейдер загружен из кэша после компиляции", self.sh_type);
            return Ok(self);
        };

        /*let spirv = compiler.compile_from_string(self.source.as_str(), sh_type);
        match spirv {
            Err(error) => {
                let mut numbered_src = String::new();
                let mut line_num = 1;
                for line in self.source.split("\n") {
                    numbered_src += format!("{}: {}\n", line_num, line).as_str();
                    line_num += 1;
                }
                panic!("{}\nОшибка шейдера (исходник с нуменованными строками представлен выше)\n{}", numbered_src, error);
            },
            Ok(spv_ok) => {
                if cache_path.is_dir() {
                    let mut file = File::create(spv_path).unwrap();
                    for val in spv_ok.as_slice() {
                        file.write(&val.to_le_bytes()).unwrap();
                    }
                }
                self.module = unsafe { Some(vulkano::shader::ShaderModule::from_words(self.device.clone(), spv_ok.as_slice()).unwrap()) };
                println!("Скопилирован {} шейдер (self.module={})", self.sh_type, self.module.is_some());
                return Ok(self);
            }
        };*/
        return Err(format!("Шейдер не скомпилирован"));
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
                pipeline : None,
                subpass_id : (0, 0),
                
                write_set_descriptors : HashMap::new(),
                uniforms_sets : HashMap::new(),
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

/// Шейдерная программа
#[allow(dead_code)]
pub struct ShaderProgram
{
    device : Arc<Device>,
    pipeline : Option<Arc<vulkano::pipeline::GraphicsPipeline>>,
    subpass_id : (u32, u32),
    
    write_set_descriptors: HashMap<usize, Vec<WriteDescriptorSet>>,
    uniforms_sets: HashMap<usize, Arc<PersistentDescriptorSet>>,
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
            hash : 0,
        }
    }

    pub fn device(&self) -> &Arc<Device>
    {
        &self.device
    }

    pub fn pipeline(&self) -> Option<Arc<GraphicsPipeline>>
    {
        self.pipeline.clone()
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
    pub fn uniform_structure<T>(&mut self, obj: Vec<T>, set_num: usize, binding_num: usize)
        where T: std::marker::Send + std::marker::Sync + Pod + 'static
    {
        /*if self.uniforms_sets.contains_key(&set_num) {
            return;
        }*/
        let uniform_buffer = CpuBufferPool::uniform_buffer(self.device.clone());
        let uniform_buffer = WriteDescriptorSet::buffer(binding_num as u32, uniform_buffer.chunk(obj.clone()).unwrap());
        
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
    pub fn uniform<T>(&mut self, obj: &T, set_num: usize, binding_num: usize)
        where T: ShaderStructUniform + std::marker::Send + std::marker::Sync + Pod + 'static
    {
        if self.uniforms_sets.contains_key(&set_num) {
            return;
        }
        let uniform_buffer = {
            let uniform_buffer = CpuBufferPool::uniform_buffer(self.device.clone());
            WriteDescriptorSet::buffer(binding_num as u32, uniform_buffer.next(obj.clone()).unwrap())
        };
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

    pub fn uniform_sampler(&mut self, texture: &TextureRef, set_num: usize, binding_num: usize)
    {
        let texture = texture.take_mut();
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

    pub fn uniform_by_name<T>(&mut self, obj: &T, name: &String) -> Result<(), String>
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

    pub fn uniform_sampler_by_name(&mut self, texture: &TextureRef, name: &String) -> Result<(), String>
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
        Ok(())
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

    /// Перестраивает `Pipeline` для использования с заданным `Subpass`'ом.
    /// Необходимо вызывать каждый раз при использовании нового `Subpass`'а перед вызовами `uniform` и `uniform_structure`.
    /// 
    /// Вызов `use_subpass` приводит к полной очистке буферов uniform-переменных
    pub fn use_subpass(&mut self, render_pass : Arc<RenderPass>, subpass_id: u32)
    {
        let render_pass_id = render_pass.as_ref() as *const RenderPass as u32;
        let subpass_full_id = (render_pass_id, subpass_id);
        if subpass_full_id == self.subpass_id {
            return;
        }
        //println!("Исользуется новый subpass");
        self.subpass_id = subpass_full_id;
        let subpass = Subpass::from(render_pass.clone(), subpass_id).unwrap();
        self.uniforms_sets.clear();
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
        self.pipeline = Some(pipeline.build(self.device.clone()).unwrap());
    }

    /// Собирает дескрипторы наборов uniform-переменных (`PersistentDescriptorSet`).
    /// Полезно для формирования неизменяемого списка текстур материала
    pub fn build_uniform_sets(&mut self)
    {
        let layouts = self.pipeline.as_ref().unwrap().layout().set_layouts();
        while self.write_set_descriptors.len() > 0 {
            let set_num = *self.write_set_descriptors.keys().nth(0).unwrap();
            let set_desc = self.write_set_descriptors.remove(&set_num).unwrap();
            let pers_decc_set = PersistentDescriptorSet::new(layouts.get(set_num).unwrap().clone(), set_desc).unwrap();
            self.uniforms_sets.insert(set_num, pers_decc_set);
        }
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
    fn bind_shader_uniforms(&mut self, shader: &mut ShaderProgram) -> Result<&mut Self, String>;
}

impl ShaderProgramBinder for AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>
{
    fn bind_shader_program(&mut self, shader: &ShaderProgram) -> Result<&mut Self, String>
    {
        match shader.pipeline() {
            Some(pipeline) => Ok(self.bind_pipeline_graphics(pipeline)),
            None => Err("Не установлен Subpass".to_string())
        }
    }

    fn bind_shader_uniforms(&mut self, shader: &mut ShaderProgram) -> Result<&mut Self, String>
    {
        let pipeline_layout = shader.pipeline.as_ref().unwrap().layout();
        let layouts = pipeline_layout.set_layouts();
        while shader.write_set_descriptors.len() > 0 {
            let set_num = *shader.write_set_descriptors.keys().last().unwrap();
            let descriptor_writes = shader.write_set_descriptors.remove(&set_num).unwrap();
            let layout = 
            match layouts.get(set_num)
            {
                Some(layout) => layout,
                None => return Err(format!("В шейдере есть неиспользуемые uniform-переменные. Набор {} не используется нигде.", set_num))
            };
            
            let set = PersistentDescriptorSet::new(layout.clone(), descriptor_writes);
            match set {
                Ok(set) => self.bind_descriptor_sets(PipelineBindPoint::Graphics, pipeline_layout.clone(), set_num as u32, set),
                Err(e) => return Err(format!("Не удалось сформировать набор uniform-переменных №{}: {:?}", set_num, e))
            };
        }
        for (set_num, desc_set) in &shader.uniforms_sets {
            self.bind_descriptor_sets(PipelineBindPoint::Graphics, pipeline_layout.clone(), *set_num as u32, desc_set.clone());
        }
        Ok(self)
    }
}
