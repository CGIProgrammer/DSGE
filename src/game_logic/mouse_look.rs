use crate::{game_object::GameObjectRef};
use crate::references::*;
use winit::event::VirtualKeyCode;
use super::events::*;
use AbstractEvent::*;
use crate::types::{Transform3, Vec4};
use nalgebra::Rotation3;

pub struct MouseLook
{
    look_x : f32,
    look_y : f32,
    fwd_key: f32,
    bwd_key: f32,
    left_key: f32,
    right_key: f32,

    down_key: f32,
    up_key: f32,

    accel: f32,

    sens: f32,
    dt: f32
}

impl MouseLook
{
    pub fn new(sensitivity: f32) -> Self
    {
        Self {
            look_x: 0.0,
            look_y: std::f32::consts::FRAC_PI_2 * 0.75,
            fwd_key: 0.0,
            bwd_key: 0.0,
            left_key: 0.0,
            right_key: 0.0,
            up_key: 0.0,
            down_key: 0.0,
            accel: 1.0,
            dt: 0.0,
            sens: sensitivity
        }
    }

    pub fn look(&mut self, _owner: &GameObjectRef, event: AbstractEvent)
    {
        let (dx, dy) = match event {
            MouseMove(MouseMoveEvent{dx, dy, ..}) => (dx as f32, dy as f32),
            _ => return
        };
        self.look_y -= dy * self.sens;
        self.look_x -= dx * self.sens;
        if self.look_y < 0.0 {
            self.look_y = 0.0;
        }
        if self.look_y > 3.1415926535 {
            self.look_y = 3.1415926535;
        }
    }
    
    pub fn motion(&mut self, _owner: &GameObjectRef, event: AbstractEvent)
    {
        if let AbstractEvent::FrameTick(tick) = event {
            self.dt = tick.delta;
        };

        let mut obj = _owner.lock();
        let local = obj.transform().local.clone();
        
        let position = local.column(3).to_owned();
        let back = local.column(2).to_owned();
        let front = -back;
        let right = local.column(0).to_owned();
        let left = -right;
        let down: Vec4 = [0.0, 0.0, -1.0, 0.0].into();
        let up: Vec4 = [0.0, 0.0, 1.0, 0.0].into();
        
        let direction = 
            self.fwd_key * front + 
            self.bwd_key * back + 
            self.left_key * left + 
            self.right_key * right + 
            self.down_key * down +
            self.up_key * up;
            
        let direction_magnitude = if direction.magnitude() == 0.0 { 1.0 } else { direction.magnitude() };
        let delta = self.dt * self.accel * direction / direction_magnitude;
        
        obj.transform_mut().local.set_rotation(&Rotation3::from_euler_angles(self.look_y, 0.0, self.look_x));
        obj.transform_mut().local.set_column(3, &(position + delta));
    }

    pub fn keys(&mut self, _owner: &GameObjectRef, event: AbstractEvent)
    {
        match event {
            AbstractEvent::Keyboard(KeyboardEvent { key_id:VirtualKeyCode::W, state }) => {
                self.fwd_key   = match state {1 => 1.0, _ => 0.0};
            },
            AbstractEvent::Keyboard(KeyboardEvent { key_id:VirtualKeyCode::A, state }) => {
                self.left_key  = match state {1 => 1.0, _ => 0.0};
            },
            AbstractEvent::Keyboard(KeyboardEvent { key_id:VirtualKeyCode::S, state }) => {
                self.bwd_key   = match state {1 => 1.0, _ => 0.0};
            },
            AbstractEvent::Keyboard(KeyboardEvent { key_id:VirtualKeyCode::D, state }) => {
                self.right_key = match state {1 => 1.0, _ => 0.0};
            },
            AbstractEvent::Keyboard(KeyboardEvent { key_id:VirtualKeyCode::Q, state }) => {
                self.down_key  = match state {1 => 1.0, _ => 0.0};
            },
            AbstractEvent::Keyboard(KeyboardEvent { key_id:VirtualKeyCode::E, state }) => {
                self.up_key    = match state {1 => 1.0, _ => 0.0};
            },
            AbstractEvent::Keyboard(KeyboardEvent { 
                key_id : VirtualKeyCode::LShift | VirtualKeyCode::RShift,
                state
            }) => {
                self.accel = match state {1 => 20.0, _ => 1.0};
            },
            _ => { }
        }
    }
}

crate::impl_behaviour!(
    MouseLook {
        look: MouseMove,
        motion: FrameTick,
        keys: Keyboard
    }
);