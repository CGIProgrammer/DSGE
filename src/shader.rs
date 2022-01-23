extern crate spirv_compiler;

use std::collections::HashMap;
pub use super::glenums::{ShaderType, GLSLVersion, GLSLType, AttribType};
use std::sync::Arc;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::RenderPass;
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use crate::references::*;
use crate::vulkano::pipeline::Pipeline;
use vulkano::device::Device;

pub type ShaderProgramRef = RcBox<ShaderProgram>;

/// Trait для единообразной передачи uniform-структур и текстур в GLSL шейдер.
/// Он же используется для сборки GLSL шейдера
pub trait ShaderStructUniform
{
    fn structure() -> String;       // Должна возвращать текстовое представление структуры типа для GLSL
    fn glsl_type_name() -> String;  // Должна возвращать название типа
    fn texture(&self) -> Option<&crate::texture::TextureRef>; // Позволяет получить текстуру, если структура является таковой
}

/// Структура для построения GLSL шейдера и компиляции его в SPIR-V
pub struct Shader
{
    glsl_version: GLSLVersion,
    sh_type: ShaderType,
    device: Arc<vulkano::device::Device>,
    module: Option<Arc<vulkano::shader::ShaderModule>>,
    source: String,
    inputs    : HashMap<String, AttribType>,
    outputs   : HashMap<String, AttribType>,
    uniforms  : HashMap<usize, i32>,
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
            sh_type : shader_type,
            module : None,
            device : device,
            source : source,
            inputs : HashMap::new(),
            outputs : HashMap::new(),
            uniforms :  HashMap::new(),
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

    /// Объявление uniform-переменных
    pub fn uniform<T: ShaderStructUniform>(&mut self, name: &str, set: usize) -> &mut Self
    {
        let mut uniforms_in_set = match self.uniforms.get(&set) { Some(uc) => uc.clone(), None => 0 };
        self.source += format!("layout (set = {}, binding = {}) uniform {} {} {};\n", set, uniforms_in_set, T::glsl_type_name(), T::structure(), name).as_str();
        uniforms_in_set += 1;
        self.uniforms.insert(set, uniforms_in_set);
        self
    }

    /// Объявление одномерного сэмплера (текстуры)
    /// `name` - название внутри шейдера
    /// `set` - номер множества uniform-переменных
    /// `array` - объявить как массив
    pub fn uniform_sampler1d(&mut self, name: &str, set: usize, array : bool) -> &mut Self
    {
        let mut uniforms_in_set = match self.uniforms.get(&set) { Some(uc) => uc.clone(), None => 0 };
        self.source += format!("layout (set = {}, binding = {}) uniform sampler1D{} {};\n", set, uniforms_in_set, if array {"Array"} else {""}, name).as_str();
        uniforms_in_set += 1;
        self.uniforms.insert(set, uniforms_in_set);
        self
    }

    /// Объявление двухмерного сэмплера
    /// `name` - название внутри шейдера
    /// `set` - номер множества uniform-переменных
    /// `array` - объявить как массив
    pub fn uniform_sampler2d(&mut self, name: &str, set: usize, array : bool) -> &mut Self
    {
        let mut uniforms_in_set = match self.uniforms.get(&set) { Some(uc) => uc.clone(), None => 0 };
        self.source += format!("layout (set = {}, binding = {}) uniform sampler2D{} {};\n", set, uniforms_in_set, if array {"Array"} else {""}, name).as_str();
        uniforms_in_set += 1;
        self.uniforms.insert(set, uniforms_in_set);
        self
    }

    /// Объявление трёхмерного сэмплера
    /// `name` - название внутри шейдера
    /// `set` - номер множества uniform-переменных
    /// Не может быть массивом
    pub fn uniform_sampler3d(&mut self, name: &str, set: usize) -> &mut Self
    {
        let mut uniforms_in_set = match self.uniforms.get(&set) { Some(uc) => uc.clone(), None => 0 };
        self.source += format!("layout (set = {}, binding = {}) uniform sampler3D {};\n", set, uniforms_in_set, name).as_str();
        uniforms_in_set += 1;
        self.uniforms.insert(set, uniforms_in_set);
        self
    }

    /// Объявляет выход шейдера
    pub fn output(&mut self, name: &str, type_: AttribType) -> &mut Self
    {
        let layout_location = 
            if self.glsl_version.have_explicit_attri_location() {
                format!("layout(location = {}) ", self.outputs.len()).to_string()
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
                format!("layout(location = {}) ", self.inputs.len()).to_string()
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

    /// Строит шейдер.
    /// Здесь GLSL код компилируется в SPIR-V, и из него формируется `ShaderModule`
    pub fn build(&mut self) -> Result<&Self, String>
    {
        let mut compiler = spirv_compiler::CompilerBuilder::new().with_source_language(spirv_compiler::SourceLanguage::GLSL).build().unwrap();
        let sh_type = match self.sh_type {
            ShaderType::Vertex => spirv_compiler::ShaderKind::Vertex,
            ShaderType::Fragment => spirv_compiler::ShaderKind::Fragment,
            ShaderType::TesselationControl => spirv_compiler::ShaderKind::TessControl,
            ShaderType::TesselationEval => spirv_compiler::ShaderKind::TessEvaluation,
            ShaderType::Compute => spirv_compiler::ShaderKind::Compute,
            ShaderType::Geometry => spirv_compiler::ShaderKind::Geometry,
        };
        let spirv = compiler.compile_from_string(self.source.as_str(), sh_type);
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
                self.module = unsafe { Some(vulkano::shader::ShaderModule::from_words(self.device.clone(), spv_ok.as_slice()).unwrap()) };
                println!("Скопилирован {} шейдер (self.module={})", self.sh_type, self.module.is_some());
                return Ok(self);
            }
        };
    }
}

/// Строитель шейдера
#[allow(dead_code)]
pub struct ShaderProgramBuilder
{
    vertex_shader: Option<Arc<vulkano::shader::ShaderModule>>,
    fragment_shader: Option<Arc<vulkano::shader::ShaderModule>>,
    fragment_outputs: HashMap<String, AttribType>,
    tess_controll: Option<Arc<vulkano::shader::ShaderModule>>,
    tess_eval: Option<Arc<vulkano::shader::ShaderModule>>,
    compute: Option<Arc<vulkano::shader::ShaderModule>>,
    //uniforms  : HashMap<String, ShaderUniform>,
}

#[allow(dead_code)]
impl ShaderProgramBuilder
{
    /// Фрагментный шейдер
    pub fn fragment(&mut self, shader: &Shader) -> &mut Self
    {
        self.fragment_shader = shader.module.clone();
        self.fragment_outputs = shader.outputs.clone();
        self
    }

    /// Вершинный шейдер
    pub fn vertex(&mut self, shader: &Shader) -> &mut Self
    {
        self.vertex_shader = shader.module.clone();
        self
    }

    /// Пара тесселяционных шейдеров
    pub fn tesselation(&'static mut self, eval: &Shader, control: &Shader) -> &mut Self
    {
        self.tess_controll = control.module.clone();
        self.tess_eval = eval.module.clone();
        self
    }

    /// Вычислительный шейдер
    /// TODO: реализовать
    pub fn compute(&mut self) -> &mut Self
    {
        panic!("Не реализовано");
    }

    /// Строит шейдерную программу
    pub fn build(&mut self, device: Arc<Device>) -> Result<ShaderProgramRef, String>
    {        
        Ok(
            ShaderProgramRef::construct(
                ShaderProgram{
                    device : device.clone(),
                    pipeline : None,
                    render_pass : None,
                    //uniforms_values : HashMap::new(),
                    uniforms_sets : HashMap::new(),
                    vertex_shader : self.vertex_shader.clone(),
                    fragment_shader : self.fragment_shader.clone(),
                    tess_controll : self.tess_controll.clone(),
                    tess_eval : self.tess_eval.clone(),
                    compute : self.compute.clone(),
                    fragment_outputs : self.fragment_outputs.clone()
                }
            )
        )
    }
}

use vulkano::descriptor_set::persistent::PersistentDescriptorSetBuilder;
pub trait PipelineUniform: ShaderStructUniform + std::marker::Send + std::marker::Sync {}

/// Шейдерная программа
#[allow(dead_code)]
pub struct ShaderProgram
{
    device : Arc<Device>,
    pipeline : Option<Arc<vulkano::pipeline::GraphicsPipeline>>,
    render_pass : Option<Arc<RenderPass>>,
    //uniforms_values: HashMap<usize, Vec<Arc<dyn ShaderStructUniform + std::marker::Send + std::marker::Sync + 'static>>>,
    uniforms_sets: HashMap<usize, PersistentDescriptorSetBuilder>,
    vertex_shader : Option<Arc<vulkano::shader::ShaderModule>>,
    fragment_shader : Option<Arc<vulkano::shader::ShaderModule>>,
    fragment_outputs : HashMap<String, AttribType>,
    tess_controll : Option<Arc<vulkano::shader::ShaderModule>>,
    tess_eval : Option<Arc<vulkano::shader::ShaderModule>>,
    compute : Option<Arc<vulkano::shader::ShaderModule>>,
}

impl ShaderProgram
{
    pub fn builder() -> ShaderProgramBuilder
    {
        ShaderProgramBuilder {
            vertex_shader : None,
            fragment_shader : None,
            fragment_outputs : HashMap::new(),
            tess_controll : None,
            tess_eval : None,
            compute : None,
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
    
    /// Передаёт uniform-переменную в шейдер
    /// Может передавать как `TextureRef`, так и структуры, для которых определён trait ShaderStructUniform
    pub fn uniform<T>(&mut self, obj: &T, set_num: usize)
        where T: ShaderStructUniform + std::marker::Send + std::marker::Sync + Clone + 'static
    {
        let desc_set_layout = self.pipeline.as_ref().unwrap().layout().descriptor_set_layouts().get(set_num).unwrap();
        
        let set_builder = match self.uniforms_sets.get_mut(&set_num) {
            Some(builder) => {
                builder
            },
            None => {
                let set_b = PersistentDescriptorSet::start(desc_set_layout.clone());
                self.uniforms_sets.insert(set_num, set_b);
                self.uniforms_sets.get_mut(&set_num).unwrap()
            }
        };

        match obj.texture() {
            Some(sampler) => {
                let si = sampler.take();
                set_builder.add_sampled_image(si.image_view().clone(), si.sampler().clone()).unwrap();
            },
            None => {
                let uniform_buffer = CpuBufferPool::new(self.device.clone(), BufferUsage::all());
                let uniform_buffer_subbuffer = uniform_buffer.next(obj.clone()).unwrap();
                set_builder.add_buffer(uniform_buffer_subbuffer).unwrap();
            }
        }
        
    }
    
    /// Построить `Pipeline`
    /// Необходимо вызывать при изменении `Subpass`а.
    pub fn make_pipeline(&mut self, subpass : vulkano::render_pass::Subpass)
    {
        let depth_test = subpass.has_depth();
        let mut pipeline = vulkano::pipeline::GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<super::mesh::VkVertex>());
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

    #[allow(dead_code)]
    pub fn outputs(&self) -> &HashMap<String, AttribType>
    {
        &self.fragment_outputs
    }
}

use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::buffer::{CpuBufferPool, BufferUsage};
use vulkano::command_buffer::pool::CommandPoolBuilderAlloc;
use vulkano::pipeline::PipelineBindPoint;

/*
 * trait для удобной передачи шейдеров и uniform данных в AutoCommandBufferBuilder
 */
pub trait ShaderProgramBinder
{
    fn bind_shader_program(&mut self, shader: &ShaderProgramRef) -> &mut Self;
    fn bind_shader_uniforms(&mut self, shader: &ShaderProgramRef) -> &mut Self;
}

impl <P: CommandPoolBuilderAlloc>ShaderProgramBinder for AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<P::Alloc>, P>
{
    fn bind_shader_program(&mut self, shader: &ShaderProgramRef) -> &mut Self
    {
        let pipeline = shader.take().pipeline().unwrap().clone();
        self.bind_pipeline_graphics(pipeline)
    }


    fn bind_shader_uniforms(&mut self, shader: &ShaderProgramRef) -> &mut Self
    {
        let mut sh = shader.take_mut();
        
        let mut sets = Vec::new();
        for (set_num, _) in &sh.uniforms_sets {
            sets.push(set_num.clone());
        }
        for set_num in sets {
            let set_builder = sh.uniforms_sets.remove(&set_num).unwrap();
            let set = set_builder.build().unwrap();
            self.bind_descriptor_sets(PipelineBindPoint::Graphics, sh.pipeline.as_ref().unwrap().layout().clone(), set_num as u32, set);
        }
        self
    }
}
