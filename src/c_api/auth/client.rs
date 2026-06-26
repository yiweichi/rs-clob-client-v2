//! Client lifecycle and authentication-facing C ABI functions.

use std::ffi::c_char;
use std::ptr;
use std::str::FromStr as _;
use std::sync::Arc;

use alloy::primitives::Address;
use alloy::signers::Signer as _;
use alloy::signers::local::PrivateKeySigner;

use crate::auth::{Kind, Normal, state::Authenticated};
use crate::c_api::core::error::{required_cstr, write_cstr_buffer};
use crate::c_api::core::types::{PMClient, PMStatus};
use crate::clob::types::SignatureType;
use crate::clob::{Client, Config};

pub(crate) struct PMClientInner<K: Kind = Normal> {
    host: String,
    private_key: String,
    chain_id: u64,
    last_error: String,
    runtime: tokio::runtime::Runtime,
    pub(crate) signer: Option<PrivateKeySigner>,
    pub(crate) clob_client: Option<Arc<Client<Authenticated<K>>>>,
}

fn inner_from_handle<'a>(client: *mut PMClient) -> Result<&'a mut PMClientInner, PMStatus> {
    if client.is_null() {
        return Err(PMStatus::NullPointer);
    }

    Ok(unsafe { &mut *(client.cast::<PMClientInner>()) })
}

fn set_error(inner: &mut PMClientInner, message: impl Into<String>) {
    inner.last_error = message.into();
}

/// Create a Polymarket client handle and authenticate it.
///
/// This builds `clob::Client::new(...).authentication_builder(...).authenticate().await`
/// internally and stores the authenticated client behind the opaque handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_client_create(
    host: *const c_char,
    private_key: *const c_char,
    funder: *const c_char,
    chain_id: u64,
    out_client: *mut *mut PMClient,
) -> PMStatus {
    let result = std::panic::catch_unwind(|| {
        if out_client.is_null() {
            return PMStatus::NullPointer;
        }

        let host = match unsafe { required_cstr(host) } {
            Ok(value) => value,
            Err(status) => return status,
        };
        let private_key = match unsafe { required_cstr(private_key) } {
            Ok(value) => value,
            Err(status) => return status,
        };
        let funder = match unsafe { required_cstr(funder) }
            .and_then(|value| Address::from_str(value).map_err(|_| PMStatus::InvalidArgument))
        {
            Ok(value) => value,
            Err(status) => return status,
        };

        let runtime = match tokio::runtime::Runtime::new() {
            Ok(value) => value,
            Err(_) => return PMStatus::InternalError,
        };
        let mut inner = PMClientInner {
            host: host.to_owned(),
            private_key: private_key.to_owned(),
            chain_id,
            last_error: String::new(),
            runtime,
            signer: None,
            clob_client: None,
        };

        let auth_result = (|| -> Result<(), PMStatus> {
            let signer = PrivateKeySigner::from_str(inner.private_key.as_str())
                .map_err(|_| PMStatus::InvalidArgument)?
                .with_chain_id(Some(chain_id));

            let authenticated = inner.runtime.block_on(async {
                let unauthenticated = Client::new(&inner.host, Config::default())
                    .map_err(|_| PMStatus::InternalError)?;
                unauthenticated
                    .authentication_builder(&signer)
                    .funder(funder)
                    .signature_type(SignatureType::Poly1271)
                    .authenticate()
                    .await
                    .map_err(|_| PMStatus::AuthenticationError)
            })?;

            set_error(
                &mut inner,
                format!("authenticated clob client ready on chain {chain_id}"),
            );
            inner.signer = Some(signer);
            inner.clob_client = Some(Arc::new(authenticated));
            Ok(())
        })();

        if let Err(status) = auth_result {
            set_error(&mut inner, format!("client create/auth failed: {status:?}"));
            return status;
        }

        unsafe {
            *out_client = Box::into_raw(Box::new(inner)).cast::<PMClient>();
        }
        PMStatus::Ok
    });

    result.unwrap_or(PMStatus::Panic)
}

/// Destroy a Polymarket client handle created by `pm_client_create`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_client_destroy(client: *mut PMClient) {
    if client.is_null() {
        return;
    }

    let _ = std::panic::catch_unwind(|| unsafe {
        drop(Box::from_raw(client.cast::<PMClientInner>()));
    });
}

/// Copy the client's last error into a caller-owned buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_client_last_error(
    client: *mut PMClient,
    out_buffer: *mut c_char,
    out_buffer_len: usize,
) -> PMStatus {
    let result = std::panic::catch_unwind(|| {
        let inner = match inner_from_handle(client) {
            Ok(value) => value,
            Err(status) => return status,
        };

        unsafe { write_cstr_buffer(out_buffer, out_buffer_len, &inner.last_error) }
    });

    result.unwrap_or(PMStatus::Panic)
}

/// Clear the client's last error string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_client_clear_error(client: *mut PMClient) -> PMStatus {
    let result = std::panic::catch_unwind(|| {
        let inner = match inner_from_handle(client) {
            Ok(value) => value,
            Err(status) => return status,
        };

        inner.last_error.clear();
        PMStatus::Ok
    });

    result.unwrap_or(PMStatus::Panic)
}

pub(crate) fn set_last_error(client: *mut PMClient, message: impl Into<String>) {
    if let Ok(inner) = inner_from_handle(client) {
        inner.last_error = message.into();
    }
}

pub(crate) fn ensure_client(client: *mut PMClient) -> PMStatus {
    match inner_from_handle(client) {
        Ok(_) => PMStatus::Ok,
        Err(status) => status,
    }
}

pub(crate) fn authenticated_client(
    client: *mut PMClient,
) -> Result<Arc<Client<Authenticated<Normal>>>, PMStatus> {
    let inner = inner_from_handle(client)?;
    inner
        .clob_client
        .clone()
        .ok_or(PMStatus::AuthenticationError)
}

pub(crate) fn signer(client: *mut PMClient) -> Result<PrivateKeySigner, PMStatus> {
    let inner = inner_from_handle(client)?;
    inner.signer.clone().ok_or(PMStatus::AuthenticationError)
}

pub(crate) fn with_runtime<T>(
    client: *mut PMClient,
    f: impl FnOnce(&tokio::runtime::Runtime) -> T,
) -> Result<T, PMStatus> {
    let inner = inner_from_handle(client)?;
    Ok(f(&inner.runtime))
}

#[allow(dead_code)]
pub(crate) fn client_config(client: *mut PMClient) -> Result<(String, String, u64), PMStatus> {
    let inner = inner_from_handle(client)?;
    Ok((
        inner.host.clone(),
        inner.private_key.clone(),
        inner.chain_id,
    ))
}

#[allow(dead_code)]
pub(crate) fn null_client_out(out_client: *mut *mut PMClient) {
    if !out_client.is_null() {
        unsafe {
            *out_client = ptr::null_mut();
        }
    }
}
