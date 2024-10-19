pub use super::glenums::{AttribType, GLSLType, GLSLVersion, ShaderType};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::sync::Arc;
use byteorder::ReadBytesExt;
use vulkano::buffer::{
    Buffer, BufferContents, BufferUsage, BufferCreateInfo,
};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::memory::allocator::{StandardMemoryAllocator, AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::color_blend::{ColorBlendState, ColorBlendAttachmentState};
use vulkano::pipeline::graphics::depth_stencil::{DepthStencilState, DepthState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::{CullMode, RasterizationState};
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
//use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::{PipelineBindPoint, PipelineShaderStageCreateInfo, DynamicState};
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition, VertexInputState};
use vulkano::pipeline::{ComputePipeline, GraphicsPipeline, PipelineLayout};
use vulkano::render_pass::{RenderPass, Subpass};
use vulkano::shader::ShaderModuleCreateInfo;
use vulkano::Validated;

use crate::command_buffer::CommandBufferFather;
use crate::mesh::VkVertex;
use crate::renderer::BumpMemoryAllocator;
use crate::texture::{Texture, TextureView, TextureViewGlsl};
use crate::vulkano::pipeline::Pipeline as VkPipeline;
use crate::{references::*, VULKANO_BUFFER_ATOMIC_SIZE, utils};
use bytemuck::Pod;
use vulkano::device::Device;

use std::collections::hash_map::DefaultHasher;
use std::fs::{File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::prelude::*;
use std::path::Path;

pub mod shader_struct_uniform;
pub use shader_struct_uniform::ShaderStructUniform;

pub type ShaderProgramRef = RcBox<ShaderProgram>;

#[derive(Clone)]
struct ShaderSourceUniform {
    name: String,
    type_name: String,
    set: u32,
    binding: u32,
}

pub enum ShaderUniformArrayLength {
    NotArray,
    Fixed(u32),
    Unknown,
}

impl ShaderUniformArrayLength {
    fn glsl_source(&self) -> String {
        match self {
            ShaderUniformArrayLength::NotArray => String::new(),
            ShaderUniformArrayLength::Fixed(len) => format!("[{len}]"),
            ShaderUniformArrayLength::Unknown => "[]".to_owned(),
        }
    }
}

#[derive(Copy, Clone)]
pub enum FragmentInterpolation {
    Flat,
    Smooth,
    NoPerspective,
}

impl Default for FragmentInterpolation {
    fn default() -> Self {
        FragmentInterpolation::Smooth
    }
}

impl Display for FragmentInterpolation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let qualifier = match self {
            Self::Flat => "flat",
            Self::Smooth => "smooth",
            Self::NoPerspective => "nooerspective",
        };
        f.write_str(qualifier)
    }
}

/// Структура для построения GLSL шейдера и компиляции его в SPIR-V
pub struct Shader {
    glsl_version: GLSLVersion,
    sh_type: ShaderType,
    device: Arc<vulkano::device::Device>,
    module: Option<Arc<vulkano::shader::ShaderModule>>,
    source: String,
    inputs: HashMap<String, AttribType>,
    input_locations: u32,
    outputs: HashMap<String, AttribType>,
    output_locations: u32,
    uniforms: HashMap<String, ShaderSourceUniform>,
    structures: HashSet<String>,
    push_constants: Option<(String, ShaderSourceUniform)>,
    spirv_hash: u64,
    vertex_input_state: Option<VertexInputState>
}

impl std::fmt::Debug for Shader {
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

impl Shader {
    pub fn get_source(&self) -> String {
        self.source.clone()
    }

    /// Построить щейдер указанного типа
    pub fn builder(shader_type: ShaderType, device: Arc<vulkano::device::Device>) -> Self {
        let gl_version = GLSLVersion::V450;
        let mut source = gl_version.stringify() + "\n";

        if gl_version.need_precision_qualifier() {
            source += "precision mediump float;\n";
        }

        Self {
            glsl_version: gl_version,
            sh_type: shader_type,
            module: None,
            device: device,
            source: source,
            inputs: HashMap::new(),
            input_locations: 0,
            outputs: HashMap::new(),
            output_locations: 0,
            uniforms: HashMap::new(),
            structures: HashSet::new(),
            push_constants: None,
            spirv_hash: 0,
            vertex_input_state: None
        }
    }

    /// Атрибуты вершин по умолчанию
    pub fn default_vertex_attributes(&mut self) -> &mut Self {
        self.input("v_pos", AttribType::FVec3, FragmentInterpolation::default())
            .input("v_nor", AttribType::FVec3, FragmentInterpolation::default())
            .input("v_bin", AttribType::FVec3, FragmentInterpolation::default())
            .input("v_tan", AttribType::FVec3, FragmentInterpolation::default())
            .input(
                "v_tex1",
                AttribType::FVec2,
                FragmentInterpolation::default(),
            )
            .input(
                "v_tex2",
                AttribType::FVec2,
                FragmentInterpolation::default(),
            )
            .input("v_grp", AttribType::UVec3, FragmentInterpolation::default())
    }

    pub fn instance_attributes(&mut self) -> &mut Self {
        self.input(
            "transform",
            AttribType::FMat4,
            FragmentInterpolation::default(),
        )
        .input(
            "transform_prev",
            AttribType::FMat4,
            FragmentInterpolation::default(),
        )
    }

    fn last_set_index(&self, set: u32) -> u32 {
        match self
            .uniforms
            .values()
            .filter(|val| val.set == set)
            .max_by_key(|val1| val1.binding)
        {
            Some(ShaderSourceUniform { binding, .. }) => *binding + 1,
            None => 0,
        }
    }

    /// Объявляет буфер-хранилище
    pub fn storage_buffer<T: ShaderStructUniform>(
        &mut self,
        name: &str,
        array_length: ShaderUniformArrayLength,
        set: u32,
        binding: u32,
    ) -> Result<&mut Self, String> {
        self._uniform_structure(
            name,
            "readonly buffer",
            &T::glsl_type_name(),
            array_length,
            &T::structure(),
            set,
            binding,
        )
    }

    pub fn storage_buffer_autoincrement<T: ShaderStructUniform>(
        &mut self,
        name: &str,
        array_length: ShaderUniformArrayLength,
        set: u32,
    ) -> Result<&mut Self, String> {
        let binding = self.last_set_index(set);
        self.storage_buffer::<T>(name, array_length, set, binding)
    }

    /// Объявление uniform-структуры
    pub fn uniform<T: ShaderStructUniform>(
        &mut self,
        name: &str,
        array_length: ShaderUniformArrayLength,
        set: u32,
        binding: u32,
    ) -> Result<&mut Self, String> {
        self._uniform_structure(
            name,
            "uniform",
            &T::glsl_type_name(),
            array_length,
            &T::structure(),
            set,
            binding,
        )
    }

    /// Объявление постоянной uniform-структуры
    pub fn uniform_constant<T: ShaderStructUniform>(
        &mut self,
        name: &str,
    ) -> Result<&mut Self, String> {
        if self.push_constants.is_some() {
            return Err(format!("Можно объявить только одну константу."));
        }
        let name = name.to_owned();

        let uniform_source = ShaderSourceUniform {
            name: name.clone(),
            type_name: T::glsl_type_name(),
            set: 0,
            binding: 0,
        };
        self.source += format!(
            "layout (push_constant) uniform {} {} {};\n",
            uniform_source.type_name,
            T::structure(),
            name
        )
        .as_str();
        self.push_constants = Some((name, uniform_source));
        Ok(self)
    }

    /// Объявление uniform-структуры с автоопределением расположения
    pub fn uniform_autoincrement<T: ShaderStructUniform>(
        &mut self,
        name: &str,
        array_length: ShaderUniformArrayLength,
        set: u32,
    ) -> Result<&mut Self, String> {
        let binding = self.last_set_index(set);
        self.uniform::<T>(name, array_length, set, binding)
    }

    /// Объявление uniform-структуры с явной передачей кода структуры
    fn _uniform_structure(
        &mut self,
        name: &str,
        buffer_type: &str,
        _type: &str,
        array_length: ShaderUniformArrayLength,
        structure: &str,
        set: u32,
        binding: u32,
    ) -> Result<&mut Self, String> {
        let name = name.to_owned();
        if self.uniforms.contains_key(&name) {
            return Err(format!("Переменная {} уже объявлена в этом шейдере.", name));
        }
        let uniform_source = ShaderSourceUniform {
            name: name.clone(),
            type_name: _type.to_owned(),
            set: set,
            binding: binding,
        };
        if !self.structures.contains(_type) {
            self.source += &format!("struct {_type} {structure};");
            self.structures.insert(_type.to_owned());
        }
        self.source += format!("layout (std140, set = {set}, binding = {binding}) {buffer_type} _{0} {{ {0}{1} data; }} _{name};\n", uniform_source.type_name, array_length.glsl_source()).as_str();
        self.define(&name, &format!("_{name}.data"));
        self.uniforms.insert(name.to_owned(), uniform_source);
        Ok(self)
    }

    pub fn uniform_structure(
        &mut self,
        name: &str,
        _type: &str,
        array_length: ShaderUniformArrayLength,
        structure: &str,
        set: u32,
        binding: u32,
    ) -> Result<&mut Self, String> {
        self._uniform_structure(
            name,
            "uniform",
            _type,
            array_length,
            structure,
            set,
            binding,
        )
    }

    /// Объявление uniform-структуры с явной передачей кода структуры и автоопределением расположения
    pub fn uniform_structure_autoincrement(
        &mut self,
        name: &str,
        _type: &str,
        array_length: ShaderUniformArrayLength,
        structure: &str,
        set: u32,
    ) -> Result<&mut Self, String> {
        let binding = self.last_set_index(set);
        self.uniform_structure(name, _type, array_length, structure, set, binding)
    }

    pub fn uniform_sampler(
        &mut self,
        name: &str,
        set: u32,
        binding: u32,
        dims: TextureView,
        shadowmap: bool,
    ) -> Result<&mut Self, String> {
        let name = name.to_owned();
        if self.uniforms.contains_key(&name) {
            return Err(format!("Переменная {} уже объявлена в этом шейдере.", name));
        }

        let utype = dims.glsl_sampler_name().to_owned();
        let uniform_source = ShaderSourceUniform {
            name: name.clone(),
            type_name: utype.clone(),
            set: set,
            binding: binding,
        };
        if shadowmap {
            self.source += format!(
                "layout (set = {set}, binding = {binding}) uniform {utype}Shadow {name};\n"
            )
            .as_str();
        } else {
            self.source +=
                format!("layout (set = {set}, binding = {binding}) uniform {utype} {name};\n")
                    .as_str();
        }
        self.uniforms.insert(name, uniform_source);
        Ok(self)
    }

    pub fn uniform_sampler_autoincrement(
        &mut self,
        name: &str,
        set: u32,
        dims: TextureView,
        shadowmap: bool,
    ) -> Result<&mut Self, String> {
        let binding = self.last_set_index(set);
        self.uniform_sampler(name, set, binding, dims, shadowmap)
    }

    /// Объявляет выход шейдера
    pub fn output(&mut self, name: &str, type_: AttribType) -> &mut Self {
        let layout_location = if self.glsl_version.have_explicit_attri_location() {
            let disp = (type_.get_cells_count() as f32 / 4.0).ceil() as u32;
            let line = format!("layout (location = {}) ", self.output_locations).to_owned();
            self.output_locations += disp;
            line
        } else {
            String::new()
        };
        self.outputs.insert(name.to_owned(), type_);
        self.source += format!(
            "{}out {} {};\n",
            layout_location,
            type_.get_glsl_name(),
            name
        )
        .as_str();
        self
    }

    /// Объявляет вход шейдера
    pub fn input(
        &mut self,
        name: &str,
        type_: AttribType,
        interpolation: FragmentInterpolation,
    ) -> &mut Self {
        let glsl_type = type_.get_glsl_name();
        if let Some(ref mut vis) = self.vertex_input_state {
            // vis.attributes.insert(self.input_locations, VertexInputAttributeDescription{
            //     binding: 0,
            //     type_
            // });
        }
        let disp = (type_.get_cells_count() as f32 / 4.0).ceil() as u32;
        let layout_location = if self.glsl_version.have_explicit_attri_location() {
            let line = format!("layout (location = {})", self.input_locations).to_owned();
            line
        } else {
            String::new()
        };
        self.input_locations += disp;
        self.inputs.insert(name.to_owned(), type_);
        match (type_.is_int(), self.sh_type) {
            (true, ShaderType::Fragment) => {
                self.source +=
                    format!("{layout_location} {interpolation} in {glsl_type} {name};\n").as_str();
            }
            _ => {
                self.source += format!("{layout_location} in {glsl_type} {name};\n").as_str();
            }
        }
        self
    }

    /// Добавляет код
    pub fn code(&mut self, code: &str) -> &mut Self {
        self.source += code;
        self.source += "\n";
        self
    }

    pub fn include(&mut self, inclusion: &str) -> &mut Self {
        self.code(format!("#include \"{inclusion}\"").as_str())
    }

    pub fn ifdef(&mut self, name: &str) -> &mut Self {
        self.code(format!("#ifdef {}", name).as_str())
    }

    pub fn ifndef(&mut self, name: &str) -> &mut Self {
        self.code(format!("#ifndef {}", name).as_str())
    }

    pub fn endif(&mut self) -> &mut Self {
        self.code(format!("#endif").as_str())
    }

    /// Объявляет макрос-переменную
    pub fn define(&mut self, name: &str, expr: &str) -> &mut Self {
        self.code(format!("#define {} ({})", name, expr).as_str())
    }

    pub fn source_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        self.source.hash(&mut hasher);
        hasher.finish()
    }

    pub fn spirv_hash(&self) -> u64 {
        self.spirv_hash
    }

    /// Строит шейдер.
    /// Здесь GLSL код компилируется в SPIR-V.
    pub fn build(&mut self) -> Result<&Self, String> {
        let hash = self.source_hash();
        let fname = format!("./shader_cache/{:X}.spv", hash);
        let cache_path = Path::new("./shader_cache/");
        let spv_path = Path::new(fname.as_str());

        if !cache_path.is_dir() {
            std::fs::create_dir(cache_path).unwrap();
        }

        let (glslc, spirv_opt) = {
            #[cfg(target_os = "windows")]
            {
                let glslc = ".\\vulkan_sdk\\Bin\\glslc.exe";
                let spirv_opt = ".\\vulkan_sdk\\Bin\\spirv-opt.exe";
                if !std::fs::metadata(glslc).unwrap().is_file() {
                    return Err(format!("Компилятор шейдеров {glslc} не найден."));
                }
                if std::fs::metadata(glslc).unwrap().is_file() {
                    (glslc, spirv_opt)
                } else {
                    (glslc, "")
                }
            }
            #[cfg(target_os = "linux")]
            {
                ("glslc", "spirv-opt")
            }
        };

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
            
            // let mut file = OpenOptions::new()
            //     .write(true)
            //     .create_new(true)
            //     .open(format!("./shader_cache/{hash:X}_composed.glsl")).unwrap();
            // file.write(source).unwrap();
            // file.sync_all().unwrap();
            // drop(file);
            let child_compiler = std::process::Command::new(glslc)
                .args([
                    format!("-fshader-stage={}", sh_type).as_str(),
                    //"-fauto-bind-uniforms",
                    "-fpreserve-bindings",
                    "--target-env=vulkan1.2",
                    "--target-spv=spv1.3",
                    "-fentry-point=main",
                    "-",
                    "-O0",
                    "-o",
                    spv_path.to_str().unwrap(),
                ])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();
            let mut child_compiler = match child_compiler {
                Ok(child) => child,
                Err(_) => return Err("Не установлен компилятор шейдеров glslc.".to_owned()),
            };
            let mut spirv_log = String::new();
            {
                let compiler_stdin = child_compiler.stdin.as_mut().unwrap();
                compiler_stdin.write_all(source).unwrap();
            }
            match child_compiler.wait() {
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
                        return Err(format!("{}\nОшибка шейдера (исходник с нуменованными строками представлен выше)\n{}", numbered_src, spirv_log));
                    }
                }
                Err(_) => return Err(format!("Шейдер не скомпилирован")),
            }
        };

        if spv_path.is_file() {
            if spirv_opt.len() > 0 {
                std::process::Command::new(spirv_opt)
                    .args([
                        "-O", spv_path.to_str().unwrap(),
                        "-o", spv_path.to_str().unwrap()]
                    )
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
            }
            let mut file = File::open(spv_path).unwrap();
            let mut spv = Vec::with_capacity(45000);
            file.read_to_end(&mut spv).unwrap();
            unsafe {
                self.module = Some(
                    vulkano::shader::ShaderModule::new(
                        self.device.clone(),
                        ShaderModuleCreateInfo::new(utils::cast_slice(spv.as_ref()))
                    ).unwrap()
                );
            }
            
            /*match self.module {
                Some(ref module) => {
                    let ep = module.entry_point("main").unwrap();
                    let module_uniforms = ep
                        .descriptor_binding_requirements()
                        .map(|((set, binding), _)| (set, binding))
                        .collect::<HashSet<_>>();

                    let source_uniforms = self.uniforms.clone();
                    for (
                        _,
                        ShaderSourceUniform {
                            name,
                            type_name: _,
                            set,
                            binding,
                        },
                    ) in &source_uniforms
                    {
                        let key = (*set as u32, *binding as u32);
                        if !module_uniforms.contains(&key) {
                            self.uniforms.remove(name);
                        }
                    }
                }
                None => (),
            };*/
            let mut hasher = DefaultHasher::new();
            spv.hash(&mut hasher);
            self.spirv_hash = hasher.finish();
            return Ok(self);
        };

        Err(format!("Шейдер не скомпилирован"))
    }
}

/// Строитель шейдера
pub struct ShaderProgramBuilder {
    vertex_shader_source: String,
    fragment_shader_source: String,
    tess_controll_source: String,
    tess_eval_source: String,
    compute_source: String,

    vertex_shader: Option<Arc<vulkano::shader::ShaderModule>>,
    fragment_shader: Option<Arc<vulkano::shader::ShaderModule>>,
    fragment_outputs: HashMap<String, AttribType>,
    tess_controll: Option<Arc<vulkano::shader::ShaderModule>>,
    tess_eval: Option<Arc<vulkano::shader::ShaderModule>>,
    compute: Option<Arc<vulkano::shader::ShaderModule>>,
    uniforms_locations: HashMap<String, (u32, u32)>,
    uniforms_types: HashMap<String, String>,
    uniform_constant: Option<(String, String)>,
    hash: u64
}

impl ShaderProgramBuilder {
    fn check_uniforms_compatibility<T>(
        uniforms_s1: &HashMap<String, T>,
        uniforms_s2: &HashMap<String, T>,
    ) -> bool
    where
        T: Eq + PartialEq,
    {
        let keys_a = HashSet::<_>::from_iter(
            uniforms_s1
                .keys()
                .map(|element| element.clone())
                .collect::<Vec<String>>(),
        );
        let keys_b = HashSet::<_>::from_iter(
            uniforms_s2
                .keys()
                .map(|element| element.clone())
                .collect::<Vec<String>>(),
        );
        for name in keys_a.intersection(&keys_b) {
            if uniforms_s1[name] != uniforms_s2[name] {
                return false;
            }
        }
        true
    }

    /// Фрагментный шейдер
    pub fn fragment(&mut self, shader: &Shader) -> Result<&mut Self, String> {
        let mut uniforms_locations = HashMap::new();
        let mut uniforms_types = HashMap::new();
        for (
            uname,
            ShaderSourceUniform {
                set,
                binding,
                type_name,
                ..
            },
        ) in &shader.uniforms
        {
            uniforms_locations.insert(uname.clone(), (*set, *binding));
            uniforms_types.insert(uname.clone(), type_name.clone());
        }
        let loc_compat =
            Self::check_uniforms_compatibility(&self.uniforms_locations, &uniforms_locations);
        let type_compat = Self::check_uniforms_compatibility(&self.uniforms_types, &uniforms_types);

        if !loc_compat {
            return Err(format!("Фрагментный шейдер использует те же имена uniform-переменных с разным расположением."));
        }
        if !type_compat {
            return Err(format!(
                "Фрагментный шейдер использует те же имена uniform-переменных с другими типами."
            ));
        }

        self.hash ^= shader.spirv_hash();
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
            Some(_) => {
                return Err(format!(
                    "Uniform-константу можно объявить только в одном шейдере."
                ))
            }
            None => {
                self.uniform_constant = match &shader.push_constants {
                    Some(pc) => Some((pc.1.name.to_owned(), pc.1.type_name.to_owned())),
                    None => None,
                }
            }
        }
        Ok(self)
    }

    /// Вершинный шейдер
    pub fn vertex(&mut self, shader: &Shader) -> Result<&mut Self, String> {
        let mut uniforms_locations = HashMap::new();
        let mut uniforms_types = HashMap::new();
        for (
            uname,
            ShaderSourceUniform {
                set,
                binding,
                type_name,
                ..
            },
        ) in &shader.uniforms
        {
            uniforms_locations.insert(uname.clone(), (*set, *binding));
            uniforms_types.insert(uname.clone(), type_name.clone());
        }

        let loc_compat =
            Self::check_uniforms_compatibility(&self.uniforms_locations, &uniforms_locations);
        let type_compat = Self::check_uniforms_compatibility(&self.uniforms_types, &uniforms_types);

        if !loc_compat {
            return Err(format!("Вершинный шейдер использует существующие имена uniform-переменных с разными расположениями."));
        }
        if !type_compat {
            return Err(format!("Вершинный шейдер использует существующие имена uniform-переменных с разными типами."));
        }

        self.hash ^= shader.spirv_hash();
        self.vertex_shader_source = shader.source.clone();
        self.vertex_shader = shader.module.clone();
        for (name, location) in &uniforms_locations {
            self.uniforms_locations.insert(name.clone(), *location);
        }
        for (name, _type) in &uniforms_types {
            self.uniforms_types.insert(name.clone(), _type.clone());
        }
        /*match self.uniform_constant {
            Some(_) => return Err(format!("Uniform-константу можно объявить только в одном шейдере.")),
            None => self.uniform_constant =
            match &shader.push_constants {
                Some(pc) => Some((pc.1.name.to_owned(), pc.1.type_name.to_owned())),
                None => None
            }
        }*/
        Ok(self)
    }

    /// Пара тесселяционных шейдеров
    pub fn tesselation(
        &'static mut self,
        eval: &Shader,
        control: &Shader,
    ) -> Result<&mut Self, String> {
        let mut eval_uniforms_locations = HashMap::new();
        let mut eval_uniforms_types = HashMap::new();
        let mut control_uniforms_locations = HashMap::new();
        let mut control_uniforms_types = HashMap::new();

        for (
            uname,
            ShaderSourceUniform {
                set,
                binding,
                type_name,
                ..
            },
        ) in &eval.uniforms
        {
            eval_uniforms_locations.insert(uname.clone(), (*set, *binding));
            eval_uniforms_types.insert(uname.clone(), type_name.clone());
        }
        for (
            uname,
            ShaderSourceUniform {
                set,
                binding,
                type_name,
                ..
            },
        ) in &control.uniforms
        {
            control_uniforms_locations.insert(uname.clone(), (*set, *binding));
            control_uniforms_types.insert(uname.clone(), type_name.clone());
        }

        let loc_compat =
            Self::check_uniforms_compatibility(&self.uniforms_locations, &eval_uniforms_locations)
                && Self::check_uniforms_compatibility(
                    &self.uniforms_locations,
                    &control_uniforms_locations,
                );

        let type_compat =
            Self::check_uniforms_compatibility(&self.uniforms_types, &eval_uniforms_types)
                && Self::check_uniforms_compatibility(
                    &self.uniforms_types,
                    &control_uniforms_types,
                );

        if !loc_compat {
            return Err(format!("Шейдеры тесселяции использует одни и те же имена uniform-переменных с разными расположениями."));
        }
        if !type_compat {
            return Err(format!("Шейдеры тесселяции использует одни и те же имена uniform-переменных с разными типами."));
        }
        self.hash ^= eval.spirv_hash();
        self.hash ^= control.spirv_hash();

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
            Some(_) => {
                return Err(format!(
                    "Uniform-константу можно объявить только в одном шейдере."
                ))
            }
            None => {
                self.uniform_constant = match &eval.push_constants {
                    Some(pc) => Some((pc.1.name.to_owned(), pc.1.type_name.to_owned())),
                    None => None,
                }
            }
        }
        match self.uniform_constant {
            Some(_) => {
                return Err(format!(
                    "Uniform-константу можно объявить только в одном шейдере."
                ))
            }
            None => {
                self.uniform_constant = match &control.push_constants {
                    Some(pc) => Some((pc.1.name.to_owned(), pc.1.type_name.to_owned())),
                    None => None,
                }
            }
        }

        Ok(self)
    }

    /// Вычислительный шейдер
    /// TODO: реализовать
    pub fn compute(&mut self) -> &mut Self {
        panic!("Не реализовано");
    }

    pub fn build_mutex(self, device: Arc<Device>) -> Result<ShaderProgramRef, String> {
        Ok(RcBox::construct(self.build(device)?))
    }

    /// Строит шейдерную программу
    pub fn build(self, device: Arc<Device>) -> Result<ShaderProgram, String> {
        Ok(ShaderProgram {
            device: device.clone(),
            pipeline: PipelineType::None,
            subpass_id: (0, 0),

            vertex_shader: self.vertex_shader.clone(),
            fragment_shader: self.fragment_shader.clone(),
            tess_controll: self.tess_controll.clone(),
            tess_eval: self.tess_eval.clone(),
            compute: self.compute.clone(),
            fragment_outputs: self.fragment_outputs.clone(),

            uniforms_types: self.uniforms_types,
            uniforms_locations: self.uniforms_locations,

            vertex_shader_source: self.vertex_shader_source,
            fragment_shader_source: self.fragment_shader_source,
            tess_controll_source: self.tess_controll_source,
            tess_eval_source: self.tess_eval_source,
            compute_source: self.compute_source,

            cull_faces: CullMode::None,

            hash: self.hash,
        })
    }
}

use vulkano::descriptor_set::WriteDescriptorSet;
pub trait PipelineUniform: ShaderStructUniform + std::marker::Send + std::marker::Sync {}

/// Буфер uniform-переменных для шейдеров
pub struct ShaderProgramUniformBuffer {
    write_set_descriptors: HashMap<u32, Vec<WriteDescriptorSet>>,
    //write_set_descriptors: HashMap<u32, Vec<(Vec<Subbuffer<[u8]>>, u32, u32)>>,
    uniforms_sets: HashMap<u32, Arc<PersistentDescriptorSet>>,
    uniforms_locations: HashMap<String, (u32, u32)>,
    //uniform_buffer: Arc<SubbufferAllocator>,
    device: Arc<Device>,
    pipeline: PipelineType,
}

impl Debug for ShaderProgramUniformBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderProgramUniformBuffer")
            .field("uniforms_locations", &self.uniforms_locations)
            .field("device", &self.device)
            .field("pipeline", &self.pipeline)
            .finish()
    }
}

impl Clone for ShaderProgramUniformBuffer {
    fn clone(&self) -> Self {
        Self {
            write_set_descriptors: HashMap::new(),
            uniforms_sets: self.uniforms_sets.clone(),
            uniforms_locations: self.uniforms_locations.clone(),
            //uniform_buffer: self.uniform_buffer.clone(),
            device: self.device.clone(),
            pipeline: self.pipeline.clone(),
        }
    }
}

impl ShaderProgramUniformBuffer {
    /// Собирает дескрипторы наборов uniform-переменных (`PersistentDescriptorSet`).
    /// Полезно для формирования неизменяемого списка текстур материала
    #[inline]
    pub fn build_uniform_sets(
        &mut self,
        ds_allocator: Arc<StandardDescriptorSetAllocator>,
        sets: &[u32]
    ) -> Result<(), String> {
        let layouts = match self.pipeline.layout() {
            Some(layout) => layout,
            None => {
                return Err(format!(
                "Не получилось собрать буфер uniform-переменных: неподдерживаемый тип конвейера."
            ))
            }
        }
        .set_layouts();

        for set_num in sets {
            if let Some(set_desc) = self.write_set_descriptors.remove(set_num) {
                /*let set_desc = set_desc.into_iter().map(|subbuffers| {
                    if let WriteDescriptorSetElements::Buffer(subbuffer, ..) = subbuffers.elements() {
                        let mut pcbb = command_buffer_father.new_primary().unwrap();
                        let binding = subbuffers.binding();
                        let first_array_element = subbuffers.first_array_element();
                        let elements = subbuffer.into_iter().map(|(subbuffer, _)| {
                            pcbb.move_buffer_to_device(subbuffer.clone(), allocator).unwrap()
                        }).collect::<Vec<_>>();
                        let subbuffers = WriteDescriptorSet::buffer_array(binding, first_array_element, elements);
                        pcbb.execute_after(None).unwrap();
                        subbuffers
                    } else {
                        subbuffers
                    }
                });*/
                
                let res = PersistentDescriptorSet::new(
                    ds_allocator.as_ref(),
                    layouts.get(*set_num as usize).unwrap().clone(),
                    set_desc,
                    []
                );
                let pers_decc_set = res.unwrap();
                self.uniforms_sets.insert(*set_num, pers_decc_set);
            }
        }
        Ok(())
    }

    /// Проверяет инициализирован ли набор uniform-переменных
    #[inline]
    pub fn is_set_initialized(&self, set_num: u32) -> bool {
        self.uniforms_sets.contains_key(&set_num)
    }

    /// Очищает заданный набор uniform-переменных
    #[inline]
    pub fn clear_uniform_set(&mut self, set_num: u32) {
        //self.uniform_set_builders.remove(&set_num);
        self.uniforms_sets.remove(&set_num);
    }

    /// Передаёт в шейдер массив данных неопределённой структуры
    #[inline]
    pub fn uniform_structure(
        &mut self,
        allocator: Arc<BumpMemoryAllocator>,
        obj: &[&[f32]],
        index: u32,
        set_num: u32,
        binding_num: u32,
    ) {
        let pipeline_layout = self.pipeline.layout().unwrap();
        if !((set_num as usize) < pipeline_layout.set_layouts().len()
            && pipeline_layout.set_layouts()[set_num as usize]
                .bindings()
                .contains_key(&binding_num))
        {
            return;
        }

        let uniform_buffer = {
            let obj = obj
                .iter()
                .map(|vector| {
                    // Фикс для помойных интеловских видюх.
                    let mask = (VULKANO_BUFFER_ATOMIC_SIZE - 1) >> 2;
                    let dummy_zeros = (VULKANO_BUFFER_ATOMIC_SIZE >> 2) - (vector.len() & mask);
                    let dummy_zeros = (0..dummy_zeros).map(|_| 0.0f32);
                    let vector = vector
                        .iter()
                        .map(|fl| *fl)
                        .chain(dummy_zeros)
                        .collect::<Vec<_>>();
                    let ba = Buffer::from_iter(
                        allocator.clone(),
                        BufferCreateInfo {
                            usage: BufferUsage::UNIFORM_BUFFER | BufferUsage::TRANSFER_SRC,
                            ..Default::default()
                        },
                        AllocationCreateInfo {
                            memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE | MemoryTypeFilter::PREFER_DEVICE,
                            ..Default::default()
                        },
                        vector
                    ).unwrap().into_bytes();
                    ba
                })
                .collect::<Vec<_>>();
            WriteDescriptorSet::buffer_array(binding_num, index, obj)
        };
        match self.write_set_descriptors.get_mut(&set_num) {
            Some(set_buffer) => {
                set_buffer.push(uniform_buffer);
            }
            None => {
                let mut uniform_set = Vec::new();
                uniform_set.push(uniform_buffer);
                self.write_set_descriptors.insert(set_num, uniform_set);
            }
        }
    }

    /// Передаёт uniform-переменную в шейдер
    #[inline]
    pub fn uniform<T>(&mut self, allocator: Arc<BumpMemoryAllocator>, obj: T, set_num: u32, binding_num: u32)
    where
        T: ShaderStructUniform
            + std::fmt::Debug
            + std::marker::Send
            + std::marker::Sync
            + Clone
            + 'static,
    {
        let s = [obj];
        let vector = utils::cast_slice::<T, f32>(&s);
        self.uniform_structure(allocator, &[vector], 0, set_num, binding_num);
    }

    /// Передаёт массив uniform-переменных в шейдер
    #[inline]
    pub fn uniform_array<T>(&mut self, allocator: Arc<BumpMemoryAllocator>, objs: &[T], index: u32, set_num: u32, binding_num: u32)
    where
        T: ShaderStructUniform
            + std::fmt::Debug
            + std::marker::Send
            + std::marker::Sync
            + Clone
            + 'static,
    {
        //let vector = bytemuck::cast_slice::<T, f32>(objs).chunks(size_of::<T>() / size_of::<f32>()).map(|a: &[f32]| a).collect::<Vec<_>>();
        let vector = utils::cast_slice::<T, f32>(objs);
        self.uniform_structure(allocator, &[vector], index, set_num, binding_num);
    }

    #[inline]
    pub fn uniform_sampler(&mut self, texture: &Texture, set_num: u32, binding_num: u32) {
        let pipeline_layout = self.pipeline.layout().unwrap();
        if !((set_num as usize) < pipeline_layout.set_layouts().len()
            && pipeline_layout.set_layouts()[set_num as usize]
                .bindings()
                .contains_key(&binding_num))
        {
            return;
        }

        let wds = WriteDescriptorSet::image_view_sampler(
            binding_num as u32,
            texture.image_view().clone(),
            texture.sampler().clone(),
        );
        match self.write_set_descriptors.get_mut(&set_num) {
            Some(set_buffer) => {
                set_buffer.push(wds);
            }
            None => {
                let mut uniform_set = Vec::new();
                uniform_set.push(wds);
                self.write_set_descriptors.insert(set_num, uniform_set);
            }
        };
    }

    #[inline]
    pub fn uniform_sampler_array(
        &mut self,
        textures: &[&Texture],
        first_index: u32,
        set_num: u32,
        binding_num: u32,
    ) {
        let pipeline_layout = self.pipeline.layout().unwrap();
        if !((set_num as usize) < pipeline_layout.set_layouts().len()
            && pipeline_layout.set_layouts()[set_num as usize]
                .bindings()
                .contains_key(&binding_num))
        {
            return;
        }
        let _textures = textures
            .iter()
            .map(|texture| (texture.image_view().clone(), texture.sampler().clone()))
            .collect::<Vec<_>>();

        let wds = WriteDescriptorSet::image_view_sampler_array(
            binding_num as _,
            first_index as _,
            _textures,
        );
        match self.write_set_descriptors.get_mut(&set_num) {
            Some(set_buffer) => {
                set_buffer.push(wds);
            }
            None => {
                let mut uniform_set = Vec::new();
                uniform_set.push(wds);
                self.write_set_descriptors.insert(set_num, uniform_set);
            }
        };
    }

    #[inline]
    pub fn uniform_none_array(
        &mut self,
        num_elements: u32,
        first_array_element: u32,
        set_num: u32,
        binding_num: u32,
    ) {
        let wds = WriteDescriptorSet::none_array(
            binding_num as _,
            first_array_element as _,
            num_elements as _,
        );
        match self.write_set_descriptors.get_mut(&set_num) {
            Some(set_buffer) => {
                set_buffer.push(wds);
            }
            None => {
                let mut uniform_set = Vec::new();
                uniform_set.push(wds);
                self.write_set_descriptors.insert(set_num, uniform_set);
            }
        };
    }

    #[inline]
    pub fn uniform_by_name<T>(&mut self, allocator: Arc<BumpMemoryAllocator>, obj: T, name: &str) -> Result<(), String>
    where
        T: ShaderStructUniform
            + std::fmt::Debug
            + std::marker::Send
            + std::marker::Sync
            + Clone
            + 'static,
    {
        //println!("uniform {}", name);
        let (set_num, binding_num) = *match self.uniforms_locations.get(name) {
            Some(val) => val,
            None => {
                return Err(format!(
                    "Uniform-переменная {} не объявлена в этом шейдере.",
                    name
                ))
            }
        };
        self.uniform(allocator, obj, set_num, binding_num);
        Ok(())
    }

    #[inline]
    pub fn uniform_array_by_name<T>(
        &mut self,
        allocator: Arc<BumpMemoryAllocator>,
        objs: &[T],
        index: u32,
        name: &str,
    ) -> Result<(), String>
    where
        T: ShaderStructUniform
            + std::fmt::Debug
            + std::marker::Send
            + std::marker::Sync
            + Pod
            + 'static,
    {
        //println!("uniform {}", name);
        let (set_num, binding_num) = *match self.uniforms_locations.get(name) {
            Some(val) => val,
            None => {
                return Err(format!(
                    "Uniform-переменная {} не объявлена в этом шейдере.",
                    name
                ))
            }
        };
        self.uniform_array(allocator, objs, index, set_num, binding_num);
        Ok(())
    }

    #[inline]
    pub fn uniform_sampler_by_name(
        &mut self,
        texture: &Texture,
        name: &str,
    ) -> Result<(u32, u32), String> {
        //println!("uniform {}", name);
        let (set_num, binding_num) = *match self.uniforms_locations.get(name) {
            Some(val) => val,
            None => {
                return Err(format!(
                    "Uniform-переменная {} не объявлена в этом шейдере.",
                    name
                ));
            }
        };
        self.uniform_sampler(texture, set_num, binding_num);
        Ok((set_num, binding_num))
    }

    pub fn uniform_location_by_name(&self, name: &str) -> Option<(u32, u32)> {
        match self.uniforms_locations.get(name) {
            Some(loc) => Some(*loc),
            None => None,
        }
    }

    #[inline]
    pub fn uniform_sampler_array_by_name(
        &mut self,
        textures: &[&Texture],
        first_index: u32,
        name: &str,
    ) -> Result<(u32, u32), String> {
        let (set_num, binding_num) = *match self.uniforms_locations.get(name) {
            Some(val) => val,
            None => {
                return Err(format!(
                    "Uniform-переменная {} не объявлена в этом шейдере.",
                    name
                ));
            }
        };
        self.uniform_sampler_array(textures, first_index, set_num, binding_num);
        Ok((set_num, binding_num))
    }

    #[inline]
    pub fn uniform_none_array_by_name(
        &mut self,
        num_elements: u32,
        first_array_element: u32,
        name: &str,
    ) -> Result<(u32, u32), String> {
        let (set_num, binding_num) = *match self.uniforms_locations.get(name) {
            Some(val) => val,
            None => {
                return Err(format!(
                    "Uniform-переменная {} не объявлена в этом шейдере.",
                    name
                ));
            }
        };
        self.uniform_none_array(num_elements, first_array_element, set_num, binding_num);
        Ok((set_num, binding_num))
    }

    pub fn storage_buffer<T>(&mut self, allocator: Arc<StandardMemoryAllocator>, obj: T, set_num: u32, binding_num: u32)
    where
        T: std::marker::Send + std::marker::Sync + Pod + 'static,
    {
        if self.uniforms_sets.contains_key(&set_num) {
            return;
        }
        let uniform_buffer = Buffer::from_data(
            allocator,
            BufferCreateInfo{usage: BufferUsage::STORAGE_BUFFER, ..Default::default()},
            AllocationCreateInfo{
                memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE | MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            obj,
        )
        .unwrap();
        let ub = WriteDescriptorSet::buffer(binding_num as u32, uniform_buffer);

        match self.write_set_descriptors.get_mut(&set_num) {
            Some(set_buffer) => {
                set_buffer.push(ub);
            }
            None => {
                let mut uniform_set = vec![];
                uniform_set.push(ub);
                self.write_set_descriptors.insert(set_num, uniform_set);
            }
        };
    }

    pub fn storage_buffer_by_name<T>(&mut self, allocator: Arc<StandardMemoryAllocator>, obj: T, name: &str) -> Result<(), String>
    where
        T: std::marker::Send + std::marker::Sync + Pod + 'static,
    {
        let (set_num, binding_num) = *match self.uniforms_locations.get(name) {
            Some(val) => val,
            None => {
                return Err(format!(
                    "Uniform-переменная {} не объявлена в этом шейдере.",
                    name
                ))
            }
        };
        self.storage_buffer(allocator, obj, set_num, binding_num);
        Ok(())
    }
}

/// Шейдерная программа
#[derive(Clone)]
pub struct ShaderProgram {
    device: Arc<Device>,
    pipeline: PipelineType,
    subpass_id: (u32, u32),

    uniforms_types: HashMap<String, String>,
    uniforms_locations: HashMap<String, (u32, u32)>,

    vertex_shader: Option<Arc<vulkano::shader::ShaderModule>>,
    fragment_shader: Option<Arc<vulkano::shader::ShaderModule>>,
    fragment_outputs: HashMap<String, AttribType>,
    tess_controll: Option<Arc<vulkano::shader::ShaderModule>>,
    tess_eval: Option<Arc<vulkano::shader::ShaderModule>>,
    compute: Option<Arc<vulkano::shader::ShaderModule>>,

    vertex_shader_source: String,
    fragment_shader_source: String,
    tess_controll_source: String,
    tess_eval_source: String,
    compute_source: String,

    pub cull_faces: CullMode,

    hash: u64,
}

impl Debug for ShaderProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ds = f.debug_struct("ShaderProgram");
        ds.field("device", &self.device)
            .field("pipeline", &self.pipeline);

        if self.vertex_shader.is_some() {
            ds.field("vertex_shader", self.vertex_shader.as_ref().unwrap());
        }
        if self.tess_controll.is_some() {
            ds.field("tess_controll", self.tess_controll.as_ref().unwrap());
        }
        if self.tess_eval.is_some() {
            ds.field("tess_eval", self.tess_eval.as_ref().unwrap());
        }
        if self.fragment_shader.is_some() {
            ds.field("fragment_shader", self.fragment_shader.as_ref().unwrap());
        }
        if self.compute.is_some() {
            ds.field("compute", self.compute.as_ref().unwrap());
        }
        ds.finish()
    }
}

impl ShaderProgram {
    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn vertex_shader_source(&self) -> &String {
        &self.vertex_shader_source
    }

    pub fn fragment_shader_source(&self) -> &String {
        &self.fragment_shader_source
    }

    pub fn tess_controll_source(&self) -> &String {
        &self.tess_controll_source
    }

    pub fn tess_eval_source(&self) -> &String {
        &self.tess_eval_source
    }

    pub fn compute_source(&self) -> &String {
        &self.compute_source
    }
}

impl ShaderProgram {
    pub fn builder() -> ShaderProgramBuilder {
        ShaderProgramBuilder {
            vertex_shader_source: String::new(),
            fragment_shader_source: String::new(),
            tess_controll_source: String::new(),
            tess_eval_source: String::new(),
            compute_source: String::new(),

            vertex_shader: None,
            fragment_shader: None,
            fragment_outputs: HashMap::new(),
            tess_controll: None,
            tess_eval: None,
            compute: None,
            uniforms_locations: HashMap::new(),
            uniforms_types: HashMap::new(),
            uniform_constant: None,
            hash: 0
        }
    }

    pub fn new_uniform_buffer(&self) -> ShaderProgramUniformBuffer {
        /*let uniform_buffer = SubbufferAllocator::new(
            allocator,
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
        );*/
        ShaderProgramUniformBuffer {
            write_set_descriptors: HashMap::new(),
            uniforms_sets: HashMap::new(),
            uniforms_locations: self.uniforms_locations.clone(),
            //uniform_buffer: uniform_buffer,
            device: self.device.clone(),
            pipeline: self.pipeline.clone(),
        }
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn pipeline(&self) -> PipelineType {
        self.pipeline.clone()
    }

    /// Перестраивает `Pipeline` для использования с заданным `Subpass`'ом.
    /// Возвращает true, если переданный subpass отличается от использованного в прошлый раз
    pub fn use_subpass<T: VertexDefinition>(
        &mut self,
        subpass: Subpass,
        cull_mode: CullMode,
        vbd: Option<T>,
    ) -> (PipelineType, bool) {
        let render_pass_id = subpass.render_pass().as_ref() as *const RenderPass as u32;
        let subpass_full_id = (render_pass_id, subpass.index());

        if subpass_full_id == self.subpass_id {
            return (self.pipeline.clone(), false);
        }
        #[cfg(debug)]
        {
            println!("Исользуется новый subpass {:?}", subpass_full_id);
        }
        self.subpass_id = subpass_full_id;
        /*self.uniforms_sets.clear();*/
        let depth_test = subpass.subpass_desc().depth_stencil_attachment.is_some();

        let mut stages = vec![];

        if let Some(ref shader) = self.vertex_shader {
            let ep = shader.entry_point("main").unwrap();
            stages.push(PipelineShaderStageCreateInfo::new(ep));
        }
        if let Some(ref shader) = self.fragment_shader {
            let ep = shader.entry_point("main").unwrap();
            stages.push(PipelineShaderStageCreateInfo::new(ep));
        }
        if self.tess_eval.is_some() && self.tess_controll.is_some() {
            let ep_c = self.tess_controll.as_ref().unwrap().entry_point("main").unwrap();
            let ep_e = self.tess_eval.as_ref().unwrap().entry_point("main").unwrap();
            stages.push(PipelineShaderStageCreateInfo::new(ep_c));
            stages.push(PipelineShaderStageCreateInfo::new(ep_e));
        }
        let vertex_input_interface = &stages[0].entry_point.info().input_interface;
        let vertex_input_state = match vbd {
            Some(vbd) => vbd.definition(vertex_input_interface).unwrap(),
            None => VkVertex::per_vertex().definition(vertex_input_interface).unwrap(),
        };
        let layout = PipelineLayout::new(
            self.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(self.device.clone())
                .unwrap(),
        )
        .unwrap();
        let is_desc = vertex_input_state.bindings.iter().collect::<Vec<_>>();
        let pipeline = vulkano::pipeline::GraphicsPipeline::new(
            self.device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState::default()),
                rasterization_state: Some(RasterizationState{
                    depth_clamp_enable: true,
                    cull_mode: cull_mode,
                    ..RasterizationState::default()
                }),
                depth_stencil_state: if depth_test {
                    Some(DepthStencilState {
                        depth: Some(DepthState::simple()),
                        ..Default::default()
                    })
                } else {
                    None
                },
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: if subpass.num_color_attachments() > 0 {
                    Some(ColorBlendState::with_attachment_states(
                        subpass.num_color_attachments(),
                        ColorBlendAttachmentState::default(),
                    ))
                } else {
                    None
                },
                subpass: Some(subpass.into()),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                ..GraphicsPipelineCreateInfo::layout(layout)
            }
        ).unwrap();
        /*let mut pipeline = vulkano::pipeline::GraphicsPipeline::start();
        pipeline = pipeline
            .input_assembly_state(vulkano::pipeline::graphics::input_assembly::InputAssemblyState::new())
            .viewport_state(vulkano::pipeline::graphics::viewport::ViewportState::viewport_dynamic_scissor_irrelevant())
            .render_pass(subpass);

        if depth_test {
            pipeline = pipeline.depth_stencil_state(DepthStencilState::simple_depth_test());
        }
        //if self.
        self.pipeline = PipelineType::Graphics(pipeline.build(self.device.clone()).unwrap());*/
        self.pipeline = PipelineType::Graphics(pipeline);
        (self.pipeline.clone(), true)
    }

    /// Возвращает имена выходов фрагментного фейдера
    pub fn outputs(&self) -> &HashMap<String, AttribType> {
        &self.fragment_outputs
    }
}

/// trait для удобной передачи шейдеров и uniform-переменные в `AutoCommandBufferBuilder`
pub trait ShaderProgramBinder {
    /// Присоединение шейдерной программы (`GraphicsPipeline`) к `AutoCommandBufferBuilder`'у
    fn bind_shader_program(&mut self, shader: &ShaderProgram) -> Result<&mut Self, String>;

    /// Присоединение uniform-переменных к `AutoCommandBufferBuilder`'у
    fn bind_shader_uniforms(
        &mut self,
        allocator: Arc<StandardDescriptorSetAllocator>,
        uniform_buffer: &mut ShaderProgramUniformBuffer,
        only_dynamic: bool,
    ) -> Result<&mut Self, String>;

    fn bind_uniform_constant<T: BufferContents>(
        &mut self,
        shader: &ShaderProgram,
        data: T,
    ) -> Result<&mut Self, String>;
}

impl<BufferType> ShaderProgramBinder for AutoCommandBufferBuilder<BufferType> {
    #[inline]
    fn bind_shader_program(&mut self, shader: &ShaderProgram) -> Result<&mut Self, String> {
        match shader.pipeline() {
            PipelineType::Graphics(pipeline) => self.bind_pipeline_graphics(pipeline).map_err(|e| e.to_string()),
            PipelineType::Compute(pipeline) => self.bind_pipeline_compute(pipeline).map_err(|e| e.to_string()),
            PipelineType::None => Err("Не установлен Subpass".to_owned()),
        }
    }

    #[inline]
    fn bind_uniform_constant<T: BufferContents>(
        &mut self,
        shader: &ShaderProgram,
        data: T,
    ) -> Result<&mut Self, String> {
        let pipeline_layout = shader.pipeline.layout().unwrap();
        self
            .push_constants(pipeline_layout.clone(), 0, data)
            .map_err(|e| e.to_string())?;
        Ok(self)
    }

    #[inline]
    fn bind_shader_uniforms(
        &mut self,
        allocator: Arc<StandardDescriptorSetAllocator>,
        uniform_buffer: &mut ShaderProgramUniformBuffer,
        only_dynamic: bool,
    ) -> Result<&mut Self, String> {
        let pipeline_layout = uniform_buffer.pipeline.layout().unwrap();
        let layouts = pipeline_layout.set_layouts();
        let mut desc_sets = Vec::new();

        let keys = uniform_buffer
            .write_set_descriptors
            .keys()
            .map(|elem| *elem)
            .collect::<Vec<_>>();
        for set_num in keys {
            let descriptor_writes = uniform_buffer
                .write_set_descriptors
                .remove(&set_num)
                .unwrap();
            let layout = match layouts.get(set_num as usize)
            {
                Some(layout) => layout,
                None => return Err(format!("В шейдере есть неиспользуемые uniform-переменные. Набор {} не используется нигде.", set_num))
            };

            let set = PersistentDescriptorSet::new(&allocator, layout.clone(), descriptor_writes.clone(), []);
            match set {
                Ok(set) => desc_sets.push((set_num, set)),
                Err(e) => {
                    //println!("{}", shader.fragment_shader_source);
                    let ve = 
                    if let Validated::ValidationError(ref e) = e {
                        match descriptor_writes.get(3).unwrap().elements() {
                            vulkano::descriptor_set::WriteDescriptorSetElements::None(_) => todo!(),
                            vulkano::descriptor_set::WriteDescriptorSetElements::Buffer(_) => todo!(),
                            vulkano::descriptor_set::WriteDescriptorSetElements::BufferView(_) => todo!(),
                            vulkano::descriptor_set::WriteDescriptorSetElements::ImageView(_) => todo!(),
                            vulkano::descriptor_set::WriteDescriptorSetElements::ImageViewSampler(e) => dbg!(e),
                            vulkano::descriptor_set::WriteDescriptorSetElements::Sampler(_) => todo!(),
                            vulkano::descriptor_set::WriteDescriptorSetElements::InlineUniformBlock(_) => todo!(),
                            vulkano::descriptor_set::WriteDescriptorSetElements::AccelerationStructure(_) => todo!(),
                        };
                        "Validated::ValidationError".to_owned()
                    } else {
                        "Validated::Error".to_owned()
                    };
                    return Err(format!(
                        "{ve}: Не удалось сформировать набор uniform-переменных №{}: {:?}",
                        set_num, e
                    ));
                }
            };
        }
        if !only_dynamic {
            for (set_num, desc_set) in &uniform_buffer.uniforms_sets {
                desc_sets.push((*set_num, desc_set.clone()));
            }
        }
        if desc_sets.len() == 0 {
            return Ok(self);
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
        //println!("desc_sets {}", desc_sets.len());
        let first_set_num = desc_sets.first().unwrap().0;
        let desc_sets = desc_sets
            .iter()
            .map(|elem| elem.1.clone())
            .collect::<Vec<_>>();
        self.bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            pipeline_layout.clone(),
            first_set_num as _,
            desc_sets,
        ).map_err(|e| e.to_string())?;
        Ok(self)
    }
}

#[derive(Clone, Debug)]
pub enum PipelineType {
    Compute(Arc<ComputePipeline>),
    Graphics(Arc<GraphicsPipeline>),
    None,
}

impl PipelineType {
    pub fn layout(&self) -> Option<&Arc<PipelineLayout>> {
        match self {
            Self::Compute(pipeline) => Some(pipeline.layout()),
            Self::Graphics(pipeline) => Some(pipeline.layout()),
            Self::None => None,
        }
    }
}
