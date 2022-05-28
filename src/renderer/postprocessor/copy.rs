use super::Postprocessor;
use super::{StageIndex};
use crate::texture::{TexturePixelFormat, TextureFilter};

#[allow(dead_code)]
impl Postprocessor
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
                vec2 size = vec2(textureSize(albedo, 0));
                vec2 crd = fragCoordWp;
                crd.y = (crd.y + 150.0 / size.y) / 1536.0 * 1080.0;
                swapchain_out = texture(albedo, fragCoordWp);
            }");
        
        let result = stage_builder.build(self);
        result
    }
}