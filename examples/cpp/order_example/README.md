# C++ C API order example

This example shows how to submit Polymarket CLOB orders from C++ through the Rust C API.

## Build the Rust C API library

From the repository root:

```powershell
cargo build --release --features c-api
```

On Windows this produces files under `target/release`, including an import library and DLL for `polymarket_client_sdk_v2`.

## Configure and build this example

From the repository root on Windows PowerShell:

```powershell
cmake -S examples/cpp/order_example -B build/cpp-order-example `
  -DPM_C_API_LIB_DIR="$PWD/target/release" `
  -DPM_C_API_LIB_NAME="polymarket_client_sdk_v2" `
  -DPM_C_API_DLL="$PWD/target/release/polymarket_client_sdk_v2.dll"

cmake --build build/cpp-order-example --config Release
```

If you link the static library instead of the DLL import library, omit `PM_C_API_DLL`.

## Run

Set the private key and token id through environment variables:

```powershell
$env:POLYMARKET_PRIVATE_KEY = "0xyour_private_key_here"
$env:POLYMARKET_TOKEN_ID = "your_uint256_token_id_here"
$env:POLYMARKET_CLOB_HOST = "https://clob.polymarket.com/"
$env:POLYMARKET_CHAIN_ID = "137"

.\build\cpp-order-example\Release\order_example.exe
```

By default the example submits a limit buy order with price `0.50` and size `5.00`.

To submit a market buy order instead:

```powershell
.\build\cpp-order-example\Release\order_example.exe $env:POLYMARKET_TOKEN_ID market
```

For market orders, the current C API convention is:

- `BUY`: `amount` is USDC.
- `SELL`: `amount` is shares.

Review `main.cpp` before running with a real private key or production account.
