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
use crossbeam::queue::SegQueue;
use tokio::sync::RwLock;
use super::Transport;
use crate::application::config::IpcConfig;
use crate::domain::errors::MessengerError;
use crate::domain::message::Message;
use crate::infrastructure::serialization::Serializer;

// ipc transport struct for inter-process communication
// ipc transport struct for inter-process communication
pub struct IpcTransport {
    shmem: RwLock<Vec<u8>>,               // shared memory for storing messages
    queue: Arc<SegQueue<(usize, usize)>>, // queue for tracking message offsets and lengths
    config: IpcConfig,                    // configuration for ipc transport
    serializer: Box<dyn Serializer>,      // serializer for converting messages to/from bytes
}

#[async_trait]
impl Transport for IpcTransport {
    // send a message using shared memory
    async fn send(&self, msg: Message) -> Result<(), MessengerError> {
        let serialized = self.serializer.serialize(&msg)?;
        let len = serialized.len();

        // check if message size exceeds the maximum allowed
        if len > self.config.max_message_size {
            return Err(MessengerError::TransportError("Message too large".into()));
        }

        let mut shmem = self.shmem.write().await; // await the future to get the RwLockWriteGuard
        let offset = self.queue.len() * self.config.max_message_size;

        // copy serialized message to shared memory
        shmem[offset..offset + len].copy_from_slice(&serialized);
        self.queue.push((offset, len));

        Ok(())
    }

    // receive a message from shared memory
    async fn receive(&self) -> Result<Message, MessengerError> {
        if let Some((offset, len)) = self.queue.pop() {
            let shmem = self.shmem.read().await; // await the future to get the RwLockReadGuard
            let slice = shmem.as_slice();
            let data = slice[offset..offset + len].to_vec();
            self.serializer.deserialize(&data)
        } else {
            Err(MessengerError::TransportError(
                "No messages available".into(),
            ))
        }
    }

    // clean up shared memory by zeroing out its contents
    async fn cleanup(&self) {
        let mut shmem = self.shmem.write().await;
        shmem.iter_mut().for_each(|x| *x = 0);
    }
}

unsafe impl Send for IpcTransport {}

impl IpcTransport {
    // create a new ipc transport instance
    pub fn new(config: IpcConfig, serializer: Box<dyn Serializer>) -> Result<Self, MessengerError> {
        Ok(Self {
            shmem: RwLock::new(Vec::new()),
            queue: Arc::new(SegQueue::new()),
            config,
            serializer,
        })
    }
}
