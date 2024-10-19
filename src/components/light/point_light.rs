use std::{ops::{Deref, DerefMut}, sync::Arc};

use nalgebra::Rotation3;
use vulkano::memory::allocator::StandardMemoryAllocator;

use crate::{types::{Vec3, Transform3, Mat4}, fast_impl_ssu, shader::ShaderStructUniform, components::ProjectionUniformData, framebuffer::Framebuffer, impl_behaviour, game_object::GOTransform, command_buffer::CommandBufferFather};

use super::{GenericLight, ShadowMapMode, ShadowBuffer, GenericLightUniform, AbstractLight, LightShaderStruct, LightType};

#[derive(Clone)]
pub struct PointLight
{
    base: GenericLight,
}

impl Deref for PointLight {
    type Target = GenericLight;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for PointLight {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl PointLight {
    pub fn new(
        power: f32,
        color: [f32; 3],
        z_near: f32,
        distance: f32,
        shadow_map_mode: ShadowMapMode,
        command_buffer_father: &CommandBufferFather,
        allocator: Arc<StandardMemoryAllocator>
    ) -> Self {
        let static_shadow_buffer = match shadow_map_mode {
            ShadowMapMode::Static(resolution) | ShadowMapMode::SemiDynamic(resolution) => {
                Some(ShadowBuffer::new(command_buffer_father, allocator, resolution, 6))
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
                transform: Default::default(),
                color: color.into(),
                power,
                z_near,
                distance,
                static_shadow_buffer,
                shadow_map_mode,
                need_to_refresh_static_shadows
            }
        }
    }

    pub fn projection_matrix(&self) -> Mat4
    {
        nalgebra::Perspective3::new(
            1.0,
            std::f32::consts::FRAC_PI_2,
            self.base.z_near(),
            self.base.distance(),
        ).as_matrix().clone()
    }
}

impl_behaviour!(PointLight {});

impl AbstractLight for PointLight {
    fn bbox_corners(&self) -> [Vec3; 8] {
        [
            Vec3::new(-self.distance, -self.distance, -self.distance),
            Vec3::new(-self.distance, -self.distance, self.distance),
            Vec3::new(-self.distance, self.distance, -self.distance),
            Vec3::new(-self.distance, self.distance, self.distance),
            Vec3::new(self.distance, -self.distance, -self.distance),
            Vec3::new(self.distance, -self.distance, self.distance),
            Vec3::new(self.distance, self.distance, -self.distance),
            Vec3::new(self.distance, self.distance, self.distance),
        ]
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
        self.base.static_shadow_buffer()
    }

    fn shadow_map_mode(&self) -> ShadowMapMode {
        self.base.shadow_map_mode
    }

    fn take_refresh_flag(&mut self) -> bool {
        self.base.take_refresh_flag()
    }

    fn update_transform(&mut self, transform: &GOTransform) {
        self.base.update_transform(transform);
    }

    fn static_shadow_framebuffers(&self) -> Vec<(Framebuffer, ProjectionUniformData)> {
        if let Some(static_buffer) = self.base.static_shadow_buffer() {
            static_buffer.frame_buffers().clone().into_iter().zip(self.projections().into_iter()).collect::<Vec<_>>()
        } else {
            vec![]
        }
    }

    fn projections(&self) -> Vec<ProjectionUniformData> {
        let rotations = [
            Rotation3::look_at_rh(&[1.0, 0.0, 0.0].into(), &[0.0, -1.0, 0.0].into()),
            Rotation3::look_at_rh(&[-1.0, 0.0, 0.0].into(), &[0.0, -1.0, 0.0].into()),
            Rotation3::look_at_rh(&[0.0, -1.0, 0.0].into(), &[0.0, 0.0, -1.0].into()),
            Rotation3::look_at_rh(&[0.0, 1.0, 0.0].into(), &[0.0, 0.0, 1.0].into()),
            Rotation3::look_at_rh(&[0.0, 0.0, 1.0].into(), &[0.0, -1.0, 0.0].into()),
            Rotation3::look_at_rh(&[0.0, 0.0, -1.0].into(), &[0.0, -1.0, 0.0].into())
        ];
        let mut transform = self.base.transform;
        let mut transform_prev = transform.clone();

        let projection_matrix = self.projection_matrix();

        rotations.iter().map(|rotation| {
            transform.set_rotation(&rotation);
            transform_prev.set_rotation(&rotation);
            ProjectionUniformData {
                transform: transform.into(),
                transform_prev: transform_prev.into(),
                transform_inverted: transform
                    .try_inverse()
                    .unwrap()
                    .into(),
                transform_prev_inverted: transform_prev
                    .try_inverse()
                    .unwrap()
                    .into(),
                projection: projection_matrix.into(),
                projection_inverted: projection_matrix
                    .try_inverse()
                    .unwrap()
                    .into(),
            }
        }).collect::<Vec<_>>()
    }

    fn ty(&self) -> LightType {
        LightType::Point
    }

    fn uniform_struct(&self, shadowmap_index: i32) -> LightShaderStruct {
        LightShaderStruct::Point(PointLightUniform {
            base : self.base.uniform_struct_base(Mat4::identity(), shadowmap_index)
        })
    }

}

fast_impl_ssu!{
    #[derive(Default)]
    layout(std140) struct PointLightUniform {
        base: GenericLightUniform
    }
}

impl PointLightUniform {
    pub fn base_mut(&mut self) -> &mut GenericLightUniform
    {
        &mut self.base
    }
}