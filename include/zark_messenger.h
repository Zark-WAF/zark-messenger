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

#ifdef __cplusplus
extern "C" {
#endif

// initializes a new messenger instance
// name: identifier for the messenger
// is_tcp: flag to determine if TCP should be used (true) or IPC (false)
// returns: pointer to the messenger instance or NULL on failure
void* zark_messenger_init(const char* name, bool is_tcp);

// sends a message through the messenger
// messenger: pointer to the messenger instance
// topic: the topic/channel for the message
// message: the content of the message
// returns: true if the message was sent successfully, false otherwise
bool zark_messenger_send(void* messenger, const char* topic, const char* message);

// receives a message from the messenger
// messenger: pointer to the messenger instance
// topic: buffer to store the received topic
// topic_len: size of the topic buffer
// buffer: buffer to store the received message
// buffer_len: size of the message buffer
// returns: length of the received message, or negative value on error
int zark_messenger_receive(void* messenger, char* topic, size_t topic_len, char* buffer, size_t buffer_len);

// performs any necessary cleanup operations for the messenger
// messenger: pointer to the messenger instance
void zark_messenger_cleanup(void* messenger);

// frees the memory associated with the messenger instance
// messenger: pointer to the messenger instance
void zark_messenger_free(void* messenger);

#ifdef __cplusplus
}
#endif

#endif // ZARK_MESSENGER_H