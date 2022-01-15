use std::sync::Arc;
use vulkano::render_pass::{
    Framebuffer as VkFramebuffer,
    RenderPass as VkRenderPass,
};

use vulkano::pipeline::{graphics::viewport::{Viewport}};
use crate::texture::*;

pub type FramebufferRef = RcBox<Framebuffer>;
pub type FramebufferAttachmentDefaultValue = vulkano::format::ClearValue;

struct Attachment
{
    storage : TextureRef,
    default_color : FramebufferAttachmentDefaultValue
}

pub struct Framebuffer
{
    _viewport: Viewport,
    _dimensions: [u16; 2],
    _color_attachments: Vec<Attachment>,
    _depth_attachment: Option<Attachment>,
    _vk_fb: Option<Arc<VkFramebuffer>>,
}

#[allow(dead_code)]
impl Framebuffer
{
    pub fn new(width: u16, height: u16) -> FramebufferRef
    {
        RcBox::construct(Self {
            _viewport: Viewport {
                origin: [0.0, 0.0],
                dimensions: [0.0, 0.0],
                depth_range: 0.0..1.0,
            },
            _dimensions: [width, height],
            _color_attachments: Vec::new(),
            _depth_attachment: None,
            _vk_fb: None
        })
    }

    // Присоединение изображения к выходу фреймбуфера
    pub fn add_color_attachment(&mut self, att: TextureRef, default_val : FramebufferAttachmentDefaultValue) -> Result<(), String>
    {
        if self._color_attachments.len() < 15 {
            self._color_attachments.push(Attachment{ storage : att, default_color : default_val });
            self._vk_fb = None;
            Ok(())
        } else {
            Err("Слишком много целей для буфера кадра. Максимум 16.".to_string())
        }
    }

    pub fn set_depth_attachment(&mut self, depth: TextureRef, default_val : FramebufferAttachmentDefaultValue)
    {
        let attachment = Attachment {
            storage : depth,
            default_color : default_val
        };
        self._depth_attachment = Some(attachment);
        self._vk_fb = None;
    }

    pub fn reset_attachments(&mut self)
    {
        self._color_attachments.clear();
        self._vk_fb = None;
    }

    pub fn vk_fb(&self) -> &Arc<VkFramebuffer>
    {
        self._vk_fb.as_ref().unwrap()
    }

    pub fn make_vk_fb(&mut self, render_pass : Arc<VkRenderPass>)
    {
        let mut vk_fb_builder = VkFramebuffer::with_intersecting_dimensions(render_pass.clone());
        for attachment in &self._color_attachments {
            vk_fb_builder = vk_fb_builder.add(attachment.storage.take().image_view().clone()).unwrap();
        }
        if self._depth_attachment.is_some()
        {
            vk_fb_builder = vk_fb_builder.add(self._depth_attachment.as_ref().unwrap().storage.take().image_view().clone()).unwrap();
        }
        self._vk_fb = Some(vk_fb_builder.build().unwrap());
    }
}

use vulkano::command_buffer::{AutoCommandBufferBuilder, SubpassContents, PrimaryAutoCommandBuffer};
use vulkano::render_pass::{RenderPass};
use vulkano::command_buffer::pool::CommandPoolBuilderAlloc;

pub trait FramebufferBinder
{
    fn bind_framebuffer(&mut self, framebuffer: FramebufferRef, render_pass: Arc<RenderPass>) -> Result<&mut Self, vulkano::command_buffer::BeginRenderPassError>;
}

impl <P: CommandPoolBuilderAlloc>FramebufferBinder for AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<P::Alloc>, P>
{
    fn bind_framebuffer(&mut self, framebuffer: FramebufferRef, render_pass: Arc<RenderPass>) -> Result<&mut Self, vulkano::command_buffer::BeginRenderPassError>
    {
        let mut clear_values = Vec::new();
        let mut fb = framebuffer.take_mut();
        if fb._vk_fb.is_none() {
            println!("Создание буфера кадра");
            fb.make_vk_fb(render_pass);
        }
        for i in &fb._color_attachments
        {
            clear_values.push(i.default_color);
        }
        if fb._depth_attachment.is_some() {
            clear_values.push(fb._depth_attachment.as_ref().unwrap().default_color);
        }
        self.begin_render_pass(
            fb._vk_fb.as_ref().unwrap().clone(),
            SubpassContents::Inline,
            clear_values
        ).unwrap();
        fb._viewport.dimensions = [fb._dimensions[0] as f32, fb._dimensions[1] as f32];
        
        Ok(self.set_viewport(0, [fb._viewport.clone()]))
    }
}