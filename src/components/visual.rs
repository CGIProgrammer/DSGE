use crate::mesh::MeshRef;
use crate::material::{MaterialRef};
use crate::game_object::GameObjectRef;
use super::Component;

pub struct VisualComponent
{
    mesh : MeshRef,
    material : MaterialRef,
    owner : Option<GameObjectRef>
}

impl VisualComponent
{
    pub fn new(mesh: MeshRef, material: MaterialRef) -> Self
    {
        Self {
            mesh : mesh,
            material : material,
            owner : None
        }
    }
}

impl Component for VisualComponent
{
    fn set_owner(&mut self, go : GameObjectRef)
    {
        self.owner = Some(go);
    }

    fn as_visual(&self) -> Option<&Self>
    {
        Some(self)
    }
}