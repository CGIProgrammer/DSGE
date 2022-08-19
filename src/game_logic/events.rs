use std::collections::HashMap;

use winit::event::VirtualKeyCode;
pub use winit::event::{Event, WindowEvent, DeviceEvent, KeyboardInput};
use crate::game_object::{GameObjectRef, GameObject};
use crate::references::*;
use crate::game_logic::Behaviour;
pub use crate::time::UniformTime as FrameTick;
use crate::time::{UniformTime, Timer};
use crate::utils::RefId;

use super::behaviour::DynBehaviour;

pub(crate) type EventHandlerBoxed = RcBox<dyn FnMut(&GameObjectRef, &mut dyn Behaviour, AbstractEvent) + Sync + Send>;
type EventsByComponentId = HashMap<i32, EventHandler>;
type EventsByObjectId = HashMap<i32, EventsByComponentId>;
type EventsByEventTypeId = HashMap<u32, EventsByObjectId>;


impl Default for KeyboardEvent
{
    fn default() -> Self {
        Self {
            key_id: VirtualKeyCode::Escape,
            state: 0
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct MouseMoveEvent
{
    pub dx: i32,
    pub dy: i32,
}
#[derive(Clone, Copy, Default)]
pub struct MouseClickEvent
{
    pub lmb: i32,
    pub rmb: i32,
    pub mmb: i32,
    pub wheel: (i32, i32)
}

#[derive(Clone, Copy)]
pub struct KeyboardEvent
{
    pub key_id: VirtualKeyCode,
    pub state: i32
}

macro_rules! events_enums {
    {$($variant: ident ($variant_type: tt)),*} => {
        #[derive(Clone, Copy)]
        pub enum AbstractEvent
        {
            None,
            $($variant($variant_type)),*
        }

        impl AbstractEvent
        {
            pub fn event_type(&self) -> EventType
            {
                match self {
                    Self::None => EventType::None,
                    $(Self::$variant(_) => EventType::$variant),*
                }
            }

            pub fn variant_id(&self) -> u32
            {
                self.event_type() as _
            }
        }

        #[derive(Clone, Copy)]
        pub enum EventType
        {
            None = 0,
            $($variant),*
        }
    };
}

events_enums!{
    MouseMove(MouseMoveEvent),
    MouseClick(MouseClickEvent),
    Keyboard(KeyboardEvent),
    FrameTick(FrameTick),
    InitialTick(FrameTick)
}

#[derive(Default, Clone)]
pub struct EventProcessor
{
    event_handlers: RcBox<EventsByEventTypeId>,
    event_stack: RcBox<Vec<AbstractEvent>>,
    pub(crate) timer: Timer,
    pub(crate) time: RcBox<UniformTime>
}

impl EventProcessor
{
    pub(crate) fn update_object(&mut self, obj: &GameObject)
    {
        let components = obj.get_all_components().clone();
        let self_event_handlers = &mut *self.event_handlers.lock_write();
        let obj_id = obj as *const GameObject as i32;
        for cmp in components {
            let component = &*cmp.lock().unwrap();
            let component_id = component.box_id() as i32;
            for (event_handler_type, event_handler) in component.event_handlers() {
                let eht = event_handler_type as u32;
                if !self_event_handlers.contains_key(&eht) {
                    self_event_handlers.insert(eht, HashMap::new());
                }
                let by_event_type = self_event_handlers.get_mut(&eht).unwrap();
                if !by_event_type.contains_key(&obj_id) {
                    by_event_type.insert(obj_id, HashMap::new());
                }
                let by_object_id = by_event_type.get_mut(&obj_id).unwrap();
                if !by_object_id.contains_key(&component_id) {
                    let eh = EventHandler {
                        event_handler: event_handler.clone(),
                        component: cmp.clone(),
                        owner: obj.transform._owner.clone().unwrap()
                    };
                    by_object_id.insert(component_id, eh);
                    self.send_event(AbstractEvent::InitialTick(*self.time.lock()));
                }
            }
        }
    }

    pub(crate) fn remove_object(&mut self, obj: GameObjectRef)
    {
        let components = obj.lock().get_all_components().clone();
        for component in components
        {
            let mut compon = component.lock().unwrap();
            compon.on_unlink(obj.clone(), AbstractEvent::FrameTick(*self.time.lock()));
        }
        for (_eh_id, obj_list) in &mut *self.event_handlers.lock()
        {
            obj_list.remove(&obj.box_id());
        }
    }

    pub fn send_event(&self, event: AbstractEvent)
    {
        self.event_stack.lock_write().push(event);
    }

    pub fn step(&mut self)
    {
        *self.time.lock() = self.timer.next_frame();
        self.send_event(AbstractEvent::FrameTick(*self.time.lock()));
    }

    pub fn execute(&self)
    {
        loop {
            let mut event_stack = self.event_stack.lock();
            if event_stack.len() == 0 {
                break;
            }
            let event = event_stack.remove(0);
            if let Some(event_handlers) = self.event_handlers.lock().get_mut(&event.variant_id())
            {
                for (_, handlers_by_obj_id) in event_handlers
                {
                    for (_, event_handler) in handlers_by_obj_id {
                        event_handler.call(event);
                    }
                }
            }
        };
    }
}

#[derive(Clone)]
struct EventHandler
{
    event_handler: EventHandlerBoxed,
    component: DynBehaviour,
    owner: GameObjectRef,
}

#[allow(dead_code)]
impl EventHandler
{
    pub fn new(owner: GameObjectRef, component: DynBehaviour, event_handler: EventHandlerBoxed) -> Self
    {
        Self {
            event_handler: event_handler,
            component: component,
            owner,
        }
    }

    fn call(&mut self, event: AbstractEvent)
    {
        (self.event_handler.lock().unwrap())(&self.owner, &mut *self.component.lock().unwrap(), event)
    }
}