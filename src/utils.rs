use std::io::{Read, Result};
use std::mem::{size_of, MaybeUninit};
use std::slice;
//use gl::types::*;
pub use std::ffi::CStr;

#[macro_export]
macro_rules! c_str {
    ($literal:expr) => {
        CStr::from_bytes_with_nul_unchecked(concat!($literal, "\0").as_bytes())
    };
}

#[allow(dead_code)]
pub fn read_struct<T, R: Read>(read: &mut R) -> Result<T> {
    let num_bytes = size_of::<T>();
    unsafe {
        let mut s = MaybeUninit::<T>::uninit();
        let ptr = s.as_mut_ptr();
        let buffer = slice::from_raw_parts_mut(ptr as *mut u8, num_bytes);
        match read.read_exact(buffer) {
            Ok(()) => Ok(s.assume_init()),
            Err(e) => {
                ::std::mem::forget(s);
                Err(e)
            }
        }
    }
}

pub fn cast_slice<'a, A, B>(a: &'a [A]) -> &'a [B]
{
    let size_of_val_a = core::mem::size_of_val(a);
    let size_of_type_b = size_of::<B>();
    let new_len = size_of_val_a / size_of_type_b;
    let rem = size_of_val_a % size_of_type_b;
    if rem == 0 {
        unsafe { core::slice::from_raw_parts(a.as_ptr() as *const B, new_len) }
    } else {
        panic!("Невозможно выполнить приведение срезов: размер приводимого среза не делится на размер целевого типа без остатка.")
    }
}

pub(crate) trait RefId {
    fn box_id(&self) -> usize {
        self as *const Self as *const usize as usize
    }
}

pub fn box_id(obj: &dyn std::any::Any) -> usize {
    obj as *const dyn std::any::Any as *const usize as usize
}
