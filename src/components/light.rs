use crate::types::{Vec3};
use crate::texture::TextureRef;


#[allow(dead_code)]
pub struct SunLight {
    dynamic_shadow_maps: Option<TextureRef>,
    static_shadow_map: Option<TextureRef>,
    power: f32,
    color: Vec3
}

#[allow(dead_code)]
pub struct PointLight {
    dynamic_shadow_map: Option<TextureRef>,
    static_shadow_map: Option<TextureRef>,
    power: f32,
    color: Vec3
}

#[allow(dead_code)]
pub struct SpotLight {
    dynamic_shadow_map: Option<TextureRef>,
    static_shadow_map: Option<TextureRef>,
    angle: f32,
    inner_angle: f32,
    power: f32,
    color: Vec3
}

pub enum Light
{
    Spot(SpotLight),
    Point(PointLight),
    Sun(SunLight)
}