use std::sync::Arc;

use getset::{Getters, Setters, MutGetters, CopyGetters};
use vulkano::memory::allocator::StandardMemoryAllocator;

use crate::{components::ProjectionUniformData, game_object::GOTransform, types::{Vec3, Vec4, Mat4, ArrayInto}, fast_impl_ssu, framebuffer::Framebuffer, shader::ShaderStructUniform, command_buffer::CommandBufferFather};

use super::{GenericLight, GenericLightUniform, ShadowMapMode, ShadowBuffer, AbstractLight, LightType, LightShaderStruct};

/// Прожекторный источник света.
#[derive(Getters, Setters, MutGetters, CopyGetters, Clone)]
pub struct Spotlight {
    /// Это БАЗА!
    base: GenericLight,

    /// Внутренний угол.
    #[getset(set, get_copy)]
    inner_angle: f32,
    
    /// Внешний угол
    #[getset(set, get_copy)]
    outer_angle: f32,
}

fast_impl_ssu!{
    #[derive(Default)]
    layout(std140) struct SpotlightUniform {
        base: GenericLightUniform,
        direction: Vec4,
        inner_angle: f32,
        outer_angle: f32,
        reserved1: f32,
        reserved2: f32
    }
}

impl SpotlightUniform {
    pub fn base_mut(&mut self) -> &mut GenericLightUniform
    {
        &mut self.base
    }
}

impl Spotlight {
    
    pub fn new(
        power: f32,
        color: [f32; 3],
        angle: f32,
        inner_angle: f32,
        z_near: f32,
        distance: f32,
        shadow_mode: ShadowMapMode,
        command_buffer_father: &CommandBufferFather,
        allocator: Arc<StandardMemoryAllocator>
    ) -> Self {
        let static_shadow_buffer = match shadow_mode {
            ShadowMapMode::Static(resolution) | ShadowMapMode::SemiDynamic(resolution) => {
                Some(ShadowBuffer::new(command_buffer_father, allocator, resolution, 1))
            }
            _ => None,
        };
        let need_to_refresh_static_shadows =
            if let ShadowMapMode::Static(_) | ShadowMapMode::SemiDynamic(_) = shadow_mode {
                true
            } else {
                false
            };
        Self {
            base: GenericLight {
                color: color.into(),
                power,
                z_near,
                distance,
                static_shadow_buffer,
                shadow_map_mode: shadow_mode,
                need_to_refresh_static_shadows,
                transform: Default::default(),
            },
            outer_angle: angle,
            inner_angle,
        }
    }

    
    fn projection_matrix(&self) -> Mat4
    {
        let projection =
        nalgebra::Perspective3::new(1.0, self.outer_angle * 2.0, self.base.z_near, self.base.distance);
        projection.as_matrix().clone()
    }

    #[inline(always)]
    pub fn direction(&self) -> Vec3 {
        -self.base.transform.z_direction()
    }
}

impl AbstractLight for Spotlight
{
    fn ty(&self) -> LightType {
        LightType::Spot
    }

    fn uniform_struct(&self, shadowmap_index: i32) -> LightShaderStruct {
        let direction = self.direction();
        let projection_matrix = self.projection_matrix();
        LightShaderStruct::Spot(
            SpotlightUniform {
                base : self.base.uniform_struct_base(projection_matrix, shadowmap_index).into(),
                direction : [direction.x, direction.y, direction.z, 0.0].into(),
                inner_angle : self.inner_angle.into(),
                outer_angle : self.outer_angle.into(),
                reserved1: 0.0.into(),
                reserved2: 0.0.into()
            }
        )
    }

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
        let far_top_right = self.outer_angle.tan() * self.base.distance;
        let near_top_right = self.outer_angle.tan() * self.base.z_near;
        [
            Vec3::new(-far_top_right,  -far_top_right, -self.base.distance),
            Vec3::new(-far_top_right,   far_top_right, -self.base.distance),
            Vec3::new( far_top_right,   far_top_right, -self.base.distance),
            Vec3::new( far_top_right,  -far_top_right, -self.base.distance),
            Vec3::new(-near_top_right, -near_top_right, -self.base.z_near),
            Vec3::new(-near_top_right,  near_top_right, -self.base.z_near),
            Vec3::new( near_top_right,  near_top_right, -self.base.z_near),
            Vec3::new( near_top_right, -near_top_right, -self.base.z_near),
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
}

crate::impl_behaviour!(Spotlight {});