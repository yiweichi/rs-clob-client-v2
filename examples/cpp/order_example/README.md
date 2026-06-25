# C++ C API order example

This example shows how to submit Polymarket CLOB orders from C++ through the Rust C API on Linux.

## Build the Rust C API library with SONAME

From the repository root, build the Rust C API shared library and explicitly set its SONAME so downstream ELF binaries record a stable dependency name instead of an absolute path:

```bash
RUSTFLAGS="-C link-arg=-Wl,-soname,libpolymarket_client_sdk_v2.so" cargo build --release --features c-api
```

## Configure and build this example on Linux

From the repository root:

```bash
mkdir -p examples/cpp/order_example/bin
c++ -std=c++17 examples/cpp/order_example/main.cpp \
  -I examples/cpp/order_example \
  -L target/release \
  -lpolymarket_client_sdk_v2 \
  -Wl,-rpath,'$ORIGIN' \
  -Wl,-rpath,'$ORIGIN/../../../target/release' \
  -o examples/cpp/order_example/bin/order_example
```

If you want the executable to load the Rust shared library from the same directory first, copy the shared library next to the executable:

```bash
cp target/release/libpolymarket_client_sdk_v2.so examples/cpp/order_example/bin/
```

Then confirm the executable records the dependency by library name instead of an absolute path:

```bash
readelf -d examples/cpp/order_example/bin/order_example | grep NEEDED
```

The expected output should contain:

```text
Shared library: [libpolymarket_client_sdk_v2.so]
```

## Run

The commands above produce `examples/cpp/order_example/bin/order_example`.

The executable has separate modes so safe market data checks are isolated from authenticated order actions:

- `market-data`: run unauthenticated market data C API smoke tests. This is the default.
- `limit-order`: submit a limit buy order with price `0.50` and size `5.00`.
- `market-order`: submit a market buy order with amount `1.00` USDC.
- `cancel-all`: cancel every open order belonging to the authenticated account.

### Market data smoke test

This mode does not require a private key. Set a token id to also run token-specific checks; without a token id, it only calls `pm_get_server_time`.

```bash
export POLYMARKET_TOKEN_ID="your_uint256_token_id_here"
export POLYMARKET_CLOB_HOST="https://clob.polymarket.com/"
./examples/cpp/order_example/bin/order_example market-data
```

The market data mode exercises these unauthenticated C API functions:

- `pm_market_client_create` / `pm_market_client_destroy`
- `pm_get_server_time`
- `pm_get_price`
- `pm_get_spread`
- `pm_get_midpoint`
- `pm_get_last_trade_price`
- `pm_get_orderbook`

Use `PMMarketResponse` for single-value endpoints and `PMOrderBookResponse` for orderbook summaries.

### Authenticated order actions

Set the private key, token id, host, and chain id before running an authenticated mode:

```bash
export POLYMARKET_PRIVATE_KEY="0xyour_private_key_here"
export POLYMARKET_TOKEN_ID="your_uint256_token_id_here"
export POLYMARKET_CLOB_HOST="https://clob.polymarket.com/"
export POLYMARKET_CHAIN_ID="137"
```

Submit a limit buy order:

```bash
./examples/cpp/order_example/bin/order_example limit-order
```

Submit a market buy order:

```bash
./examples/cpp/order_example/bin/order_example market-order
```

Cancel all open orders:

```bash
./examples/cpp/order_example/bin/order_example cancel-all
```

For market orders, the current C API convention is:

- `BUY`: `amount` is USDC.
- `SELL`: `amount` is shares.

Review `main.cpp` before running with a real private key or production account.
