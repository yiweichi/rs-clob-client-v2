cmake_minimum_required(VERSION 3.20)
project(polymarket_c_api_order_example LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

add_executable(order_example main.cpp)
target_include_directories(order_example PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})

# Pass the directory containing the Rust-generated import/static library with:
#   cmake -S examples/cpp/order_example -B build/cpp-order-example \
#     -DPM_C_API_LIB_DIR=/absolute/path/to/target/release
#
# On Windows this usually points at a directory containing a .lib file.
# On Linux/macOS it usually points at a directory containing a .so/.dylib/.a file.
set(PM_C_API_LIB_DIR "" CACHE PATH "Directory containing the Polymarket C API library")
set(PM_C_API_LIB_NAME "polymarket_client_sdk_v2" CACHE STRING "Polymarket C API library name without prefix/suffix")
set(PM_C_API_DLL "" CACHE FILEPATH "Optional path to the Polymarket C API DLL on Windows")

if(PM_C_API_LIB_DIR)
    find_library(
        PM_C_API_LIBRARY
        NAMES ${PM_C_API_LIB_NAME}
        PATHS ${PM_C_API_LIB_DIR}
        NO_DEFAULT_PATH
    )
else()
    find_library(PM_C_API_LIBRARY NAMES ${PM_C_API_LIB_NAME})
endif()

if(NOT PM_C_API_LIBRARY)
    message(FATAL_ERROR "Unable to find Polymarket C API library. Set PM_C_API_LIB_DIR and PM_C_API_LIB_NAME.")
endif()

target_link_libraries(order_example PRIVATE ${PM_C_API_LIBRARY})

if(WIN32 AND PM_C_API_DLL)
    add_custom_command(
        TARGET order_example POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            "${PM_C_API_DLL}"
            "$<TARGET_FILE_DIR:order_example>"
        VERBATIM
    )
endif()
