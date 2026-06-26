#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Opaque authenticated trading client handle created by pm_client_create and destroyed by pm_client_destroy. */
typedef struct PMClient PMClient;

/** Opaque unauthenticated market data client handle created by pm_market_client_create. */
typedef struct PMMarketClient PMMarketClient;

typedef enum PMStatus {
    PM_STATUS_OK = 0,
    PM_STATUS_NULL_POINTER = 1,
    PM_STATUS_INVALID_ARGUMENT = 2,
    PM_STATUS_AUTHENTICATION_ERROR = 3,
    PM_STATUS_NETWORK_ERROR = 4,
    PM_STATUS_INTERNAL_ERROR = 100,
    PM_STATUS_PANIC = 101,
} PMStatus;

typedef enum PMSide {
    PM_SIDE_BUY = 1,
    PM_SIDE_SELL = 2,
} PMSide;

typedef enum PMOrderType {
    PM_ORDER_TYPE_GTC = 1,
    PM_ORDER_TYPE_FOK = 2,
    PM_ORDER_TYPE_FAK = 3,
} PMOrderType;

typedef struct PMMarketResponse {
    char value[128];
    char raw_json[4096];
} PMMarketResponse;

typedef struct PMOrderBookResponse {
    char hash[128];
    char raw_json[4096];
} PMOrderBookResponse;

typedef struct PMOrderResponse {
    char order_id[128];
    char status[64];
    char raw_json[4096];
} PMOrderResponse;

typedef struct PMCancelResponse {
    char status[64];
    char raw_json[4096];
} PMCancelResponse;

PMStatus pm_client_create(
    const char* host,
    const char* private_key,
    uint64_t chain_id,
    const char* funder,
    PMClient** out_client
);

void pm_client_destroy(PMClient* client);

PMStatus pm_client_last_error(
    PMClient* client,
    char* out_buffer,
    size_t out_buffer_len
);

PMStatus pm_client_clear_error(PMClient* client);

PMStatus pm_market_client_create(
    const char* host,
    PMMarketClient** out_client
);

void pm_market_client_destroy(PMMarketClient* client);

PMStatus pm_market_client_last_error(
    PMMarketClient* client,
    char* out_buffer,
    size_t out_buffer_len
);

PMStatus pm_get_server_time(
    PMMarketClient* client,
    uint64_t* out_timestamp
);

PMStatus pm_get_price(
    PMMarketClient* client,
    const char* token_id,
    PMSide side,
    PMMarketResponse* out
);

PMStatus pm_get_spread(
    PMMarketClient* client,
    const char* token_id,
    PMMarketResponse* out
);

PMStatus pm_get_midpoint(
    PMMarketClient* client,
    const char* token_id,
    PMMarketResponse* out
);

PMStatus pm_get_last_trade_price(
    PMMarketClient* client,
    const char* token_id,
    PMMarketResponse* out
);

PMStatus pm_get_orderbook(
    PMMarketClient* client,
    const char* token_id,
    PMOrderBookResponse* out
);

PMStatus pm_market_order(
    PMClient* client,
    const char* token_id,
    PMSide side,
    const char* amount,
    PMOrderType order_type,
    PMOrderResponse* out
);

PMStatus pm_limit_order(
    PMClient* client,
    const char* token_id,
    PMSide side,
    const char* price,
    const char* size,
    PMOrderResponse* out
);

PMStatus pm_cancel_order(
    PMClient* client,
    const char* order_id,
    PMCancelResponse* out
);

PMStatus pm_cancel_all_orders(
    PMClient* client,
    PMCancelResponse* out
);

#ifdef __cplusplus
}
#endif
