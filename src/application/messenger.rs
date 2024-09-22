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

use crate::domain::message::Message;
use crate::domain::errors::MessengerError;
use crate::domain::topic::Topic;
use crate::domain::serializable::Serializable;



#[async_trait]
pub trait Messenger: Send + Sync {
    async fn publish<T: Serializable + Send + Sync>(&self, topic: &Topic, payload: &T) -> Result<(), MessengerError>;
    async fn subscribe(&self, topic: &Topic) -> Result<Box<dyn MessageSubscriber>, MessengerError>;
    async fn rpc_call<P: Serializable + Send + Sync, R: Serializable + Send + Sync>(&self, method: &[u8], params: &P) -> Result<R, MessengerError>;
    async fn register_rpc_handler(&self, method: &[u8], handler: Box<dyn RpcHandler>) -> Result<(), MessengerError>;
}

#[async_trait]
pub trait MessageSubscriber: Send + Sync {
    async fn receive(&self) -> Result<Message, MessengerError>;
}

#[async_trait]
pub trait RpcHandler: Send + Sync {
    async fn handle(&self, params: &[u8]) -> Result<Vec<u8>, MessengerError>;
}