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

use super::Transport;
use crate::application::config::IpcConfig;
use crate::domain::errors::MessengerError;
use crate::domain::message::Message;
use crate::infrastructure::memory::pool_allocator::PoolAllocator;
use crate::infrastructure::serialization::Serializer;
use std::sync::Arc;

use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use tokio::sync::RwLock;

pub struct IpcTransport {
    shmem: RwLock<Vec<u8>>,
    queue: Arc<SegQueue<usize>>,
    config: IpcConfig,
    serializer: Box<dyn Serializer>,
    buffer_pool: PoolAllocator<Vec<u8>>,
}

#[async_trait]
impl Transport for IpcTransport {
    async fn send(&self, message: &Message) -> Result<(), MessengerError> {
        // Serialize the message
        let serialized_data = self.serializer.serialize(message)
            .map_err(|e| MessengerError::Serialization(e.to_string()))?;

        let total_len = serialized_data.len();
        if total_len > self.max_message_size() {
            return Err(MessengerError::MessageTooLarge(total_len, self.max_message_size()));
        }

        let mut shmem = self.shmem.write().await;
        
        // Write length prefix and serialized data
        let mut buffer = vec![0u8; 4 + total_len];
        buffer[..4].copy_from_slice(&(total_len as u32).to_be_bytes());
        buffer[4..].copy_from_slice(&serialized_data);
        
        // Write to shared memory
        shmem[..buffer.len()].copy_from_slice(&buffer);
        
        // Push to queue
        self.queue.push(total_len);
        
        Ok(())
    }

    async fn receive(&self) -> Result<Message, MessengerError> {
        if let Some(msg_len) = self.queue.pop() {
            let shmem = self.shmem.read().await;
            
            // Read length and serialized data
            let mut buffer = vec![0u8; msg_len];
            buffer.copy_from_slice(&shmem[4..4 + msg_len]);
            
            // Deserialize using configured serializer
            self.serializer.deserialize(&buffer)
                .map_err(|e| MessengerError::Deserialization(e.to_string()))
        } else {
            Err(MessengerError::NoMessagesAvailable)
        }
    }

    async fn cleanup(&self) -> Result<(), MessengerError> {
        let mut shmem = self.shmem.write().await;
        shmem.fill(0);
        while self.queue.pop().is_some() {}
        Ok(())
    }

    async fn is_ready(&self) -> bool {
        true // IPC transport is always ready
    }

    async fn reconnect(&self) -> Result<(), MessengerError> {
        Ok(()) // No reconnection needed for IPC
    }

    fn max_message_size(&self) -> usize {
        self.config.max_message_size
    }

    async fn close(&self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        self.cleanup().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

impl IpcTransport {
    pub fn new(
        config: IpcConfig,
        serializer: Box<dyn Serializer>,
        buffer_pool: PoolAllocator<Vec<u8>>,
    ) -> Result<Self, MessengerError> {
        let shmem_size = config.max_message_size * config.max_queue_size;
        Ok(Self {
            shmem: RwLock::new(vec![0; shmem_size]),
            queue: Arc::new(SegQueue::new()),
            config,
            serializer,
            buffer_pool,
        })
    }

    fn find_next_free_slot(&self, max_size: usize) -> usize {
        let mut used_slots = vec![false; self.config.max_queue_size];
        let mut temp_storage = Vec::new();
        
        // Pop items, mark slots, and store temporarily
        while let Some(len) = self.queue.pop() {
            used_slots[len / max_size] = true;
            temp_storage.push(len);
        }
        
        // Push items back
        for item in temp_storage {
            self.queue.push(item);
        }
        
        used_slots.iter().position(|&used| !used).unwrap_or(0)
    }
}

unsafe impl Send for IpcTransport {}
unsafe impl Sync for IpcTransport {}
