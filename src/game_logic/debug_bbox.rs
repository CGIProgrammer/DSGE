use super::events::*;
use crate::references::*;
use crate::types::Vec4;
use crate::{game_object::GameObjectRef, types::Vec3};

pub struct DisplayOnBboxCorners {
    marker_objects: [GameObjectRef; 8],
}

impl DisplayOnBboxCorners {
    pub fn new(marker: GameObjectRef) -> Self {
        let mut original = marker.lock();
        original.set_static(false);
        let marker_objects = [
            original.fork(),
            original.fork(),
            original.fork(),
            original.fork(),
            original.fork(),
            original.fork(),
            original.fork(),
            original.fork(),
        ];
        /*let mut column = original.transform_mut().local.column_mut(3);
        column[0] = 0.0;
        column[1] = 0.0;
        column[2] = 0.0;*/
        Self {
            marker_objects: marker_objects,
        }
    }

    fn move_markers_to_corners(&mut self, owner: &GameObjectRef, _event: AbstractEvent) {
        let model = owner.lock().transform().local;
        let mvp = model; //projection * view;
        let obj = owner.lock();
        let corners = match (obj.visual(), obj.light()) {
            (Some(vis), _) => vis.bbox_corners(),
            (None, Some(li)) => li.lock().unwrap().bbox_corners(),
            _ => [Vec3::new(0.0, 0.0, 0.0); 8],
        };
        for (corner, marker) in corners.iter().zip(&self.marker_objects) {
            let corner = mvp * Vec4::new(corner.x, corner.y, corner.z, 1.0);
            let mut m = marker.lock();
            m.transform_mut().unwrap().local = model;
            let mut column = m.transform_mut().unwrap().local.column_mut(3);
            column[0] = corner.x;
            column[1] = corner.y;
            column[2] = corner.z;
        }
    }
}

mod behaviour {
  use crate::game_logic::*;
  use std::any::Any;
  use crate::game_logic::events::EventHandlerBoxed;
  use super::*;
  impl Behaviour for DisplayOnBboxCorners {
    fn as_mut_any(&mut self) ->  &mut dyn Any {
      self
    }
    fn as_any(&self) ->  &dyn Any {
      self
    }
    fn event_handlers(&self) -> Vec<(EventType,EventHandlerBoxed)>{
      vec![(EventType::FrameTick,RcBox::construct(|owner: &GameObjectRef,component: &mut dyn Behaviour,event:AbstractEvent|{
        component.as_mut_any().downcast_mut::<Self>().unwrap().move_markers_to_corners(owner,event);
      }))]
    }
  
    }

  }pub use behaviour::*;
