use std::sync::Arc;

use vulkano::device::Device;

use crate::framebuffer::Framebuffer;
use crate::types::{Vec3};
use crate::texture::{Texture, TextureDimensions, TexturePixelFormat};
use crate::game_object::GameObject;
use crate::components::ProjectionUniformData;

#[derive(Clone)]
struct ShadowBuffer(Texture);

impl ShadowBuffer
{
    fn new(device: Arc<Device>, resolution: u16, layers: u16) -> Self
    {
        let buffer = Texture::new_empty(
            format!("Shadow buffer {}", resolution).as_str(),
            TextureDimensions::Dim2d{
                width: resolution as _,
                height: resolution as _,
                array_layers: layers as _
            },
            TexturePixelFormat::D16_UNORM,
            device
        ).unwrap();
        Self(buffer)
    }

    fn shadow_map_framebuffers(&self) -> Vec<Framebuffer>
    {
        let width = self.0.width() as u16;
        let height = self.0.height() as u16;
        (0..self.0.array_layers()).map(
            |layer| {
                let mut fb = Framebuffer::new(width, height);
                let subbuffer = self.0.array_layer_as_texture(layer).unwrap();
                fb.set_depth_attachment(&subbuffer, 1.0.into());
                fb
            }
        ).collect::<Vec<_>>()
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct SpotLight {
    dynamic_shadow_buffer: Option<ShadowBuffer>,
    angle: f32,
    inner_angle: f32,
    power: f32,
    distance: f32,
    color: Vec3
}

impl SpotLight
{
    pub fn new(power: f32, color: [f32; 3], angle: f32, inner_angle: f32, distance: f32, resolution: u16, device: Arc<Device>) -> Self
    {
        Self {
            dynamic_shadow_buffer: if resolution > 0 {Some(ShadowBuffer::new(device, resolution, 1))} else {None},
            power,
            color: color.into(),
            angle,
            inner_angle,
            distance
        }
    }

    pub fn projection_data(&self, owner: &GameObject) -> ProjectionUniformData
    {
        let transform = owner.transform();
        let projection = nalgebra::Perspective3::new(1.0, self.angle, 0.1, self.distance);
        let projection_matrix = projection.as_matrix().clone();
        ProjectionUniformData {
            transform : transform.global_for_render.as_slice().try_into().unwrap(),
            transform_prev : transform.global_for_render_prev.as_slice().try_into().unwrap(),
            transform_inverted : transform.global_for_render.try_inverse().unwrap().as_slice().try_into().unwrap(),
            transform_prev_inverted : transform.global_for_render_prev.try_inverse().unwrap().as_slice().try_into().unwrap(),
            projection : projection_matrix.as_slice().try_into().unwrap(),
            projection_inverted : projection_matrix.try_inverse().unwrap().as_slice().try_into().unwrap(),
        }
    }

    pub fn render_pass_data(&self, owner: &GameObject) -> Option<(Framebuffer, ProjectionUniformData)>
    {
        match self.dynamic_shadow_buffer {
            Some(ref shadow_buffer) => {
                Some((shadow_buffer.shadow_map_framebuffers().remove(0), self.projection_data(owner)))
            },
            None => None
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct SunLight {
    shadow_buffer: Option<ShadowBuffer>,
    power: f32,
    color: Vec3
}

impl SunLight
{
    pub fn new(power: f32, color: [f32; 3], resolution: u16, device: Arc<Device>) -> Self
    {
        Self {
            shadow_buffer: if resolution > 0 {Some(ShadowBuffer::new(device, resolution, 1))} else {None},
            power,
            color: color.into()
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct PointLight {
    shadow_buffer: Option<ShadowBuffer>,
    power: f32,
    color: Vec3
}

impl PointLight
{
    pub fn new(power: f32, color: [f32; 3], resolution: u16, device: Arc<Device>) -> Self
    {
        Self {
            shadow_buffer: if resolution > 0 {Some(ShadowBuffer::new(device, resolution, 1))} else {None},
            power,
            color: color.into()
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum Light
{
    Spot(SpotLight),
    Point(PointLight),
    Sun(SunLight)
}

impl Light
{
    pub fn framebuffers(&self, owner: &GameObject) -> Vec<(Framebuffer, ProjectionUniformData)>
    {
        match self {
            Self::Spot(light) => {
                match light.render_pass_data(owner) {
                    Some(shadow_buffer) => vec![shadow_buffer],
                    None => Vec::new()
                }
            },
            _ => {
                panic!("Не реализовано!")
            }
        }
    }

    pub fn ty(&self) -> &str
    {
        match self
        {
            Self::Spot(_) => "SpotLight",
            Self::Point(_) => "PointLight",
            Self::Sun(_) => "SunLight",
        }
    }
}