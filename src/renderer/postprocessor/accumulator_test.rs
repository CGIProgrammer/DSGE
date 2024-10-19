use super::PostprocessingPass;
use super::StageIndex;
use crate::texture::{TextureFilter, TexturePixelFormat, TextureView};

#[allow(dead_code)]
impl PostprocessingPass {
    pub fn acc_mblur_new(
        &mut self,
        width: u16,
        height: u16,
        sc_format: TexturePixelFormat,
    ) -> Result<StageIndex, String> {
        let mut stage_builder = Self::stage_builder(self._device.clone());
        stage_builder
            .dimenstions(width, height)
            .input("image", TextureView::Dim2d, false)
            .input("vectors", TextureView::Dim2d, false)
            .input("accumulator", TextureView::Dim2d, false)
            .output("swapchain_out", sc_format, TextureFilter::Nearest, 0)
            .output(
                "accumulator_out",
                TexturePixelFormat::R16G16B16A16_SFLOAT,
                TextureFilter::Linear,
                1,
            )
            .code(
                "
            void main()
            {
                vec2 delta = texture(vectors, fragCoordWp).xy;
                float scale = 1.0;
                vec2 teapot_uv = fragCoordWp / scale;
                vec2 teapot_past_uv = (fragCoordWp - delta*scale);

                vec4 original = texture(image, teapot_uv);
                vec4 past = texture(accumulator, teapot_past_uv);
                accumulator_out = mix(past, original, 0.1);

                if (timer.frame==0 || any(isnan(accumulator_out))) {
                    accumulator_out = vec4(0.0);
                }
                accumulator_out.a = 1.0;
                swapchain_out.rgb = pow(accumulator_out.rgb, vec3(1.0));
                swapchain_out.a = 1.0;
            }",
            );

        let result = stage_builder.build(self);
        match result {
            Ok(stage) => {
                self.link_stages(stage, 1, None, stage, format!("accumulator"))?;
            }
            _ => (),
        }
        result
    }
}
