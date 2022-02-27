use super::Postprocessor;
use super::super::super::shader::*;
use super::{RenderResolution, StageIndex};
use crate::texture::TexturePixelFormat;
use crate::time::UniformTime;

impl Postprocessor
{
    pub fn acc_mblur_new(&mut self, width: u16, height: u16, sc_format: TexturePixelFormat) -> Result<StageIndex, String>
    {
        let mut stage_builder = Self::stage_builder(self._device.clone());
        stage_builder
            .dimenstions(width, height)
            .input("image")
            .input("vectors")
            .input("background")
            .input("accumulator")
            .output("accumulator_out", TexturePixelFormat::RGBA16f, true)
            .output("swapchain_out", sc_format, false)
            .code("
            void main()
            {
                vec2 delta = texture(vectors, fragCoordWp).xy;
                float scale = 0.25;
                vec2 teapot_uv = fragCoordWp / scale;
                vec2 teapot_past_uv = (fragCoordWp - delta*scale);

                vec4 original = texture(image, teapot_uv);
                vec4 past = texture(accumulator, teapot_past_uv);
                accumulator_out = mix(past, original, clamp(0.1 + past.a, 0.0, 1.0));

                if (timer.frame==0 || any(isnan(accumulator_out))) {
                    accumulator_out = vec4(0.0);
                }
                accumulator_out.a = 1.0;
                swapchain_out.rgb = mix(texture(background, fragCoordWp).rgb, accumulator_out.rgb, original.a);
                swapchain_out.a = 1.0;
            }");
        
        let result = stage_builder.build(self);
        match result {
            Ok(stage) => {
                self.link_stages(stage, 0, stage, format!("accumulator"));
            },
            _ => ()
        }
        result
    }
}