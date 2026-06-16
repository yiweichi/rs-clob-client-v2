//! Order functions for the CLOB C API.

use std::ffi::c_char;
use std::str::FromStr as _;

use crate::c_api::auth::client::{
    authenticated_client, ensure_client, set_last_error, signer, with_runtime,
};
use crate::c_api::core::error::{required_cstr, write_cstr_buffer};
use crate::c_api::core::types::{
    PMCancelResponse, PMClient, PMOrderResponse, PMOrderType, PMSide, PMStatus,
};
use crate::clob::types::response::{CancelOrdersResponse, PostOrderResponse};
use crate::clob::types::{Amount, OrderType, Side};
use crate::types::{Decimal, U256};

fn map_side(side: PMSide) -> Result<Side, PMStatus> {
    match side {
        PMSide::Buy => Ok(Side::Buy),
        PMSide::Sell => Ok(Side::Sell),
    }
}

fn map_order_type(order_type: PMOrderType) -> Result<OrderType, PMStatus> {
    match order_type {
        PMOrderType::Gtc => Ok(OrderType::GTC),
        PMOrderType::Fok => Ok(OrderType::FOK),
        PMOrderType::Fak => Ok(OrderType::FAK),
    }
}

fn parse_u256(value: &str) -> Result<U256, PMStatus> {
    U256::from_str(value).map_err(|_| PMStatus::InvalidArgument)
}

fn parse_decimal(value: &str) -> Result<Decimal, PMStatus> {
    Decimal::from_str(value).map_err(|_| PMStatus::InvalidArgument)
}

fn status_from_error(error: &crate::error::Error) -> PMStatus {
    if error.downcast_ref::<reqwest::Error>().is_some() {
        PMStatus::NetworkError
    } else {
        PMStatus::InternalError
    }
}

unsafe fn write_order_response(out: *mut PMOrderResponse, response: &PostOrderResponse) -> PMStatus {
    if out.is_null() {
        return PMStatus::NullPointer;
    }

    unsafe {
        *out = PMOrderResponse::default();
    }
    let raw_json = match serde_json::to_string(response) {
        Ok(value) => value,
        Err(_) => return PMStatus::InternalError,
    };

    let out = unsafe { &mut *out };
    let order_id_status = unsafe {
        write_cstr_buffer(
            out.order_id.as_mut_ptr(),
            out.order_id.len(),
            &response.order_id,
        )
    };
    if order_id_status != PMStatus::Ok {
        return order_id_status;
    }

    let status = response.status.to_string();
    let status_status = unsafe {
        write_cstr_buffer(out.status.as_mut_ptr(), out.status.len(), &status)
    };
    if status_status != PMStatus::Ok {
        return status_status;
    }

    unsafe { write_cstr_buffer(out.raw_json.as_mut_ptr(), out.raw_json.len(), &raw_json) }
}

unsafe fn write_cancel_response(
    out: *mut PMCancelResponse,
    response: &CancelOrdersResponse,
) -> PMStatus {
    if out.is_null() {
        return PMStatus::NullPointer;
    }

    unsafe {
        *out = PMCancelResponse::default();
    }
    let raw_json = match serde_json::to_string(response) {
        Ok(value) => value,
        Err(_) => return PMStatus::InternalError,
    };
    let status = if response.not_canceled.is_empty() {
        "canceled"
    } else if response.canceled.is_empty() {
        "not_canceled"
    } else {
        "partial"
    };

    let out = unsafe { &mut *out };
    let status_status = unsafe {
        write_cstr_buffer(out.status.as_mut_ptr(), out.status.len(), status)
    };
    if status_status != PMStatus::Ok {
        return status_status;
    }

    unsafe { write_cstr_buffer(out.raw_json.as_mut_ptr(), out.raw_json.len(), &raw_json) }
}

/// Submit a market order.
///
/// `amount` is interpreted as USDC for buy orders and shares for sell orders.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_market_order(
    client: *mut PMClient,
    token_id: *const c_char,
    side: PMSide,
    amount: *const c_char,
    order_type: PMOrderType,
    out: *mut PMOrderResponse,
) -> PMStatus {
    let result = std::panic::catch_unwind(|| {
        let status = ensure_client(client);
        if status != PMStatus::Ok {
            return status;
        }
        if out.is_null() {
            return PMStatus::NullPointer;
        }

        let token_id = match unsafe { required_cstr(token_id) }.and_then(parse_u256) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let side = match map_side(side) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let amount = match unsafe { required_cstr(amount) }.and_then(parse_decimal) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let order_type = match map_order_type(order_type) {
            Ok(value) => value,
            Err(status) => return status,
        };

        let client_ref = match authenticated_client(client) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let signer_ref = match signer(client) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let amount = match side {
            Side::Buy => Amount::usdc(amount),
            Side::Sell => Amount::shares(amount),
            Side::Unknown => return PMStatus::InvalidArgument,
        };
        let amount = match amount {
            Ok(value) => value,
            Err(error) => {
                set_last_error(client, error.to_string());
                return PMStatus::InvalidArgument;
            }
        };

        let response = match with_runtime(client, |runtime| {
            runtime.block_on(async {
                client_ref
                    .market_order()
                    .token_id(token_id)
                    .side(side)
                    .amount(amount)
                    .order_type(order_type)
                    .build_sign_and_post(&signer_ref)
                    .await
            })
        }) {
            Ok(value) => value,
            Err(status) => return status,
        };

        match response {
            Ok(response) => unsafe { write_order_response(out, &response) },
            Err(error) => {
                set_last_error(client, error.to_string());
                status_from_error(&error)
            }
        }
    });

    result.unwrap_or(PMStatus::Panic)
}

/// Submit a limit order.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_limit_order(
    client: *mut PMClient,
    token_id: *const c_char,
    side: PMSide,
    price: *const c_char,
    size: *const c_char,
    out: *mut PMOrderResponse,
) -> PMStatus {
    let result = std::panic::catch_unwind(|| {
        let status = ensure_client(client);
        if status != PMStatus::Ok {
            return status;
        }
        if out.is_null() {
            return PMStatus::NullPointer;
        }

        let token_id = match unsafe { required_cstr(token_id) }.and_then(parse_u256) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let side = match map_side(side) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let price = match unsafe { required_cstr(price) }.and_then(parse_decimal) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let size = match unsafe { required_cstr(size) }.and_then(parse_decimal) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let client_ref = match authenticated_client(client) {
            Ok(value) => value,
            Err(status) => return status,
        };
        let signer_ref = match signer(client) {
            Ok(value) => value,
            Err(status) => return status,
        };

        let response = match with_runtime(client, |runtime| {
            runtime.block_on(async {
                client_ref
                    .limit_order()
                    .token_id(token_id)
                    .side(side)
                    .price(price)
                    .size(size)
                    .build_sign_and_post(&signer_ref)
                    .await
            })
        }) {
            Ok(value) => value,
            Err(status) => return status,
        };

        match response {
            Ok(response) => unsafe { write_order_response(out, &response) },
            Err(error) => {
                set_last_error(client, error.to_string());
                status_from_error(&error)
            }
        }
    });

    result.unwrap_or(PMStatus::Panic)
}

/// Cancel an order by Polymarket order id.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pm_cancel_order(
    client: *mut PMClient,
    order_id: *const c_char,
    out: *mut PMCancelResponse,
) -> PMStatus {
    let result = std::panic::catch_unwind(|| {
        let status = ensure_client(client);
        if status != PMStatus::Ok {
            return status;
        }
        if out.is_null() {
            return PMStatus::NullPointer;
        }

        let order_id = match unsafe { required_cstr(order_id) } {
            Ok(value) => value,
            Err(status) => return status,
        };
        let client_ref = match authenticated_client(client) {
            Ok(value) => value,
            Err(status) => return status,
        };

        let response = match with_runtime(client, |runtime| {
            runtime.block_on(async { client_ref.cancel_order(order_id).await })
        }) {
            Ok(value) => value,
            Err(status) => return status,
        };

        match response {
            Ok(response) => unsafe { write_cancel_response(out, &response) },
            Err(error) => {
                set_last_error(client, error.to_string());
                status_from_error(&error)
            }
        }
    });

    result.unwrap_or(PMStatus::Panic)
}
