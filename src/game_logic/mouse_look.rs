use super::events::*;
use crate::game_object::GameObjectRef;
use crate::references::*;
use crate::types::{Transform3, Vec4};
use nalgebra::Rotation3;
use winit::event::VirtualKeyCode;
use AbstractEvent::*;

pub struct MouseLook {
    vnc_mode: bool,

    look_x: f32,
    look_y: f32,

    look_x_inert: f32,
    look_y_inert: f32,

    ddx: f32,
    ddy: f32,

    fwd_key: f32,
    bwd_key: f32,
    left_key: f32,
    right_key: f32,

    down_key: f32,
    up_key: f32,

    accel: f32,

    sens: f32,
    dt: f32,
}

impl MouseLook {
    pub fn new(sensitivity: f32, vnc_mode: bool) -> Self {
        Self {
            vnc_mode: vnc_mode,
            look_x: 0.0,
            look_y: std::f32::consts::FRAC_PI_2 * 0.75,
            look_x_inert: 0.0,
            look_y_inert: 0.0,
            ddx: 0.0,
            ddy: 0.0,
            fwd_key: 0.0,
            bwd_key: 0.0,
            left_key: 0.0,
            right_key: 0.0,
            up_key: 0.0,
            down_key: 0.0,
            accel: 1.0,
            dt: 0.0,
            sens: sensitivity,
        }
    }

    pub fn look(&mut self, _owner: &GameObjectRef, event: AbstractEvent) {
        let (dx, dy) = match event {
            MouseMove(MouseMoveEvent { dx, dy, .. }) => (dx as f32, dy as f32),
            _ => return,
        };
        if self.vnc_mode {
            self.look_y_inert -= (dy - self.ddy) * self.sens;
            self.look_x_inert -= (dx - self.ddx) * self.sens;
        } else {
            self.look_y_inert -= dy * self.sens;
            self.look_x_inert -= dx * self.sens;
        }
        self.ddx = dx;
        self.ddy = dy;
    }

    pub fn motion(&mut self, _owner: &GameObjectRef, event: AbstractEvent) {
        if let AbstractEvent::FrameTick(tick) = event {
            self.dt = tick.delta();
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

        self.look_y += self.look_y_inert * (60.0 * self.dt);
        self.look_x += self.look_x_inert * (60.0 * self.dt);

        self.look_x_inert *= 0.5f32.powf(self.dt * 60.0);
        self.look_y_inert *= 0.5f32.powf(self.dt * 60.0);

        if self.look_y < 0.0 {
            self.look_y = 0.0;
        }
        if self.look_y > 3.1415926535 {
            self.look_y = 3.1415926535;
        }

        let direction = self.fwd_key * front
            + self.bwd_key * back
            + self.left_key * left
            + self.right_key * right
            + self.down_key * down
            + self.up_key * up;

        let direction_magnitude = if direction.magnitude() == 0.0 {
            1.0
        } else {
            direction.magnitude()
        };
        let delta = self.dt * self.accel * direction / direction_magnitude;

        if let Some(transform) = obj.transform_mut() {
            transform.local.set_rotation(&Rotation3::from_euler_angles(
                self.look_y,
                0.0,
                self.look_x,
            ));
            transform.local.set_column(3, &(position + delta));
        }
    }

    pub fn keys(&mut self, _owner: &GameObjectRef, event: AbstractEvent) {
        match event {
            AbstractEvent::Keyboard(event) => match event {
                KeyboardEvent {
                    key_id: VirtualKeyCode::Up,
                    state: _,
                } => {
                    self.look_y_inert += 0.1;
                }
                KeyboardEvent {
                    key_id: VirtualKeyCode::Down,
                    state: _,
                } => {
                    self.look_y_inert -= 0.1;
                }
                KeyboardEvent {
                    key_id: VirtualKeyCode::Left,
                    state: _,
                } => {
                    self.look_x_inert += 0.1;
                }
                KeyboardEvent {
                    key_id: VirtualKeyCode::Right,
                    state: _,
                } => {
                    self.look_x_inert -= 0.1;
                }
                KeyboardEvent {
                    key_id: VirtualKeyCode::W,
                    state,
                } => {
                    self.fwd_key = match state {
                        1 => 1.0,
                        _ => 0.0,
                    };
                }
                KeyboardEvent {
                    key_id: VirtualKeyCode::A,
                    state,
                } => {
                    self.left_key = match state {
                        1 => 1.0,
                        _ => 0.0,
                    };
                }
                KeyboardEvent {
                    key_id: VirtualKeyCode::S,
                    state,
                } => {
                    self.bwd_key = match state {
                        1 => 1.0,
                        _ => 0.0,
                    };
                }
                KeyboardEvent {
                    key_id: VirtualKeyCode::D,
                    state,
                } => {
                    self.right_key = match state {
                        1 => 1.0,
                        _ => 0.0,
                    };
                }
                KeyboardEvent {
                    key_id: VirtualKeyCode::Q,
                    state,
                } => {
                    self.down_key = match state {
                        1 => 1.0,
                        _ => 0.0,
                    };
                }
                KeyboardEvent {
                    key_id: VirtualKeyCode::E,
                    state,
                } => {
                    self.up_key = match state {
                        1 => 1.0,
                        _ => 0.0,
                    };
                }
                KeyboardEvent {
                    key_id: VirtualKeyCode::LShift | VirtualKeyCode::RShift,
                    state,
                } => {
                    self.accel = match state {
                        1 => 20.0,
                        _ => 1.0,
                    };
                }
                _ => {}
            },
            _ => {}
        }
    }
}

crate::impl_behaviour!(MouseLook {
    look: MouseMove,
    motion: FrameTick,
    keys: Keyboard
});
