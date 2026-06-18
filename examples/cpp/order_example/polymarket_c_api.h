#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Opaque client handle created by pm_client_create and destroyed by pm_client_destroy. */
typedef struct PMClient PMClient;

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
    PMClient** out_client
);

void pm_client_destroy(PMClient* client);

PMStatus pm_client_last_error(
    PMClient* client,
    char* out_buffer,
    size_t out_buffer_len
);

PMStatus pm_client_clear_error(PMClient* client);

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
