use super::PostprocessingPass;
use super::{StageIndex};
use crate::texture::{TexturePixelFormat, TextureFilter};
use crate::components::ProjectionUniformData;

#[allow(dead_code)]
impl PostprocessingPass
{
    pub fn copy_node(&mut self, width: u16, height: u16, sc_format: TexturePixelFormat) -> Result<StageIndex, String>
    {
        let mut stage_builder = Self::stage_builder(self._device.clone());
        stage_builder
            .dimenstions(width, height)
            .input("albedo")
            .input("shadowmap")
            .input("font")
            .uniform::<ProjectionUniformData>("light")
            .output("swapchain_out", sc_format, TextureFilter::Nearest, false)
            .code("#include \"data/shaders/testing_node.glsl\"");
        
        let result = stage_builder.build(self);
        result
    }
}