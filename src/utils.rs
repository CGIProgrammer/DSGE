use std::io::{Read, Result};
use std::slice;
use std::mem::{MaybeUninit, size_of};
//use gl::types::*;
pub use std::ffi::CStr;
#[macro_export]
macro_rules! c_str {
    ($literal:expr) => {
        CStr::from_bytes_with_nul_unchecked(concat!($literal, "\0").as_bytes())
    }
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