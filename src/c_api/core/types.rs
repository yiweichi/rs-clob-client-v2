//! C-compatible shared types for the C API.

use std::os::raw::c_char;

/// Opaque client handle used by C/C++ callers.
///
/// The concrete fields are intentionally private to keep Rust types out of the
/// public ABI. Callers must create and destroy this handle through the exported
/// C API functions.
#[repr(C)]
pub struct PMClient {
    _private: [u8; 0],
}

/// Common status codes returned by C API functions.
#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PMStatus {
    Ok = 0,
    NullPointer = 1,
    InvalidArgument = 2,
    AuthenticationError = 3,
    NetworkError = 4,
    InternalError = 100,
    Panic = 101,
}

/// Order side exposed to C/C++ callers.
#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PMSide {
    Buy = 1,
    Sell = 2,
}

/// Order type exposed to C/C++ callers.
#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PMOrderType {
    Gtc = 1,
    Fok = 2,
    Fak = 3,
}

/// Fixed-size response buffer for initial order APIs.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PMOrderResponse {
    pub order_id: [c_char; 128],
    pub status: [c_char; 64],
    pub raw_json: [c_char; 4096],
}

impl Default for PMOrderResponse {
    fn default() -> Self {
        Self {
            order_id: [0; 128],
            status: [0; 64],
            raw_json: [0; 4096],
        }
    }
}

/// Fixed-size response buffer for initial cancel APIs.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PMCancelResponse {
    pub status: [c_char; 64],
    pub raw_json: [c_char; 4096],
}

impl Default for PMCancelResponse {
    fn default() -> Self {
        Self {
            status: [0; 64],
            raw_json: [0; 4096],
        }
    }
}
