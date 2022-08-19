use super::PostprocessingPass;
use super::{StageIndex};
use crate::components::light::{PointLight, SunLight, SpotLight, SpotLightUniform};
use crate::texture::{TexturePixelFormat, TextureFilter};
use crate::texture::{TextureView};
use crate::components::ProjectionUniformData;

#[allow(dead_code)]
impl PostprocessingPass
{
    pub fn copy_node(&mut self, width: u16, height: u16, sc_format: TexturePixelFormat) -> Result<StageIndex, String>
    {
        let mut stage_builder = Self::stage_builder(self._device.clone());
        stage_builder
            .dimenstions(width, height)
            .input("accumulator_in", TextureView::Dim2d)
            .input("gAlbedo", TextureView::Dim2d)
            .input("gNormals", TextureView::Dim2d)
            .input("gMasks", TextureView::Dim2d)
            .input("gDepth", TextureView::Dim2d)
            .input("point_shadowmaps[4]", TextureView::Cube)
            .input("spot_shadowmaps[4]", TextureView::Dim2d)
            .input("font", TextureView::Dim2d)
            .input("lights_data", TextureView::Dim2d)
            .uniform_named_type::<SpotLightUniform>("testing_light", "TestingLight")
            .uniform_named_type::<ProjectionUniformData>("camera", "MainCamera")
            //.uniform_named_type::<ProjectionUniformData>("light", "TestingLight")
            .output("accumulator_out", TexturePixelFormat::R16G16B16A16_SFLOAT, TextureFilter::Nearest, true)
            .output("swapchain_out", sc_format, TextureFilter::Nearest, false)
            .code(PointLight::glsl_code())
            .code(SunLight::glsl_code())
            .code(SpotLight::glsl_code())
            .code("#include \"data/shaders/testing_node.glsl\"");
        
        let result = stage_builder.build(self);
        result
    }
}