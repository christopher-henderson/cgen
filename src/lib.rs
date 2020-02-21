use std::convert::{TryFrom, TryInto};
use std::ffi::CString;
use std::fmt::{Display, Error, Formatter, Write, Debug};

use libc::c_char;
use libc::c_double;
use libc::c_float;
use libc::c_int;
use libc::c_long;
use libc::c_short;
use libc::c_uchar;
use libc::c_uint;
use libc::c_ulong;
use libc::c_ushort;
use libc::size_t;
use libc::ssize_t;
use std::intrinsics::copy;

pub trait C: Sized + Debug {}

impl C for c_char {}
impl C for c_double {}
impl C for c_float {}
impl C for c_int {}
impl C for c_long {}
impl C for c_short {}
impl C for c_uchar {}
impl C for c_uint {}
impl C for c_ulong {}
impl C for c_ushort {}
impl C for size_t {}
impl C for ssize_t {}

#[repr(C)]
#[derive(Debug)]
pub struct RustBuffer<T: C> {
    data: *mut T,
    len: size_t,
    capacity: size_t,
    free: *const *mut Self,
    push: *const (*mut Self, T),
    push_all: *const (*mut Self, *const T, usize)
}

type RustCharBuffer = RustBuffer<c_char>;
type RustDoubleBuffer = RustBuffer<c_double>;
type RustFloatBuffer = RustBuffer<c_float>;
type RustIntBuffer = RustBuffer<c_int>;
type RustLongBuffer = RustBuffer<c_long>;
type RustShortBuffer = RustBuffer<c_short>;
type RustUcharBuffer = RustBuffer<c_uchar>;
type RustUintBuffer = RustBuffer<c_uint>;
type RustUlongBuffer = RustBuffer<c_ulong>;
type RustUshortBuffer = RustBuffer<c_ushort>;

type RustString = RustCharBuffer;

impl<T: C> C for RustBuffer<T> {}

impl<T: C> RustBuffer<T> {
    pub fn new() -> RustBuffer<T> {
        Self::with_capacity(0)
    }
    pub fn with_capacity(capacity: usize) -> RustBuffer<T> {
        let mut new = Vec::with_capacity(capacity);
        // let mut new: Vec<Goal> = array.into_iter().map(|item| item.into()).collect();
        // This MIGHT be a leak for the fat pointer data that array was keeping around.
        // Although in casual testing this has been very hard to detect.
        // Keep an eye on https://github.com/rust-lang/rust/issues/65816 as
        // that is the API that we want.
        let data = new.as_mut_ptr();
        let len = new.len();
        let capacity = new.capacity();
        // I would say that function pointer declarations are even MORE confusing in Rust
        // than they are in C. But hey, at least we don't have to use them often.
        let free = Self::free as *const *mut Self;
        let push = Self::push as *const (*mut Self, T);
        let push_all = Self::push_all as *const (*mut Self, *const T, usize);
        // After this point the buffer is managed by our drop
        // function rather than the original vector.
        std::mem::forget(new);
        RustBuffer {
            data,
            len,
            capacity,
            free,
            push,
            push_all
        }
    }
    pub fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }
    pub fn from_raw(raw: *mut Self) -> Self {
        unsafe { *Box::from_raw(raw) }
    }
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        unsafe { std::slice::from_raw_parts(self.data, self.len) }.iter()
    }

    pub extern "C" fn push(&mut self, value: T) {
        let mut array = unsafe{Vec::from_raw_parts(self.data, self.len, self.capacity)};
        array.push(value);
        let data = array.as_mut_ptr();
        let len = array.len();
        let capacity = array.capacity();
        self.data = data;
        self.len = len;
        self.capacity = capacity;
        std::mem::forget(array);
    }

    pub extern "C" fn push_all(&mut self, values: *const T, len: usize) {
        let mut array = unsafe{Vec::from_raw_parts(self.data, self.len, self.capacity)};
        // This is a convenient way to call std::alloc::realloc without poking that dragon just yet.
        array.reserve(len);
        unsafe {copy(values, array.as_mut_ptr().add(array.len()), len);}
        let data = array.as_mut_ptr();
        let len = array.len() + len;
        let capacity = array.capacity();
        self.data = data;
        self.len = len;
        self.capacity = capacity;
        std::mem::forget(array);
    }

    pub extern "C" fn free(ptr: *mut Self) {
        Self::from_raw(ptr);
    }
}


impl<T: C> Drop for RustBuffer<T> {
    fn drop(&mut self) {
        if self.data.is_null() {
            return;
        }
        unsafe {
            let slice = std::slice::from_raw_parts_mut(self.data, self.capacity);
            Box::from_raw(slice as *mut [T]);
        }
    }
}

impl<Goal, Src> From<Vec<Src>> for RustBuffer<Goal>
where
    Goal: C,
    Src: Into<Goal>,
{
    fn from(array: Vec<Src>) -> Self {
        let mut new = Vec::with_capacity(array.capacity());
        new.extend(array.into_iter().map(|item| item.into()));
        // let mut new: Vec<Goal> = array.into_iter().map(|item| item.into()).collect();
        // This MIGHT be a leak for the fat pointer data that array was keeping around.
        // Although in casual testing this has been very hard to detect.
        // Keep an eye on https://github.com/rust-lang/rust/issues/65816 as
        // that is the API that we want.
        let data = new.as_mut_ptr();
        let len = new.len();
        let capacity = new.capacity();
        // I would say that function pointer declarations are even MORE confusing in Rust
        // than they are in C. But hey, at least we don't have to use them often.
        let free = Self::free as *const *mut Self;
        let push = Self::push as *const (*mut Self, Goal);
        let push_all = Self::push_all as *const (*mut Self, *const Goal, usize);
        // After this point the buffer is managed by our drop
        // function rather than the original vector.
        std::mem::forget(new);
        RustBuffer {
            data,
            len,
            capacity,
            free,
            push,
            push_all
        }
    }
}

impl TryFrom<String> for RustString {
    type Error = std::ffi::NulError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let cstring = CString::new(value)?;
        let buf = cstring.into_bytes_with_nul();
        // libc::c_char aliases to an i8 and I am not sure why.
        // If I am incorrect in coercing this to u8s then please do scream at me.
        let buf = unsafe{std::mem::transmute::<Vec<u8>, Vec<i8>>(buf)};
        Ok(buf.into())
    }
}

impl TryInto<String> for RustString {
    type Error = std::ffi::IntoStringError;

    fn try_into(mut self) -> Result<String, Self::Error> {
        let data = self.data;
        self.data = std::ptr::null_mut();
        // I am honestly a bit perplexed why this interface wants i8
        // when the above into_bytes_with_nul call gives me u8s.
        unsafe { CString::from_raw(data) }.into_string()
    }
}

impl Display for RustCharBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for (i, item) in self.iter().enumerate() {
            if i >= self.len {
                break;
            }
            f.write_char((*item as u8) as char)?;
        }
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn new_rust_char_buffer(capacity: size_t) -> *mut RustCharBuffer {
    RustCharBuffer::with_capacity(capacity).into_raw()
}

#[no_mangle]
pub extern "C" fn new_rust_double_buffer(capacity: size_t) -> *mut RustDoubleBuffer {
    RustDoubleBuffer::with_capacity(capacity).into_raw()
}

#[no_mangle]
pub extern "C" fn new_rust_float_buffer(capacity: size_t) -> *mut RustFloatBuffer {
    RustFloatBuffer::with_capacity(capacity).into_raw()
}

#[no_mangle]
pub extern "C" fn new_rust_int_buffer(capacity: size_t) -> *mut RustIntBuffer {
    RustIntBuffer::with_capacity(capacity).into_raw()
}

#[no_mangle]
pub extern "C" fn new_rust_long_buffer(capacity: size_t) -> *mut RustLongBuffer {
    RustLongBuffer::with_capacity(capacity).into_raw()
}

#[no_mangle]
pub extern "C" fn new_rust_short_buffer(capacity: size_t) -> *mut RustShortBuffer {
    RustShortBuffer::with_capacity(capacity).into_raw()
}

#[no_mangle]
pub extern "C" fn new_rust_uchar_buffer(capacity: size_t) -> *mut RustUcharBuffer {
    RustUcharBuffer::with_capacity(capacity).into_raw()
}

#[no_mangle]
pub extern "C" fn new_rust_uint_buffer(capacity: size_t) -> *mut RustUintBuffer {
    RustUintBuffer::with_capacity(capacity).into_raw()
}

#[no_mangle]
pub extern "C" fn new_rust_ulong_buffer(capacity: size_t) -> *mut RustUlongBuffer {
    RustUlongBuffer::with_capacity(capacity).into_raw()
}

#[no_mangle]
pub extern "C" fn new_rust_ushort_buffer(capacity: size_t) -> *mut RustUshortBuffer {
    RustUshortBuffer::with_capacity(capacity).into_raw()
}

#[no_mangle]
pub extern "C" fn new_rust_string(capacity: size_t) -> *mut RustString {
    let buf: Vec<i8> = (0..=capacity).map(|_| 0).collect();
    let buf: RustString = buf.into();
    buf.into_raw()
}


#[cfg(test)]
mod tests {
    // use crate::RustCharBuffer;

    #[test]
    fn asdas() {
        // let size: usize = 5;
        // let s: Vec<u8> = [0; size].into_vec();
        // println!("{:?}", s);
        // let string = String::with_capacity(20);
        // let cap = string.capacity();
        // let string = CString::new(string).unwrap().into_bytes_with_nul().len();
        // println!("{} {}", cap, string);
    }

    #[test]
    fn wat() {
        let mut v: Vec<u8> = Vec::with_capacity(0);
        v.reserve(1);
        println!("{:?}", v.as_mut_ptr());
        // let mut array: Vec<u8> = v.into_iter().map(|item| item.into()).collect();
        // println!("{:?}", array.as_mut_ptr());
        // array.push(1);
        // println!("{:?}", array.as_mut_ptr());
        // let b: RustCharBuffer = RustCharBuffer::with_capacity(10);
        // unsafe {println!("{:?}", (*b.into_raw()).data);}
    }

    // #[test]
    // fn thsdfsdf() {
    //     let buf: RustCharBuffer = "abdc".to_string().try_into().unwrap();
    //     println!("{}1", buf);
    //     let s: String = buf.try_into().unwrap();
    //     println!("{} 1", s);
    // }
    //    use super::*;
    //
    //
    //    #[test]
    //    fn arr() {
    //        for _ in 0..1 {
    //            let _buf: RustBuffer<c_float> = vec![1.0, 2.1, 3.4, 4.6].into();
    //        }
    //    }
    //
    //    #[test]
    //    fn string() {
    //        let _string = RustString::new("asdsad").unwrap();
    //    }
    //
    //    #[test]
    //    fn string_buf() {
    //        for _ in 0..10000000 {
    //            let _buf: RustBuffer<RustString> = vec![
    //                RustString::new("asdsad1").unwrap(),
    //                RustString::new("asdsad2").unwrap(),
    //                RustString::new("asdsad3").unwrap(),
    //            ]
    //            .into();
    ////            rustfree(buf.into_raw() as *mut c_void);
    //        }
    //    }
    //
    //    #[test]
    //    fn nested() {
    //        let buf1: RustBuffer<RustString> = vec![
    //            RustString::new("asdsad1").unwrap(),
    //            RustString::new("asdsad2").unwrap(),
    //            RustString::new("asdsad3").unwrap(),
    //        ]
    //        .into();
    //        let buf2: RustBuffer<RustString> = vec![
    //            RustString::new("asdsad4").unwrap(),
    //            RustString::new("asdsad5").unwrap(),
    //            RustString::new("asdsad6").unwrap(),
    //        ]
    //        .into();
    //        let buf3: RustBuffer<RustBuffer<RustString>> = vec![buf2, buf1].into();
    //        println!("{}", buf3);
    //    }
}
