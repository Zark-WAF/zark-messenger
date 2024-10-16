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
use super::Transport;
use crate::application::config::IpcConfig;
use crate::domain::errors::MessengerError;
use crate::domain::message::Message;
use crate::infrastructure::serialization::Serializer;
use crate::infrastructure::memory::pool_allocator::PoolAllocator;

use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use tokio::sync::RwLock;

pub struct IpcTransport {
    shmem: RwLock<Vec<u8>>,
    queue: Arc<SegQueue<(usize, usize)>>,
    config: IpcConfig,
    serializer: Box<dyn Serializer>,
    buffer_pool: PoolAllocator<Vec<u8>>,
}

#[async_trait]
impl Transport for IpcTransport {

    async fn send(&self, message: &Message) -> Result<(), MessengerError> {
        let topic = message.topic.as_bytes().to_vec();
        let payload = &message.payload;

        let total_len = topic.len() + payload.len() + 8; // 4 bytes for topic length, 4 bytes for payload length
        let max_message_size = self.max_message_size();

        if total_len > max_message_size {
            return Err(MessengerError::MessageTooLarge(total_len, max_message_size));
        }

        // Prepare the message in a separate block to limit the lifetime of buffer_ptr
        let prepared_message = {
            let mut buffer_ptr = self.buffer_pool.allocate();
            let buffer = unsafe { buffer_ptr.as_mut() };

            if buffer.len() < total_len {
                buffer.resize(total_len, 0);
            }

            // Write topic length
            buffer[0..4].copy_from_slice(&(topic.len() as u32).to_le_bytes());
            // Write topic
            buffer[4..4 + topic.len()].copy_from_slice(&topic);
            // Write payload length
            buffer[4 + topic.len()..8 + topic.len()].copy_from_slice(&(payload.len() as u32).to_le_bytes());
            // Write payload
            buffer[8 + topic.len()..].copy_from_slice(payload);

            let prepared = buffer[..total_len].to_vec();
            self.buffer_pool.deallocate(buffer_ptr);
            prepared
        };

        let mut shmem = self.shmem.write().await;
        let offset = self.queue.len() * max_message_size;

        if offset + total_len > shmem.len() {
            shmem.resize(offset + total_len, 0);
        }

        shmem[offset..offset + total_len].copy_from_slice(&prepared_message);
        self.queue.push((offset, total_len));

        Ok(())
    }

    async fn receive(&self) -> Result<Message, MessengerError> {
        if let Some((offset, len)) = self.queue.pop() {
            let shmem = self.shmem.read().await;
            let data = shmem[offset..offset + len].to_vec();
            
            // Deserialize the data into a Message
            let topic_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
            let topic = data[4..4 + topic_len].to_vec();
            let payload_len = u32::from_le_bytes([data[4 + topic_len], data[5 + topic_len], data[6 + topic_len], data[7 + topic_len]]) as usize;
            let payload = data[8 + topic_len..8 + topic_len + payload_len].to_vec();

            Ok(Message {
                topic: String::from_utf8(topic).map_err(|_| MessengerError::Deserialization("Invalid UTF-8 in topic".to_string()))?,
                payload,
                id: String::new(), // or generate a unique id
            })
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
}

impl IpcTransport {
    pub fn new(config: IpcConfig, serializer: Box<dyn Serializer>, buffer_pool: PoolAllocator<Vec<u8>>) -> Result<Self, MessengerError> {
        let shmem_size = config.max_message_size * config.max_queue_size;
        Ok(Self {
            shmem: RwLock::new(vec![0; shmem_size]),
            queue: Arc::new(SegQueue::new()),
            config,
            serializer,
            buffer_pool,
        })
    }
}

unsafe impl Send for IpcTransport {}
unsafe impl Sync for IpcTransport {}