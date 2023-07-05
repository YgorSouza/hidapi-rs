use std::ffi::CStr;
use std::iter::once;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::str::Utf8Error;
use windows_sys::core::PCWSTR;
use windows_sys::Win32::Devices::Properties::{DEVPROP_TYPE_STRING, DEVPROP_TYPE_STRING_LIST, DEVPROPTYPE};
use crate::WcharString;
use crate::windows_native::types::DeviceProperty;

#[repr(transparent)]
pub struct U16Str([u16]);

impl U16Str {

    unsafe fn from_slice_unsafe(slice: &[u16]) -> &Self {
        let ptr: *const [u16] = slice;
        &*(ptr as *const Self)
    }

    unsafe fn from_slice_mut_unsafe(slice: &mut [u16]) -> &mut Self {
        let ptr: *mut [u16] = slice;
        &mut *(ptr as *mut Self)
    }

    pub fn from_slice(slice: &[u16]) -> &Self {
        assert!(slice.last().is_some_and(is_null), "Slice is not null terminated");
        debug_assert_eq!(slice.iter().filter(|c| is_null(c)).count(), 1, "Found null character in the middle");
        unsafe { Self::from_slice_unsafe(slice) }
    }

    pub fn from_slice_mut(slice: &mut [u16]) -> &mut Self {
        assert!(slice.last().is_some_and(is_null), "Slice is not null terminated");
        debug_assert_eq!(slice.iter().filter(|c| is_null(c)).count(), 1, "Found null character in the middle");
        unsafe { Self::from_slice_mut_unsafe(slice) }
    }

    pub fn from_slice_list(slice: &[u16]) -> impl Iterator<Item=&U16Str> {
        slice
            .split_inclusive(is_null)
            .map(Self::from_slice)
    }

    pub fn from_slice_list_mut(slice: &mut [u16]) -> impl Iterator<Item=&mut U16Str> {
        slice
            .split_inclusive_mut(is_null)
            .map(Self::from_slice_mut)
    }

    pub fn as_ptr(&self) -> PCWSTR {
        self.0.as_ptr()
    }

    pub fn as_slice(&self) -> &[u16] {
        &self.0[..self.0.len() - 1]
    }
    pub fn as_slice_mut(&mut self) -> &mut [u16] {
        let end = self.0.len() - 1;
        &mut self.0[..end]
    }

    pub fn make_uppercase_ascii(&mut self) {
        for c in self.as_slice_mut() {
            if let Ok(t) = u8::try_from(*c) {
                *c = t.to_ascii_uppercase().into();
            }
        }
    }

}

impl ToString for U16Str {
    fn to_string(&self) -> String {
        String::from_utf16(self.as_slice()).expect("Invalid UTF-16")
    }
}

impl From<&U16Str> for WcharString {
    fn from(value: &U16Str) -> Self {
        String::from_utf16(value.as_slice())
            .map(WcharString::String)
            .unwrap_or_else(|_| WcharString::Raw(value.as_slice().to_vec()))
    }
}

pub struct U16String(Vec<u16>);

impl U16String {
    pub fn as_ptr(&self) -> PCWSTR {
        self.0.as_ptr()
    }
}

impl Deref for U16String {
    type Target = U16Str;

    fn deref(&self) -> &Self::Target {
        unsafe { U16Str::from_slice_unsafe(self.0.as_slice()) }
    }
}

impl DerefMut for U16String {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { U16Str::from_slice_mut_unsafe(self.0.as_mut_slice()) }
    }
}

impl TryFrom<&CStr> for U16String {
    type Error = Utf8Error;

    fn try_from(value: &CStr) -> Result<Self, Self::Error> {
        Ok(Self(value
            .to_str()?
            .encode_utf16()
            .chain(once(0))
            .collect()))
    }
}

impl From<U16String> for WcharString {
    fn from(value: U16String) -> Self {
        (&*value).into()
    }
}

unsafe impl DeviceProperty for U16String {
    const TYPE: DEVPROPTYPE = DEVPROP_TYPE_STRING;

    fn create_sized(bytes: usize) -> Self {
        assert_eq!(bytes % size_of::<u16>(), 0);
        U16String(vec![0u16; bytes / size_of::<u16>()])
    }

    fn as_ptr_mut(&mut self) -> *mut u8 {
        self.0.as_mut_ptr() as _
    }

    fn validate(&self) {
        assert!(self.0.last().is_some_and(is_null), "Slice is not null terminated");
        debug_assert_eq!(self.0.iter().filter(|c| is_null(c)).count(), 1, "Found null character in the middle");
    }
}

pub struct U16StringList(pub Vec<u16>);

unsafe impl DeviceProperty for U16StringList {
    const TYPE: DEVPROPTYPE = DEVPROP_TYPE_STRING_LIST;

    fn create_sized(bytes: usize) -> Self {
        assert_eq!(bytes % size_of::<u16>(), 0);
        U16StringList(vec![0u16; bytes / size_of::<u16>()])
    }

    fn as_ptr_mut(&mut self) -> *mut u8 {
        self.0.as_mut_ptr() as _
    }

    fn validate(&self) {
        assert!(self.0.last().is_some_and(is_null), "Slice is not null terminated");
    }
}

impl U16StringList {
    pub fn iter(&self) -> impl Iterator<Item=&U16Str> {
        U16Str::from_slice_list(self.0.as_slice())
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut U16Str> {
        U16Str::from_slice_list_mut(self.0.as_mut_slice())
    }
}

fn is_null(c: &u16) -> bool {
    *c == 0
}
