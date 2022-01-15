pub type GLuint = u32;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum TextureType
{
    Texture1D,
    Texture2D,
    Texture3D,
    TextureCube
}

#[allow(dead_code)]
impl TextureType
{
    pub fn stringify(self) -> String
    {
        match self
        {
            TextureType::Texture1D => "Texture1D",
            TextureType::Texture2D => "Texture2D",
            TextureType::Texture3D => "Texture3D",
            TextureType::TextureCube => "TextureCube",
        }.to_string()
    }

    pub fn get_gl_const(self) -> GLuint
    {
        self as GLuint
    }
}