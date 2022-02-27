/// Компоненты для `GameObject`
/// Пока в зачаточном состоянии

pub mod camera;
pub mod visual;

pub use crate::game_object::GameObject;
pub use visual::AbstractVisual;
pub use camera::AbstractCamera;

/// Абстрактная камера
pub trait AbstractCameraObject : GameObject + AbstractCamera {}

/// Абстрактный отображаемый объект
pub trait AbstractVisualObject : GameObject + AbstractVisual {}
