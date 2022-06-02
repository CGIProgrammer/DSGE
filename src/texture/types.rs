pub use vulkano::image::view::ImageViewType as TextureView;

pub trait TextureViewGlsl
{
    fn glsl_sampler_name(&self) -> &'static str;
}

impl TextureViewGlsl for TextureView
{
    fn glsl_sampler_name(&self) -> &'static str
    {
        match self
        {
            TextureView::Dim1d => "sampler1D",
            TextureView::Dim1dArray => "sampler1DArray",
            TextureView::Dim2d => "sampler2D",
            TextureView::Dim2dArray => "sampler2DArray",
            TextureView::Cube => "samplerCube",
            TextureView::Dim3d => "sampler3D",
            TextureView::CubeArray => "samplerCubeArray",
        }
    }
}