#include "PolymarketCApi.h"

#include <cstdlib>
#include <cstring>
#include <iostream>
#include <string>

namespace {

const char* status_to_string(PMStatus status) {
    switch (status) {
        case PM_STATUS_OK:
            return "Ok";
        case PM_STATUS_NULL_POINTER:
            return "NullPointer";
        case PM_STATUS_INVALID_ARGUMENT:
            return "InvalidArgument";
        case PM_STATUS_AUTHENTICATION_ERROR:
            return "AuthenticationError";
        case PM_STATUS_NETWORK_ERROR:
            return "NetworkError";
        case PM_STATUS_INTERNAL_ERROR:
            return "InternalError";
        case PM_STATUS_PANIC:
            return "Panic";
        default:
            return "Unknown";
    }
}

const char* default_mode() {
    return "market-data";
}

void print_usage() {
    std::cerr << "Usage: order_example [market-data|limit-order|market-order|cancel-all] [token_id]\n";
    std::cerr << "\n";
    std::cerr << "Modes:\n";
    std::cerr << "  market-data   Run unauthenticated market data C API smoke tests. This is the default.\n";
    std::cerr << "  limit-order   Submit a limit buy order. Requires POLYMARKET_PRIVATE_KEY.\n";
    std::cerr << "  market-order  Submit a market buy order. Requires POLYMARKET_PRIVATE_KEY.\n";
    std::cerr << "  cancel-all    Cancel every open order. Requires POLYMARKET_PRIVATE_KEY.\n";
}

void print_last_error(PMClient* client) {
    if (client == nullptr) {
        return;
    }

    char error_buffer[2048] = {};
    const PMStatus status = pm_client_last_error(
        client,
        error_buffer,
        sizeof(error_buffer)
    );

    if (status == PM_STATUS_OK && std::strlen(error_buffer) > 0) {
        std::cerr << "last error: " << error_buffer << '\n';
    }
}

void print_market_last_error(PMMarketClient* client) {
    if (client == nullptr) {
        return;
    }

    char error_buffer[2048] = {};
    const PMStatus status = pm_market_client_last_error(
        client,
        error_buffer,
        sizeof(error_buffer)
    );

    if (status == PM_STATUS_OK && std::strlen(error_buffer) > 0) {
        std::cerr << "last market error: " << error_buffer << '\n';
    }
}

const char* env_or_default(const char* name, const char* fallback) {
    const char* value = std::getenv(name);
    return value == nullptr || std::strlen(value) == 0 ? fallback : value;
}

const char* resolve_token_id(int argc, char** argv) {
    if (argc >= 3 && std::strlen(argv[2]) > 0) {
        return argv[2];
    }

    const char* token_id = std::getenv("POLYMARKET_TOKEN_ID");
    if (token_id != nullptr && std::strlen(token_id) > 0) {
        return token_id;
    }

    return nullptr;
}

bool print_market_value(
    const char* label,
    PMStatus status,
    const PMMarketResponse& response,
    PMMarketClient* client
) {
    if (status != PM_STATUS_OK) {
        std::cerr << label << " failed: " << status_to_string(status) << '\n';
        print_market_last_error(client);
        return false;
    }

    std::cout << label << ": " << response.value << '\n';
    std::cout << label << " raw_json: " << response.raw_json << '\n';
    return true;
}

bool run_market_data_smoke_test(const char* host, const char* token_id) {
    PMMarketClient* client = nullptr;
    PMStatus status = pm_market_client_create(host, &client);
    if (status != PM_STATUS_OK) {
        std::cerr << "pm_market_client_create failed: " << status_to_string(status) << '\n';
        return false;
    }

    bool ok = true;

    uint64_t server_time = 0;
    status = pm_get_server_time(client, &server_time);
    if (status != PM_STATUS_OK) {
        std::cerr << "pm_get_server_time failed: " << status_to_string(status) << '\n';
        print_market_last_error(client);
        ok = false;
    } else {
        std::cout << "server_time: " << server_time << '\n';
    }

    if (token_id == nullptr) {
        std::cerr << "Skipping token-specific market data tests because POLYMARKET_TOKEN_ID is not set and no token_id argument was provided.\n";
        pm_market_client_destroy(client);
        return ok;
    }

    PMMarketResponse market_response = {};
    ok = print_market_value(
        "best_buy_price",
        pm_get_price(client, token_id, PM_SIDE_BUY, &market_response),
        market_response,
        client
    ) && ok;

    market_response = {};
    ok = print_market_value(
        "best_sell_price",
        pm_get_price(client, token_id, PM_SIDE_SELL, &market_response),
        market_response,
        client
    ) && ok;

    market_response = {};
    ok = print_market_value(
        "spread",
        pm_get_spread(client, token_id, &market_response),
        market_response,
        client
    ) && ok;

    market_response = {};
    ok = print_market_value(
        "midpoint",
        pm_get_midpoint(client, token_id, &market_response),
        market_response,
        client
    ) && ok;

    market_response = {};
    ok = print_market_value(
        "last_trade_price",
        pm_get_last_trade_price(client, token_id, &market_response),
        market_response,
        client
    ) && ok;

    PMOrderBookResponse orderbook_response = {};
    status = pm_get_orderbook(client, token_id, &orderbook_response);
    if (status != PM_STATUS_OK) {
        std::cerr << "pm_get_orderbook failed: " << status_to_string(status) << '\n';
        print_market_last_error(client);
        ok = false;
    } else {
        std::cout << "orderbook_hash: " << orderbook_response.hash << '\n';
        std::cout << "orderbook raw_json: " << orderbook_response.raw_json << '\n';
    }

    pm_market_client_destroy(client);
    return ok;
}

bool submit_limit_order(PMClient* client, const char* token_id) {
    PMOrderResponse response = {};

    const PMStatus status = pm_limit_order(
        client,
        token_id,
        PM_SIDE_BUY,
        "0.50",
        "5.00",
        &response
    );

    if (status != PM_STATUS_OK) {
        std::cerr << "pm_limit_order failed: " << status_to_string(status) << '\n';
        print_last_error(client);
        return false;
    }

    std::cout << "limit order submitted\n";
    std::cout << "order_id: " << response.order_id << '\n';
    std::cout << "status: " << response.status << '\n';
    std::cout << "raw_json: " << response.raw_json << '\n';
    return true;
}

bool submit_market_order(PMClient* client, const char* token_id) {
    PMOrderResponse response = {};

    const PMStatus status = pm_market_order(
        client,
        token_id,
        PM_SIDE_BUY,
        "1.00",
        PM_ORDER_TYPE_FAK,
        &response
    );

    if (status != PM_STATUS_OK) {
        std::cerr << "pm_market_order failed: " << status_to_string(status) << '\n';
        print_last_error(client);
        return false;
    }

    std::cout << "market order submitted\n";
    std::cout << "order_id: " << response.order_id << '\n';
    std::cout << "status: " << response.status << '\n';
    std::cout << "raw_json: " << response.raw_json << '\n';
    return true;
}

bool cancel_all_orders(PMClient* client) {
    PMCancelResponse response = {};
    const PMStatus status = pm_cancel_all_orders(client, &response);
    if (status != PM_STATUS_OK) {
        std::cerr << "pm_cancel_all_orders failed: " << status_to_string(status) << '\n';
        print_last_error(client);
        return false;
    }

    std::cout << "pm_cancel_all_orders succeeded\n";
    std::cout << "status: " << response.status << '\n';
    std::cout << "raw_json: " << response.raw_json << '\n';
    return true;
}

bool run_authenticated_action(
    const char* mode,
    const char* host,
    const char* private_key,
    uint64_t chain_id,
    const char* token_id
) {
    if (private_key == nullptr || std::strlen(private_key) == 0) {
        std::cerr << "Missing POLYMARKET_PRIVATE_KEY environment variable\n";
        return false;
    }

    if ((std::strcmp(mode, "limit-order") == 0 || std::strcmp(mode, "market-order") == 0)
        && token_id == nullptr) {
        std::cerr << "Missing POLYMARKET_TOKEN_ID environment variable or token_id argument\n";
        return false;
    }

    PMClient* client = nullptr;
    PMStatus status = pm_client_create(host, private_key, chain_id, &client);
    if (status != PM_STATUS_OK) {
        std::cerr << "pm_client_create failed: " << status_to_string(status) << '\n';
        print_last_error(client);
        return false;
    }

    bool ok = false;
    if (std::strcmp(mode, "limit-order") == 0) {
        ok = submit_limit_order(client, token_id);
    } else if (std::strcmp(mode, "market-order") == 0) {
        ok = submit_market_order(client, token_id);
    } else if (std::strcmp(mode, "cancel-all") == 0) {
        ok = cancel_all_orders(client);
    } else {
        std::cerr << "Unknown authenticated mode: " << mode << '\n';
        ok = false;
    }

    pm_client_destroy(client);
    return ok;
}

} // namespace

int main(int argc, char** argv) {
    const char* mode = argc >= 2 ? argv[1] : default_mode();
    const char* host = env_or_default("POLYMARKET_CLOB_HOST", "https://clob.polymarket.com/");
    const char* token_id = resolve_token_id(argc, argv);

    const char* chain_id_env = std::getenv("POLYMARKET_CHAIN_ID");
    const uint64_t chain_id = (chain_id_env == nullptr || std::strlen(chain_id_env) == 0)
        ? 137
        : std::strtoull(chain_id_env, nullptr, 10);

    if (std::strcmp(mode, "market-data") == 0) {
        return run_market_data_smoke_test(host, token_id) ? 0 : 1;
    }

    if (std::strcmp(mode, "limit-order") == 0
        || std::strcmp(mode, "market-order") == 0
        || std::strcmp(mode, "cancel-all") == 0) {
        return run_authenticated_action(
            mode,
            host,
            std::getenv("POLYMARKET_PRIVATE_KEY"),
            chain_id,
            token_id
        ) ? 0 : 1;
    }

    print_usage();
    return 1;
}
