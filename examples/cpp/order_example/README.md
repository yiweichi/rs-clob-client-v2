# C++ C API order example

This example shows how to submit Polymarket CLOB orders from C++ through the Rust C API.

## Build the Rust C API library

From the repository root:

```powershell
cargo build --release --features c-api
```

On Windows this produces files under `target/release`, including an import library and DLL for `polymarket_client_sdk_v2`.

## Configure and build this example

From the repository root on Windows PowerShell, first create the output directory, then link against the DLL import library and emit the executable into that directory:

```powershell
mkdir examples\cpp\order_example\bin
cl /EHsc /MD examples\cpp\order_example\main.cpp /I examples\cpp\order_example /DPM_C_API_DLL /Fe:examples\cpp\order_example\bin\order_example.exe /link target\release\polymarket_client_sdk_v2.dll.lib
```

Copy `target\release\polymarket_client_sdk_v2.dll` next to `examples\cpp\order_example\bin\order_example.exe`, or add `target\release` to `PATH` before running.

If you intentionally link the Rust static library instead of the DLL import library, omit `PM_C_API_DLL`, create the same output directory first, and add the native Windows import libraries required by Rust and its dependencies:

```powershell
mkdir examples\cpp\order_example\bin
cl /EHsc /MD examples\cpp\order_example\main.cpp /I examples\cpp\order_example /Fe:examples\cpp\order_example\bin\order_example.exe /link target\release\polymarket_client_sdk_v2.lib userenv.lib ntdll.lib advapi32.lib bcrypt.lib ws2_32.lib user32.lib shell32.lib ole32.lib crypt32.lib secur32.lib ncrypt.lib
```

## Run

The commands above produce `examples\cpp\order_example\bin\order_example.exe`.

The executable has separate modes so safe market data checks are isolated from authenticated order actions:

- `market-data`: run unauthenticated market data C API smoke tests. This is the default.
- `limit-order`: submit a limit buy order with price `0.50` and size `5.00`.
- `market-order`: submit a market buy order with amount `1.00` USDC.
- `cancel-all`: cancel every open order belonging to the authenticated account.

### Market data smoke test

This mode does not require a private key. Set a token id to also run token-specific checks; without a token id, it only calls `pm_get_server_time`.

PowerShell:

```powershell
$env:POLYMARKET_TOKEN_ID = "your_uint256_token_id_here"
$env:POLYMARKET_CLOB_HOST = "https://clob.polymarket.com/"
.\examples\cpp\order_example\bin\order_example.exe market-data
```

Command Prompt:

```cmd
set "POLYMARKET_TOKEN_ID=your_uint256_token_id_here" && set "POLYMARKET_CLOB_HOST=https://clob.polymarket.com/" && examples\cpp\order_example\bin\order_example.exe market-data
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

```powershell
$env:POLYMARKET_PRIVATE_KEY = "0xyour_private_key_here"
$env:POLYMARKET_TOKEN_ID = "your_uint256_token_id_here"
$env:POLYMARKET_CLOB_HOST = "https://clob.polymarket.com/"
$env:POLYMARKET_CHAIN_ID = "137"
```

Submit a limit buy order:

```powershell
.\examples\cpp\order_example\bin\order_example.exe limit-order
```

Submit a market buy order:

```powershell
.\examples\cpp\order_example\bin\order_example.exe market-order
```

Cancel all open orders:

```powershell
.\examples\cpp\order_example\bin\order_example.exe cancel-all
```

If you use Command Prompt instead of PowerShell, use quoted `set` assignments so spaces around `&&` are not included in the environment variable values:

```cmd
set "POLYMARKET_PRIVATE_KEY=0xyour_private_key_here" && set "POLYMARKET_TOKEN_ID=your_uint256_token_id_here" && set "POLYMARKET_CLOB_HOST=https://clob.polymarket.com/" && set "POLYMARKET_CHAIN_ID=137" && examples\cpp\order_example\bin\order_example.exe limit-order
```

For market orders, the current C API convention is:

- `BUY`: `amount` is USDC.
- `SELL`: `amount` is shares.

Review `main.cpp` before running with a real private key or production account.
