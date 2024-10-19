use super::{events::EventHandlerBoxed, AbstractEvent};
use crate::game_logic::events::EventType;
use crate::{game_object::GameObjectRef, references::RcBox, utils::RefId};
use std::any::Any;

pub type DynBehaviour = RcBox<dyn Behaviour>;

/// Описывает поведение объекта
pub trait Behaviour: Send + Sync + 'static {
    /// Выполняется при удалении объекта со сцены
    fn on_unlink(&mut self, _owner: GameObjectRef, _event: AbstractEvent) {}
    fn event_handlers(&self) -> Vec<(EventType, EventHandlerBoxed)> //Vec<(EventType, Box<dyn FnMut(&GameObjectRef, &mut dyn Behaviour, AbstractEvent)>)>
    {
        vec![]
    }

    /// Должно возвращать ссылку на самого себя в динамическом типе
    fn as_any(&self) -> &dyn Any;

    /// Должно возвращать ссылку на самого себя в динамическом типе
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl RefId for dyn Behaviour {}

#[macro_export]
macro_rules! impl_behaviour {
    ($ty: ty {
        $($handler: ident: $handler_type: tt),*
    }) => {
        mod behaviour {
            use crate::game_logic::*;
            use std::any::Any;
            use crate::game_logic::events::EventHandlerBoxed;
            use super::*;
            impl Behaviour for $ty {
                fn as_mut_any(&mut self) -> &mut dyn Any
                {
                    self
                }

                fn as_any(&self) -> &dyn Any
                {
                    self
                }

                fn event_handlers(&self) -> Vec<(EventType, EventHandlerBoxed)> // Vec<(EventType, Box<dyn FnMut(&GameObjectRef, &mut dyn Behaviour, AbstractEvent)>)>
                {
                    vec![$(
                        (
                            EventType::$handler_type,
                            RcBox::construct(|owner: &GameObjectRef, component: &mut dyn Behaviour, event: AbstractEvent | {
                                component.as_mut_any().downcast_mut::<Self>().unwrap().$handler(owner, event);
                            })
                        )
                    ),*]
                }
            }
        }
        pub use behaviour::*;
    };
}
