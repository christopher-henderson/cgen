
use std::ffi::CString;
use std::fmt::{Display, Error, Formatter, Write};

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

/// Traits
pub trait C: Sized {}

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

pub trait IntoRaw: Sized {
    fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }
}


impl <T: C> IntoRaw for T {}

// @TODO These are platform specific, conditional compilation these out or something
//impl C for c_longlong {}
//impl C for c_schar {}
//impl C for c_ulonglong {}

#[repr(C)]
pub struct RustBuffer<T: C> {
    data: *mut T,
    len: usize,
}

impl<T: C> C for RustBuffer<T> {}

impl<T: C> RustBuffer<T> {
    pub fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }
    pub fn from_raw(raw: *mut Self) -> Self {
        unsafe { *Box::from_raw(raw) }
    }
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        unsafe { std::slice::from_raw_parts(self.data, self.len) }.iter()
    }
}

impl<T: C> Drop for RustBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            let slice = std::slice::from_raw_parts_mut(self.data, self.len);
            Box::from_raw(slice as *mut [T]);
        }
    }
}

impl<Goal, Src, Iter> From<Iter> for RustBuffer<Goal>
where
    Goal: C,
    Src: Into<Goal>,
    Iter: IntoIterator<Item = Src>,
{
    fn from(array: Iter) -> Self {
        let mut array: Vec<Goal> = array.into_iter().map(|item| item.into()).collect();
        array.shrink_to_fit(); // Is this necessary? Will map give us the right capacity here?
        let data = array.as_mut_ptr();
        let len = array.len();
        // This MIGHT be a leak for the fat pointer data that array was keeping around.
        // Although in casual testing this has been very hard to detect.
        // Keep an eye on https://github.com/rust-lang/rust/issues/65816 as
        // that is the API that we want.
        std::mem::forget(array);
        RustBuffer { data, len }
    }
}

impl<T: Display + C> Display for RustBuffer<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str("[\n")?;
        for item in self.iter() {
            f.write_fmt(format_args!("\t{},\n", item))?;
        }
        f.write_char(']')?;
        Ok(())
    }
}

#[repr(C)]
/// A RustString wraps a RustBuffer<c_uchar>.
///
/// RustString's str_len is the length of the string WITHOUT the null byte.
/// To get the length of the entire buffer, including the null byte, access
/// the internal buffer's own len field.
pub struct RustString {
    buf: RustBuffer<c_uchar>,
    str_len: usize,
    free: *const *mut RustString
}

impl C for RustString {}

impl RustString {
    pub fn new<T: Into<Vec<u8>>>(string: T) -> Result<Self, std::ffi::NulError> {
        let buf: RustBuffer<c_uchar> = CString::new(string)?.into_bytes_with_nul().into();
        let str_len = buf.len - 1;
        Ok(RustString { buf, str_len, free: RustString::free as *const *mut RustString })
    }

//    #[no_mangle]
    pub extern fn free(ptr: *mut Self) {
        Box::new(unsafe{Box::from_raw(ptr)});
    }
}

impl Display for RustString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for (i, item) in self.buf.iter().enumerate() {
            if i >= self.str_len {
                break;
            }
            f.write_char(*item as char)?;
        }
        Ok(())
    }
}


#[no_mangle]
pub extern fn ruststringnew(len: usize) -> *mut RustString {
    RustString::new(String::with_capacity(len)).unwrap().into_raw()
}




//#[repr(C)]
//pub struct Thing {
//    free: *const *mut Thing
//}


//impl Thing {
//    pub fn new() -> Thing {
//        Thing { free: Thing::free as *const *mut Thing }
//    }
//
//    #[no_mangle]
//    pub extern fn free(thing: *mut Thing) {
//        println!("....hi?");
////        Box::new(unsafe{Box::from_raw(thing)});
//    }
//}

//#[no_mangle]
//pub extern fn thingnew() -> *mut Thing {
//    Box::into_raw(Box::new(Thing::new()))
//}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn arr() {
        for _ in 0..1 {
            let _buf: RustBuffer<c_float> = vec![1.0, 2.1, 3.4, 4.6].into();
        }
    }

    #[test]
    fn string() {
        let _string = RustString::new("asdsad").unwrap();
    }

    #[test]
    fn string_buf() {
        for _ in 0..10000000 {
            let _buf: RustBuffer<RustString> = vec![
                RustString::new("asdsad1").unwrap(),
                RustString::new("asdsad2").unwrap(),
                RustString::new("asdsad3").unwrap(),
            ]
            .into();
//            rustfree(buf.into_raw() as *mut c_void);
        }
    }

    #[test]
    fn nested() {
        let buf1: RustBuffer<RustString> = vec![
            RustString::new("asdsad1").unwrap(),
            RustString::new("asdsad2").unwrap(),
            RustString::new("asdsad3").unwrap(),
        ]
        .into();
        let buf2: RustBuffer<RustString> = vec![
            RustString::new("asdsad4").unwrap(),
            RustString::new("asdsad5").unwrap(),
            RustString::new("asdsad6").unwrap(),
        ]
        .into();
        let buf3: RustBuffer<RustBuffer<RustString>> = vec![buf2, buf1].into();
        println!("{}", buf3);
    }

}
