pub mod camera;
pub mod visual;

use crate::game_object::GameObjectRef;

pub trait Component
{
    fn set_owner(&mut self, owner : GameObjectRef);
    fn as_visual(&self) -> Option<&visual::VisualComponent>;
}