use super::PostprocessingPass;
use super::StageIndex;
use super::StageInputIndex;
use super::StageOutputIndex;
use crate::components::light::GenericLightUniform;
use crate::components::light::PointLightUniform;
use crate::components::light::SunLightUniform;
use crate::components::light::{
    LightsUniformData, Spotlight, SpotlightUniform, SunLight,
};
use crate::components::ProjectionUniformData;
use crate::shader::ShaderUniformArrayLength;
use crate::shader::shader_struct_uniform::ShaderStructUniform;
use crate::texture::TextureView;
use crate::texture::{TextureFilter, TexturePixelFormat};

pub struct Composer
{
    pub stage_id: StageIndex,
    pub diffuse_input: StageInputIndex,
    pub specular_input: StageInputIndex,
    pub output: StageOutputIndex
}

pub struct DeferredLighting
{
    pub stage_id: StageIndex,
    pub diffuse_out: StageOutputIndex,
    pub specular_out: StageOutputIndex,
}

pub struct ScreenSpaceReflections
{
    pub stage_id: StageIndex,
    pub diffuse_in: StageInputIndex,
    pub specular_in: StageInputIndex,
    pub specular_out: StageOutputIndex,
    pub diffuse_out: StageOutputIndex,
}

impl PostprocessingPass {
    fn lighting_filter_pass(&mut self, width: u16, height: u16) -> Result<StageIndex, String> {
        let mut stage_builder = Self::stage_builder(self.device().clone());
        stage_builder
            .dimenstions(width, height)
            .input("gAlbedo", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("gDepth", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("lighting", TextureView::Dim2d, TextureFilter::Nearest, false)
            .output(
                "result",
                TexturePixelFormat::R8G8B8A8_SRGB,
                0,
            )
            .code("#include \"data/shaders/lighting/lighting_filter_pass.glsl\"");
        stage_builder.build(self)
    }

    pub(crate) fn new_lighting(
        &mut self,
        width: u16,
        height: u16,
        max_spotlights: u32,
        max_sun_lights: u32,
        max_pointlights: u32,
        sc_format: TexturePixelFormat,
    ) -> Result<DeferredLighting, String> {
        let mut stage_builder = Self::stage_builder(self.device().clone());
        stage_builder
            .dimenstions(width, height)
            .input("gAlbedo", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("gNormals", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("gMasks", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("gDepth", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("point_shadowmaps", TextureView::Dim2dArray, TextureFilter::Nearest, false)
            .input("spot_shadowmaps", TextureView::Dim2dArray, TextureFilter::Nearest, false)
            .input("sun_shadowmaps", TextureView::Dim2dArray, TextureFilter::Nearest, false)
            .input("blue_noise", TextureView::Dim2d, TextureFilter::Nearest, false)
            .code(format!("layout(std140) struct {} {};", GenericLightUniform::glsl_type_name(), GenericLightUniform::structure()).as_str())
            .uniform_named_type::<SpotlightUniform>(
                "ppSpotlights",
                "Spotlight",
                ShaderUniformArrayLength::Fixed(max_spotlights),
            )
            .uniform_named_type::<SunLightUniform>(
                "ppSunlights",
                "SunLight",
                ShaderUniformArrayLength::Fixed(max_sun_lights),
            )
            .uniform_named_type::<PointLightUniform>(
                "ppPointlights",
                "PointLight",
                ShaderUniformArrayLength::Fixed(max_pointlights),
            )
            .uniform::<LightsUniformData>("lights_count", ShaderUniformArrayLength::NotArray)
            .uniform_named_type::<ProjectionUniformData>(
                "camera",
                "MainCamera",
                ShaderUniformArrayLength::NotArray,
            )
            .output("diffuse_out", sc_format, 0)
            .output("specular_out", sc_format, 0)
            .code("#include \"data/shaders/lighting/deferred_lighting_pass.glsl\"");

        let stage_id = stage_builder.build(self)?;
        Ok(DeferredLighting {
            stage_id: stage_id,
            diffuse_out: 0,
            specular_out: 1,
        })
    }

    pub fn new_ssr(
        &mut self,
        width: u16,
        height: u16,
    ) -> Result<ScreenSpaceReflections, String> {
        let mut stage_builder = Self::stage_builder(self.device().clone());
        stage_builder
            .dimenstions(width, height)
            .uniform_named_type::<ProjectionUniformData>(
                "camera",
                "MainCamera",
                ShaderUniformArrayLength::NotArray,
            )
            .input("gAlbedo", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("gNormals", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("gMasks", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("gDepth", TextureView::Dim2d, TextureFilter::Linear, false)
            .input("diffuse_in", TextureView::Dim2d, TextureFilter::Linear, false)
            .input("specular_in", TextureView::Dim2d, TextureFilter::Linear, false)
            .input("blue_noise", TextureView::Dim2d, TextureFilter::Nearest, false)
            .output("diffuse_out", TexturePixelFormat::R16G16B16A16_SFLOAT, 0)
            .output("specular_out", TexturePixelFormat::R16G16B16A16_SFLOAT, 0)
            .code("#include \"data/shaders/lighting/reflection.glsl\"");
        let stage_id = stage_builder.build(self)?;
        Ok(ScreenSpaceReflections {
            stage_id: stage_id,
            diffuse_in: "diffuse_in".to_owned(),
            specular_in: "specular_in".to_owned(),
            diffuse_out: 0,
            specular_out: 1
        })
    }

    pub fn new_composing(
        &mut self,
        width: u16,
        height: u16,
    ) -> Result<Composer, String>
    {
        let mut stage_builder = Self::stage_builder(self.device().clone());
        stage_builder
            .dimenstions(width, height)
            .input("gAlbedo", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("gDepth", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("gNormals", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("gMasks", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("diffuse_input", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("specular_input", TextureView::Dim2d, TextureFilter::Nearest, false)
            .uniform_named_type::<ProjectionUniformData>(
                "camera",
                "MainCamera",
                ShaderUniformArrayLength::NotArray,
            )
            .output("composition_out", TexturePixelFormat::R8G8B8A8_SRGB, 0)
            .code("#include \"data/shaders/lighting/composing.glsl\"");
        let stage_id = stage_builder.build(self)?;
        Ok(Composer {
            stage_id: stage_id,
            diffuse_input: "diffuse_input".to_owned(),
            specular_input: "specular_input".to_owned(),
            output: 0
        })
    }
}
