use std::collections::HashMap;

pub use crate::components::camera::*;
use crate::components::light::PointLight;
pub use crate::components::visual::*;
use crate::components::*;
use crate::game_logic::behaviour::DynBehaviour;
use crate::game_logic::Behaviour;

use crate::references::*;
use crate::scene::SceneRef;
use crate::shader::ShaderStructUniform;
use crate::types::*;
use bytemuck::{Pod, Zeroable};
use vulkano::pipeline::graphics::vertex_input::Vertex;

pub type GameObjectRef = RcBox<GameObject>;
pub type Transform = Mat4;

#[derive(Clone)]
pub enum GOParent {
    Object(GameObjectRef),
    Scene(SceneRef),
    None,
}

impl GOParent {
    pub fn is_some(&self) -> bool {
        match self {
            Self::Object(_) | Self::Scene(_) => true,
            Self::None => false,
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct GOTransform {
    pub local: Transform,
    pub global: Transform,
    pub global_prev: Transform,
    pub(crate) _owner: Option<GameObjectRef>,
    pub(crate) _parent: GOParent,
    pub(crate) _children: HashMap<i32, GameObjectRef>,
    pub(crate) _is_static: bool,
}

crate::fast_impl_ssu! {
    #[derive(Default, Zeroable, Pod, Vertex)]
    struct GOTransformUniform as GOTransform
    {
        #[format(R32G32B32A32_SFLOAT)]
        transform: [f32; 16],
        #[format(R32G32B32A32_SFLOAT)]
        transform_prev: [f32; 16],
        /*#[format(R32G32B32A32_SFLOAT)]
        transform_0: [f32; 4],
        #[format(R32G32B32A32_SFLOAT)]
        transform_1: [f32; 4],
        #[format(R32G32B32A32_SFLOAT)]
        transform_2: [f32; 4],
        #[format(R32G32B32A32_SFLOAT)]
        transform_3: [f32; 4],

        #[format(R32G32B32A32_SFLOAT)]
        transform_0_prev: [f32; 4],
        #[format(R32G32B32A32_SFLOAT)]
        transform_1_prev: [f32; 4],
        #[format(R32G32B32A32_SFLOAT)]
        transform_2_prev: [f32; 4],
        #[format(R32G32B32A32_SFLOAT)]
        transform_3_prev: [f32; 4],*/
    }
}

impl GOTransformUniform
{
    /*pub fn transform(&self) -> [f32; 16]
    {
        [
            self.transform_0[0], self.transform_0[1], self.transform_0[2], self.transform_0[3],
            self.transform_1[0], self.transform_1[1], self.transform_1[2], self.transform_1[3],
            self.transform_2[0], self.transform_2[1], self.transform_2[2], self.transform_2[3],
            self.transform_3[0], self.transform_3[1], self.transform_3[2], self.transform_3[3],
        ]
    }

    pub fn transform_prev(&self) -> [f32; 16]
    {
        [
            self.transform_0_prev[0], self.transform_0_prev[1], self.transform_0_prev[2], self.transform_0_prev[3],
            self.transform_1_prev[0], self.transform_1_prev[1], self.transform_1_prev[2], self.transform_1_prev[3],
            self.transform_2_prev[0], self.transform_2_prev[1], self.transform_2_prev[2], self.transform_2_prev[3],
            self.transform_3_prev[0], self.transform_3_prev[1], self.transform_3_prev[2], self.transform_3_prev[3],
        ]
    }*/
}

impl From<Mat4> for GOTransform {
    fn from(value: Mat4) -> Self {
        Self {
            local: value,
            global: value,
            global_prev: value,
            _owner: None,
            _parent: GOParent::None,
            _children: HashMap::new(),
            _is_static: true,
        }
    }
}

#[allow(dead_code)]
impl GOTransform {
    fn set_owner(&mut self, owner: RcBox<GameObject>) {
        self._owner = Some(owner);
    }

    pub fn is_static(&self) -> bool {
        self._is_static
    }

    pub fn identity() -> Self {
        Self {
            local: Transform::identity(),
            global: Transform::identity(),
            global_prev: Transform::identity(),
            _parent: GOParent::None,
            _children: HashMap::new(),
            _is_static: true,
            _owner: None,
        }
    }

    pub fn uniform_value(&self) -> GOTransformUniform {
        let transform: [f32; 16] = self.global.as_slice().try_into().unwrap();
        let transform_prev: [f32; 16] = self.global_prev.as_slice().try_into().unwrap();
        GOTransformUniform {
            transform, transform_prev
        }
        /*GOTransformUniform {
            transform_0: transform[0..4].try_into().unwrap(),
            transform_1: transform[4..8].try_into().unwrap(),
            transform_2: transform[8..12].try_into().unwrap(),
            transform_3: transform[12..16].try_into().unwrap(),
            
            transform_0_prev: transform_prev[0..4].try_into().unwrap(),
            transform_1_prev: transform_prev[4..8].try_into().unwrap(),
            transform_2_prev: transform_prev[8..12].try_into().unwrap(),
            transform_3_prev: transform_prev[12..16].try_into().unwrap(),
        }*/
    }
}

//#[derive(Clone)]
pub struct GameObject {
    pub(crate) scene: Option<SceneRef>,
    pub(crate) transform: GOTransform,
    name: String,
    camera: Option<CameraComponent>,
    mesh_visual: Option<MeshVisual>,
    light: Option<Light>,
    components: Vec<DynBehaviour>,
    //scene: Option<SceneRef>
}

impl Drop for GameObject {
    fn drop(&mut self) {
        println!("GameObject dropped");
    }
}

//#[allow(dead_code)]
impl GameObject {
    /*pub fn set_scene(&mut self, scene: SceneRef)
    {
        self.scene = Some(scene);
    }*/

    #[inline]
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn camera(&self) -> Option<&CameraComponent> {
        self.camera.as_ref()
    }

    pub fn camera_mut(&mut self) -> Option<&mut CameraComponent> {
        self.camera.as_mut()
    }

    pub fn visual(&self) -> Option<&MeshVisual> {
        self.mesh_visual.as_ref()
    }

    pub fn light(&self) -> Option<&Light> {
        if let Some(ref light) = self.light {
            light.lock().unwrap().update_transform(&self.transform);
            Some(light)
        } else {
            None
        }
    }

    pub fn new<T: ToString>(name: T) -> RcBox<Self> {
        let obj = RcBox::construct(Self {
            name: name.to_string(),
            transform: GOTransform::identity(),
            camera: None,
            mesh_visual: None,
            light: None,
            components: Vec::new(),
            scene: None,
        });
        obj.lock_write().transform._owner = Some(obj.clone());
        obj
    }

    pub fn parent_object(&self) -> Option<&GameObjectRef> {
        if let GOParent::Object(ref parent) = self.transform._parent {
            Some(parent)
        } else {
            None
        }
    }

    pub fn is_root(&self) -> bool {
        if let GOParent::Scene(ref _s) = self.transform._parent {
            return true;
        } else {
            return false;
        }
    }

    pub fn set_parent(&mut self, parent: RcBox<GameObject>) {
        self.remove_parent();
        let mut _par = GOParent::Object(parent.clone());
        let parent_id = parent.box_id();
        loop {
            match _par {
                GOParent::Object(ref par) => {
                    if parent_id == par.box_id() {
                        panic!("Обнаружена циклическая зависимость объектов.");
                    }
                }
                _ => {
                    break;
                }
            }
        }
        self.transform._parent = GOParent::Object(parent.clone());
        let owner = self.transform._owner.clone().unwrap();
        parent
            .lock()
            .transform
            ._children
            .insert(owner.box_id(), owner);
    }

    pub fn add_child(&mut self, child: RcBox<GameObject>) {
        child
            .lock()
            .set_parent(self.transform._owner.clone().unwrap());
    }

    /// Добавляет компонент и возвращает RcBox с этим компонентом
    pub fn add_component<T: Sized + Behaviour>(&mut self, component: T) -> Option<RcBox<T>> {
        /*fn downcast_copy<E, F>(component: F) -> Option<E> 
        where E: Sized + 'static, F: Sized + 'static
        {
            if let Some(_) = ((&component) as &dyn std::any::Any).downcast_ref::<E>() {
                Some(unsafe{std::mem::transmute_copy(&component)})
                /*let mut _cbb = MaybeUninit::<E>::uninit();
                
                Some(unsafe {
                    std::ptr::copy(
                        &component as *const F as *const E,
                        &mut _cbb as *mut MaybeUninit<E> as *mut E,
                        1,
                    );
                    _cbb.assume_init()
                })*/
            } else {
                None
            }
        }*/
        
        if (&component as &dyn std::any::Any).is::<CameraComponent>() {
            self.camera = Some((&component as &dyn std::any::Any).downcast_ref::<CameraComponent>().unwrap().clone());
            //self.camera = downcast_copy(component);
            return None;
        }
        if (&component as &dyn std::any::Any).is::<MeshVisual>() {
            self.mesh_visual = Some((&component as &dyn std::any::Any).downcast_ref::<MeshVisual>().unwrap().clone());
            return None;
        }
        if (&component as &dyn std::any::Any).is::<Spotlight>() {
            let light: Spotlight = (&component as &dyn std::any::Any).downcast_ref::<Spotlight>().unwrap().clone(); //downcast_copy(component).unwrap();
            self.light = Some(RcBox::construct(light));
            return None;
        }
        if (&component as &dyn std::any::Any).is::<SunLight>() {
            let light: SunLight  = (&component as &dyn std::any::Any).downcast_ref::<SunLight>().unwrap().clone(); //downcast_copy(component).unwrap();
            self.light = Some(RcBox::construct(light));
            return None;
        }
        if (&component as &dyn std::any::Any).is::<PointLight>() {
            let light: PointLight  = (&component as &dyn std::any::Any).downcast_ref::<PointLight>().unwrap().clone(); //downcast_copy(component).unwrap();
            self.light = Some(RcBox::construct(light));
            return None;
        }
        /*if (&component as &dyn std::any::Any).is::<SpotLight>() {
            let light: SunLight = downcast_copy(component).unwrap();
            self.light = Some(RcBox::construct(light));
            return None;
        }*/
        let result = RcBox::construct(component);
        self.components.push(result.clone());
        if let Some(ref scene) = self.scene {
            scene.lock().event_processor.update_object(self);
        };
        Some(result)
    }

    pub fn get_component<T: Behaviour>(&self) -> Option<&DynBehaviour> {
        self.components
            .iter()
            .find(|b| (*b).lock().unwrap().as_any().downcast_ref::<T>().is_some())
    }

    pub fn get_components<T: Behaviour>(&self) -> Vec<&DynBehaviour> {
        self.components
            .iter()
            .filter(|b| (*b).lock().unwrap().as_any().downcast_ref::<T>().is_some())
            .collect::<Vec<_>>()
    }

    pub fn get_all_components(&self) -> &Vec<DynBehaviour> {
        &self.components
    }

    pub fn remove_parent(&mut self) {
        if self.transform._parent.is_none() {
            return;
        }
        match self.transform._parent {
            GOParent::Scene(ref scene) => {
                scene
                    .lock_write()
                    .root_objects
                    .remove(&(self as *const Self as usize as i32));
            }
            GOParent::Object(ref object) => {
                object
                    .lock_write()
                    .transform
                    ._children
                    .remove(&(self as *const Self as usize as i32));
            }
            GOParent::None => (),
        }
        match self.scene {
            Some(ref scene) => self.transform._parent = GOParent::Scene(scene.clone()),
            None => self.transform._parent = GOParent::None,
        };
    }

    pub fn children(&self) -> Vec<RcBox<GameObject>> {
        self.transform
            ._children
            .iter()
            .map(|(_, v)| v.clone())
            .collect()
    }

    pub fn transform(&self) -> &GOTransform {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> Option<&mut GOTransform> {
        if !self.transform._is_static {
            Some(&mut self.transform)
        } else {
            None
        }
    }

    pub fn is_static(&self) -> bool {
        self.transform._is_static
    }

    pub fn set_static(&mut self, is_static: bool) {
        self.transform._is_static = is_static;
    }

    pub(crate) fn step(&mut self) {
        self.transform.global_prev = self.transform.global;
        match &self.transform._parent {
            GOParent::Object(parent) => {
                let par = parent.lock();
                self.transform.global = par.transform().global * self.transform.local;
            }
            _ => {
                self.transform.global = self.transform.local;
            }
        };
        fn apply_transform_closure(obj: &mut GameObject) {
            for child in obj.children() {
                let mut ch = child.lock();
                /*{
                    let ch_transform = ch.transform_mut();
                    ch_transform.global = obj.transform().global * ch_transform.local;
                }*/
                apply_transform_closure(&mut *ch);
            }
        }
        apply_transform_closure(self);
    }

    pub fn next_frame(&mut self) {
        match &self.transform._parent {
            GOParent::Object(parent) => {
                let par = parent.lock();
                self.transform.global = par.transform().global * self.transform.local;
            }
            _ => {
                self.transform.global = self.transform.local;
            }
        };
        fn next_frame_closure(obj: &mut GameObject) {
            if let Some(mut transform) = obj.transform_mut() {
                transform.global_prev = transform.global;
                transform.global = transform.global;
                for child in obj.children() {
                    let mut ch = child.lock();
                    if let Some(ch_transform) = ch.transform_mut() {
                        ch_transform.global = obj.transform().global * ch_transform.local;
                    }
                    next_frame_closure(&mut *ch);
                }
            }
        }
        next_frame_closure(self);
    }

    pub fn fork(&self) -> RcBox<GameObject> {
        let fork = Self::new(format!("Fork of {}", self.name()));
        let mut _fork = fork.lock();
        _fork.mesh_visual = self.mesh_visual.clone();
        _fork.set_static(self.is_static());
        drop(_fork);
        if let Some(ref scene) = self.scene {
            scene.lock().add_object(fork.clone()).unwrap();
        }
        return fork;
    }

    #[inline(always)]
    pub fn project_bounding_box_corners(&self, model_view_projection: Mat4) -> Option<[Vec3; 8]> {
        match self.mesh_visual {
            Some(ref mesh_visual) => {
                let corners = mesh_visual.bbox_corners();
                Some(
                    corners
                        .iter()
                        .map(|corner| {
                            let v = model_view_projection
                                * Vec4::new(corner.x, corner.y, corner.z, 1.0);
                            v.xyz() / v.w
                        })
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                )
            }
            None => None,
        }
    }
}
