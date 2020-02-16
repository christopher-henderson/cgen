use std::os::raw::*;
use std::convert::TryFrom;
use std::ffi::CString;

/// Primitive Impls
pub trait Primitive: Copy + Clone {}

impl Primitive for c_char {}
impl Primitive for c_double {}
impl Primitive for c_float {}
impl Primitive for c_int {}
impl Primitive for c_long {}
// @TODO conditional compilation these or something
// These are platform specific
//impl Primitive for c_longlong {}
//impl Primitive for c_schar {}
impl Primitive for c_short {}
impl Primitive for c_uchar {}
impl Primitive for c_uint {}
impl Primitive for c_ulong {}
//impl Primitive for c_ulonglong {}
impl Primitive for c_ushort {}

impl C for c_void {}

/// Traits
pub trait C: Sized {}

pub trait AsC {
    type Target: C;
    fn as_c(&self) -> &Self::Target;
}

pub trait IntoC {
    type Target: C;
    fn into_c(self) -> Self::Target;
}


pub trait TryIntoC<T: C> {
    type Error;
    fn try_into_c(self) -> Result<T, Self::Error>;
}

pub trait TryFromC<T>: C {
    type Error;
    fn try_from_c(value: T) -> Result<Self, Self::Error>;
}


/// Blanket Implementations
impl <U: C, T: TryFromC<U>> TryIntoC<T> for U {
    type Error = T::Error;

    fn try_into_c(self) -> Result<T, Self::Error> {
        T::try_from_c(self)
    }
}



impl <T: Primitive> C for T {}

impl<T: Primitive> AsC for T {
    type Target = Self;
    fn as_c(&self) -> &Self::Target {
        self
    }
}

impl<T: Primitive> IntoC for T {
    type Target = Self;
    fn into_c(self) -> Self::Target {
        self
    }
}

impl <T: Primitive> TryFromC<T> for T {
    type Error = std::convert::Infallible;

    fn try_from_c(value: T) -> Result<Self, Self::Error> {
        Ok(value)
    }
}

/// Array Type

#[repr(C)]
pub struct RustBuffer<T> {
    data: *mut T,
    len: usize
}

impl <T> RustBuffer<T> {
    pub fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }
    pub fn from_raw(raw: *mut Self) -> Self {
        unsafe{*Box::from_raw(raw)}
    }
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        unsafe{std::slice::from_raw_parts(self.data, self.len)}.iter()
    }
}

impl <T: IntoC> From<Vec<T>> for RustBuffer<T::Target> {
    fn from(array: Vec<T>) -> Self {
        let mut array: Vec<T::Target> = array.into_iter().map(|item| item.into_c()).collect();
        array.shrink_to_fit(); // Is this necessary? Will map give us the right capacity here?
        let ptr = array.as_mut_ptr();
        let len = array.len();
        std::mem::forget(array);
        RustBuffer {
            data: ptr,
            len: len
        }
    }
}

#[cfg(not(test))]
impl <T> Drop for RustBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            let slice = std::slice::from_raw_parts_mut(self.data, self.len);
            Box::from_raw(slice as *mut [T]);
        }
    }
}


#[no_mangle]
pub extern fn rustfree(buf: *mut RustBuffer<c_void>) {
    if buf.is_null() {
        return;
    }
    RustBuffer::from_raw(buf);
}

/// A RustString is guaranteed to not have any interior null bytes,
/// however it is NOT null terminated as it always comes with its length data.
type RustString = RustBuffer<c_uchar>;

impl TryFrom<String> for RustString {
    type Error = std::ffi::NulError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(CString::new(value)?.into_bytes().into())
    }
}

use std::fmt::{Write, Formatter, Error};

impl std::fmt::Display for RustString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for i in self.iter() {
            f.write_char(*i as char)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for RustBuffer<RustString> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_char('[')?;
        for (i, item) in self.iter().enumerate() {
            f.write_fmt(format_args!("\t{}", item))?;
            if i != self.len - 1 {
                f.write_char(',')?;
            }
        }
        f.write_char(']')?;
        Ok(())
    }
}

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod tests {

    use super::*;
    use std::convert::TryInto;
    use std::sync::atomic::Ordering;


    lazy_static! {
        static ref FREES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    }

    /// 1:1 malloc to free assertion testing
    ///
    /// This needs to manually stay in-sync with the live implementation. It's intent is to
    /// count the number of deallocations for tests that wish to assert a 1:1 malloc-to-free
    ///
    /// If you are getting testing failures regarding incorrect drop counts then you likely
    /// need to run tests sequentially (cargo test -- --test-threads 1)
    impl <T> Drop for RustBuffer<T> {
        fn drop(&mut self) {
            FREES.fetch_add(1, Ordering::Relaxed);
            unsafe {
                let slice = std::slice::from_raw_parts_mut(self.data, self.len);
                Box::from_raw(slice as *mut [T]);
            }
        }
    }

    #[test]
    fn test_primitive_as() {
        assert_eq!(5.as_c(), &5);
        assert_eq!(5.0.as_c(), &5.0);
    }

    #[test]
    fn test_primitive_into() {
        assert_eq!(5.into_c(), 5);
        assert_eq!(5.0_f32.into_c(), 5.0);
    }
    
    #[test]
    fn test_primitive_try_into() {
        let five: c_char = 5;
        let five: c_char = five.try_into_c().unwrap();
        assert_eq!(five, 5);
    }

    #[test]
    fn test_primitive_try_from() {
        let five: c_char = c_char::try_from_c(5).unwrap();
        assert_eq!(five, 5);
    }
    
    #[test]
    fn primitive_array() {
        let _arr: RustBuffer<u8> = vec![1,2,3,4].into();
    }

    #[test]
    fn one_to_one_malloc_to_free_from_drop() {
        FREES.fetch_and(0, Ordering::SeqCst);
        for _ in 0..100 {
            let _: RustBuffer<u8> = vec![1,2,3].into();
        }
        assert_eq!(FREES.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn one_to_one_malloc_to_free_from_extern_free() {
        FREES.fetch_and(0, Ordering::SeqCst);
        for _ in 0..100 {
            let buf: RustBuffer<u8> = vec![1,2,3].into();
            rustfree(buf.into_raw() as *mut RustBuffer<c_void>);
        }
        assert_eq!(FREES.load(Ordering::SeqCst), 100);
    }

    static LARGE: &str = include_str!("../proc");

    #[test]
    fn rust_string_free() {
        FREES.fetch_and(0, Ordering::SeqCst);
        for _ in 0..100 {
            let buf: RustString = String::from(LARGE).try_into().unwrap();
            rustfree(buf.into_raw() as *mut RustBuffer<c_void>);
        }
        assert_eq!(FREES.load(Ordering::SeqCst), 100);
    }
    
    #[test]
    fn rust_string_display() {
        let want = String::from("This isn't the greatest song in the world");
        let buf: RustString = want.clone().try_into().unwrap();
        assert_eq!(format!("{}", buf), want);
    }

    #[test]
    fn zero_len_buffer() {
        let v: Vec<u8>= vec![];
        let _: RustBuffer<u8> = v.into();
    }
    
    #[test]
    fn zero_len_string() {
        let buf: RustString = String::with_capacity(0).try_into().unwrap();
        assert_eq!(format!("{}", buf), String::from(""));
    }
    
    #[test]
    fn null_free() {
        rustfree(std::ptr::null_mut());
    }
    
    #[test]
    fn inner_null_free() {
        rustfree(RustBuffer {
            data: std::ptr::null_mut(),
            len: 0
        }.into_raw());
    }
    
    #[test]
    fn array_of_strings() {
        let arr: Vec<RustString> = vec!["a".to_string().try_into().unwrap(), "b".to_string().try_into().unwrap(), "c".to_string().try_into().unwrap()];
        let buf: RustBuffer<RustString> = arr.into();
//        let buf: RustBuffer<RustString> = arr.try_into().unwrap();
    }
}