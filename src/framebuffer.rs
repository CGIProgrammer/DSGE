use std::sync::Arc;
use vulkano::render_pass::{
    Framebuffer as VkFramebuffer, FramebufferCreateInfo, RenderPass as VkRenderPass,
};
use crate::renderer::RenderResolution;
use crate::texture::*;
use vulkano::pipeline::graphics::viewport::Viewport;

pub type FramebufferRef = RcBox<Framebuffer>;
pub type FramebufferAttachmentDefaultValue = vulkano::format::ClearValue;

#[derive(Clone)]
struct Attachment {
    storage: Texture,
    default_color: Option<FramebufferAttachmentDefaultValue>,
}

/// Буфер кадра
#[derive(Clone)]
pub struct Framebuffer {
    _viewport: Viewport,                   // Окно вида
    _dimensions: [u16; 2],                 // Разрешение
    _color_attachments: Vec<Attachment>,   // "Разъёмы" для вывода данных
    _depth_attachment: Option<Attachment>, // "Разъём" для буфера глубины
    _vk_fb: Option<Arc<VkFramebuffer>>,    // Буфер кадра vulkano
}

/// Буфер кадра
/// Может использоваться и как буфер кадра "по умолчанию", так и
/// для рендеринга в текстуру
//#[allow(dead_code)]
impl Framebuffer {
    pub fn new(width: u16, height: u16) -> Framebuffer {
        Self {
            _viewport: Viewport {
                offset: [0.0, 0.0],
                extent: [0.0, 0.0],
                depth_range: 0.0..=1.0,
            },
            _dimensions: [width, height],
            _color_attachments: Vec::new(),
            _depth_attachment: None,
            _vk_fb: None,
        }
    }

    pub fn width(&self) -> u16 {
        self._dimensions[0]
    }

    pub fn height(&self) -> u16 {
        self._dimensions[1]
    }

    pub fn render_resolution(&self) -> RenderResolution {
        RenderResolution::new(self._dimensions[0] as _, self._dimensions[1] as _)
    }

    pub fn viewport(&self) -> &Viewport {
        return &self._viewport;
    }

    /// Присоединение "цветного" изображения к выходу фреймбуфера
    pub fn add_color_attachment(
        &mut self,
        att: &Texture,
        default_val: Option<FramebufferAttachmentDefaultValue>,
    ) -> Result<(), String> {
        if self._color_attachments.len() < 15 {
            self._color_attachments.push(Attachment {
                storage: att.clone(),
                default_color: default_val,
            });
            self._vk_fb = None;
            Ok(())
        } else {
            Err("Слишком много целей для буфера кадра. Максимум 16.".to_owned())
        }
    }

    /// Присоединение изображения в качестве буфера глубины
    pub fn set_depth_attachment(
        &mut self,
        depth: &Texture,
        default_val: Option<FramebufferAttachmentDefaultValue>,
    ) {
        let attachment = Attachment {
            storage: depth.clone(),
            default_color: default_val,
        };
        self._depth_attachment = Some(attachment);
        self._vk_fb = None;
    }

    pub fn depth_attachment(&self) -> &Texture {
        &self._depth_attachment.as_ref().unwrap().storage
    }

    /// Очистить список изображений
    pub fn reset_attachments(&mut self) {
        self._color_attachments.clear();
        self._vk_fb = None;
    }

    /// Установка окна вида
    pub fn view_port(&mut self, width: u16, height: u16) {
        self._dimensions = [width, height];
    }

    /// Инициализировать буфер кадра для render pass'а
    pub fn make_vk_fb(&mut self, render_pass: Arc<VkRenderPass>) -> Result<(), String> {
        let mut attachments = Vec::new();
        for attachment in &self._color_attachments {
            let image_view = attachment.storage.image_view().clone();
            attachments.push(image_view);
        }
        if self._depth_attachment.is_some() {
            let image_view = self
                ._depth_attachment
                .as_ref()
                .unwrap()
                .storage
                .image_view()
                .clone();
            attachments.push(image_view);
        }

        let vk_fb = VkFramebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: attachments,
                ..Default::default()
            },
        );
        match vk_fb {
            Ok(fb) => {
                self._vk_fb = Some(fb);
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

use vulkano::command_buffer::{
    AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassContents, SubpassBeginInfo,
};
use vulkano::render_pass::RenderPass;

pub trait FramebufferBinder {
    fn bind_framebuffer(
        &mut self,
        framebuffer: &mut Framebuffer,
        render_pass: Arc<RenderPass>,
        secondary: bool,
        flip_y: bool,
    ) -> Result<&mut Self, String>;
}

impl FramebufferBinder for AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> {

    fn bind_framebuffer(
        &mut self,
        fb: &mut Framebuffer,
        render_pass: Arc<RenderPass>,
        secondary: bool,
        flip_y: bool,
    ) -> Result<&mut Self, String> {
        let mut clear_values = Vec::new();
        if fb._vk_fb.is_none() {
            //println!("Создание буфера кадра");
            fb.make_vk_fb(render_pass.clone())?;
        }
        for Attachment {
            storage: _,
            default_color,
        } in &fb._color_attachments
        {
            clear_values.push(*default_color);
        }

        if let Some(ref depth_attachment) = fb._depth_attachment {
            clear_values.push(depth_attachment.default_color);
        }
        let mut rpbi = RenderPassBeginInfo::framebuffer(fb._vk_fb.as_ref().unwrap().clone());
        rpbi.clear_values = clear_values;
        
        let spbi = SubpassBeginInfo {
            contents: if secondary {SubpassContents::SecondaryCommandBuffers} else {SubpassContents::Inline},
            ..Default::default()
        };

        if flip_y {
            fb._viewport.extent = [fb._dimensions[0] as f32, -(fb._dimensions[1] as f32)];
            fb._viewport.offset = [0.0, fb._dimensions[1] as f32];
        } else {
            fb._viewport.extent = [fb._dimensions[0] as f32, fb._dimensions[1] as f32];
            fb._viewport.offset = [0.0, 0.0];
        }
        //fb._viewport.
        self
            .begin_render_pass(rpbi, spbi)
            .map_err(|e| e.to_string())?
            .set_viewport(0, [fb._viewport.clone()].into_iter().collect())
            .map_err(|e| e.to_string())
    }
}
