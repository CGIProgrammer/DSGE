/// Компоненты для `GameObject`
/// Пока в зачаточном состоянии
pub mod camera;
pub mod light;
pub mod visual;

use crate::shader::ShaderStructUniform;
use crate::types::Mat4;

pub use crate::game_object::{GOTransformUniform, GameObject, GameObjectRef};
pub use camera::CameraComponent;
pub use light::{Spotlight, SunLight, Light};
pub use visual::{AbstractVisual, MeshVisual};

// Структура для передачи данных шейдерной программе
crate::fast_impl_ssu! {
    #[repr(align(16))]
    struct ProjectionUniformData as Camera
    {
        transform : std140::mat4,
        transform_prev : std140::mat4,
        transform_inverted : std140::mat4,
        transform_prev_inverted : std140::mat4,
        projection : std140::mat4,
        projection_inverted : std140::mat4
    }
}

impl ProjectionUniformData {
    pub fn full_matrix(&self) -> Mat4 {
        let projection: Mat4 = self.projection.into();
        let transform: Mat4 = self.transform.into();
        projection * transform.try_inverse().unwrap()
    }

    pub fn full_matrix_inverted(&self) -> Mat4 {
        self.full_matrix().try_inverse().unwrap()
    }
}

impl Default for ProjectionUniformData {
    fn default() -> Self {
        let proj = nalgebra::Perspective3::new(1.0, 80.0 * 3.1415926535 / 180.0, 0.1, 100.0)
            .as_matrix()
            .clone();
        let identity: std140::mat4 = Mat4::identity().into();
        Self {
            transform: identity,
            transform_prev: identity,
            transform_inverted: identity,
            transform_prev_inverted: identity,
            projection: proj.into(),
            projection_inverted: proj.try_inverse().unwrap().into(),
        }
    }
}
