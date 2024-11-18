// MIT License
// 
// Copyright (c) 2024 ZARK-WAF
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// Authors: I. Zeqiri, E. Gjergji

#ifndef ZARK_MESSENGER_H
#define ZARK_MESSENGER_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Error codes returned by messenger functions
typedef enum ZarkMessengerError {
    ZARK_SUCCESS = 0,
    ZARK_ERROR_INVALID_ARGUMENT = -1,
    ZARK_ERROR_MEMORY_ALLOCATION = -2,
    ZARK_ERROR_CONNECTION_FAILED = -3,
    ZARK_ERROR_SEND_FAILED = -4,
    ZARK_ERROR_RECEIVE_FAILED = -5,
    ZARK_ERROR_TIMEOUT = -6,
    ZARK_ERROR_BUFFER_TOO_SMALL = -7,
    ZARK_ERROR_INTERNAL = -8,
    ZARK_ERROR_MESSAGE_TOO_LARGE = -9,
    ZARK_ERROR_NO_MESSAGES = -10
} ZarkMessengerError;

// Configuration for IPC transport
typedef struct ZarkIpcConfig {
    const char* shared_memory_name;
    size_t max_message_size;
    size_t max_queue_size;
    size_t max_buffer_size;
} ZarkIpcConfig;

// Configuration for TCP transport
typedef struct ZarkTcpConfig {
    const char* host;
    uint16_t port;
    size_t max_message_size;
} ZarkTcpConfig;

// Transport type enum
typedef enum ZarkTransportType {
    ZARK_TRANSPORT_IPC,
    ZARK_TRANSPORT_TCP
} ZarkTransportType;

// Main configuration structure
typedef struct ZarkConfig {
    ZarkTransportType transport_type;
    ZarkIpcConfig* ipc_config;
    ZarkTcpConfig* tcp_config;
} ZarkConfig;

// Opaque pointer to messenger instance
typedef void* ZarkMessenger;

// Initialize messenger with configuration
ZarkMessenger* zark_messenger_init(const ZarkConfig* config);

// Send a message
bool zark_messenger_send(ZarkMessenger* messenger, const struct Message* message);

// Receive a message
int32_t zark_messenger_receive(
    ZarkMessenger* messenger,
    char* topic,
    size_t topic_len,
    char* buffer,
    size_t buffer_len
);

// Cleanup messenger
void zark_messenger_cleanup(ZarkMessenger* messenger);

// Free messenger instance
void zark_messenger_free(ZarkMessenger* messenger);

#ifdef __cplusplus
}
#endif

#endif // ZARK_MESSENGER_H