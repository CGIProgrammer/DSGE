use super::PostprocessingPass;
use super::StageIndex;
use crate::texture::{TextureFilter, TexturePixelFormat, TextureView};

#[allow(dead_code)]
impl PostprocessingPass {
    pub fn new_stack_buffer(
        &mut self,
        width: u16,
        height: u16,
        input_name: &str,
        count: u8,
        pix_fmt: TexturePixelFormat,
    ) -> Result<StageIndex, String> {
        let mut stage_builder = Self::stage_builder(self.device().clone());
        let comps = pix_fmt.components();
        let mut swizzle = String::with_capacity(8);
        swizzle += "x";
        if comps[1] != 0 {
            swizzle += "y";
        }
        if comps[2] != 0 {
            swizzle += "z";
        }
        if comps[3] != 0 {
            swizzle += "w";
        }
        stage_builder
            .dimenstions(width, height)
            .input(input_name, TextureView::Dim2d, TextureFilter::Nearest, false)
            .output("buffer_out", pix_fmt, count as _)
            .code(
                format!(
                    "void main() {{
                buffer_out = texelFetch({input_name}, ivec2(pixelCoord), 0).{swizzle};
            }}"
                )
                .as_str(),
            );

        Ok(stage_builder.build(self)?)
    }
}
