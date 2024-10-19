use super::PostprocessingPass;
use super::StageIndex;
use crate::components::light::{LightsUniformData, SpotlightUniform};
use crate::shader::ShaderUniformArrayLength;
use crate::texture::{TextureFilter, TexturePixelFormat, TextureView};

#[allow(dead_code)]
impl PostprocessingPass {
    pub fn new_debug_overlay(
        &mut self,
        width: u16,
        height: u16,
        sc_pix_fmt: TexturePixelFormat,
    ) -> Result<StageIndex, String> {
        let mut builder = Self::stage_builder(self.device().clone());
        builder
            .dimenstions(width, height)
            .input("image_in", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("blue_noise", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("font", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("point_shadowmaps[4]", TextureView::Cube, TextureFilter::Nearest, false)
            .input("spot_shadowmaps[4]", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("lights_data", TextureView::Dim2d, TextureFilter::Nearest, false)
            .output("image_out", sc_pix_fmt, 0)
            .uniform::<LightsUniformData>("lights_count", ShaderUniformArrayLength::NotArray)
            .uniform_named_type::<SpotlightUniform>(
                "ppSpotlights",
                "SpotlightUniform",
                ShaderUniformArrayLength::Fixed(4),
            )
            .code(&format!(
                "#include \"data/shaders/overlay/debug_overlay.glsl\""
            ));

        let flipper = builder.build(self)?;
        Ok(flipper)
    }
}
