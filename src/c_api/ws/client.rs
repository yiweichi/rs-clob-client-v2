//! WebSocket-related C ABI placeholders.

use crate::c_api::core::types::{PMClient, PMStatus};

/// Start the WebSocket subsystem for a client.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_ws_start(client: *mut PMClient) -> PMStatus {
    let _ = client;
    PMStatus::InternalError
}

/// Stop the WebSocket subsystem for a client.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_ws_stop(client: *mut PMClient) -> PMStatus {
    let _ = client;
    PMStatus::InternalError
}
