use super::{PostprocessingPass, StageIndex, StageInputIndex, StageOutputIndex};
use crate::components::visual::ProjectionUniformData;
use crate::shader::ShaderUniformArrayLength;
use crate::texture::{TextureFilter, TexturePixelFormat, TextureView};

pub struct TemporalDeoiser {
    pub stage_id: StageIndex,
    pub input: StageInputIndex,
    pub output: StageOutputIndex,
}

pub struct FxaaFilter {
    pub stage_id: StageIndex,
    pub input: StageInputIndex,
    pub output: StageOutputIndex,
}

#[allow(dead_code)]
impl PostprocessingPass {
    pub fn new_fxaa(&mut self, width: u16, height: u16) -> Result<FxaaFilter, String> {
        let mut node = Self::stage_builder(self.device().clone());
        node.dimenstions(width, height)
            .input("orig", TextureView::Dim2d, TextureFilter::Nearest, false)
            .input("gVectors", TextureView::Dim2d, TextureFilter::Nearest, false)
            .output(
                "antialiased",
                TexturePixelFormat::R8G8B8A8_SRGB,
                0,
            )
            .code(
                "#include \"data/shaders/filters/fxaa.h\"
            void main() {
                float c = length(texture(gVectors, fragCoord).xy) * 10.0;
                c = 1.0 - c / (1.0 + c);
                antialiased = fxaa(orig, ivec2(pixelCoord), fragCoord, c);
            }",
            );
        let stage_id = node.build(self)?;

        Ok(FxaaFilter {
            stage_id: stage_id,
            input: "orig".to_owned(),
            output: 0,
        })
    }

    pub fn new_temporal_filter(
        &mut self,
        width: u16,
        height: u16,
        super_resolution: bool,
    ) -> Result<TemporalDeoiser, String> {
        let mut denoiser_builder = Self::stage_builder(self.device().clone());
        denoiser_builder
            .dimenstions(width, height)
            .uniform_named_type::<ProjectionUniformData>(
                "camera",
                "MainCamera",
                ShaderUniformArrayLength::NotArray,
            )
            .input("gVectors", TextureView::Dim2d, TextureFilter::Linear, false)
            //.input("gVectors1",  TextureView::Dim2d, false)
            .input("gDepth", TextureView::Dim2d, TextureFilter::Linear, false)
            //.input("gDepth1", TextureView::Dim2d, false)
            .input("lowres_in", TextureView::Dim2d, TextureFilter::Linear, false)
            .input("hires_in", TextureView::Dim2d, TextureFilter::Linear, false)
            .output(
                "hires_out",
                TexturePixelFormat::R16G16B16A16_SFLOAT,
                1,
            );
        //.output("vectors_out", TexturePixelFormat::R16G16B16A16_SFLOAT, TextureFilter::Nearest, 1)
        //.output("depth_out", TexturePixelFormat::R16_UNORM, TextureFilter::Nearest, 1);
        if super_resolution {
            denoiser_builder.code("#define SUPER_RESOLUTION 1");
        }
        denoiser_builder.code("#include \"data/shaders/filters/stsr.glsl\"");
        let denoiser = denoiser_builder.build(self)?;
        /*for i in 1..4 {
            self.link_stages(denoiser, 1, Some(i-1), denoiser, format!("noised{i}"))?;
            self.link_stages(denoiser, 2, Some(i-1), denoiser, format!("gVectors{i}"))?;
        }*/
        self.link_stages(denoiser, 0, None, denoiser, "hires_in".to_owned())?;
        //self.link_stages(denoiser, 1, None, denoiser, "gVectors1".to_owned())?;
        //self.link_stages(denoiser, 1, None, denoiser, "gDepth1".to_owned())?;
        Ok(TemporalDeoiser {
            stage_id: denoiser,
            input: "lowres_in".to_owned(),
            output: 0,
        })
    }
}
