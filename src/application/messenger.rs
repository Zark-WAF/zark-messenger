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

use crate::domain::message::Message;
use crate::domain::errors::MessengerError;
use crate::infrastructure::transport::Transport;
use std::sync::Arc;

// messenger struct represents a messaging system that uses a transport layer for communication
pub struct Messenger {
    // transport is an arc-wrapped trait object that implements the Transport trait
    // this allows for different transport implementations to be used interchangeably
    transport: Arc<dyn Transport>,
}

impl Messenger {
    // new creates a new messenger instance with the provided transport
    // this constructor allows for dependency injection of the transport layer
    pub fn new(transport: Arc<dyn Transport>) -> Self {
        Self { transport }
    }

    // send asynchronously sends a message using the underlying transport
    // it returns a result indicating success or a messenger-specific error
    pub async fn send(&self, msg: Message) -> Result<(), MessengerError> {
        self.transport.send(msg).await
    }

    // receive asynchronously waits for and receives a message from the transport
    // it returns either the received message or a messenger-specific error
    pub async fn receive(&self) -> Result<Message, MessengerError> {
        self.transport.receive().await
    }

    // cleanup performs any necessary cleanup operations on the transport
    // this method should be called when the messenger is no longer needed
    pub fn cleanup(&self) {
        self.transport.cleanup();
    }
}