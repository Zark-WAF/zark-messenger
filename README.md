# ZarkMessenger

ZarkMessenger is a high-performance, versatile messaging system designed for the ZARK-WAF (Web Application Firewall) project. It supports both local inter-process communication (IPC) using shared memory and network communication via TCP, allowing for efficient module interaction both locally and across distributed systems.

## Table of Contents

1. [Features](#features)
2. [Architecture](#architecture)
3. [Installation](#installation)
4. [Usage](#usage)
5. [API Reference](#api-reference)
6. [FFI Support](#ffi-support)
7. [Memory Management and Cleanup](#memory-management-and-cleanup)
8. [Limitations and Considerations](#limitations-and-considerations)
9. [Contributing](#contributing)
10. [License](#license)

## Features

- **Dual Transport Modes**: Supports both shared memory IPC and TCP communication.
- **Dynamic Message Queue**: Utilizes a thread-safe, dynamically-sized queue for message management.
- **Large Message Support**: Handles messages up to 1MB in size (configurable).
- **Global Instance**: Provides a singleton-like global instance for consistent messaging across the application.
- **FFI Support**: Offers C-compatible functions for language-agnostic module interaction.
- **Memory Cleanup**: Implements proper memory cleanup mechanisms for both IPC and TCP modes.

## Architecture

ZarkMessenger's architecture consists of several key components:

1. **Transport Layer**: Supports both shared memory IPC and TCP communication.
2. **Message Queue**: A thread-safe, dynamically-sized queue for managing messages.
3. **Serialization**: Uses serde for serializing and deserializing messages.
4. **FFI Layer**: Provides C-compatible functions for cross-language support.

## Installation

### C

```c
#include "zark_messenger.h"
#include <stdio.h>

int main() {
    // For local IPC
    void* messenger = zark_messenger_init("zark_waf_messenger", false);
    
    // For TCP
    // void* messenger = zark_messenger_init("8080", true);
    
    // Sending a message
    zark_messenger_send(messenger, "topic", "Hello, ZarkWAF!");
    
    // Receiving a message
    char topic[256];
    char buffer[1024];
    int received = zark_messenger_receive(messenger, topic, sizeof(topic), buffer, sizeof(buffer));
    if (received >= 0) {
        printf("Received on topic '%s': %s\n", topic, buffer);
    }
    
    // Cleanup
    zark_messenger_cleanup(messenger);
    zark_messenger_free(messenger);
    
    return 0;
}
```

## API Reference

### Rust API

- `ZarkMessenger::new(name: &str, transport: TransportType) -> Result<Self, Box<dyn std::error::Error>>`
  Creates a new ZarkMessenger instance.

- `ZarkMessenger::global(name: &str, transport: TransportType) -> Arc<Self>`
  Returns a global instance of ZarkMessenger.

- `send(&self, topic: &str, message: &[u8]) -> Result<(), Box<dyn std::error::Error>>`
  Sends a message on a specific topic.

- `receive(&self) -> Result<(String, Vec<u8>), Box<dyn std::error::Error>>`
  Receives a message, returning the topic and payload.

- `cleanup(&self)`
  Cleans up the shared memory (for IPC mode).

### C API

- `void* zark_messenger_init(const char* name, bool is_tcp)`
  Initializes the messenger and returns a handle.

- `bool zark_messenger_send(void* messenger, const char* topic, const char* message)`
  Sends a message on a specific topic.

- `int zark_messenger_receive(void* messenger, char* topic, size_t topic_len, char* buffer, size_t buffer_len)`
  Receives a message, populating the topic and buffer.

- `void zark_messenger_cleanup(void* messenger)`
  Cleans up the shared memory (for IPC mode).

- `void zark_messenger_free(void* messenger)`
  Frees the messenger resources.

## FFI Support

ZarkMessenger provides a C-compatible FFI layer, allowing it to be used from various programming languages. The header file `zark_messenger.h` defines the C API, which can be used to interact with the messenger from C or any language with C FFI support.

## Memory Management and Cleanup

- For IPC mode, shared memory is cleaned up after each message is read.
- The `cleanup` method (or `zark_messenger_cleanup` in C) should be called when the messenger is no longer needed to ensure proper resource release.
- For TCP mode, the operating system handles buffer management and cleanup. It allows scaling Zark-Waf into network as needed.

## Limitations and Considerations

- Maximum message size is set to 1MB by default (configurable).
- IPC mode is limited to processes on the same machine. It is achieved to install Zark-WAF alongside your Web Server and have a 0-lose time performance
- TCP mode allows for distributed communication and makes ZARK-WAF scalable horizontally.

## License

ZarkMessenger is released under the MIT License. See the LICENSE file for details.
