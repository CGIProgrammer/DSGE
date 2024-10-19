use std::sync::Arc;

use vulkano::{image::view::{ImageViewCreateInfo, ImageView}, memory::allocator::StandardMemoryAllocator};

use crate::{framebuffer::Framebuffer, texture::{TextureViewType, Texture, TexturePixelFormat, TextureUseCase}, command_buffer::CommandBufferFather};

#[derive(Clone)]
pub struct ShadowBuffer {
    buffer: Texture,
    frame_buffers: Vec<Framebuffer>,
}

impl ShadowBuffer {
    pub fn new(command_buffer_father: &CommandBufferFather, allocator: Arc<StandardMemoryAllocator>, resolution: u16, layers: u16) -> Self {
        let ty = match layers {
            1 => TextureViewType::Dim2d,
            _ => TextureViewType::Dim2dArray
        };
        let buffer = Texture::new(
            format!("Shadow buffer {}", resolution).as_str(),
            [resolution as u32, resolution as u32, layers as _],
            false,
            ty,
            TexturePixelFormat::D16_UNORM,
            TextureUseCase::Attachment,
            allocator,
        )
        .unwrap();
        drop(buffer.clear_depth_stencil(command_buffer_father, 1.0.into()).unwrap());
        let frame_buffers = (0..layers)
            .map(|layer| {
                let mut fb = Framebuffer::new(resolution, resolution);
                let subbuffer = buffer.array_layer_as_texture(layer as _).unwrap();
                fb.set_depth_attachment(&subbuffer, None);
                fb
            })
            .collect::<Vec<_>>();
        let result = Self {
            buffer: buffer.clone(),
            frame_buffers,
        };
        result
    }

    pub fn from_texture(texture: &Texture) -> Self {
        let width = texture.dims()[0];
        let height = texture.dims()[1];
        let layers = texture
            ._vk_image_view
            .subresource_range()
            .array_layers
            .clone();
        let frame_buffers = layers
            .map(|layer| {
                let mut fb = Framebuffer::new(width as _, height as _);
                let subbuffer = texture.array_layer_as_texture(layer as _).unwrap();
                fb.set_depth_attachment(&subbuffer, None);
                fb
            })
            .collect::<Vec<_>>();
        let result = Self {
            buffer: texture.clone(),
            frame_buffers,
        };
        result
    }

    #[inline(always)]
    pub fn frame_buffers(&self) -> &Vec<Framebuffer> {
        return &self.frame_buffers;
    }

    #[inline(always)]
    pub fn buffer(&self) -> &Texture {
        &self.buffer
    }

    pub fn as_cubemap(&self) -> Option<Texture> {
        let buffer = &self.buffer;
        let iw = buffer._vk_image_view.clone();
        let array_layers = buffer.array_layers();
        if array_layers == 6 {
            let mut ivci = ImageViewCreateInfo::from_image(buffer._vk_image_access.as_ref());
            ivci.view_type = TextureViewType::Cube;
            ivci.subresource_range = iw.subresource_range().clone();
            ivci.sampler_ycbcr_conversion = match iw.sampler_ycbcr_conversion() {
                Some(conv) => Some(conv.clone()),
                None => None,
            };
            let iw = ImageView::new(buffer._vk_image_access.clone(), ivci).unwrap();
            Some(Texture::from_vk_image_view(iw, buffer._vk_device.clone()).unwrap())
        } else {
            None
        }
    }

    pub fn linked_copy(&self) -> Self
    {
        Self {
            buffer: self.buffer.clone(),
            frame_buffers: self.frame_buffers.clone(),
        }
    }
}