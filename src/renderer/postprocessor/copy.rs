use super::PostprocessingPass;
use super::{StageIndex};
use crate::texture::{TexturePixelFormat, TextureFilter};

#[allow(dead_code)]
impl PostprocessingPass
{
    pub fn copy_node(&mut self, width: u16, height: u16, sc_format: TexturePixelFormat) -> Result<StageIndex, String>
    {
        let mut stage_builder = Self::stage_builder(self._device.clone());
        stage_builder
            .dimenstions(width, height)
            .input("albedo")
            .output("swapchain_out", sc_format, TextureFilter::Nearest, false)
            .code("
            void main()
            {
                swapchain_out = texture(albedo, fragCoordWp * vec2(1.0, -1.0) + vec2(0.0, 1.0));
            }");
        
        let result = stage_builder.build(self);
        result
    }
}