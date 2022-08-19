use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use nalgebra::Rotation3;
use vulkano::device::Device;
use vulkano::image::view::{ImageView, ImageViewCreateInfo};

use crate::framebuffer::Framebuffer;
use crate::types::{Vec3, Vec4, Transform3, Mat4};
use crate::texture::{Texture, TextureDimensions, TexturePixelFormat, ShaderStructUniform};
use crate::game_object::GOTransform;
use crate::components::ProjectionUniformData;


#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct LightsUniformData
{
    spotlights: i32,
    point_lights: i32,
    sun_lights: i32,
}

impl LightsUniformData
{
    pub(crate) fn new(spotlights: i32, point_lights: i32, sun_lights: i32) -> Self
    {
        Self { spotlights, point_lights, sun_lights }
    }
}

impl ShaderStructUniform for LightsUniformData
{
    fn structure() -> String {
        "{
            int spotlights;
            int point_lights;
            int sun_lights;
        }".to_owned()
    }

    fn glsl_type_name() -> String
    {
        "LightsCount".to_owned()
    }

    fn texture(&self) -> Option<&crate::texture::Texture> {
        None
    }
}

/*#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub(crate)struct SunDataUniform
{
    color: [f32; 3],
    power: f32,
    direction: [f32; 3],
    cascade_projections: [[f32; 16]; 6]
}

impl ShaderStructUniform for SunDataUniform
{
    fn structure() -> String
    {
        "{
            vec3 color;
            float power;
            vec3 direction;
            mat4 cascade_projections[6];
        }".to_owned()
    }

    fn glsl_type_name() -> String
    {
        "Sun".to_owned()
    }

    fn texture(&self) -> Option<&crate::texture::Texture> {
        None
    }
}*/

#[derive(Clone)]
struct ShadowBuffer
{
    buffer: Texture,
    frame_buffers: Vec<Framebuffer>
}

impl ShadowBuffer
{
    fn new(device: Arc<Device>, resolution: u16, layers: u16, cubemap: bool) -> Self
    {
        let layers = if cubemap {6} else {layers};
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
        let frame_buffers = (0..layers).map(
            |layer| {
                let mut fb = Framebuffer::new(resolution, resolution);
                let subbuffer = buffer.array_layer_as_texture(layer as _).unwrap();
                fb.set_depth_attachment(&subbuffer, 1.0.into());
                fb
            }
        ).collect::<Vec<_>>();
        let mut result = Self{
            buffer,
            frame_buffers
        };
        if cubemap {
            if let Some(cbm) = result.as_cubemap() {
                result.buffer = cbm;
            }
        }
        result
    }

    fn shadow_map_framebuffers(&self) -> &Vec<Framebuffer>
    {
        return &self.frame_buffers;
    }

    fn as_cubemap(&self) -> Option<Texture>
    {
        let iw = self.buffer._vk_image_view.clone();
        if iw.array_layers().len() == 6 {
            let iw = ImageView::new(self.buffer._vk_image_access.clone(), ImageViewCreateInfo {
                view_type: vulkano::image::view::ImageViewType::Cube,
                component_mapping: iw.component_mapping(),
                format: iw.format(),
                array_layers: iw.array_layers(),
                aspects: iw.aspects().clone(),
                mip_levels: iw.mip_levels(),
                sampler_ycbcr_conversion: match iw.sampler_ycbcr_conversion() {Some(conv) => Some(conv.clone()), None => None},
                ..Default::default()
            }).unwrap();
            Some(Texture::from_vk_image_view(iw, self.buffer._vk_device.clone()).unwrap())
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Zeroable, Pod)]
pub struct SpotLightUniform
{
    pub location_znear : [f32; 4],
    pub direction_zfar : [f32; 4],
    pub color : [f32; 4],
    pub projection_inv : [f32; 16],
    pub angle : f32,
    pub inner_angle : f32,
    pub shadow_buffer : i32,
    pub shadowmap_index : i32
}

impl ShaderStructUniform for SpotLightUniform
{
    fn structure() -> String
    {
        "{
            vec4 location_znear;
            vec4 direction_zfar;
            vec4 color;
            mat4 projection_inv;
            float angle;
            float inner_angle;
            int shadow_buffer;
            int shadowmap_index;
        }".to_owned()
    }

    fn glsl_type_name() -> String
    {
        "SpotLight".to_owned()
    }

    fn texture(&self) -> Option<&crate::texture::Texture>
    {
        None
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct SpotLight {
    shadow_buffer: Option<ShadowBuffer>,
    angle: f32,
    inner_angle: f32,
    color: Vec4,
    distance: f32,
    znear: f32,
    location: Vec3,
    direction: Vec3,
}

impl Debug for SpotLight {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        f.debug_struct("SpotLight")
            .field("shadow_buffer", &match self.shadow_buffer { Some(_) => Some(()), None => None})
            .field("angle", &self.angle)
            .field("inner_angle", &self.inner_angle)
            .field("color", &self.color)
            .field("distance", &self.distance)
            .field("znear", &self.znear)
            .field("location", &self.location)
            .field("direction", &self.direction);
        Ok(())
    }
}

impl SpotLight
{
    pub fn as_uniform_struct(&self, view_projection_matrix: Mat4, shadowmap_index: i32) -> SpotLightUniform
    {
        let location: [f32; 3] = self.location.into();
        let direction: [f32; 3] = self.direction.into();
        SpotLightUniform {
            location_znear: [location[0], location[1], location[2], self.znear],
            direction_zfar: [direction[0], direction[1], direction[2], self.distance],
            color: self.color.into(),
            angle: self.angle,
            inner_angle: self.inner_angle,
            shadow_buffer: match self.shadow_buffer {Some(_) => i32::MAX, None => 0},
            projection_inv: view_projection_matrix.as_slice().try_into().unwrap(),
            shadowmap_index: shadowmap_index,
        }
    }

    pub fn new(power: f32, color: [f32; 3], angle: f32, inner_angle: f32, znear: f32, distance: f32, resolution: u16, device: Arc<Device>) -> Self
    {
        Self {
            shadow_buffer: if resolution > 0 {Some(ShadowBuffer::new(device, resolution, 1, false))} else {None},
            color: [color[0], color[1], color[2], power].into(),
            angle,
            inner_angle,
            distance,
            znear,
            location: [0.0, 0.0, 0.0].into(),
            direction: [0.0, 0.0, -1.0].into()
        }
    }

    pub fn projection_data(&self, transform: &GOTransform) -> ProjectionUniformData
    {
        let projection = nalgebra::Perspective3::new(1.0, self.angle*2.0, self.znear, self.distance);
        let projection_matrix = projection.as_matrix().clone();
        ProjectionUniformData {
            transform : transform.global.as_slice().try_into().unwrap(),
            transform_prev : transform.global_prev.as_slice().try_into().unwrap(),
            transform_inverted : transform.global.try_inverse().unwrap().as_slice().try_into().unwrap(),
            transform_prev_inverted : transform.global_prev.try_inverse().unwrap().as_slice().try_into().unwrap(),
            projection : projection_matrix.as_slice().try_into().unwrap(),
            projection_inverted : projection_matrix.try_inverse().unwrap().as_slice().try_into().unwrap(),
        }
    }

    pub fn render_pass_data(&mut self, owner: &GOTransform) -> Vec<(Framebuffer, ProjectionUniformData)>
    {
        let matrix = owner.global;
        let location = matrix.column(3);
        let direction = -matrix.column(2);
        self.location = [location[0], location[1], location[2]].into();
        self.direction = [direction[0], direction[1], direction[2]].into();
        //println!("{}", owner.global);
        match self.shadow_buffer {
            Some(ref shadow_buffer) => {
                vec![(shadow_buffer.shadow_map_framebuffers()[0].clone(), self.projection_data(owner))]
            },
            None => Vec::new()
        }
    }

    pub fn serialize(&self) -> Vec<f32>
    {
        vec![
            self.location[0], self.location[1], self.location[2], 1.0,
            self.direction[0], self.direction[1], self.direction[2], 1.0,
            self.color[0], self.color[1], self.color[2], self.color[3],
            match self.shadow_buffer {Some(_) => 1.0, None => 0.0}, self.distance, self.znear, self.angle, self.inner_angle
        ]
    }

    pub fn glsl_code() -> &'static str
    {
        "struct SpotLight {
            vec3 location;
            vec3 direction;
            vec3 color;
            float power;
            float distance;
            float znear;
            float angle;
            float inner_angle;
            bool shadow_buffer;
            mat4 projection_inv;
            int shadowmap_index;
        };
        SpotLight unpack_spotlight(sampler2D l_buffer, int offset)
        {
            offset *= 2;
            vec3 location = texelFetch(l_buffer, ivec2(0, offset), 0).xyz;
            vec3 direction = texelFetch(l_buffer, ivec2(1, offset), 0).xyz;
            vec4 power = texelFetch(l_buffer, ivec2(2, offset), 0);
            vec4 d0 = texelFetch(l_buffer, ivec2(3, offset), 0);
            bool shadow_buffer = d0.x != 0.0;
            float distance = d0.y;
            float znear = d0.z;
            float angle = d0.w;
            vec4 inner_angle_and_shadowmap_index = texelFetch(l_buffer, ivec2(4, offset), 0);
            float inner_angle = inner_angle_and_shadowmap_index.x;
            int shadowmap_index = int(inner_angle_and_shadowmap_index.y);
            mat4 projection_inv = mat4(
                texelFetch(l_buffer, ivec2(0, offset+1), 0),
                texelFetch(l_buffer, ivec2(1, offset+1), 0),
                texelFetch(l_buffer, ivec2(2, offset+1), 0),
                texelFetch(l_buffer, ivec2(3, offset+1), 0)
            );
            
            return SpotLight(
                location,
                direction,
                power.rgb, power.w,
                distance,
                znear,
                angle,
                inner_angle,
                shadow_buffer,
                projection_inv,
                shadowmap_index
            );
        }
        /*float spotlight_shadow(sampler2D shadow_map, SpotLight light)
        {
            texture(l_buffer, 
            return light.
        }*/"
    }
}


#[allow(dead_code)]
#[derive(Clone)]
pub struct SunLight {
    shadow_buffer: Option<ShadowBuffer>,
    power: f32,
    color: Vec3,
    size: f32,
    direction: Vec3
}

impl Debug for SunLight {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        f.debug_struct("SunLight")
            .field("shadow_buffer", &match self.shadow_buffer { Some(_) => Some(()), None => None})
            .field("power", &self.power)
            .field("color", &self.color)
            .field("size", &self.size)
            .field("direction", &self.direction);
        Ok(())
    }
}

impl SunLight
{
    pub fn new(size: f32, power: f32, color: [f32; 3], resolution: u16, device: Arc<Device>) -> Self
    {
        Self {
            shadow_buffer: if resolution > 0 {Some(ShadowBuffer::new(device, resolution, 1, false))} else {None},
            power,
            color: color.into(),
            direction: [0.0, 0.0, -1.0].into(),
            size: size,
        }
    }

    pub fn serialize(&self) -> Vec<f32>
    {
        vec![
            self.direction[0], self.direction[1], self.direction[2], 0.0,
            self.color[0], self.color[1], self.color[2], self.power,
            match self.shadow_buffer {Some(_) => 1.0, None => 0.0}
        ]
    }

    pub fn glsl_code() -> &'static str
    {
        "struct SunLight {
            vec3 direction;
            vec3 color;
            float power;
            bool shadow_buffer;
            mat4 view_projection;
        };
        SunLight unpack_sun_light(sampler2D l_buffer, int offset)
        {
            vec3 direction = texelFetch(l_buffer, ivec2(0, offset*2), 0).xyz;
            vec4 power = texelFetch(l_buffer, ivec2(1, offset*2), 0);
            bool shadow_buffer = texelFetch(l_buffer, ivec2(2, offset*2), 0).x != 0.0;
            mat4 view_projection = mat4(
                texelFetch(l_buffer, ivec2(0, offset*2+1), 0),
                texelFetch(l_buffer, ivec2(1, offset*2+1), 0),
                texelFetch(l_buffer, ivec2(2, offset*2+1), 0),
                texelFetch(l_buffer, ivec2(3, offset*2+1), 0)
            );
            return SunLight(
                direction,
                power.rgb, power.w,
                shadow_buffer,
                view_projection
            );
        }"
    }

    pub fn projection_data(&self, transform: &GOTransform) -> ProjectionUniformData
    {
        let size = self.size;
        let projection = nalgebra::Orthographic3::new(-size / 2.0, size / 2.0, -size / 2.0, size / 2.0, -size / 2.0, size / 2.0);
        let projection_matrix = projection.as_matrix().clone();
        ProjectionUniformData {
            transform : transform.global.as_slice().try_into().unwrap(),
            transform_prev : transform.global_prev.as_slice().try_into().unwrap(),
            transform_inverted : transform.global.try_inverse().unwrap().as_slice().try_into().unwrap(),
            transform_prev_inverted : transform.global_prev.try_inverse().unwrap().as_slice().try_into().unwrap(),
            projection : projection_matrix.as_slice().try_into().unwrap(),
            projection_inverted : projection_matrix.try_inverse().unwrap().as_slice().try_into().unwrap(),
        }
    }

    pub fn render_pass_data(&mut self, transform: &GOTransform) -> Vec<(Framebuffer, ProjectionUniformData)>
    {
        let matrix = transform.global;
        let direction = -matrix.column(2);
        self.direction = [direction[0], direction[1], direction[2]].into();
        match self.shadow_buffer {
            Some(ref shadow_buffer) => {
                vec![(shadow_buffer.shadow_map_framebuffers()[0].clone(), self.projection_data(transform))]
            },
            None => Vec::new()
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct PointLight {
    shadow_buffer: Option<ShadowBuffer>,
    power: f32,
    color: Vec3,
    distance: f32,
    location: Vec3,
}

impl Debug for PointLight {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        f.debug_struct("PointLight")
            .field("shadow_buffer", &match self.shadow_buffer { Some(_) => Some(()), None => None})
            .field("power", &self.power)
            .field("color", &self.color)
            .field("distance", &self.distance)
            .field("location", &self.location);
        Ok(())
    }
}

impl PointLight
{
    pub fn new(power: f32, color: [f32; 3], resolution: u16, device: Arc<Device>) -> Self
    {
        Self {
            shadow_buffer: if resolution > 0 {Some(ShadowBuffer::new(device, resolution, 6, true))} else {None},
            power,
            color: color.into(),
            location: Vec3::default(),
            distance: 10.0
        }
    }

    pub fn serialize(&self) -> Vec<f32>
    {
        vec![
            self.location[0], self.location[1], self.location[2], 1.0,
            self.color[0], self.color[1], self.color[2], self.power,
            match self.shadow_buffer {Some(_) => 1.0, None => 0.0}, self.distance
        ]
    }

    pub fn glsl_code() -> &'static str
    {
        "struct PointLight {
            vec3 location;
            vec3 color;
            float power;
            bool shadow_buffer;
            mat4 view_projection;
        };
        PointLight unpack_point_light(sampler2D l_buffer, int offset)
        {
            vec3 location = texelFetch(l_buffer, ivec2(0, offset*2), 0).xyz;
            vec4 power = texelFetch(l_buffer, ivec2(1, offset*2), 0);
            bool shadow_buffer = texelFetch(l_buffer, ivec2(2, offset*2), 0).x != 0.0;
            mat4 view_projection = mat4(
                texelFetch(l_buffer, ivec2(0, offset*2+1), 0),
                texelFetch(l_buffer, ivec2(1, offset*2+1), 0),
                texelFetch(l_buffer, ivec2(2, offset*2+1), 0),
                texelFetch(l_buffer, ivec2(3, offset*2+1), 0)
            );
            return PointLight(
                location,
                power.rgb, power.w,
                shadow_buffer,
                view_projection
            );
        }"
    }

    pub fn projection_data(&self, owner_transform: &GOTransform, direction: usize) -> ProjectionUniformData
    {
        let rotation = match direction {
            /*  x Направо */ 0 => Rotation3::from_euler_angles(std::f32::consts::FRAC_PI_2, 0.0, -std::f32::consts::FRAC_PI_2),
            /* -x Налево  */ 1 => Rotation3::from_euler_angles(std::f32::consts::FRAC_PI_2, 0.0, std::f32::consts::FRAC_PI_2),
            /*  y Вперёд  */ 2 => Rotation3::from_euler_angles(std::f32::consts::FRAC_PI_2, 0.0, 0.0),
            /* -y Назад   */ 3 => Rotation3::from_euler_angles(std::f32::consts::FRAC_PI_2, 0.0, -std::f32::consts::PI),
            /*  z Вверх   */ 4 => Rotation3::from_euler_angles(std::f32::consts::PI, 0.0, 0.0),
            /* -z Вниз    */ 5 => Rotation3::default(),
            _ => panic!("Неправильное направление.")
        };

        let mut transform = owner_transform.global;
        let mut transform_prev = owner_transform.global_prev;
        transform.set_rotation(&rotation);
        transform_prev.set_rotation(&rotation);
        let projection = nalgebra::Perspective3::new(1.0, std::f32::consts::FRAC_PI_2, 0.05, self.distance);
        let projection_matrix = projection.as_matrix().clone();
        ProjectionUniformData {
            transform : transform.as_slice().try_into().unwrap(),
            transform_prev : transform_prev.as_slice().try_into().unwrap(),
            transform_inverted : transform.try_inverse().unwrap().as_slice().try_into().unwrap(),
            transform_prev_inverted : transform_prev.try_inverse().unwrap().as_slice().try_into().unwrap(),
            projection : projection_matrix.as_slice().try_into().unwrap(),
            projection_inverted : projection_matrix.try_inverse().unwrap().as_slice().try_into().unwrap(),
        }
    }

    pub fn render_pass_data(&mut self, transform: &GOTransform) -> Vec<(Framebuffer, ProjectionUniformData)>
    {
        let matrix = transform.global;
        let location = matrix.column(3);
        self.location = [location[0], location[1], location[2]].into();
        match self.shadow_buffer {
            Some(ref shadow_buffer) => {
                shadow_buffer.shadow_map_framebuffers()
                    .iter()
                    .enumerate()
                    .map(|(direction, frame_buffer)|
                        (frame_buffer.clone(), self.projection_data(transform, direction))
                    ).collect::<Vec<_>>()[1..2].to_vec()
            },
            None => Vec::new()
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Light
{
    Spot(SpotLight),
    Point(PointLight),
    Sun(SunLight)
}

impl Eq for Light {}

impl PartialEq for Light
{
    fn eq(&self, other: &Light) -> bool
    {
        match (self, other)
        {
            (Self::Spot(_), Self::Spot(_)) => true,
            (Self::Point(_), Self::Point(_)) => true,
            (Self::Sun(_), Self::Sun(_)) => true,
            _ => false
        }
    }
}

impl PartialOrd for Light
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering>
    {
        match (self, other) {
            (Self::Spot(_),  Self::Spot(_) ) |
            (Self::Point(_), Self::Point(_)) |
            (Self::Sun(_),   Self::Sun(_)  ) => {
                Some(core::cmp::Ordering::Equal)
            },

            (Self::Spot(_), Self::Point(_) | Self::Sun(_)) => {
                Some(core::cmp::Ordering::Less)
            },
            (Self::Point(_), Self::Sun(_)) => {
                Some(core::cmp::Ordering::Less)
            },
            
            (Self::Sun(_), Self::Spot(_) | Self::Point(_)) => {
                Some(core::cmp::Ordering::Greater)
            },
            (Self::Point(_), Self::Spot(_)) => {
                Some(core::cmp::Ordering::Greater)
            },
        }
    }
}

impl Ord for Light
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering
    {
        self.partial_cmp(other).unwrap()
    }
}

impl Light
{
    pub fn framebuffers(&mut self, owner: &GOTransform) -> Vec<(Framebuffer, ProjectionUniformData)>
    {
        match self {
            Self::Spot(light) => {
                light.render_pass_data(owner)
            },
            Self::Point(light) => {
                light.render_pass_data(owner)
            },
            Self::Sun(light) => {
                light.render_pass_data(owner)
            }
        }
    }

    pub fn shadowmap(&self) -> Option<&Texture>
    {
        match self {
            Self::Spot(ref light) => 
                match light.shadow_buffer {
                    Some(ref sm_buff) => Some(&sm_buff.buffer),
                    None => None
                },
            Self::Point(ref light) => 
                match light.shadow_buffer {
                    Some(ref sm_buff) => Some(&sm_buff.buffer),
                    None => None
                },
            Self::Sun(ref light) => 
                match light.shadow_buffer {
                    Some(ref sm_buff) => Some(&sm_buff.buffer),
                    None => None
                },
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

    pub fn serialize(&self) -> Vec<f32>
    {
        match self {
            Self::Spot(light) => {
                light.serialize()
            },
            Self::Point(light) => {
                light.serialize()
            },
            Self::Sun(light) => {
                light.serialize()
            }
        }
    }

    pub fn glsl_code(&self) -> &'static str
    {
        match self {
            Self::Spot(_) => {
                SpotLight::glsl_code()
            },
            Self::Point(_) => {
                PointLight::glsl_code()
            },
            Self::Sun(_) => {
                SunLight::glsl_code()
            }
        }
    }
}

crate::impl_behaviour!(
    Light { }
);