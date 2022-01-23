use super::RenderPostprocessingGraph;
use super::super::super::shader::*;
use super::{RenderResolution, StageIndex};
use crate::texture::TexturePixelFormat;

impl RenderPostprocessingGraph
{
    /// Размытие в движении, основанное на накоптельном буфере
    pub fn acc_mblur(&mut self, width: u16, height: u16) -> StageIndex
    {
        let acc_mblur_prog = self.make_acc_mblur_shader();
        self.add_stage(
            &acc_mblur_prog,
            vec![
                (true,  TexturePixelFormat::RGBA16f),   // сокет для накопительного буфера
                (false, TexturePixelFormat::SBGRA8)     // сокет для swapchain изображения
            ],
            width, height
        )
    }

    fn make_acc_mblur_shader(&self) -> ShaderProgramRef
    {
        let mut v_shader = self.vertex_plane_shader();
        let mut f_shader = Shader::builder(ShaderType::Fragment, self._device.clone());
        f_shader
            .input("position", AttribType::FVec2)
            .input("fragCoordWp", AttribType::FVec2)
            .input("fragCoord", AttribType::FVec2)
            .output("accumulator_out", AttribType::FVec4)
            .output("swapchain_out", AttribType::FVec4)
            .uniform_sampler2d("image", 1, false)
            .uniform_sampler2d("accumulator", 2, false)
            .uniform::<RenderResolution>("iResolution", 0)
            .code("
            void main()
            {
                accumulator_out = mix(texture(accumulator, fragCoordWp), texture(image, fragCoordWp), 0.1);
                accumulator_out.a = 1.0;
                swapchain_out = accumulator_out;
            }");

        v_shader.build().unwrap();
        f_shader.build().unwrap();
        
        let mut shader_builder = ShaderProgram::builder();
        shader_builder
            .vertex(&v_shader)
            .fragment(&f_shader);
        shader_builder.build(self._device.clone()).unwrap()
    }
}