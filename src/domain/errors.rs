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

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MessengerError {
    #[error("Transport error: {0}")]
    TransportError(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Message too large: actual size {0}, max size {1}")]
    MessageTooLarge(usize, usize), // (actual_size, max_size)

    #[error("No messages available")]
    NoMessagesAvailable,

    #[error("Got a memory overflow")]
    MemoryOverflow,

    #[error("No free memory slots")]
    NoFreeSlots,

    #[error("Channel Closed")]
    ChannelClosed,

    #[error("Message not found in channel")]
    MessageNotFound,

    #[error("You sucked all memory, there is left no more")]
    MemoryUnavailable
}