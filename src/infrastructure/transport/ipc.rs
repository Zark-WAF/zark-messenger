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
use shared_memory::{Shmem, ShmemConf};
use crossbeam::queue::SegQueue;
use std::sync::{Arc, Mutex};
use crate::domain::message::Message;
use crate::domain::errors::MessengerError;
use super::Transport;
use crate::application::config::IpcConfig;

// ipc transport struct for inter-process communication
pub struct IpcTransport {
    shmem: Arc<Mutex<Shmem>>, // shared memory for storing messages
    queue: Arc<SegQueue<(usize, usize)>>, // queue for tracking message offsets and lengths
    config: IpcConfig, // configuration for ipc transport
    serializer: Box<dyn Serializer>, // serializer for converting messages to/from bytes
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

        let mut shmem = self.shmem.lock().unwrap();
        let offset = self.queue.len() * self.config.max_message_size;
        let slice = unsafe { shmem.as_slice_mut() };

        // copy serialized message to shared memory
        slice[offset..offset + len].copy_from_slice(&serialized);
        self.queue.push((offset, len));

        Ok(())
    }

    // receive a message from shared memory
    async fn receive(&self) -> Result<Message, MessengerError> {
        if let Some((offset, len)) = self.queue.pop() {
            let shmem = self.shmem.lock().unwrap();
            let slice = unsafe { shmem.as_slice() };
            let data = slice[offset..offset + len].to_vec();
            self.serializer.deserialize(&data)
        } else {
            Err(MessengerError::TransportError("No messages available".into()))
        }
    }

    // clean up shared memory by zeroing out its contents
    fn cleanup(&self) {
        let mut shmem = self.shmem.lock().unwrap();
        let slice = unsafe { shmem.as_slice_mut() };
        slice.fill(0);
    }
}

impl IpcTransport {
    // create a new ipc transport instance
    pub fn new(config: IpcConfig) -> Result<Self, MessengerError> {
        let shmem = ShmemConf::new()
        .size(config.max_message_size * 100) // allow for 100 messages
        .os_id(&config.shared_memory_name)
        .create()
        .map_err(|e| MessengerError::TransportError(e.to_string()))?;

        Ok(Self {
            shmem: Arc::new(Mutex::new(shmem)),
            queue: Arc::new(SegQueue::new()),
            config,
            serializer,
        })
    }
}