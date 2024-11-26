use super::Transport;
use crate::application::config::IpcConfig;
use crate::domain::errors::MessengerError;
use crate::domain::message::Message;
use crate::infrastructure::memory::pool_allocator::PoolAllocator;
use crate::infrastructure::serialization::Serializer;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::collections::HashMap;
use tokio::time::{timeout, Duration};

use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc};

pub struct IpcTransport {
    messages: Mutex<HashMap<u64, Vec<u8>>>, // Map of message IDs to data
    next_id: AtomicU64,                     // Atomic counter for message IDs
    total_memory: AtomicUsize,              // Total memory currently used
    max_memory: usize,                      // Maximum allowed memory usage
    tx: mpsc::Sender<u64>,                  // Sender for message IDs
    rx: Mutex<Option<mpsc::Receiver<u64>>>, // Receiver for message IDs
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
        if total_len > self.config.max_message_size {
            return Err(MessengerError::MessageTooLarge(total_len, self.config.max_message_size));
        }

        // Wait until enough memory is available
        loop {
            let current_memory = self.total_memory.load(Ordering::SeqCst);
            if current_memory + total_len <= self.max_memory {
                match self.total_memory.compare_exchange(
                    current_memory,
                    current_memory + total_len,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => break,
                    Err(_) => continue,
                }
            } else {
                // Not enough memory, wait with a timeout
                if timeout(Duration::from_millis(100), tokio::task::yield_now()).await.is_err() {
                    return Err(MessengerError::MemoryUnavailable);
                }
            }
        }
        

        // Generate a unique message ID
        let message_id = self.next_id.fetch_add(1, Ordering::SeqCst);

        // Store the message
        {
            let mut messages = self.messages.lock().await;
            messages.insert(message_id, serialized_data);
        }

        // Send the message ID to the receiver
        self.tx.send(message_id).await.map_err(|_| MessengerError::ChannelClosed)?;

        Ok(())
    }

    async fn receive(&self) -> Result<Message, MessengerError> {
        // Wait for a message ID from the channel
        let message_id = {
            let mut rx_lock = self.rx.lock().await;
    
            if rx_lock.is_none() {
                return Err(MessengerError::ChannelClosed);
            }
    
            let rx = rx_lock.as_mut().unwrap();
    
            match rx.recv().await {
                Some(id) => id,
                None => return Err(MessengerError::ChannelClosed),
            }
        };    

        // Retrieve and remove the message
        let serialized_data = {
            let mut messages = self.messages.lock().await;
            messages.remove(&message_id).ok_or(MessengerError::MessageNotFound)?
        };

        // Deserialize the message
        let message = self.serializer.deserialize(&serialized_data)
            .map_err(|e| MessengerError::Deserialization(e.to_string()))?;

        // Update total memory usage
        self.total_memory.fetch_sub(serialized_data.len(), Ordering::SeqCst);

        Ok(message)
    }

    async fn cleanup(&self) -> Result<(), MessengerError> {
        // Clear messages
        {
            let mut messages = self.messages.lock().await;
            messages.clear();
        }
    
        // Reset total memory
        self.total_memory.store(0, Ordering::SeqCst);
    
        // Reset next_id
        self.next_id.store(0, Ordering::SeqCst);
    
        // Close the channel by dropping the sender
        // Drop the Sender
        // Cloning `tx` to drop it explicitly
        let _ = self.tx.clone();
    
        // Drop the Receiver
        {
            let mut rx_lock = self.rx.lock().await;
            *rx_lock = None;
        }
    
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
        let (tx, rx) = mpsc::channel(config.max_queue_size);

        Ok(Self {
            messages: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(0),
            total_memory: AtomicUsize::new(0),
            max_memory: config.max_message_size * config.max_queue_size,
            tx,
            rx: Mutex::new(Some(rx)),
            config,
            serializer,
            buffer_pool,
        })
    }
}
