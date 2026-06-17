# C++ C API order example

This example shows how to submit Polymarket CLOB orders from C++ through the Rust C API.

## Build the Rust C API library

From the repository root:

```powershell
cargo build --release --features c-api
```

On Windows this produces files under `target/release`, including an import library and DLL for `polymarket_client_sdk_v2`.

## Configure and build this example

From the repository root on Windows PowerShell, prefer linking against the DLL import library and emit the executable into this example directory:

```powershell
cl /EHsc /MD examples\cpp\order_example\main.cpp /I examples\cpp\order_example /DPM_C_API_DLL /Fe:examples\cpp\order_example\order_example.exe /link target\release\polymarket_client_sdk_v2.dll.lib
```

Copy `target\release\polymarket_client_sdk_v2.dll` next to `examples\cpp\order_example\order_example.exe`, or add `target\release` to `PATH` before running.

If you intentionally link the Rust static library instead of the DLL import library, omit `PM_C_API_DLL` and add the native Windows import libraries required by Rust and its dependencies:

```powershell
cl /EHsc /MD examples\cpp\order_example\main.cpp /I examples\cpp\order_example /Fe:examples\cpp\order_example\order_example.exe /link target\release\polymarket_client_sdk_v2.lib userenv.lib ntdll.lib advapi32.lib bcrypt.lib ws2_32.lib user32.lib shell32.lib ole32.lib crypt32.lib secur32.lib ncrypt.lib
```

## Run

The commands above produce `examples\cpp\order_example\order_example.exe`.

In PowerShell, set the environment variables first, then run the executable as a separate command:

```powershell
$env:POLYMARKET_PRIVATE_KEY = "0xyour_private_key_here"
$env:POLYMARKET_TOKEN_ID = "your_uint256_token_id_here"
$env:POLYMARKET_CLOB_HOST = "https://clob.polymarket.com/"
$env:POLYMARKET_CHAIN_ID = "137"
```

Then run:

```powershell
.\examples\cpp\order_example\order_example.exe
```

If you use Command Prompt instead of PowerShell, use quoted `set` assignments so spaces around `&&` are not included in the environment variable values:

```cmd
set "POLYMARKET_PRIVATE_KEY=0xyour_private_key_here" && set "POLYMARKET_TOKEN_ID=your_uint256_token_id_here" && set "POLYMARKET_CLOB_HOST=https://clob.polymarket.com/" && set "POLYMARKET_CHAIN_ID=137" && examples\cpp\order_example\order_example.exe
```

By default the example submits a limit buy order with price `0.50` and size `5.00`.

To submit a market buy order instead from PowerShell:

```powershell
.\examples\cpp\order_example\order_example.exe $env:POLYMARKET_TOKEN_ID market
```

Or from Command Prompt:

```cmd
examples\cpp\order_example\order_example.exe %POLYMARKET_TOKEN_ID% market
```

For market orders, the current C API convention is:

- `BUY`: `amount` is USDC.
- `SELL`: `amount` is shares.

Review `main.cpp` before running with a real private key or production account.
