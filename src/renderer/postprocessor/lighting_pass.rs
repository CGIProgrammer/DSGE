use super::PostprocessingPass;
use super::{StageIndex};
use crate::components::light::{PointLight, SunLight, SpotLight};
use crate::texture::{TexturePixelFormat, TextureFilter, TextureView};
use crate::components::ProjectionUniformData;

#[allow(dead_code)]
impl PostprocessingPass
{
    pub fn spotlight_pass(&mut self, width: u16, height: u16, sc_format: TexturePixelFormat) -> Result<StageIndex, String>
    {
        let mut stage_builder = Self::stage_builder(self._device.clone());
        stage_builder
            .dimenstions(width, height)
            .input("gAlbedo", TextureView::Dim2d, false)
            .input("gNormals", TextureView::Dim2d, false)
            .input("gMasks", TextureView::Dim2d, false)
            .input("gDepth", TextureView::Dim2d, false)
            .input("shadowmaps[16]", TextureView::Dim2d, false)
            .input("lights_data", TextureView::Dim2d, false)
            .uniform_named_type::<ProjectionUniformData>("camera", "MainCamera")
            .output("swapchain_out", sc_format, TextureFilter::Nearest, false)
            .code(SpotLight::glsl_code())
            .code("#include \"data/shaders/testing_node.glsl\"");
        
        let result = stage_builder.build(self);
        result
    }
}