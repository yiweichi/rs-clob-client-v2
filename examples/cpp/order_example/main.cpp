#include "polymarket_c_api.h"

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

    // Current C API convention:
    // - BUY market order amount is denominated in USDC.
    // - SELL market order amount is denominated in shares.
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

} // namespace

int main(int argc, char** argv) {
    const char* private_key = std::getenv("POLYMARKET_PRIVATE_KEY");
    if (private_key == nullptr || std::strlen(private_key) == 0) {
        std::cerr << "Missing POLYMARKET_PRIVATE_KEY environment variable\n";
        return 1;
    }

    const char* token_id = std::getenv("POLYMARKET_TOKEN_ID");
    if (token_id == nullptr || std::strlen(token_id) == 0) {
        if (argc >= 2) {
            token_id = argv[1];
        } else {
            std::cerr << "Missing POLYMARKET_TOKEN_ID environment variable or token id argument\n";
            std::cerr << "Usage: order_example <token_id> [market]\n";
            return 1;
        }
    }

    const char* host = std::getenv("POLYMARKET_CLOB_HOST");
    if (host == nullptr || std::strlen(host) == 0) {
        host = "https://clob.polymarket.com/";
    }

    const char* chain_id_env = std::getenv("POLYMARKET_CHAIN_ID");
    const uint64_t chain_id = (chain_id_env == nullptr || std::strlen(chain_id_env) == 0)
        ? 137
        : std::strtoull(chain_id_env, nullptr, 10);

    PMClient* client = nullptr;
    PMStatus status = pm_client_create(host, private_key, chain_id, &client);
    if (status != PM_STATUS_OK) {
        std::cerr << "pm_client_create failed: " << status_to_string(status) << '\n';
        print_last_error(client);
        return 1;
    }

    const bool use_market_order = argc >= 3 && std::strcmp(argv[2], "market") == 0;
    const bool ok = use_market_order
        ? submit_market_order(client, token_id)
        : submit_limit_order(client, token_id);

    pm_client_destroy(client);
    return ok ? 0 : 1;
}
