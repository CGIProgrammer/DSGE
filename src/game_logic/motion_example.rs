use super::AbstractEvent;
use crate::game_object::GameObjectRef;
use crate::references::*;
use crate::types::{Mat4, Transform3};
use nalgebra::Rotation3;

#[derive(Default)]
pub struct Spinning;

impl Spinning {
    fn loop_clbk(&mut self, obj: &GameObjectRef, event: AbstractEvent) {
        let mut obj = obj.lock();
        if let Some(transform) = obj.transform_mut() {
            let dtime = if let AbstractEvent::FrameTick(frame_tick) = event {
                frame_tick.delta()
            } else {
                return;
            };
            let rotation = Rotation3::<f32>::from_euler_angles(0.0, 0.0, 2.0 * Into::<f32>::into(dtime));
            transform.local.rotate(&rotation);
        }
    }
}

pub struct LinearMotion {
    initial_transform: Mat4,
}

impl LinearMotion {
    pub fn new() -> Self {
        Self {
            initial_transform: Mat4::identity(),
        }
    }

    fn init(&mut self, obj: &GameObjectRef, _event: AbstractEvent) {
        let obj = obj.lock();
        self.initial_transform = obj.transform().local;
    }

    fn loop_clbk(&mut self, obj: &GameObjectRef, event: AbstractEvent) {
        let mut obj = obj.lock();
        if let (Some(transform), AbstractEvent::FrameTick(time)) = (obj.transform_mut(), event) {
            transform.local[12] = self.initial_transform[12] + time.uptime().sin();
            transform.local[13] = self.initial_transform[13] + time.uptime().cos();
        }
    }
}

mod spinning {
    use super::*;
    crate::impl_behaviour!(Spinning {
        loop_clbk: FrameTick
    });
}

mod linear_motion {
    use super::*;
    crate::impl_behaviour!(LinearMotion {
        loop_clbk: FrameTick,
        init: InitialTick
    });
}
pub use linear_motion::*;
pub use spinning::*;
