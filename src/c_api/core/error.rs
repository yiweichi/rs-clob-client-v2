//! Error helpers for the C API.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

use crate::c_api::core::types::PMStatus;

/// Convert a nullable C string into a Rust string slice.
pub(crate) unsafe fn required_cstr<'a>(value: *const c_char) -> Result<&'a str, PMStatus> {
    if value.is_null() {
        return Err(PMStatus::NullPointer);
    }

    unsafe { CStr::from_ptr(value) }
        .to_str()
        .map_err(|_| PMStatus::InvalidArgument)
}

/// Copy a Rust string into a C buffer, always NUL-terminating when possible.
pub(crate) unsafe fn write_cstr_buffer(
    dst: *mut c_char,
    dst_len: usize,
    value: &str,
) -> PMStatus {
    if dst.is_null() {
        return PMStatus::NullPointer;
    }
    if dst_len == 0 {
        return PMStatus::InvalidArgument;
    }

    let bytes = value.as_bytes();
    let copy_len = bytes.len().min(dst_len - 1);
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), dst.cast::<u8>(), copy_len);
        *dst.add(copy_len) = 0;
    }
    PMStatus::Ok
}
