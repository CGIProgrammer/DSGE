use super::Postprocessor;
use super::{StageIndex};
use crate::texture::{TexturePixelFormat, TextureFilter};

impl Postprocessor
{
    pub fn copy_node(&mut self, width: u16, height: u16, sc_format: TexturePixelFormat) -> Result<StageIndex, String>
    {
        let mut stage_builder = Self::stage_builder(self._device.clone());
        stage_builder
            .dimenstions(width, height)
            .input("image")
            .output("swapchain_out", sc_format, TextureFilter::Nearest, false)
            .code("
            void main()
            {
                swapchain_out = texture(image, fragCoordWp);
                swapchain_out.rgb = pow(swapchain_out.rgb, vec3(1.0/2.2));
                swapchain_out.rgb += 1.0 / pow(timer.uptime*10, 10.0);
            }");
        
        let result = stage_builder.build(self);
        result
    }
}