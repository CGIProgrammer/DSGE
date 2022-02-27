use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

type RwBox<T> = Arc<RwLock<T>>;
type MutexBox<T> = Arc<Mutex<T>>;

pub use std::borrow::Borrow;
pub type RcBox<T> = MutexBox<T>;

pub trait RwLockBox<T>
{
    fn box_id(&self) -> i32;
    fn construct(obj: T) -> Self;
    fn take(&self) -> RwLockReadGuard<T>;
    fn take_mut(&self) -> RwLockWriteGuard<T>;
}

pub trait MutexLockBox<T>
{
    fn box_id(&self) -> i32;
    fn construct(obj: T) -> Self;
    fn take(&self) -> MutexGuard<T>;
    fn take_mut(&self) -> MutexGuard<T>;
}

impl <T>RwLockBox<T> for RwBox<T>
{
    fn box_id(&self) -> i32
    {
        self.as_ref() as *const RwLock<T> as _
    }
    
    fn construct(obj: T) -> Self
    {
        Arc::new(RwLock::new(obj))
    }
    
    fn take(&self) -> RwLockReadGuard<T>
    {
        self.read().unwrap()
    }

    fn take_mut(&self) -> RwLockWriteGuard<T>
    {
        self.write().unwrap()
    }
}

impl <T>MutexLockBox<T> for MutexBox<T>
{
    fn box_id(&self) -> i32
    {
        self.as_ref() as *const Mutex<T> as _
    }
    
    fn construct(obj: T) -> Self
    {
        Arc::new(Mutex::new(obj))
    }
    
    fn take(&self) -> MutexGuard<T>
    {
        self.lock().unwrap()
    }

    fn take_mut(&self) -> MutexGuard<T>
    {
        self.lock().unwrap()
    }
}