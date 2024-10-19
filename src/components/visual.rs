use std::collections::HashMap;

use crate::material::{MaterialRef, MaterialShaderProgramType};
use crate::mesh::{BoundingBox, MeshRef};
use crate::references::MutexLockBox;
use crate::types::Vec3;
use crate::utils::RefId;

pub use crate::components::ProjectionUniformData;

#[derive(Clone)]
pub struct MeshVisual {
    mesh: MeshRef,
    material: MaterialRef,
    shader_hashes: HashMap<MaterialShaderProgramType, u64>,
    cast_shadow: bool,
}

impl MeshVisual {
    pub fn new(mesh: MeshRef, material: MaterialRef, cast_shadow: bool) -> Self {
        let mat = material.lock();
        let shader_hashes = MaterialShaderProgramType::combinations()
            .iter()
            .map(|ty| (*ty, mat.shader_hash(ty)))
            .collect::<HashMap<_, _>>();
        Self {
            mesh: mesh.clone(),
            material: material.clone(),
            cast_shadow: cast_shadow,
            shader_hashes: shader_hashes,
        }
    }

    pub fn shader_hash(&self, sh_type: MaterialShaderProgramType) -> u64 {
        self.shader_hashes[&sh_type]
    }

    #[inline]
    pub fn mesh(&self) -> &MeshRef {
        &self.mesh
    }

    #[inline]
    pub fn material(&self) -> &MaterialRef {
        &self.material
    }

    #[inline]
    pub fn material_id(&self) -> i32 {
        self.material.box_id()
    }

    #[inline]
    pub fn bbox(&self) -> BoundingBox {
        self.mesh.bbox()
    }

    #[inline]
    pub fn bbox_corners(&self) -> [Vec3; 8] {
        self.mesh.bbox_corners()
    }

    #[inline]
    pub fn cast_shadow(&self) -> bool {
        self.cast_shadow
    }

    #[inline]
    pub fn set_shadow_casting(&mut self, cast: bool) {
        self.cast_shadow = cast;
    }
}

pub trait AbstractVisual: Sync + Send + 'static {
    /// Выполняется при рендеринге на стадии геометрии
    /*fn on_geometry_pass(
        &mut self,
        _transform: &GOTransformUniform,
        _camera_data: &ProjectionUniformData,
        _subpass: &Subpass,
        _acbb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        _new_mesh: bool,
        _new_material: bool
    ) -> Result<(), String>;*/

    /*/// Выполняется при рендеринге на стадии геометрии
    fn on_geometry_pass_secondary(
        &self,
        _transform: &GOTransformUniform,
        _camera_data: &ProjectionUniformData,
        _subpass: &Subpass,
        _acbb: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
        _new_mesh: bool,
        _new_material: bool,
    ) -> Result<(), String>;*/

    /*/// Выполняется при рендеринге на стадии карт теней
    fn on_geometry_pass(
        &self,
        camera_data: &ProjectionUniformData,
        instance_buffer: Arc<CpuAccessibleBuffer<[GOTransformUniform]>>,
        obj_index: u32,
        instance_count: u32,
        shader: &ShaderProgram,
        uniform_buffer: &mut ShaderProgramUniformBuffer,
        acbb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        new_mesh: bool,
        new_material: bool,
    ) -> Result<(), String>;*/

    /// Должно возвращать уникальный номер материала (необходимо для оптимизации)
    fn material_id(&self) -> i32;

    /// Должно возвращать уникальный номер полисетки (необходимо для оптимизации)
    fn mesh_id(&self) -> i32;

    fn mesh(&self) -> Option<MeshRef>;

    fn material(&self) -> Option<MaterialRef>;
}

mod behaviour {
  use crate::game_logic::*;
  use std::any::Any;
  use crate::game_logic::events::EventHandlerBoxed;
  use super::*;
  impl Behaviour for MeshVisual {
    fn as_mut_any(&mut self) ->  &mut dyn Any {
      self
    }
    fn as_any(&self) ->  &dyn Any {
      self
    }
    fn event_handlers(&self) -> Vec<(EventType,EventHandlerBoxed)>{
      vec![]
    }
  
    }

  }pub use behaviour::*;

impl RefId for MeshVisual {}
