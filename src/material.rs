/// `Material` - надстройка над шейдерной программой
/// Текущая реализация перенесена из OpenGL версии без изменений
/// Пока не работает и не используется
/// TODO: заставить работать и адаптировать под vulkano

use vulkano::device::Device;
use vulkano::pipeline::graphics::rasterization::CullMode;
use vulkano::render_pass::Subpass;
use std::sync::Arc;
use std::collections::HashMap;

use crate::types::*;
use crate::shader::*;
use crate::references::*;
use crate::texture::*;
use crate::components::ProjectionUniformData;
use crate::game_object::GOTransformUniform;

#[allow(dead_code)]
type ColorRGB = Vec3;
#[allow(dead_code)]
type ColorRGBA = Vec4;

pub type MaterialRef = RcBox<Material>;

pub static SHADER_CAMERA_SET : usize = 3;
pub static SHADER_TEXTURE_SET : usize = 2;
pub static SHADER_MATERIAL_DATA_SET : usize = 1;
//pub static SHADER_VARIABLES_SET : usize = 0;


/// Слот числовых параметров для материала
#[allow(dead_code)]
#[derive(Clone)]
pub enum MaterialSlot
{
    Scalar(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Mat2(Mat2),
    Mat3(Mat3),
    Mat4(Mat4),
}

impl std::convert::From<f32> for MaterialSlot
{
    fn from(num: f32) -> MaterialSlot
    {
        MaterialSlot::Scalar(num)
    }
}

impl std::convert::From<Vec2> for MaterialSlot
{
    fn from(num: Vec2) -> MaterialSlot
    {
        MaterialSlot::Vec2(num)
    }
}

impl std::convert::From<[f32; 2]> for MaterialSlot
{
    fn from(num: [f32; 2]) -> MaterialSlot
    {
        MaterialSlot::Vec2(Vec2::new(num[0], num[1]))
    }
}

impl std::convert::From<Vec3> for MaterialSlot
{
    fn from(num: Vec3) -> MaterialSlot
    {
        MaterialSlot::Vec3(num)
    }
}

impl std::convert::From<[f32; 3]> for MaterialSlot
{
    fn from(num: [f32; 3]) -> MaterialSlot
    {
        MaterialSlot::Vec3(Vec3::new(num[0], num[1], num[2]))
    }
}

impl std::convert::From<Vec4> for MaterialSlot
{
    fn from(num: Vec4) -> MaterialSlot
    {
        MaterialSlot::Vec4(num)
    }
}

impl std::convert::From<[f32; 4]> for MaterialSlot
{
    fn from(num: [f32; 4]) -> MaterialSlot
    {
        MaterialSlot::Vec4(Vec4::new(num[0], num[1], num[2], num[3]))
    }
}

/// Строитель материала
pub struct MaterialBuilder
{
    name : String,

    /// Присоединённые текстуры
    texture_slots : HashMap<String, Texture>,

    /// Числовые поля, представляющие регулируемые параметры материала.
    numeric_slots : Vec<MaterialSlot>,
    defines : String,

    /// Код структуры, объединяющий числовые поля
    uniform_structure : String,

    /// Базовый вершинный шейдер для рендеринга недеформируемых объектов
    vertex_base : Shader,

    /// Вершинный шейдер для рендеринга деформируемых объектов (скелетная анимация)
    vertex_deformed : Shader,

    /// Базовый фрагментный шейдер для рендеринга деформируемых и недеформируемых объектов
    fragment_base : Shader,

    /// Фрагментный шейдер для рендеринга в карту глубины (для теней)
    fragment_shadowmap : Shader,

    /// Код, описывающий поверхность материала
    pbr_code : String,
}

#[allow(dead_code)]
impl MaterialBuilder
{
    pub fn start(name: &str, device : Arc<Device>) -> MaterialBuilder
    {
        let mut builder = MaterialBuilder {
            name : name.to_string(),
            texture_slots : HashMap::new(),
            numeric_slots : Vec::new(),
            vertex_base : Shader::builder(ShaderType::Vertex, device.clone()),
            vertex_deformed : Shader::builder(ShaderType::Vertex, device.clone()),
            fragment_base : Shader::builder(ShaderType::Fragment, device.clone()),
            fragment_shadowmap : Shader::builder(ShaderType::Fragment, device.clone()),
            uniform_structure : String::new(),
            defines : String::new(),
            pbr_code : DEFAULT_PBR.to_string()
        };

        builder.vertex_base
            .default_vertex_attributes()
            .output("world_position", AttribType::FVec3)
            .output("position_prev", AttribType::FVec4)
            .output("position", AttribType::FVec4)
            .output("texture_uv", AttribType::FVec2)
            .output("view_vector", AttribType::FVec3)
            .output("TBN", AttribType::FMat3)
            //.uniform_autoincrement::<GOTransformUniform>("object", SHADER_TRANSFORM_SET).unwrap()
            .uniform_constant::<GOTransformUniform>("object").unwrap()
            .uniform_autoincrement::<ProjectionUniformData>("camera", SHADER_CAMERA_SET).unwrap();
        
        // Шейдер для скелетной деформации
        // TODO сделать нормальную реализацию. Сейчас это просто копия базового вершинного шейдера материала
        builder.vertex_deformed
            .default_vertex_attributes()
            .output("world_position", AttribType::FVec3)
            .output("position_prev", AttribType::FVec4)
            .output("position", AttribType::FVec4)
            .output("texture_uv", AttribType::FVec2)
            .output("view_vector", AttribType::FVec3)
            .output("TBN", AttribType::FMat3)
            .uniform_constant::<GOTransformUniform>("object").unwrap()
            //.uniform_autoincrement::<GOTransformUniform>("object", SHADER_TRANSFORM_SET).unwrap()
            .uniform_autoincrement::<ProjectionUniformData>("camera", SHADER_CAMERA_SET).unwrap();

        builder.fragment_base
            .input("world_position", AttribType::FVec3)
            .input("position_prev", AttribType::FVec4)
            .input("position", AttribType::FVec4)
            .input("texture_uv", AttribType::FVec2)
            .input("view_vector", AttribType::FVec3)
            .input("TBN", AttribType::FMat3)
            .output("gAlbedo", AttribType::FVec4)
            .output("gNormals", AttribType::FVec3)
            .output("gMasks", AttribType::FVec3)
            .output("gVectors", AttribType::FVec4)
            .code("vec4 mDiffuse;\n")
            .code("float mSpecular, mRoughness, mMetallic;\n")
            .code("vec3 mNormal, mAmbient;\n");

        builder.fragment_shadowmap
            .input("world_position", AttribType::FVec3)
            .input("position_prev", AttribType::FVec4)
            .input("position", AttribType::FVec4)
            .input("texture_uv", AttribType::FVec2)
            .input("view_vector", AttribType::FVec3)
            .input("TBN", AttribType::FMat3)
            .code("vec4 mDiffuse;\n")
            .code("float mSpecular, mRoughness, mMetallic;\n")
            .code("vec3 mNormal, mAmbient;\n")
            .uniform_constant::<GOTransformUniform>("object").unwrap()
            .uniform_autoincrement::<ProjectionUniformData>("camera", SHADER_CAMERA_SET).unwrap();
        builder
    }

    /// Меняет код, описывающий повержность материала.
    /// Если нужен процедурный материал, то это то, что нужно
    pub fn set_pbr_code<T: ToString>(&mut self, code: T) -> &mut Self
    {
        self.pbr_code = code.to_string();
        self
    }

    /// Добавить текстуру
    pub fn add_texture(&mut self, name: &str, texture: &Texture) -> &mut Self
    {
        let ty = texture.ty();
        self.fragment_base.uniform_sampler_autoincrement(name, SHADER_TEXTURE_SET, ty).unwrap();
        self.fragment_shadowmap.uniform_sampler_autoincrement(name, SHADER_TEXTURE_SET, ty).unwrap();
        self.texture_slots.insert(name.to_string(), texture.clone());
        self
    }

    pub fn add_numeric_parameter(&mut self, name: &str, param: MaterialSlot) -> &mut Self
    {
        match param {
            MaterialSlot::Scalar(_) => self.uniform_structure += format!("float {};\n", name).as_str(),
            MaterialSlot::Vec2(_)   => self.uniform_structure += format!("vec2 {};\n", name).as_str(),
            MaterialSlot::Vec3(_)   => self.uniform_structure += format!("vec3 {};\n", name).as_str(),
            MaterialSlot::Vec4(_)   => self.uniform_structure += format!("vec4 {};\n", name).as_str(),
            MaterialSlot::Mat2(_)   => self.uniform_structure += format!("mat2 {};\n", name).as_str(),
            MaterialSlot::Mat3(_)   => self.uniform_structure += format!("mat3 {};\n", name).as_str(),
            MaterialSlot::Mat4(_)   => self.uniform_structure += format!("mat4 {};\n", name).as_str(),
        };
        self.numeric_slots.push(param);
        self
    }

    pub fn define(&mut self, name : &str, expression : &str) -> &mut Self
    {
        self.defines += format!("#define {} {}\n", name, expression).as_str();
        self
    }
    
    pub fn build_mutex(self, device: Arc<Device>) -> MaterialRef
    {
        RcBox::construct(self.build(device))
    }

    pub fn build(mut self, device: Arc<Device>) -> Material
    {
        self.vertex_base
            .code("\n")
            .code(self.defines.as_str())
            .code("\n")
            .uniform_structure("material", "Material", format!("{{\n{}}}", self.uniform_structure).as_str(), SHADER_MATERIAL_DATA_SET, 0)
            .unwrap()
            .code("#include \"data/shaders/geometry_pass.vert.glsl\"")
            .build().unwrap();
        
        self.vertex_deformed
            .code("\n")
            .code(self.defines.as_str())
            .code("\n")
            .uniform_structure("material", "Material", format!("{{\n{}}}", self.uniform_structure).as_str(), SHADER_MATERIAL_DATA_SET, 0)
            .unwrap()
            .code("#include \"data/shaders/geometry_pass.vert.glsl\"")
            .build().unwrap();

        self.fragment_base
            .code("\n")
            .code(self.defines.as_str())
            .code("\n")
            .uniform_structure("material", "Material", format!("{{\n{}}}\n", self.uniform_structure).as_str(), SHADER_MATERIAL_DATA_SET, 0)
            .unwrap()
            .code(self.pbr_code.as_str())
            .code("#include \"data/shaders/geometry_pass.frag.glsl\"")
            .build().unwrap();

        self.fragment_shadowmap
            .code("\n")
            .code(self.defines.as_str())
            .code("\n")
            .uniform_structure("material", "Material", format!("{{\n{}}}", self.uniform_structure).as_str(), SHADER_MATERIAL_DATA_SET, 0)
            .unwrap()
            .code(self.pbr_code.as_str())
            .code("#include \"data/shaders/shadowmap.frag.glsl\"")
            .build().unwrap();
        
        let mut base_builder = ShaderProgram::builder();
        let mut deformed_builder = ShaderProgram::builder();
        let mut base_shadowmap_builder = ShaderProgram::builder();
        let mut deformed_shadowmap_builder = ShaderProgram::builder();

        base_builder
            .fragment(&self.fragment_base).unwrap()
            .vertex(&self.vertex_base).unwrap();

        deformed_builder
            .fragment(&self.fragment_base).unwrap()
            .vertex(&self.vertex_deformed).unwrap();

        base_shadowmap_builder
            .fragment(&self.fragment_shadowmap).unwrap()
            .vertex(&self.vertex_base).unwrap();

        deformed_shadowmap_builder
            .fragment(&self.fragment_shadowmap).unwrap()
            .vertex(&self.vertex_deformed).unwrap();

        let base_shader = base_builder.build(device.clone()).unwrap();
        let deformed_shader = deformed_builder.build(device.clone()).unwrap();
        let shadowmap_base_shader = base_shadowmap_builder.build(device.clone()).unwrap();
        let shadowmap_deformed_shader = deformed_shadowmap_builder.build(device.clone()).unwrap();

        let base_ub = base_shader.new_uniform_buffer();
        let deformed_ub = deformed_shader.new_uniform_buffer();
        let shadowmap_base_ub = shadowmap_base_shader.new_uniform_buffer();
        let shadowmap_deformed_ub = shadowmap_deformed_shader.new_uniform_buffer();

        let result = Material {
            name : self.name,
            texture_slots : self.texture_slots.clone(),
            numeric_slots : self.numeric_slots.clone(),
            shader_set    : MaterialShaderSet {
                base : (base_shader, base_ub),
                deformed : (deformed_shader, deformed_ub),
                shadowmap_base : (shadowmap_base_shader, shadowmap_base_ub),
                shadowmap_deformed : (shadowmap_deformed_shader, shadowmap_deformed_ub),
            }
        };
        
        result
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Material {
    name : String,
    texture_slots : HashMap<String, Texture>,
    numeric_slots : Vec<MaterialSlot>,
    shader_set : MaterialShaderSet
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
impl Material
{
    #[inline]
    pub fn name(&self) -> &String
    {
        &self.name
    }

    pub fn base_shader(&mut self, subpass: Subpass) -> (&mut ShaderProgram, ShaderProgramUniformBuffer)
    {
        self.shader_set.base.0.cull_faces = CullMode::Front;
        let (subpass, new) = self.shader_set.base.0.use_subpass(subpass);
        match subpass {
            PipelineType::Graphics(_) =>
                if new {
                    self.shader_set.base.1 = Material::build_uniform_buffer(self, &self.shader_set.base.0);
                },
            PipelineType::Compute(_) => panic!("Вычислительный конвейер не поддерживается материалами"),
            _ => panic!("Конвейер не инициализирован")
        }
        (&mut self.shader_set.base.0, self.shader_set.base.1.clone())
    }

    pub fn base_shadowmap_shader(&mut self, subpass: Subpass) -> (&mut ShaderProgram, ShaderProgramUniformBuffer)
    {
        self.shader_set.shadowmap_base.0.cull_faces = CullMode::Front;
        let (subpass, new) = self.shader_set.shadowmap_base.0.use_subpass(subpass);
        match subpass {
            PipelineType::Graphics(_) =>
                if new {
                    self.shader_set.shadowmap_base.1 = Material::build_uniform_buffer(self, &self.shader_set.shadowmap_base.0);
                },
            PipelineType::Compute(_) => panic!("Вычислительный конвейер не поддерживается материалами"),
            _ => panic!("Конвейер не инициализирован")
        }
        (&mut self.shader_set.shadowmap_base.0, self.shader_set.shadowmap_base.1.clone())
    }

    fn build_uniform_buffer(material: &Material, shader: &ShaderProgram) -> ShaderProgramUniformBuffer
    {
        println!("Сборка uniform буфера для материала");
        let mut uniform_buffer = shader.new_uniform_buffer();
        let mut numeric_data = Vec::<f32>::with_capacity(material.numeric_slots.len());
        for num_slot in &material.numeric_slots {
            let (val, size) = match num_slot {
                MaterialSlot::Scalar(val) => (val as *const _ as *const f32, std::mem::size_of_val(val) >> 2),
                MaterialSlot::Vec2(val)   => (val as *const _ as *const f32, std::mem::size_of_val(val) >> 2),
                MaterialSlot::Vec3(val)   => (val as *const _ as *const f32, std::mem::size_of_val(val) >> 2),
                MaterialSlot::Vec4(val)   => (val as *const _ as *const f32, std::mem::size_of_val(val) >> 2),
                MaterialSlot::Mat2(val)   => (val as *const _ as *const f32, std::mem::size_of_val(val) >> 2),
                MaterialSlot::Mat3(val)   => (val as *const _ as *const f32, std::mem::size_of_val(val) >> 2),
                MaterialSlot::Mat4(val)   => (val as *const _ as *const f32, std::mem::size_of_val(val) >> 2),
            };
            unsafe { numeric_data.extend(std::slice::from_raw_parts(val, size)) };
        }
        for (name, tex) in &material.texture_slots {
            uniform_buffer.uniform_sampler_by_name(tex, name).unwrap();
        }
        while numeric_data.len()%(16/4) != 0 {
            numeric_data.push(0.0);
        }
        uniform_buffer.uniform_structure(numeric_data, SHADER_MATERIAL_DATA_SET, 0);
        uniform_buffer.build_uniform_sets(&[SHADER_MATERIAL_DATA_SET, SHADER_TEXTURE_SET]);
        uniform_buffer
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct MaterialShaderSet
{
    /// Базовый шейдер статичных моделей
    base : (ShaderProgram, ShaderProgramUniformBuffer),

    /// Базовый шейдер деформируемых моделей
    deformed : (ShaderProgram, ShaderProgramUniformBuffer),

    /// Шейдер теневой карты для статичных моделей
    shadowmap_base : (ShaderProgram, ShaderProgramUniformBuffer),

    /// Шейдер теневой карты для деформируемых моделей
    shadowmap_deformed : (ShaderProgram, ShaderProgramUniformBuffer),
}

static DEFAULT_PBR : &str = "void principled() {
    //float duv = length(abs(dFdx(texture_uv)) + abs(dFdx(texture_uv))) * 10.0;
    
    #ifdef diffuse_map
    mDiffuse = texture(diffuse_map, texture_uv);
    #else
    mDiffuse.rgb = material.diffuse.rgb;
    #endif

    #ifdef normal_map
    vec3 nrm = texture(normal_map, texture_uv).rgb*2.0-1.0;
    nrm.z = sqrt(1.0 - nrm.x*nrm.x + nrm.y*nrm.y);
    nrm = normalize(nrm);
    mNormal = mix(vec3(0.0,0.0,1.0), nrm, clamp(fReliefValue, 0.0, 1.0));
    #endif
    
    #ifdef roughness_map
        mRoughness = texture(roughness_map, texture_uv).r;
    #else
        mRoughness = material.roughness;
    #endif

    #ifdef metallic_map
        mMetallic = texture(metallic_map, texture_uv).r;
    #else
        mMetallic = material.metallic;
    #endif
    
    mAmbient = vec3(material.glow)*0.0; //texture(fLightMap, texture_uv).rgb;

    #ifdef glowing_map
    mAmbient += texture(glowing_map, texture_uv).rgb*1000.0;
    #endif
    mRoughness *= mRoughness;
}";

/*float frag_coord = (2.0 * distance - znear - zfar) / (zfar - znear); */
/*frag_coord * (zfar - znear) + znear + zfar / 2.0 = distance; */