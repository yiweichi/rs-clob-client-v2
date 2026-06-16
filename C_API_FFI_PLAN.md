# C ABI / FFI Integration Plan for C++

This document describes a staged plan for exposing a minimal C ABI layer from `polymarket_client_sdk_v2` so it can be called from a C++ market adapter such as `PolyMarketMarket`.

## Goal

Expose a small, stable, C-compatible interface over the Rust SDK.

The C++ side should not depend on Rust types, async details, builders, generics, `Decimal`, `U256`, `Result`, or serde models. Rust should keep using the existing SDK internally, while the C ABI boundary remains simple and stable.

Recommended initial scope:

1. Create/authenticate a Polymarket CLOB client.
2. Submit market orders.
3. Submit limit orders.
4. Cancel orders.
5. Retrieve last error text.
6. Later: add query order/trades/balances/instruments.
7. Later: add WebSocket streaming using a poll/queue model, not direct cross-language callbacks.

## Why a C ABI Layer

The existing SDK is Rust-native and async-first. C++ cannot directly call Rust async APIs, generic builders, or Rust-owned data safely without an explicit ABI boundary.

A C ABI layer gives C++ a stable interface:

- Opaque client handles.
- Plain integer status codes.
- C strings for input.
- Fixed-size output structs or caller-provided buffers.
- Explicit destroy/free functions where needed.

## Alternative: Rust Sidecar / Gateway

A C ABI library is not the only option.

A Rust sidecar process is often better for WebSocket-heavy or stateful integrations:

```text
C++ PolyMarketMarket
    |
    | TCP / UDS / gRPC / shared memory
    v
Rust Polymarket Gateway
    |
    | native Rust async SDK
    v
Polymarket CLOB / WebSocket APIs
```

Sidecar advantages:

- Cleaner async runtime ownership.
- Easier WebSocket reconnect/resubscribe logic.
- Crash isolation.
- No C ABI compatibility issues for streaming models.
- Easier operational debugging.

C ABI advantages:

- Lower call overhead.
- No extra process.
- Easier deployment for simple request/response calls.

Recommended approach: start with C ABI for synchronous order operations. Re-evaluate sidecar once WebSocket streaming and recovery become complex.

## Cargo Feature and Library Type

Add an optional feature:

```toml
[features]
default = ["clob"]
c-api = ["clob"]
```

Configure library outputs as needed:

```toml
[lib]
name = "polymarket_client_sdk_v2"
crate-type = ["rlib", "cdylib", "staticlib"]
```

Possible outputs:

- `rlib`: normal Rust library.
- `cdylib`: dynamic library for C/C++.
- `staticlib`: static library for C/C++.

## Suggested Source Layout

```text
src/
  lib.rs
  c_api/
    mod.rs
    types.rs
    client.rs
    orders.rs
    error.rs
```

In `src/lib.rs`:

```rust
#[cfg(feature = "c-api")]
pub mod c_api;
```

## ABI Design Rules

### 1. Use Opaque Handles

C/C++ header:

```c
typedef struct PMClient PMClient;
```

Rust owns the real object:

```rust
pub struct PMClient {
    runtime: tokio::runtime::Runtime,
    // authenticated client, signer, config, last_error, etc.
}
```

C++ only stores `PMClient*`.

### 2. Return Status Codes

Do not expose Rust `Result` across FFI.

```c
typedef enum PMStatus {
    PM_OK = 0,
    PM_ERR_NULL = 1,
    PM_ERR_INVALID_ARG = 2,
    PM_ERR_UTF8 = 3,
    PM_ERR_AUTH = 4,
    PM_ERR_NETWORK = 5,
    PM_ERR_PARSE = 6,
    PM_ERR_BUFFER_TOO_SMALL = 7,
    PM_ERR_PANIC = 100,
    PM_ERR_INTERNAL = 101
} PMStatus;
```

### 3. Pass Complex Numeric Values as Strings

Use strings for values that are Rust-specific or precision-sensitive:

- `token_id`: `const char*`
- `price`: `const char*`
- `size`: `const char*`
- `amount`: `const char*`
- `order_id`: `const char*`

Rust parses them into `U256`, `Decimal`, SDK types, etc.

### 4. Avoid Rust-Allocated Strings in v1

Prefer caller-provided buffers or fixed-size output structs.

Example:

```c
PMStatus pm_client_last_error(PMClient* client, char* buf, size_t buf_len);
```

or:

```c
typedef struct PMOrderResponse {
    char order_id[128];
    char status[64];
    char raw_json[4096];
} PMOrderResponse;
```

### 5. Prevent Panic Across FFI

Every `extern "C"` function should catch panics and convert them to `PM_ERR_PANIC` or `PM_ERR_INTERNAL`.

Rust panics must not unwind into C++.

### 6. Hide Async Runtime Internals

The Rust side should own a Tokio runtime and call SDK futures via `block_on` for the first version.

Do not create a new runtime per request.

## Proposed C ABI v1

### Lifecycle

```c
PMStatus pm_client_create(
    const char* host,
    const char* private_key,
    uint64_t chain_id,
    PMClient** out_client
);

void pm_client_destroy(PMClient* client);

PMStatus pm_client_last_error(
    PMClient* client,
    char* buf,
    size_t buf_len
);
```

`pm_client_create` may authenticate immediately, or authentication can be split into:

```c
PMStatus pm_client_authenticate(PMClient* client);
```

For the first version, creating and authenticating in one call is simpler.

### Enums

```c
typedef enum PMSide {
    PM_SIDE_BUY = 1,
    PM_SIDE_SELL = 2
} PMSide;

typedef enum PMOrderType {
    PM_ORDER_TYPE_GTC = 1,
    PM_ORDER_TYPE_FOK = 2,
    PM_ORDER_TYPE_FAK = 3
} PMOrderType;
```

The final enum mapping must match the Polymarket SDK's supported order types.

### Market Order

```c
PMStatus pm_market_order(
    PMClient* client,
    const char* token_id,
    PMSide side,
    const char* amount,
    PMOrderType order_type,
    PMOrderResponse* out
);
```

For a market buy, `amount` can mean USDC notional if following the SDK's `Amount::usdc(...)` API. Make this explicit in the header comments.

### Limit Order

```c
PMStatus pm_limit_order(
    PMClient* client,
    const char* token_id,
    PMSide side,
    const char* price,
    const char* size,
    PMOrderResponse* out
);
```

### Cancel Order

```c
typedef struct PMCancelResponse {
    char order_id[128];
    char status[64];
    char raw_json[4096];
} PMCancelResponse;

PMStatus pm_cancel_order(
    PMClient* client,
    const char* order_id,
    PMCancelResponse* out
);
```

### Query APIs for Later

Possible future functions:

```c
PMStatus pm_get_order_json(PMClient* client, const char* order_id, char* out_json, size_t out_len);
PMStatus pm_get_balance_json(PMClient* client, char* out_json, size_t out_len);
PMStatus pm_get_open_orders_json(PMClient* client, char* out_json, size_t out_len);
PMStatus pm_get_trades_json(PMClient* client, char* out_json, size_t out_len);
```

Returning JSON is a pragmatic early option because it keeps the ABI small and lets C++ decide how much to parse.

## WebSocket Plan

Do not expose WebSocket events by directly calling C++ callbacks from Rust in the first version.

Recommended model:

1. Rust WebSocket task runs inside the Rust-owned Tokio runtime.
2. Incoming events are parsed or stored as raw JSON.
3. Events are pushed into a Rust-owned thread-safe queue.
4. C++ periodically polls the queue via C ABI.

Example C ABI:

```c
typedef enum PMWsEventType {
    PM_WS_EVENT_NONE = 0,
    PM_WS_EVENT_ORDERBOOK = 1,
    PM_WS_EVENT_PRICE = 2,
    PM_WS_EVENT_USER_ORDER = 3,
    PM_WS_EVENT_USER_TRADE = 4,
    PM_WS_EVENT_ERROR = 100
} PMWsEventType;

typedef struct PMWsEvent {
    PMWsEventType type;
    char channel[64];
    char raw_json[8192];
} PMWsEvent;

PMStatus pm_ws_start_user_stream(PMClient* client);
PMStatus pm_ws_subscribe_orderbook(PMClient* client, const char* token_id);
PMStatus pm_ws_poll_event(PMClient* client, PMWsEvent* out);
PMStatus pm_ws_stop(PMClient* client);
```

If no event is available, `pm_ws_poll_event` returns `PM_OK` with `type = PM_WS_EVENT_NONE`, or a dedicated status such as `PM_ERR_WOULD_BLOCK`.

## Why Poll/Queue Instead of Direct Callback

Direct Rust-to-C++ callbacks across FFI are possible, but more fragile:

- Rust async tasks run on Tokio worker threads, not necessarily C++ market threads.
- C++ callbacks may touch non-thread-safe market state.
- Callback lifetime is hard: Rust must know whether the C++ function pointer and user data are still valid.
- Shutdown ordering is tricky: a callback may fire while C++ is destroying the market object.
- Exceptions must not cross FFI; C++ callback exceptions would be undefined behavior if they unwind into Rust.
- Backpressure is unclear: if C++ is slow, Rust callback execution can block WebSocket receive tasks.
- Reentrancy is dangerous: C++ might call back into Rust while Rust is already holding locks.

A poll/queue model is easier:

- Rust owns WebSocket receive tasks.
- Rust only pushes events into a queue.
- C++ pulls events when it is safe, from its own market loop.
- Threading is explicit.
- Shutdown is easier.
- Backpressure can be controlled with queue size/drop policy.

This matches market adapter loops well: `PolyMarketMarket::Process()` or `BGThreadProcess()` can call `pm_ws_poll_event()` and then translate events into `OnExchangeAck`, `OnExec`, `OnCanceled`, etc.

## Integration with `PolyMarketMarket`

C++ `PolyMarketMarket` should use the C ABI as a thin dependency:

- `DoLoad`: read host, key path/private key, chain id, signature type, symbol/token mapping config.
- `Connect`: call `pm_client_create`, optionally query initial state, start WS streams later.
- `Send`: translate `IShmOrder` to `pm_market_order` or `pm_limit_order`.
- `Cancel`: call `pm_cancel_order`.
- `Process`: poll Rust WS queue if enabled, translate events into framework callbacks.
- `ReqQryOrder`: call query API or return cached/order JSON in later phases.
- `ReqQryTrade`: call query API or use WS/user-event cache in later phases.
- `ReqQryAccount`: query balances/allowances.
- `ReqQryInstrument`: build `FeedCode -> token_id` mappings from Gamma/CLOB metadata.

## Phase Plan

### Phase 0: Confirm Scope

Confirm which operations C++ needs first:

1. market buy
2. limit order
3. cancel order
4. last error
5. optional query order/balance

Do not include WebSocket in phase 1 unless absolutely required.

### Phase 1: Build Configuration

- Add `c-api` feature.
- Add `cdylib` and/or `staticlib` crate type.
- Add `src/c_api/mod.rs` gated behind the feature.

### Phase 2: ABI Types and Header

- Define `PMStatus`, `PMSide`, `PMOrderType`.
- Define opaque `PMClient`.
- Define response structs.
- Add `include/polymarket_client_c_api.h` manually or with `cbindgen` later.

### Phase 3: Client Lifecycle

- Implement `pm_client_create`.
- Implement `pm_client_destroy`.
- Implement `pm_client_last_error`.
- Ensure null pointer checks, UTF-8 checks, panic catching, and stable error mapping.

### Phase 4: Market Order

- Implement `pm_market_order`.
- Match the SDK market order example first.
- Return order id/status/raw JSON if available.

### Phase 5: Limit Order

- Implement `pm_limit_order`.
- Parse price and size strings into `Decimal`.
- Return order id/status/raw JSON.

### Phase 6: Cancel Order

- Implement `pm_cancel_order`.
- Map SDK success/error to `PMCancelResponse` and `PMStatus`.

### Phase 7: C++ Smoke Test

Build a minimal C++ executable that:

1. creates the client,
2. sends a small test order or dry-run/sandbox order if available,
3. cancels it if needed,
4. prints response and last error,
5. destroys the client.

### Phase 8: `PolyMarketMarket` Integration

- Add a C++ wrapper class around `PMClient*`.
- Implement `Send` and `Cancel` in `PolyMarketMarket`.
- Translate C ABI responses into `IShmMarket` callbacks.

### Phase 9: Query APIs

Add query APIs once order flow works:

- account/balance
- open orders
- order detail
- trades
- instruments/markets

### Phase 10: WebSocket Poll Queue

If WebSocket is needed inside the C ABI library:

- Start Rust WS tasks in the runtime.
- Push incoming events to a bounded queue.
- Expose `pm_ws_poll_event` for C++.
- Process polled events in `PolyMarketMarket::Process()`.

Re-evaluate using a Rust sidecar before committing to complex WebSocket FFI.

## Key Risks

1. Async runtime ownership.
2. ABI stability.
3. Rust/C++ memory ownership.
4. Panic crossing FFI.
5. Decimal and integer precision.
6. WebSocket lifecycle and shutdown ordering.
7. Thread safety of `PMClient*`.
8. Error observability.

## Recommended First Deliverable

A minimal C ABI dynamic library exposing:

- `pm_client_create`
- `pm_client_destroy`
- `pm_client_last_error`
- `pm_market_order`
- `pm_limit_order`
- `pm_cancel_order`

plus a small C++ smoke test.
