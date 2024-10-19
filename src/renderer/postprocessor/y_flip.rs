use super::PostprocessingPass;
use super::StageIndex;
use crate::texture::{TextureFilter, TexturePixelFormat, TextureView};

#[allow(dead_code)]
impl PostprocessingPass {
    pub fn new_y_flip(
        &mut self,
        width: u16,
        height: u16,
        sc_pix_fmt: TexturePixelFormat,
    ) -> Result<StageIndex, String> {
        let mut flipper_builder = Self::stage_builder(self.device().clone());
        let mut swizzle = "r".to_owned();
        if sc_pix_fmt.components()[1] != 0 {
            swizzle += "g";
        }
        if sc_pix_fmt.components()[2] != 0 {
            swizzle += "b";
        }
        if sc_pix_fmt.components()[3] != 0 {
            swizzle += "a";
        }
        flipper_builder
            .dimenstions(width, height)
            .input("image_in", TextureView::Dim2d, TextureFilter::Linear, false)
            .output("image_out", sc_pix_fmt, 0)
            .code(&format!(
                "void main() {{
                ivec2 ts = ivec2(textureSize(image_in, 0));
                ivec2 crd = ivec2(pixelCoord).xy;
                crd.y = ts.y - crd.y;
                image_out = texelFetch(image_in, crd, 0).{swizzle};
            }}"
            ));
        let flipper = flipper_builder.build(self)?;
        Ok(flipper)
    }
}
