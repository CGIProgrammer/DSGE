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

pub type ShaderProgramRef = RcBox<ShaderProgram>;

#[allow(dead_code)]
#[derive(Clone)]
pub struct ShaderUniform
{
    _type: GLSLType,
    _location: i32,
    _set: i32,
    _name: String
}

#[allow(dead_code)]
impl ShaderUniform
{
    pub fn new(_type: GLSLType, name: &str) -> Self
    {
        Self {
            _type: _type,
            _name: name.to_string(),
            _location: 0,
            _set: 0
        }
    }

    pub fn location(&self) -> i32
    {
        self._location
    }

    pub fn name(&self) -> &str
    {
        self._name.as_str()
    }
}

//#[allow(dead_code)]
pub struct Shader
{
    attributes: Vec<(String, AttribType)>,
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
        let mut attrib_list = Vec::new();
        let mut outputs_list = Vec::new();
        let mut inputs_list = Vec::new();
        for i in &self.attributes {
            attrib_list.push(format!("{}: {}", i.0, i.1));
        }
        for i in &self.inputs {
            inputs_list.push(format!("{}: {}", i.0, i.1));
        }
        for i in &self.outputs {
            outputs_list.push(format!("{}: {}", i.0, i.1));
        }
        f.debug_struct("Shader")
         .field("glsl_version", &self.glsl_version)
         .field("attributes", &attrib_list.join(", "))
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

    pub fn builder(shader_type: ShaderType, device: Arc<vulkano::device::Device>) -> Self
    {
        let gl_version = GLSLVersion::V450;
        let mut source = gl_version.stringify() + "\n";
        
        if gl_version.need_precision_qualifier() {
            source += "precision mediump float;\n";
        }
        
        Self {
            attributes : Vec::new(),
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

    pub fn vertex_attribute(&mut self, name: &str, type_: AttribType) -> &mut Self
    {
        match self.sh_type {
            ShaderType::Vertex =>
                self.source += format!("layout(location={}) in {} {};\n", self.attributes.len(), type_.get_glsl_name().as_str(), name).as_str(),
            _ => 
                panic!("Атрибуты вершин поддерживаются только вершинными Vertex шейдерами")
        };
        self.attributes.push((name.to_string(), type_));
        self
    }

    pub fn default_vertex_attributes(&mut self) -> &mut Self
    {
        self
            .vertex_attribute("v_pos", AttribType::FVec3)
            .vertex_attribute("v_nor", AttribType::FVec3)
            .vertex_attribute("v_bin", AttribType::FVec3)
            .vertex_attribute("v_tan", AttribType::FVec3)
            .vertex_attribute("v_tex1", AttribType::FVec2)
            .vertex_attribute("v_tex2", AttribType::FVec2)
            .vertex_attribute("v_grp", AttribType::UVec3)
    }

    // Сделать объявление структур.
    pub fn uniform<T: ShaderStructUniform>(&mut self, name: &str, set: usize) -> &mut Self
    {
        let mut uniforms_in_set = match self.uniforms.get(&set) { Some(uc) => uc.clone(), None => 0 };
        self.source += format!("layout (set = {}, binding = {}) uniform {} {} {};\n", set, uniforms_in_set, T::glsl_type_name(), T::structure(), name).as_str();
        uniforms_in_set += 1;
        self.uniforms.insert(set, uniforms_in_set);
        self
    }

    pub fn uniform_sampler1d(&mut self, name: &str, set: usize, array : bool) -> &mut Self
    {
        let mut uniforms_in_set = match self.uniforms.get(&set) { Some(uc) => uc.clone(), None => 0 };
        self.source += format!("layout (set = {}, binding = {}) uniform sampler1D{} {};\n", set, uniforms_in_set, if array {"Array"} else {""}, name).as_str();
        uniforms_in_set += 1;
        self.uniforms.insert(set, uniforms_in_set);
        self
    }

    pub fn uniform_sampler2d(&mut self, name: &str, set: usize, array : bool) -> &mut Self
    {
        let mut uniforms_in_set = match self.uniforms.get(&set) { Some(uc) => uc.clone(), None => 0 };
        self.source += format!("layout (set = {}, binding = {}) uniform sampler2D{} {};\n", set, uniforms_in_set, if array {"Array"} else {""}, name).as_str();
        uniforms_in_set += 1;
        self.uniforms.insert(set, uniforms_in_set);
        self
    }

    pub fn uniform_sampler3d(&mut self, name: &str, set: usize) -> &mut Self
    {
        let mut uniforms_in_set = match self.uniforms.get(&set) { Some(uc) => uc.clone(), None => 0 };
        self.source += format!("layout (set = {}, binding = {}) uniform sampler3D {};\n", set, uniforms_in_set, name).as_str();
        uniforms_in_set += 1;
        self.uniforms.insert(set, uniforms_in_set);
        self
    }

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

    pub fn code(&mut self, code: &str) -> &mut Self
    {   
        self.source += code;
        self.source += "\n";
        self
    }

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
    
    pub fn is_compatible_by_inout(&self, other: &Self) -> bool
    {
        // По in out переменным
        for (ks, input) in &self.inputs {
            let mut valid_input = false;
            for (ko, output) in &other.outputs {
                if ks==ko {
                    valid_input |= true;
                    if (*output as usize) != (*input as usize) {
                        return false;
                    }
                }
            }
            if !valid_input {
                return false;
            }
        };
        true
    }
    pub fn is_compatible_by_stages(&self, other: &Self) -> bool
    {
        // По типу шейдеров
        match (other.sh_type, self.sh_type)
        {
            (ShaderType::Vertex, ShaderType::Fragment) |
            (ShaderType::Vertex, ShaderType::TesselationControl) |
            (ShaderType::TesselationControl, ShaderType::TesselationEval) |
            (ShaderType::TesselationEval, ShaderType::Geometry) |
            (ShaderType::Vertex, ShaderType::Geometry) |
            (ShaderType::Geometry, ShaderType::Fragment) => 
                true,
            _ => 
                false
        }
    }
}

#[allow(dead_code)]
pub struct PipelineBuilder
{
    vertex_shader: Option<Arc<vulkano::shader::ShaderModule>>,
    fragment_shader: Option<Arc<vulkano::shader::ShaderModule>>,
    tess_controll: Option<Arc<vulkano::shader::ShaderModule>>,
    tess_eval: Option<Arc<vulkano::shader::ShaderModule>>,
    compute: Option<Arc<vulkano::shader::ShaderModule>>,
    uniforms  : HashMap<String, ShaderUniform>,
    depth_test: bool
}

#[allow(dead_code)]
impl PipelineBuilder
{
    pub fn fragment(&mut self, shader: &Shader) -> &mut Self
    {
        self.fragment_shader = shader.module.clone();
        self
    }

    pub fn vertex(&mut self, shader: &Shader) -> &mut Self
    {
        self.vertex_shader = shader.module.clone();
        self
    }

    pub fn tesselation(&'static mut self, eval: &Shader, control: &Shader) -> &mut Self
    {
        self.tess_controll = control.module.clone();
        self.tess_eval = eval.module.clone();
        self
    }

    pub fn depth_test(&mut self) -> &mut Self
    {
        self
    }

    pub fn compute(&mut self) -> &mut Self
    {
        panic!("Не реализовано");
    }

    pub fn enable_depth_test(&mut self) -> &mut Self
    {
        self.depth_test = true;
        self
    }

    pub fn build(&mut self, device: Arc<vulkano::device::Device>) -> Result<ShaderProgramRef, String>
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
                }
            )
        )
    }
}

use vulkano::descriptor_set::persistent::PersistentDescriptorSetBuilder;
pub trait PipelineUniform: ShaderStructUniform + std::marker::Send + std::marker::Sync {}

#[allow(dead_code)]
pub struct ShaderProgram
{
    device : Arc<vulkano::device::Device>,
    pipeline : Option<Arc<vulkano::pipeline::GraphicsPipeline>>,
    render_pass : Option<Arc<RenderPass>>,
    //uniforms_values: HashMap<usize, Vec<Arc<dyn ShaderStructUniform + std::marker::Send + std::marker::Sync + 'static>>>,
    uniforms_sets: HashMap<usize, PersistentDescriptorSetBuilder>,
    vertex_shader : Option<Arc<vulkano::shader::ShaderModule>>,
    fragment_shader : Option<Arc<vulkano::shader::ShaderModule>>,
    tess_controll : Option<Arc<vulkano::shader::ShaderModule>>,
    tess_eval : Option<Arc<vulkano::shader::ShaderModule>>,
    compute : Option<Arc<vulkano::shader::ShaderModule>>,
}

impl ShaderProgram
{
    pub fn builder() -> PipelineBuilder
    {
        PipelineBuilder {
            vertex_shader : None,
            fragment_shader : None,
            tess_controll : None,
            tess_eval : None,
            compute : None,
            uniforms : HashMap::new(),
            depth_test : false
        }
    }

    pub fn pipeline(&self) -> Option<Arc<GraphicsPipeline>>
    {
        self.pipeline.clone()
    }
    
    /*pub fn uniform_sampler(&self, si: &Texture, set_num: usize)
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

        set_builder.add_sampled_image(si.image_view().clone(), si.sampler().clone()).unwrap();
    }*/

    pub fn uniform<T: ShaderStructUniform + std::marker::Send + std::marker::Sync + Clone + 'static>(&mut self, obj: &T, set_num: usize)
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
    
    pub fn make_pipeline(&mut self, subpass : vulkano::render_pass::Subpass)
    {
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
            .render_pass(subpass)
            .depth_stencil_state(DepthStencilState::simple_depth_test());
        //if self.
        self.pipeline = Some(pipeline.build(self.device.clone()).unwrap());
    }
}

use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::buffer::{CpuBufferPool, BufferUsage};
use vulkano::command_buffer::pool::CommandPoolBuilderAlloc;
use vulkano::pipeline::PipelineBindPoint;

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

pub trait ShaderProgramBinder
{
    fn bind_shader_program(&mut self, shader: &ShaderProgramRef) -> &mut Self;
    fn bind_shader_uniforms(&mut self, shader: &ShaderProgramRef) -> &mut Self;
}

pub trait ShaderStructUniform
{
    fn structure() -> String;
    fn glsl_type_name() -> String;
    fn texture(&self) -> Option<&crate::texture::TextureRef>;
}
