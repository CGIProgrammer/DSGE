//use super::texture::Texture;
use super::mesh::Mesh;
//use super::game_object::GameObject;
//use super::material::Material;
use super::shader::{ShaderProgram};
use std::sync::{Arc, Mutex, MutexGuard};

type ThreadSafeRcBox<T> = Arc<Mutex<T>>;

pub use std::borrow::Borrow;
pub type RcBox<T> = ThreadSafeRcBox<T>;

pub trait BoxId<T>
{
    fn box_id(&self) -> i32;
}

impl <T>BoxId<T> for ThreadSafeRcBox<T>
{
    fn box_id(&self) -> i32
    {
        self.as_ref() as *const Mutex<T> as _
    }
}

pub trait Reference<T>
{
    fn construct(obj: T) -> Self;
}

pub trait ThreadSafeRcBoxTake<T>
{
    fn take(&self) -> MutexGuard<T>;
    fn take_mut(&self) -> MutexGuard<T>;
}

impl <T>Reference<T> for ThreadSafeRcBox<T>
{
    fn construct(obj: T) -> Self
    {
        Arc::new(Mutex::new(obj))
    }
}

impl <T>ThreadSafeRcBoxTake<T> for ThreadSafeRcBox<T>
{
    fn take(&self) -> MutexGuard<T>
    {
        self.lock().unwrap()
    }
    fn take_mut(&self) -> MutexGuard<T>
    {
        self.lock().unwrap()
    }
}