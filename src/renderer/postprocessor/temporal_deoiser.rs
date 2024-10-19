use super::{PostprocessingPass, StageInputIndex, StageOutputIndex, StageIndex};
use crate::components::visual::ProjectionUniformData;
use crate::texture::{TexturePixelFormat, TextureFilter, TextureView};

pub struct TemporalDeoiser {
    pub stage_id: StageIndex,
    pub input: StageInputIndex,
    pub output: StageOutputIndex,
}

#[allow(dead_code)]
impl PostprocessingPass
{
    pub fn new_temporal_denoiser(&mut self, width: u16, height: u16, sc_pix_fmt: TexturePixelFormat, filtering: TextureFilter)
        -> Result<TemporalDeoiser, String>
    {
        let mut denoiser_builder = Self::stage_builder(self._device.clone());
        denoiser_builder
            .dimenstions(width, height)
            .uniform_named_type::<ProjectionUniformData>("camera", "MainCamera")
            .input("noised", TextureView::Dim2d, false)
            .input("gDepth", TextureView::Dim2d, false)
            .input("gVectors", TextureView::Dim2d, false)
            .input("denoised_in", TextureView::Dim2d, false)
            .output("denoised_out", TexturePixelFormat::R16G16B16A16_SFLOAT, TextureFilter::Linear, true)
            .output("swapchain_out", sc_pix_fmt, filtering, false)
            .code("#include \"data/shaders/filters/temporal_denoiser.glsl\"");
        let denoiser = denoiser_builder.build(self)?;
        self.link_stages(denoiser,     0, denoiser, "denoised_in".to_owned())?;
        Ok(TemporalDeoiser {
            stage_id: denoiser,
            input: "noised".to_owned(),
            output: 1
        })
    }
}