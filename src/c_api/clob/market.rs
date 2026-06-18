//! Market data functions for the CLOB C API.

use std::ffi::c_char;
use std::str::FromStr as _;

use crate::c_api::core::error::{required_cstr, write_cstr_buffer};
use crate::c_api::core::types::{
    PMMarketClient, PMMarketResponse, PMOrderBookResponse, PMSide, PMStatus,
};
use crate::clob::types::request::{
    LastTradePriceRequest, MidpointRequest, OrderBookSummaryRequest, PriceRequest, SpreadRequest,
};
use crate::auth::state::{State, Unauthenticated};
use crate::clob::types::Side;
use crate::clob::{Client, Config};
use crate::types::U256;

struct PMMarketClientInner<S: State = Unauthenticated> {
    last_error: String,
    runtime: tokio::runtime::Runtime,
    clob_client: Client<S>,
}

fn inner_from_handle<'a>(client: *mut PMMarketClient) -> Result<&'a mut PMMarketClientInner, PMStatus> {
    if client.is_null() {
        return Err(PMStatus::NullPointer);
    }

    Ok(unsafe { &mut *(client.cast::<PMMarketClientInner>()) })
}

fn parse_u256(value: &str) -> Result<U256, PMStatus> {
    U256::from_str(value).map_err(|_| PMStatus::InvalidArgument)
}

fn map_side(side: PMSide) -> Result<Side, PMStatus> {
    match side {
        PMSide::Buy => Ok(Side::Buy),
        PMSide::Sell => Ok(Side::Sell),
    }
}

fn status_from_error(error: &crate::error::Error) -> PMStatus {
    if error.downcast_ref::<reqwest::Error>().is_some() {
        PMStatus::NetworkError
    } else {
        PMStatus::InternalError
    }
}

fn single_value_json(field: &str, value: &str) -> String {
    serde_json::json!({ field: value }).to_string()
}

unsafe fn write_market_response(
    out: *mut PMMarketResponse,
    value: &str,
    raw_json: &str,
) -> PMStatus {
    if out.is_null() {
        return PMStatus::NullPointer;
    }

    unsafe {
        *out = PMMarketResponse::default();
    }

    let out = unsafe { &mut *out };
    let value_status = unsafe { write_cstr_buffer(out.value.as_mut_ptr(), out.value.len(), value) };
    if value_status != PMStatus::Ok {
        return value_status;
    }

    unsafe { write_cstr_buffer(out.raw_json.as_mut_ptr(), out.raw_json.len(), raw_json) }
}

unsafe fn write_orderbook_response(
    out: *mut PMOrderBookResponse,
    hash: &str,
    raw_json: &str,
) -> PMStatus {
    if out.is_null() {
        return PMStatus::NullPointer;
    }

    unsafe {
        *out = PMOrderBookResponse::default();
    }

    let out = unsafe { &mut *out };
    let hash_status = unsafe { write_cstr_buffer(out.hash.as_mut_ptr(), out.hash.len(), hash) };
    if hash_status != PMStatus::Ok {
        return hash_status;
    }

    unsafe { write_cstr_buffer(out.raw_json.as_mut_ptr(), out.raw_json.len(), raw_json) }
}

/// Create an unauthenticated market data client handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_market_client_create(
    host: *const c_char,
    out_client: *mut *mut PMMarketClient,
) -> PMStatus {
    let result = std::panic::catch_unwind(|| {
        if out_client.is_null() {
            return PMStatus::NullPointer;
        }

        let host = match unsafe { required_cstr(host) } {
            Ok(value) => value,
            Err(status) => return status,
        };
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(value) => value,
            Err(_) => return PMStatus::InternalError,
        };
        let clob_client = match Client::new(host, Config::default()) {
            Ok(value) => value,
            Err(_) => return PMStatus::InvalidArgument,
        };

        let inner = PMMarketClientInner {
            last_error: String::new(),
            runtime,
            clob_client,
        };

        unsafe {
            *out_client = Box::into_raw(Box::new(inner)).cast::<PMMarketClient>();
        }
        PMStatus::Ok
    });

    result.unwrap_or(PMStatus::Panic)
}

/// Destroy a market data client handle created by pm_market_client_create.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_market_client_destroy(client: *mut PMMarketClient) {
    if client.is_null() {
        return;
    }

    let _ = std::panic::catch_unwind(|| unsafe {
        drop(Box::from_raw(client.cast::<PMMarketClientInner>()));
    });
}

/// Copy the market client's last error into a caller-owned buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_market_client_last_error(
    client: *mut PMMarketClient,
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

/// Fetch the CLOB server Unix timestamp.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_get_server_time(
    client: *mut PMMarketClient,
    out_timestamp: *mut u64,
) -> PMStatus {
    let result = std::panic::catch_unwind(|| {
        if out_timestamp.is_null() {
            return PMStatus::NullPointer;
        }
        let inner = match inner_from_handle(client) {
            Ok(value) => value,
            Err(status) => return status,
        };

        match inner.runtime.block_on(inner.clob_client.server_time()) {
            Ok(timestamp) => {
                let timestamp = match u64::try_from(timestamp) {
                    Ok(value) => value,
                    Err(_) => return PMStatus::InternalError,
                };
                unsafe {
                    *out_timestamp = timestamp;
                }
                inner.last_error.clear();
                PMStatus::Ok
            }
            Err(error) => {
                inner.last_error = error.to_string();
                status_from_error(&error)
            }
        }
    });

    result.unwrap_or(PMStatus::Panic)
}

/// Fetch the best price for a token side.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_get_price(
    client: *mut PMMarketClient,
    token_id: *const c_char,
    side: PMSide,
    out: *mut PMMarketResponse,
) -> PMStatus {
    let result = std::panic::catch_unwind(|| {
        let token_id = match unsafe { required_cstr(token_id) }.and_then(parse_u256) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let side = match map_side(side) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let inner = match inner_from_handle(client) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let request = PriceRequest::builder().token_id(token_id).side(side).build();

        match inner.runtime.block_on(inner.clob_client.price(&request)) {
            Ok(response) => {
                inner.last_error.clear();
                let value = response.price.to_string();
                let raw_json = single_value_json("price", &value);
                unsafe { write_market_response(out, &value, &raw_json) }
            }
            Err(error) => {
                inner.last_error = error.to_string();
                status_from_error(&error)
            }
        }
    });

    result.unwrap_or(PMStatus::Panic)
}

/// Fetch the bid/ask spread for a token.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_get_spread(
    client: *mut PMMarketClient,
    token_id: *const c_char,
    out: *mut PMMarketResponse,
) -> PMStatus {
    let result = std::panic::catch_unwind(|| {
        let token_id = match unsafe { required_cstr(token_id) }.and_then(parse_u256) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let inner = match inner_from_handle(client) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let request = SpreadRequest::builder().token_id(token_id).build();

        match inner.runtime.block_on(inner.clob_client.spread(&request)) {
            Ok(response) => {
                inner.last_error.clear();
                let value = response.spread.to_string();
                let raw_json = single_value_json("spread", &value);
                unsafe { write_market_response(out, &value, &raw_json) }
            }
            Err(error) => {
                inner.last_error = error.to_string();
                status_from_error(&error)
            }
        }
    });

    result.unwrap_or(PMStatus::Panic)
}

/// Fetch the midpoint price for a token.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_get_midpoint(
    client: *mut PMMarketClient,
    token_id: *const c_char,
    out: *mut PMMarketResponse,
) -> PMStatus {
    let result = std::panic::catch_unwind(|| {
        let token_id = match unsafe { required_cstr(token_id) }.and_then(parse_u256) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let inner = match inner_from_handle(client) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let request = MidpointRequest::builder().token_id(token_id).build();

        match inner.runtime.block_on(inner.clob_client.midpoint(&request)) {
            Ok(response) => {
                inner.last_error.clear();
                let value = response.mid.to_string();
                let raw_json = single_value_json("mid", &value);
                unsafe { write_market_response(out, &value, &raw_json) }
            }
            Err(error) => {
                inner.last_error = error.to_string();
                status_from_error(&error)
            }
        }
    });

    result.unwrap_or(PMStatus::Panic)
}

/// Fetch the most recent trade price for a token.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_get_last_trade_price(
    client: *mut PMMarketClient,
    token_id: *const c_char,
    out: *mut PMMarketResponse,
) -> PMStatus {
    let result = std::panic::catch_unwind(|| {
        let token_id = match unsafe { required_cstr(token_id) }.and_then(parse_u256) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let inner = match inner_from_handle(client) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let request = LastTradePriceRequest::builder().token_id(token_id).build();

        match inner.runtime.block_on(inner.clob_client.last_trade_price(&request)) {
            Ok(response) => {
                inner.last_error.clear();
                let value = response.price.to_string();
                let side = response.side.to_string();
                let raw_json = serde_json::json!({
                    "price": value,
                    "side": side,
                })
                .to_string();
                unsafe { write_market_response(out, &value, &raw_json) }
            }
            Err(error) => {
                inner.last_error = error.to_string();
                status_from_error(&error)
            }
        }
    });

    result.unwrap_or(PMStatus::Panic)
}

/// Fetch the orderbook summary for a token.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_get_orderbook(
    client: *mut PMMarketClient,
    token_id: *const c_char,
    out: *mut PMOrderBookResponse,
) -> PMStatus {
    let result = std::panic::catch_unwind(|| {
        let token_id = match unsafe { required_cstr(token_id) }.and_then(parse_u256) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let inner = match inner_from_handle(client) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let request = OrderBookSummaryRequest::builder().token_id(token_id).build();

        match inner.runtime.block_on(inner.clob_client.order_book(&request)) {
            Ok(response) => {
                inner.last_error.clear();
                let raw_json = match serde_json::to_string(&response) {
                    Ok(value) => value,
                    Err(_) => return PMStatus::InternalError,
                };
                let hash = match response.hash() {
                    Ok(value) => value,
                    Err(_) => return PMStatus::InternalError,
                };
                unsafe { write_orderbook_response(out, &hash, &raw_json) }
            }
            Err(error) => {
                inner.last_error = error.to_string();
                status_from_error(&error)
            }
        }
    });

    result.unwrap_or(PMStatus::Panic)
}
