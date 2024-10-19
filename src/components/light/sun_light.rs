use std::sync::Arc;

use getset::{Getters, Setters, MutGetters, CopyGetters};
use vulkano::memory::allocator::StandardMemoryAllocator;

use crate::{framebuffer::Framebuffer, game_object::GOTransform, components::ProjectionUniformData, types::{Vec3, Vec4, Mat4, ArrayInto}, fast_impl_ssu, shader::ShaderStructUniform, command_buffer::CommandBufferFather};

use super::{GenericLight, ShadowMapMode, ShadowBuffer, GenericLightUniform, AbstractLight, LightType, LightShaderStruct};

#[derive(Getters, Setters, MutGetters, CopyGetters, Clone)]
pub struct SunLight
{
    base: GenericLight,

    /// Размер светового потока.
    #[getset(set, get_copy)]
    size: f32
}

fast_impl_ssu!{
    #[derive(Default)]
    layout(std140) struct SunLightUniform {
        base: GenericLightUniform,
        direction: Vec4,
        size: f32,
        reserved1: f32,
        reserved2: f32,
        reserved3: f32
    }
}

impl SunLightUniform {
    pub fn base_mut(&mut self) -> &mut GenericLightUniform
    {
        &mut self.base
    }
}

impl SunLight
{
    pub fn new(
        size: f32,
        power: f32,
        color: Vec3,
        z_near: f32,
        distance: f32,
        shadow_map_mode: ShadowMapMode,
        command_buffer_father: &CommandBufferFather,
        allocator: Arc<StandardMemoryAllocator>
    ) -> Self {
        let static_shadow = match shadow_map_mode {
            ShadowMapMode::Static(resolution) | ShadowMapMode::SemiDynamic(resolution) => {
                Some(ShadowBuffer::new(command_buffer_father, allocator, resolution, 1))
            }
            _ => None,
        };
        let need_to_refresh_static_shadows =
            if let ShadowMapMode::Static(_) | ShadowMapMode::SemiDynamic(_) = shadow_map_mode {
                true
            } else {
                false
            };
        Self {
            base: GenericLight {
                static_shadow_buffer: static_shadow,
                shadow_map_mode,
                color: [color.x, color.y, color.z].into(),
                need_to_refresh_static_shadows,
                z_near,
                distance,
                transform: Mat4::identity(),
                power,
            },
            size
        }
    }

    #[inline(always)]
    pub fn direction(&self) -> Vec3 {
        -self.base.transform.z_direction()
    }
    
    pub fn uniform_struct(&self, view_projection_matrix: Mat4, shadowmap_index: i32) -> SunLightUniform {
        let direction = self.direction();
        SunLightUniform {
            base : self.base.uniform_struct_base(view_projection_matrix, shadowmap_index).into(),
            direction : [direction.x, direction.y, direction.z, 0.0].into(),
            size : self.size.into(),
            ..Default::default()
        }
    }

    fn projection_matrix(&self) -> Mat4
    {
        let dist = (self.base.distance - self.base.z_near) * 0.5;
        let projection = nalgebra::Orthographic3::new(
            -self.size / 2.0,
            self.size / 2.0,
            -self.size / 2.0,
            self.size / 2.0,
            self.base.z_near-dist,
            self.base.z_near+dist,
        );
        
        projection.as_matrix().clone()
    }
}

impl AbstractLight for SunLight
{
    fn color(&self) -> Vec3 {
        self.base.color()
    }

    fn power(&self) -> f32 {
        self.base.power()
    }

    fn z_near(&self) -> f32 {
        self.base.z_near()
    }

    fn distance(&self) -> f32 {
        self.base.distance()
    }

    fn location(&self) -> Vec3 {
        self.base.location()
    }

    fn static_shadow_buffer(&self) -> Option<&ShadowBuffer> {
        self.base.static_shadow_buffer.as_ref()
    }

    fn shadow_map_mode(&self) -> ShadowMapMode {
        self.base.shadow_map_mode
    }

    fn bbox_corners(&self) -> [Vec3; 8] {
        [
            Vec3::new(
                -std::f32::INFINITY,
                -std::f32::INFINITY,
                -std::f32::INFINITY,
            ),
            Vec3::new(-std::f32::INFINITY, -std::f32::INFINITY, std::f32::INFINITY),
            Vec3::new(-std::f32::INFINITY, std::f32::INFINITY, -std::f32::INFINITY),
            Vec3::new(-std::f32::INFINITY, std::f32::INFINITY, std::f32::INFINITY),
            Vec3::new(std::f32::INFINITY, -std::f32::INFINITY, -std::f32::INFINITY),
            Vec3::new(std::f32::INFINITY, -std::f32::INFINITY, std::f32::INFINITY),
            Vec3::new(std::f32::INFINITY, std::f32::INFINITY, -std::f32::INFINITY),
            Vec3::new(std::f32::INFINITY, std::f32::INFINITY, std::f32::INFINITY),
        ]
    }

    fn take_refresh_flag(&mut self) -> bool {
        self.base.take_refresh_flag()
    }

    fn update_transform(&mut self, transform: &GOTransform) {
        self.base.update_transform(transform);
    }

    fn static_shadow_framebuffers(&self) -> Vec<(Framebuffer, ProjectionUniformData)> {
        match self.base.static_shadow_buffer {
            Some(ref shadow_buffer) => {
                vec![(
                    shadow_buffer.frame_buffers()[0].clone(),
                    self.projections()[0],
                )]
            }
            None => Vec::new(),
        }
    }

    fn projections(&self) -> Vec<ProjectionUniformData> {
        let projection_matrix = self.projection_matrix();
        vec![ProjectionUniformData {
            transform: self.base.transform.into(),
            transform_prev: self.base.transform.into(),
            transform_inverted: self.base.transform
                .try_inverse()
                .unwrap()
                .into(),
            transform_prev_inverted: self.base.transform
                .try_inverse()
                .unwrap()
                .into(),
            projection: projection_matrix.into(),
            projection_inverted: projection_matrix
                .try_inverse()
                .unwrap()
                .into(),
        }]
    }

    fn ty(&self) -> LightType {
        LightType::Sun
    }

    fn uniform_struct(&self, shadowmap_index: i32) -> LightShaderStruct {
        let direction = self.direction();
        let projection_matrix = self.projection_matrix();
        LightShaderStruct::Sun(
            SunLightUniform {
                base : self.base.uniform_struct_base(projection_matrix, shadowmap_index).into(),
                direction : [direction.x, direction.y, direction.z, 0.0].into(),
                size : self.size.into(),
                ..Default::default()
            }
        )
    }
}

crate::impl_behaviour!(SunLight {});
