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

use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::message::Message;
use crate::domain::errors::MessengerError;
use crate::infrastructure::serialization::Serializer;
use crate::infrastructure::transport::Transport;



#[async_trait]
pub trait Messenger: Send + Sync {
    async fn publish(&self, topic: String, payload: &Message) -> Result<(), MessengerError>;
    async fn subscribe(&self, topic: String) -> Result<Box<dyn MessageSubscriber>, MessengerError>;
    async fn rpc_call(&self, method: &[u8], params: &[u8]) -> Result<Vec<u8>, MessengerError>;
    async fn register_rpc_handler(&self, method: &[u8], handler: Box<dyn RpcHandler>) -> Result<(), MessengerError>;
    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>>;
}

#[async_trait]
pub trait MessageSubscriber: Send + Sync {
    async fn receive(&self) -> Result<Message, MessengerError>;
}

#[async_trait]
pub trait RpcHandler: Send + Sync {
    async fn handle(&self, params: &[u8]) -> Result<Vec<u8>, MessengerError>;
}


//implement messenger
pub struct MessengerImpl {
    transport: Arc<dyn Transport>
}

impl MessengerImpl {
    pub fn new(transport: Arc<dyn Transport>) -> Self {
        Self { transport }
    }
}

#[async_trait]
impl Messenger for MessengerImpl {
    async fn publish(&self, topic: String, msg: &Message) -> Result<(), MessengerError> {
        let message = Message::new(topic.clone(), msg.payload.clone());
        self.transport.send(&message).await
    }

    async fn subscribe(&self, topic: String) -> Result<Box<dyn MessageSubscriber>, MessengerError> {
        self.subscribe(topic).await
    }

    async fn rpc_call(&self, method: &[u8], params: &[u8]) -> Result<Vec<u8>, MessengerError> {
        self.rpc_call(method, params).await
    }
    
    async fn register_rpc_handler(&self, method: &[u8], handler: Box<dyn RpcHandler>) -> Result<(), MessengerError> {
        self.register_rpc_handler(method, handler).await
    }

    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Add your cleanup logic here
        Ok(())
    }
}

