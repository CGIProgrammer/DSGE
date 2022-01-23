/// `Material` - надстройка над шейдерной программой
/// Текущая реализация перенесена из OpenGL версии без изменений
/// Пока не работает и не используется
/// TODO: заставить работать и адаптировать под vulkano

use vulkano::device::Device;
use std::sync::Arc;

use crate::types::*;
use crate::shader::*;
use crate::references::*;
use crate::texture::*;
use crate::components::camera::*;
use crate::game_object::GOTransfotmUniform;
use std::collections::HashMap;

#[allow(dead_code)]
type ColorRGB = Vec3;
#[allow(dead_code)]
type ColorRGBA = Vec4;

pub type MaterialRef = RcBox<Material>;

#[allow(dead_code)]
struct UniformMaterial
{
    diffuse   : ColorRGBA,
    glowing   : ColorRGBA,
    metallic  : f32,
    roughness : f32,
    relief    : f32,
}

impl ShaderStructUniform for UniformMaterial
{
    fn structure() -> String
    {
"{
    vec4 diffuse,
    vec4 glowing,
    float metallic,
    float roughness,
    float relief
}".to_string()
    }

    fn glsl_type_name() -> String
    {
        "MaterialValues".to_string()
    }

    fn texture(&self) -> Option<&TextureRef>
    {
        None
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Material {
    pub diffuse_value: ColorRGBA,
    pub glowing_value: ColorRGBA,
    pub metallic_value: f32,
    pub roughness_value: f32,
    _texture_slots: HashMap<String, TextureRef>,
    _shader_set: RcBox<MaterialShaderSet>,
}

static mut DEFAULT_MATERIAL : Option<RcBox<Material>> = None;

#[allow(dead_code)]
impl Material
{
    pub fn default(device : Arc<Device>) -> MaterialRef
    {
        unsafe {
            if DEFAULT_MATERIAL.is_none() {
                let res = Self {
                    diffuse_value : ColorRGBA::new(1.0, 1.0, 1.0, 0.0),
                    glowing_value : ColorRGBA::new(0.0, 0.0, 0.0, 0.0),
                    metallic_value: 0.0,
                    roughness_value: 1.0,
                    _texture_slots: HashMap::new(),
                    _shader_set: RcBox::construct(MaterialShaderSet::default(device).unwrap())
                };
                DEFAULT_MATERIAL = Some(RcBox::construct(res));
            }
            DEFAULT_MATERIAL.clone().unwrap()
        }
    }

    pub fn get_textrues(&self) -> HashMap<String, TextureRef>
    {
        self._texture_slots.clone()
    }

    pub fn diffuse_texture(&self) -> Option<TextureRef> {
        if self._texture_slots.contains_key("fDiffuseMap") {
            Some(self._texture_slots["fDiffuseMap"].clone())
        } else {
            None
        }
    }

    pub fn metallic_texture(&self) -> Option<TextureRef> {
        if self._texture_slots.contains_key("fMetallicMap") {
            Some(self._texture_slots["fMetallicMap"].clone())
        } else {
            None
        }
    }

    pub fn roughness_texture(&self) -> Option<TextureRef> {
        if self._texture_slots.contains_key("fRoughnessMap") {
            Some(self._texture_slots["fRoughnessMap"].clone())
        } else {
            None
        }
    }

    pub fn glowing_texture(&self) -> Option<TextureRef> {
        if self._texture_slots.contains_key("fGlowingMap") {
            Some(self._texture_slots["fGlowingMap"].clone())
        } else {
            None
        }
    }

    pub fn bump_texture(&self) -> Option<TextureRef> {
        if self._texture_slots.contains_key("fReliefMap") {
            Some(self._texture_slots["fReliefMap"].clone())
        } else {
            None
        }
    }

    pub fn set_diffuse_texture(&mut self, texture: &TextureRef)
    {
        self._texture_slots.insert("fDiffuseMap".to_string(), texture.clone());
    }

    pub fn set_metallic_texture(&mut self, texture: &TextureRef)
    {
        self._texture_slots.insert("fMetallicMap".to_string(), texture.clone());
    }

    pub fn set_roughness_texture(&mut self, texture: &TextureRef)
    {
        self._texture_slots.insert("fRoughnessMap".to_string(), texture.clone());
    }

    pub fn set_glowing_texture(&mut self, texture: &TextureRef)
    {
        self._texture_slots.insert("fGlowingMap".to_string(), texture.clone());
    }

    pub fn set_bump_texture(&mut self, texture: &TextureRef)
    {
        self._texture_slots.insert("fReliefMap".to_string(), texture.clone());
    }

    pub fn base_shader(&self) -> ShaderProgramRef
    {
        self._shader_set.take().base.clone()
    }

    pub fn base_deformed_shader(&self) -> ShaderProgramRef
    {
        self._shader_set.take().deformed.clone()
    }

    pub fn shadowmap_shader(&self) -> ShaderProgramRef
    {
        self._shader_set.take().shadowmap_base.clone()
    }

    pub fn shadowmap_deformed_shader(&self) -> ShaderProgramRef
    {
        self._shader_set.take().shadowmap_deformed.clone()
    }
}


#[allow(dead_code)]
#[derive(Clone)]
struct MaterialShaderSet
{
    base : RcBox<ShaderProgram>,
    deformed : RcBox<ShaderProgram>,
    shadowmap_base : RcBox<ShaderProgram>,
    shadowmap_deformed : RcBox<ShaderProgram>,
}

impl MaterialShaderSet
{
    /*pub fn new(vb : &Shader, vd : &Shader, fb : &Shader, fs : &Shader) -> Result<Self, String>
    {
        let base = ShaderProgram::create(&[vb, fb]);
        let shadowmap_base = ShaderProgram::create(&[vb, fs]);
        let deformed = ShaderProgram::create(&[vd, fb]);
        let shadowmap_deformed = ShaderProgram::create(&[vd, fs]);
        if base.is_err() {
            return Err(base.unwrap_err());
        }
        if shadowmap_base.is_err() {
            return Err(shadowmap_base.unwrap_err());
        }
        if deformed.is_err() {
            return Err(deformed.unwrap_err());
        }
        if shadowmap_deformed.is_err() {
            return Err(shadowmap_deformed.unwrap_err());
        }
        Ok(Self {
            base : RcBox::construct(base.unwrap()),
            shadowmap_base : RcBox::construct(shadowmap_base.unwrap()),
            deformed : RcBox::construct(deformed.unwrap()),
            shadowmap_deformed : RcBox::construct(shadowmap_deformed.unwrap()),
        })
    }*/
    
    pub fn principled(sas : &str, device : Arc<Device>) -> Result<Self, String>
    {
        let base_vert = Self::vertex_mesh_shader_set(false, device.clone());
        let deformed_vert = Self::vertex_mesh_shader_set(false, device.clone());
        let base_frag = Self::fragment_mesh_shader_set(sas, false, device.clone());
        let shadowmap_frag = Self::fragment_mesh_shader_set(sas, true, device.clone());
        if base_vert.is_err() {
            return Err(base_vert.unwrap_err());
        }
        if deformed_vert.is_err() {
            return Err(deformed_vert.unwrap_err());
        }
        if base_frag.is_err() {
            return Err(base_frag.unwrap_err());
        }
        if shadowmap_frag.is_err() {
            return Err(shadowmap_frag.unwrap_err());
        }
        let base_vert = base_vert.unwrap();
        let deformed_vert = deformed_vert.unwrap();
        let base_frag = base_frag.unwrap();
        let shadowmap_frag = shadowmap_frag.unwrap();
        
        Ok(Self {
            base               : ShaderProgram::builder().vertex(&base_vert).fragment(&base_frag).build(device.clone()).unwrap(),
            shadowmap_base     : ShaderProgram::builder().vertex(&base_vert).fragment(&shadowmap_frag).build(device.clone()).unwrap(),
            deformed           : ShaderProgram::builder().vertex(&deformed_vert).fragment(&base_frag).build(device.clone()).unwrap(),
            shadowmap_deformed : ShaderProgram::builder().vertex(&deformed_vert).fragment(&shadowmap_frag).build(device.clone()).unwrap(),
        })
    }

    pub fn default(device : Arc<Device>) -> Result<Self, String>
    {
        let principled =
        "void principled()
        {
            float duv = length(abs(dFdx(texture_uv)) + abs(dFdx(texture_uv))) * 10.0;
            mDiffuse = mix(vec4(material.diffuse.rgb, 1.0), texture(fDiffuseMap, texture_uv), material.diffuse.a);
            //mDiffuse.xyz = vec3(duv);
            //mDiffuse.rgb *= vec3(texture(fDiffuseMap, texture_uv).a);
            vec3 nrm = texture(fReliefMap, texture_uv).rgb*2.0-1.0;
            nrm.z = sqrt(1.0 - nrm.x*nrm.x + nrm.y*nrm.y);
            nrm = normalize(nrm);
            //mNormal = mix(vec3(0.0,0.0,1.0), nrm, clamp(fReliefValue, 0.0, 1.0));
            //mNormal = normalize(tbn * mNormal);
            if (material.roughness<0.0) {
                mRoughness = texture(fRoughnessMap, texture_uv).r;
            } else {
                mRoughness = material.roughness;
            }
            if (material.metallic<0.0) {
                mMetallic = texture(fMetallicMap, texture_uv).r;
            } else {
                mMetallic = material.metallic;
            }
            //mMetallic = max(texture(fMetallicMap, texture_uv).r, material.metallic);
            mAmbient = vec3(material.glow)*0.0; //texture(fLightMap, texture_uv).rgb;
            mAmbient += texture(fGlowingMap, texture_uv).rgb*1000.0;
            mRoughness *= mRoughness;
        }";
        Self::principled(principled, device)
    }

    fn vertex_mesh_shader_set(_skeleton: bool, device : Arc<Device>) -> Result<Shader, String>
    {
        let mut shader = Shader::builder(ShaderType::Vertex, device.clone());
        shader
            .default_vertex_attributes()
            .output("position", AttribType::FVec4)
            .output("position_prev", AttribType::FVec4)
            .output("view_vector", AttribType::FVec3)
            .output("texture_uv", AttribType::FVec2)
            .output("TBN", AttribType::FMat3)
            .uniform::<GOTransfotmUniform>("object", 0)
            .uniform::<CameraUniform>("camera", 0)
            .code("
            void main()
            {
                float nLength = length(nor);
                vec3 nnor = normalize(nor);
                mat4 model = camera.transform * object.transform;
                if (nLength>1.1) {
                    TBN = mat3(tang, normalize(cross(nor, tang)), nor);
                } else {
                    TBN = mat3(tang, normalize(cross(tang, nor)), nor);
                }
                TBN = mat3(model[0].xyz, model[1].xyz, model[2].xyz) * TBN;
                //nnor = camera.transform * object.transform * vec4(nor, 0.0);
                position = camera.projection * camera.transform * object.transform * vec4(pos, 1.0);
                position_prev = camera.projection * camera.transform_prev * object.transform_prev * vec4(pos, 1.0);
                texture_uv = vec2(uv1.x, 1.0-uv1.y);
                view_vector = (camera.transform * position).xyz;
                gl_Position = position;
            }");
        let builded = shader.build();
        if builded.is_err() {
            Err(builded.unwrap_err())
        } else {
            Ok(shader)
        }
    }
    fn fragment_mesh_shader_set(principled_procedure : &str, shadowmap : bool, device : Arc<Device>) -> Result<Shader, String>
    {
        let mut shader = Shader::builder(ShaderType::Fragment, device.clone());
        shader.input("position", AttribType::FVec4)
        .input("position_prev", AttribType::FVec4)
        .input("view_vector", AttribType::FVec3)
        .input("texture_uv", AttribType::FVec2)
        .input("TBN", AttribType::FMat3)
        .uniform_sampler2d("fDiffuseMap", 1, false)
        .uniform_sampler2d("fReliefMap", 1, false)
        .uniform_sampler2d("fGlowingMap", 1, false)
        .uniform_sampler2d("fMetallicMap", 1, false)
        .uniform_sampler2d("fRoughnessMap", 1, false)
        .uniform::<UniformMaterial>("material", 1)
        .code("mat3 tbn;")
        .code("vec4 mDiffuse;")
        .code("vec3 mNormal;")
        .code("vec3 mAmbient;")
        .code("float mSpecular;")
        .code("float mRoughness;")
        .code("float mMetallic;")
        .code("#define worldPosition position")
        .code(principled_procedure)
        .code("
        void main() {
            tbn = TBN;
            tbn[0] = normalize(tbn[0]);
            tbn[1] = normalize(tbn[1]);
            tbn[2] = normalize(tbn[2]);
            mNormal = tbn[2];
            //tbn = transpose(tbn);
            principled();
            mDiffuse.rgb = pow(mDiffuse.rgb, vec3(2.2));
            if (mDiffuse.a < 0.5)
            {
                mDiffuse.a = 1.0;
                discard;
            }
            vec2 velocity_vector = (position.xy/position.w - position_prev.xy/position_prev.w)*0.5;
            FragData[0] = vec4(mDiffuse.rgb, 1.0);");
        if !shadowmap
        {
            shader.code("
            FragData[1] = vec4(mNormal*0.5+0.5, 1.0);
            FragData[2] = vec4(mSpecular, mRoughness, mMetallic, 1.0);
            FragData[3] = vec4(velocity_vector, 0.0, 1.0);
            mDiffuse.a = 1.0;");
        }
        shader.code("}");
        let builded = shader.build();
        if builded.is_err() {
            Err(builded.unwrap_err())
        } else {
            Ok(shader)
        }
    }
}