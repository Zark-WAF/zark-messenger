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



use async_trait::async_trait;
use crate::domain::errors::MessengerError;
use crate::domain::message::Message;

pub mod ipc;
pub mod tcp;



#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a message asynchronously
    async fn send(&self, message: &Message) -> Result<(), MessengerError>;

    /// Receive a message asynchronously
    async fn receive(&self) -> Result<Message, MessengerError>;

    /// Perform any necessary cleanup operations
    async fn cleanup(&self) -> Result<(), MessengerError>;

    /// Check if the transport is connected and ready
    async fn is_ready(&self) -> bool;

    /// Reconnect if the transport is not ready
    async fn reconnect(&self) -> Result<(), MessengerError>;

    /// Get the maximum message size supported by this transport
    fn max_message_size(&self) -> usize;

    /// Close the transport
    async fn close(&self) -> Result<(), Box<dyn std::error::Error>>;
}