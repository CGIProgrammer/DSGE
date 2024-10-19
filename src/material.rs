use std::sync::Arc;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
/// `Material` - надстройка над шейдерной программой
use vulkano::device::Device;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::rasterization::CullMode;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::render_pass::Subpass;

use crate::command_buffer::CommandBufferFather;
use crate::components::ProjectionUniformData;
use crate::game_object::GOTransformUniform;
use crate::mesh::VkVertex;
use crate::references::*;
use crate::renderer::BumpMemoryAllocator;
use crate::renderer::RenderResolution;
use crate::shader::*;
use crate::texture::*;
use crate::time::UniformTime;
use crate::types::*;

#[allow(dead_code)]
type ColorRGB = Vec3;
#[allow(dead_code)]
type ColorRGBA = Vec4;

pub type MaterialRef = RcBox<Material>;

pub static SHADER_CAMERA_SET: u32 = 0;
pub static SHADER_MATERIAL_DATA_SET: u32 = 1;
pub static SHADER_TEXTURE_SET: u32 = 2;
//pub static SHADER_VARIABLES_SET : u32 = 0;

/// Слот числовых параметров для материала
#[allow(dead_code)]
#[derive(Clone)]
pub enum MaterialSlot {
    Scalar(f32),
    IScalar(i32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Mat2(Mat2),
    Mat3(Mat3),
    Mat4(Mat4),
}

impl std::convert::From<i32> for MaterialSlot {
    fn from(num: i32) -> MaterialSlot {
        MaterialSlot::IScalar(num)
    }
}

impl std::convert::From<f32> for MaterialSlot {
    fn from(num: f32) -> MaterialSlot {
        MaterialSlot::Scalar(num)
    }
}

impl std::convert::From<Vec2> for MaterialSlot {
    fn from(num: Vec2) -> MaterialSlot {
        MaterialSlot::Vec2(num)
    }
}

impl std::convert::From<[f32; 2]> for MaterialSlot {
    fn from(num: [f32; 2]) -> MaterialSlot {
        MaterialSlot::Vec2(Vec2::new(num[0], num[1]))
    }
}

impl std::convert::From<Vec3> for MaterialSlot {
    fn from(num: Vec3) -> MaterialSlot {
        MaterialSlot::Vec3(num)
    }
}

impl std::convert::From<[f32; 3]> for MaterialSlot {
    fn from(num: [f32; 3]) -> MaterialSlot {
        MaterialSlot::Vec3(Vec3::new(num[0], num[1], num[2]))
    }
}

impl std::convert::From<Vec4> for MaterialSlot {
    fn from(num: Vec4) -> MaterialSlot {
        MaterialSlot::Vec4(num)
    }
}

impl std::convert::From<[f32; 4]> for MaterialSlot {
    fn from(num: [f32; 4]) -> MaterialSlot {
        MaterialSlot::Vec4(Vec4::new(num[0], num[1], num[2], num[3]))
    }
}

/// Строитель материала
pub struct MaterialBuilder {
    name: String,

    /// Присоединённые текстуры
    texture_slots: Vec<(String, Texture)>,

    /// Числовые поля, представляющие регулируемые параметры материала.
    numeric_slots: Vec<(String, MaterialSlot)>,
    defines: String,

    /// Код структуры, объединяющий числовые поля
    uniform_structure: String,

    /// Код, описывающий поверхность материала
    pbr_code: String,
}

#[allow(dead_code)]
impl MaterialBuilder {
    fn vertex_shader_combination(
        &self,
        device: Arc<Device>,
        shadowmap: bool,
        deformable: bool,
        super_resolution: bool,
    ) -> Result<Shader, String> {
        let mut builder = Shader::builder(ShaderType::Vertex, device.clone());
        builder
            .default_vertex_attributes()
            .instance_attributes()
            .output("triangle_index", AttribType::Int)
            .output("texture_uv", AttribType::FVec2)
            .output("world_position", AttribType::FVec3);
        if super_resolution {
            builder.define("SUPER_RESOLUTION", "1");
        }
        if deformable {
            builder.define("DEFORMED", "");
        }
        if shadowmap {
            builder.define("SHADOWMAP", "");
        } else {
            builder
                .output("position", AttribType::FVec4)
                .output("position_prev", AttribType::FVec4)
                .output("view_vector", AttribType::FVec3)
                .output("TBN", AttribType::FMat3);
        }
        builder
            .uniform_autoincrement::<ProjectionUniformData>(
                "camera",
                ShaderUniformArrayLength::NotArray,
                SHADER_CAMERA_SET,
            )
            .unwrap()
            .uniform_autoincrement::<RenderResolution>(
                "resolution",
                ShaderUniformArrayLength::NotArray,
                SHADER_CAMERA_SET,
            )
            .unwrap()
            .uniform_autoincrement::<UniformTime>(
                "timer",
                ShaderUniformArrayLength::NotArray,
                SHADER_CAMERA_SET,
            )
            .unwrap()
            .code(self.defines.as_str())
            .uniform_structure(
                "material",
                "Material",
                ShaderUniformArrayLength::NotArray,
                format!("{{\n{}}}", self.uniform_structure).as_str(),
                SHADER_MATERIAL_DATA_SET,
                0,
            )
            .unwrap();
        if deformable {
            builder.include("data/shaders/geometry_pass.vert.glsl");
        } else {
            builder.include("data/shaders/geometry_pass.vert.glsl");
        }
        builder.build()?;
        Ok(builder)
    }

    fn fragment_shader_combination(
        &self,
        device: Arc<Device>,
        shadowmap: bool,
        super_resolution: bool,
    ) -> Result<Shader, String> {
        let mut builder = Shader::builder(ShaderType::Fragment, device.clone());
        if shadowmap {
            builder
                .define("SHADOWMAP", "")
                .code("mat3 TBN = mat3(1.0);");
        }
        if super_resolution {
            builder.define("SUPER_RESOLUTION", "1");
        }
        builder
            .input(
                "triangle_index",
                AttribType::Int,
                FragmentInterpolation::Flat,
            )
            .input(
                "texture_uv",
                AttribType::FVec2,
                FragmentInterpolation::default(),
            )
            .input(
                "world_position",
                AttribType::FVec3,
                FragmentInterpolation::default(),
            );
        if !shadowmap {
            builder
                .input(
                    "position",
                    AttribType::FVec4,
                    FragmentInterpolation::default(),
                )
                .input(
                    "position_prev",
                    AttribType::FVec4,
                    FragmentInterpolation::default(),
                )
                .input(
                    "view_vector",
                    AttribType::FVec3,
                    FragmentInterpolation::default(),
                )
                .input("TBN", AttribType::FMat3, FragmentInterpolation::default())
                .output("gAlbedo", AttribType::FVec4)
                .output("gNormals", AttribType::FVec3)
                .output("gMasks", AttribType::FVec3)
                .output("gVectors", AttribType::FVec4);
        }
        builder
            .code("vec4 mDiffuse;")
            .code("float mSpecular, mRoughness, mMetallic;")
            .code("vec3 mNormal, mAmbient;")
            .uniform_autoincrement::<ProjectionUniformData>(
                "camera",
                ShaderUniformArrayLength::NotArray,
                SHADER_CAMERA_SET,
            )?
            .uniform_autoincrement::<RenderResolution>(
                "resolution",
                ShaderUniformArrayLength::NotArray,
                SHADER_CAMERA_SET,
            )?
            .uniform_autoincrement::<UniformTime>(
                "timer",
                ShaderUniformArrayLength::NotArray,
                SHADER_CAMERA_SET,
            )?;
        for (name, tex) in &self.texture_slots {
            builder.uniform_sampler_autoincrement(name, SHADER_TEXTURE_SET, tex.ty(), false)?;
        }
        builder
            .code(self.defines.as_str())
            .uniform_structure(
                "material",
                "Material",
                ShaderUniformArrayLength::NotArray,
                format!("{{\n{}}}\n", self.uniform_structure).as_str(),
                SHADER_MATERIAL_DATA_SET,
                0,
            )?
            .code(self.pbr_code.as_str());
        if shadowmap {
            builder.include("data/shaders/shadowmap.frag.glsl");
        } else {
            builder.include("data/shaders/geometry_pass.frag.glsl");
        }
        builder.build()?;
        Ok(builder)
    }

    pub fn builder(name: &str) -> MaterialBuilder {
        MaterialBuilder {
            name: name.to_owned(),
            texture_slots: Vec::new(),
            numeric_slots: Vec::new(),
            uniform_structure: String::new(),
            defines: String::new(),
            pbr_code: DEFAULT_PBR.to_owned(),
        }
    }

    /// Меняет код, описывающий повержность материала.
    /// Если нужен процедурный материал, то это то, что нужно
    pub fn set_pbr_code(&mut self, code: &str) -> &mut Self {
        self.pbr_code = code.to_owned();
        self
    }

    /// Добавить текстуру
    pub fn add_texture(&mut self, name: &str, texture: &Texture) -> &mut Self {
        self.texture_slots.push((name.to_owned(), texture.clone()));
        self
    }

    pub fn add_numeric_parameter(&mut self, name: &str, param: MaterialSlot) -> &mut Self {
        match param {
            MaterialSlot::Scalar(_) => {
                self.uniform_structure += format!("float {};\n", name).as_str()
            }
            MaterialSlot::IScalar(_) => {
                self.uniform_structure += format!("int   {};\n", name).as_str()
            }
            MaterialSlot::Vec2(_) => {
                self.uniform_structure += format!("vec2  {};\n", name).as_str()
            }
            MaterialSlot::Vec3(_) => {
                self.uniform_structure += format!("vec3  {};\n", name).as_str()
            }
            MaterialSlot::Vec4(_) => {
                self.uniform_structure += format!("vec4  {};\n", name).as_str()
            }
            MaterialSlot::Mat2(_) => {
                self.uniform_structure += format!("mat2  {};\n", name).as_str()
            }
            MaterialSlot::Mat3(_) => {
                self.uniform_structure += format!("mat3  {};\n", name).as_str()
            }
            MaterialSlot::Mat4(_) => {
                self.uniform_structure += format!("mat4  {};\n", name).as_str()
            }
        };
        self.numeric_slots.push((name.to_owned(), param));
        self
    }

    pub fn define(&mut self, name: &str, expression: &str) -> &mut Self {
        self.defines += format!("#define {} {}\n", name, expression).as_str();
        self
    }

    pub fn build_mutex(self, device: Arc<Device>, super_resolution: bool) -> MaterialRef {
        RcBox::construct(self.build(device, super_resolution))
    }

    pub fn build(self, device: Arc<Device>, super_resolution: bool) -> Material {
        let vertex_shaders: [Shader; 4] = MaterialVertexShaderType::combinations()
            .into_iter()
            .map(|mst| {
                self.vertex_shader_combination(
                    device.clone(),
                    mst.shadowmap,
                    mst.deformable,
                    super_resolution,
                )
                .unwrap_or_else(|err| panic!("{err}"))
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let fragment_shaders: [Shader; 2] = MaterialFragmentShaderType::combinations()
            .into_iter()
            .map(|mst| {
                self.fragment_shader_combination(device.clone(), mst.shadowmap, super_resolution)
                    .unwrap_or_else(|err| panic!("{err}"))
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let shader_programs: [(ShaderProgram, ShaderProgramUniformBuffer, bool); 4] =
            MaterialShaderProgramType::combinations()
                .into_iter()
                .map(|MaterialShaderProgramType { vertex, fragment }| {
                    let mut spb = ShaderProgram::builder();
                    spb.vertex(&vertex_shaders[vertex.as_index()])
                        .unwrap()
                        .fragment(&fragment_shaders[fragment.as_index()])
                        .unwrap();
                    let sp = spb.build(device.clone()).unwrap();
                    let ub = sp.new_uniform_buffer();
                    (sp, ub, true)
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();

        let result = Material {
            name: self.name,
            texture_slots: self.texture_slots.clone(),
            numeric_slots: self.numeric_slots.clone(),
            shader_set: shader_programs,
        };

        result
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Material {
    name: String,
    texture_slots: Vec<(String, Texture)>,
    numeric_slots: Vec<(String, MaterialSlot)>,
    shader_set: MaterialShaderSet,
}

/*#[derive(Hash, PartialEq, Eq)]
enum MatShaderType {
    Base = 0,
    BaseShadowmap = 1,
    Deformable = 2,
    DeformableShadowmap = 3
}*/

//static mut DEFAULT_MATERIAL : Option<MaterialRef> = None;

#[allow(dead_code)]
impl Material {
    #[inline]
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn fork(&self, name: &str) -> Self {
        Self {
            name: name.to_owned(),
            ..self.clone()
        }
    }

    pub fn replace_texture(&mut self, name: &str, texture: &Texture) -> Result<(), String> {
        for (slot_name, old_texture) in &mut self.texture_slots {
            if slot_name == name {
                *old_texture = texture.clone();
                for (_, _, need_to_update) in &mut self.shader_set {
                    *need_to_update = true;
                }
                return Ok(());
            }
        }
        Err(format!("Материал {} не имеет текстуры {name}.", self.name))
    }

    pub fn set_parameter(&mut self, name: &str, value: MaterialSlot) -> Result<(), String> {
        for (param_name, param_value) in &mut self.numeric_slots {
            if param_name == name {
                *param_value = value;
                for (_, _, need_to_update) in &mut self.shader_set {
                    *need_to_update = true;
                }
                return Ok(());
            }
        }
        Err(format!("Материал {} не имеет параметра {name}.", self.name))
    }

    fn _shader_mut(
        &mut self,
        ty: &MaterialShaderProgramType,
    ) -> &mut (ShaderProgram, ShaderProgramUniformBuffer, bool) {
        self.shader_set.get_mut(ty.as_index()).unwrap()
    }

    pub fn shader_hash(&self, ty: &MaterialShaderProgramType) -> u64 {
        self.shader_set[ty.as_index()].0.hash()
    }

    fn _shader(
        &self,
        ty: &MaterialShaderProgramType,
    ) -> &(ShaderProgram, ShaderProgramUniformBuffer, bool) {
        self.shader_set.get(ty.as_index()).unwrap()
    }

    pub fn use_in_subpass(
        &mut self,
        command_buffer_father: &CommandBufferFather,
        generic_allocator: Arc<BumpMemoryAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
        ty: &MaterialShaderProgramType,
        subpass: Subpass,
    ) -> (ShaderProgram, ShaderProgramUniformBuffer) {
        let (subpass, new) = {
            let (shader, _, need_to_update) = &mut self._shader_mut(ty);
            shader.cull_faces = CullMode::Front;
            let (s, n) = shader.use_subpass(
                subpass,
                shader.cull_faces,
                Some([VkVertex::per_vertex(), GOTransformUniform::per_instance()]),
            );
            (s, n || *need_to_update)
        };
        match subpass {
            PipelineType::Graphics(_) => {
                if new {
                    #[cfg(debug)]
                    println!("Инициализация шейдера материала");
                    let ub: ShaderProgramUniformBuffer = Material::build_uniform_buffer(
                        command_buffer_father,
                        generic_allocator,
                        descriptor_set_allocator,
                        self,
                        &self._shader(ty).0
                    );
                    let shd = self._shader_mut(ty);
                    shd.1 = ub;
                    shd.2 = false;
                }
            }
            PipelineType::Compute(_) => {
                panic!("Вычислительный конвейер не поддерживается материалами")
            }
            PipelineType::None => panic!("Конвейер не инициализирован"),
        };
        let (shader, uniform_buffer, _) = self._shader(ty);
        (shader.clone(), uniform_buffer.clone())
    }

    fn build_uniform_buffer(
        command_buffer_father: &CommandBufferFather,
        generic_allocator: Arc<BumpMemoryAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
        material: &Material,
        shader: &ShaderProgram,
    ) -> ShaderProgramUniformBuffer {
        #[cfg(debug)]
        println!("Сборка uniform буфера для материала");
        let mut uniform_buffer = shader.new_uniform_buffer();
        let mut numeric_data = Vec::<f32>::with_capacity(material.numeric_slots.len());
        for (_, num_slot) in &material.numeric_slots {
            let (val, size) = match num_slot {
                MaterialSlot::Scalar(val) => (
                    val as *const _ as *const f32,
                    std::mem::size_of_val(val) >> 2,
                ),
                MaterialSlot::IScalar(val) => (
                    val as *const _ as *const f32,
                    std::mem::size_of_val(val) >> 2,
                ),
                MaterialSlot::Vec2(val) => (
                    val as *const _ as *const f32,
                    std::mem::size_of_val(val) >> 2,
                ),
                MaterialSlot::Vec3(val) => (
                    val as *const _ as *const f32,
                    std::mem::size_of_val(val) >> 2,
                ),
                MaterialSlot::Vec4(val) => (
                    val as *const _ as *const f32,
                    std::mem::size_of_val(val) >> 2,
                ),
                MaterialSlot::Mat2(val) => (
                    val as *const _ as *const f32,
                    std::mem::size_of_val(val) >> 2,
                ),
                MaterialSlot::Mat3(val) => (
                    val as *const _ as *const f32,
                    std::mem::size_of_val(val) >> 2,
                ),
                MaterialSlot::Mat4(val) => (
                    val as *const _ as *const f32,
                    std::mem::size_of_val(val) >> 2,
                ),
            };
            unsafe { numeric_data.extend(std::slice::from_raw_parts(val, size)) };
        }
        for (name, tex) in &material.texture_slots {
            drop(uniform_buffer.uniform_sampler_by_name(tex, name));
        }
        // Фикс для помойных видюх от Intel
        /*while numeric_data.len() % (64 / 4) != 0 {
            numeric_data.push(0.0);
        }*/
        uniform_buffer.uniform_structure(generic_allocator.clone(), &[&numeric_data], 0, SHADER_MATERIAL_DATA_SET, 0);
        uniform_buffer
            .build_uniform_sets(descriptor_set_allocator, &[SHADER_MATERIAL_DATA_SET, SHADER_TEXTURE_SET])
            .unwrap();
        uniform_buffer
    }
}

#[allow(dead_code)]
pub type MaterialShaderSet = [(ShaderProgram, ShaderProgramUniformBuffer, bool); 4];

static DEFAULT_PBR: &str = "
void principled() {
#if defined(SUPER_RESOLUTION) && !defined(SHADOWMAP)
    #define BIAS -1.0
#else
    #define BIAS 0.0
#endif

#ifdef SHADOWMAP
    vec4 diffuse = vec4(1.0);
    if (material.shadow_method!=material.shadow_method)
    {
        mDiffuse = vec4(1.0);
        return;
    }
    if (material.use_diffuse_map!=0) {
        mDiffuse = texture(fDiffuseMap, texture_uv);
        mDiffuse.a = min(material.diffuse.a, mDiffuse.a);
    } else {
        mDiffuse = material.diffuse;
    }
#else
    if (material.use_diffuse_map!=0) {
        mDiffuse = texture(fDiffuseMap, texture_uv, BIAS);
        mDiffuse.a = min(material.diffuse.a, mDiffuse.a);
    } else {
        mDiffuse = material.diffuse;
    }
    if (material.use_normal_map!=0) {
        vec3 nrm = pow(texture(fNornalMap, texture_uv, BIAS).rgb, vec3(1.0/2.2))*2.0-1.0;
        nrm.y = -nrm.y;
        nrm.z = sqrt(1.0 - min(nrm.x*nrm.x + nrm.y*nrm.y, 1.0));
        mNormal = TBN * nrm;
    }
    
    if (material.use_roughness_map!=0) {
        mRoughness = pow(texture(fRoughnessMap, texture_uv, BIAS).r, 0.4545454545);
    }
    else {
        mRoughness = pow(material.roughness, 0.4545454545);
    }

    if (material.use_metallic_map!=0) {
        mMetallic = texture(fMetallicMap, texture_uv, BIAS).r;
    }
    else {
        mMetallic = material.metallic;
    }

    if (material.use_specular_map!=0) {
        mSpecular = pow(texture(fSpecularMap, texture_uv, BIAS).r, 0.45454545);
    }
    else {
        mSpecular = material.specular;
    }
    
    mAmbient = vec3(material.glow)*0.0; //texture(fLightMap, texture_uv).rgb;

    if (material.use_emission_map!=0) {
        mAmbient += texture(fEmissionMap, texture_uv, BIAS).rgb*1000.0;
    }
    mRoughness *= mRoughness;
#endif
}";
/*
#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Eq)]
pub enum MaterialShaderType
{
    Vertex {
        deformable: bool,
        shadowmap: bool
    },
    Fragment {
        shadowmap: bool
    }
}*/

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Eq)]
pub struct MaterialVertexShaderType {
    deformable: bool,
    shadowmap: bool,
}

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Eq)]
pub struct MaterialFragmentShaderType {
    shadowmap: bool,
}

impl MaterialVertexShaderType {
    #[inline(always)]
    pub fn combinations() -> [Self; 4] {
        [
            MaterialVertexShaderType {
                deformable: false,
                shadowmap: false,
            },
            MaterialVertexShaderType {
                deformable: false,
                shadowmap: true,
            },
            MaterialVertexShaderType {
                deformable: true,
                shadowmap: false,
            },
            MaterialVertexShaderType {
                deformable: true,
                shadowmap: true,
            },
        ]
    }

    #[inline(always)]
    pub fn as_index(&self) -> usize {
        ((self.deformable as usize) << 1) | (self.shadowmap as usize)
    }
}

impl MaterialFragmentShaderType {
    #[inline(always)]
    pub fn combinations() -> [Self; 2] {
        [
            MaterialFragmentShaderType { shadowmap: false },
            MaterialFragmentShaderType { shadowmap: true },
        ]
    }

    #[inline(always)]
    pub fn as_index(&self) -> usize {
        self.shadowmap as usize
    }
}

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Eq)]
pub struct MaterialShaderProgramType {
    vertex: MaterialVertexShaderType,
    fragment: MaterialFragmentShaderType,
}

impl MaterialShaderProgramType {
    #[inline(always)]
    pub fn combinations() -> [Self; 4] {
        let vcomb = MaterialVertexShaderType::combinations();
        let fcomb = MaterialFragmentShaderType::combinations();
        [
            MaterialShaderProgramType {
                vertex: vcomb[0],
                fragment: fcomb[0],
            },
            MaterialShaderProgramType {
                vertex: vcomb[1],
                fragment: fcomb[1],
            },
            MaterialShaderProgramType {
                vertex: vcomb[2],
                fragment: fcomb[0],
            },
            MaterialShaderProgramType {
                vertex: vcomb[3],
                fragment: fcomb[1],
            },
        ]
    }

    #[inline(always)]
    pub fn as_index(&self) -> usize {
        self.vertex.as_index()
    }

    #[inline(always)]
    pub fn base_gbuffer() -> Self {
        Self {
            vertex: MaterialVertexShaderType {
                deformable: false,
                shadowmap: false,
            },
            fragment: MaterialFragmentShaderType { shadowmap: false },
        }
    }

    #[inline(always)]
    pub fn base_shadowmap() -> Self {
        Self {
            vertex: MaterialVertexShaderType {
                deformable: false,
                shadowmap: true,
            },
            fragment: MaterialFragmentShaderType { shadowmap: true },
        }
    }
}
