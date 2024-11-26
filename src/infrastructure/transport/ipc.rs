use super::Transport;
use crate::application::config::IpcConfig;
use crate::domain::errors::MessengerError;
use crate::domain::message::Message;
use crate::infrastructure::memory::pool_allocator::PoolAllocator;
use crate::infrastructure::serialization::Serializer;

use async_trait::async_trait;
use tokio::sync::{RwLock, Mutex, mpsc};

pub struct IpcTransport {
    shmem: RwLock<Vec<u8>>,                   // Shared memory buffer
    slots: Mutex<Vec<bool>>,                  // Bitmap for free/used slots
    tx: mpsc::Sender<usize>,                  // Sender for slot indices
    rx: Mutex<Option<mpsc::Receiver<usize>>>, // Receiver for slot indices
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

        // Find and mark a free slot atomically
        let slot_index = self.find_and_mark_next_free_slot().await?;

        // Proceed to write to shared memory
        let mut shmem = self.shmem.write().await;

        let start = slot_index * self.config.max_message_size;
        let end = start + total_len + 4; // 4 bytes for length prefix

        if end > shmem.len() {
            // Free the slot before returning
            self.free_slot(slot_index).await;
            return Err(MessengerError::MemoryOverflow);
        }

        // Write length prefix and serialized data
        shmem[start..start + 4].copy_from_slice(&(total_len as u32).to_be_bytes());
        shmem[start + 4..end].copy_from_slice(&serialized_data);

        // Push the slot index to the channel
        self.tx.send(slot_index).await.map_err(|_| MessengerError::ChannelClosed)?;

        Ok(())
    }

    async fn receive(&self) -> Result<Message, MessengerError> {
        // Wait for a slot index from the channel
        let slot_index = {
            let mut rx_lock = self.rx.lock().await;
    
            // If the Receiver has been dropped, return an error
            if rx_lock.is_none() {
                return Err(MessengerError::ChannelClosed);
            }
    
            let rx = rx_lock.as_mut().unwrap();
    
            match rx.recv().await {
                Some(index) => index,
                None => {
                    // The channel is closed
                    return Err(MessengerError::ChannelClosed);
                }
            }
        };

        let shmem = self.shmem.read().await;

        let start = slot_index * self.config.max_message_size;
        let len_bytes = &shmem[start..start + 4];
        let len = u32::from_be_bytes(len_bytes.try_into().unwrap()) as usize;
        let end = start + 4 + len;

        if end > shmem.len() {
            // Free the slot before returning
            self.free_slot(slot_index).await;
            return Err(MessengerError::MemoryOverflow);
        }

        // Read and deserialize data
        let serialized_data = &shmem[start + 4..end];
        let message = self.serializer.deserialize(serialized_data)
            .map_err(|e| MessengerError::Deserialization(e.to_string()))?;

        // Free the slot
        self.free_slot(slot_index).await;

        Ok(message)
    }

    async fn cleanup(&self) -> Result<(), MessengerError> {
        let mut shmem = self.shmem.write().await;
        shmem.fill(0);

        {
            let mut slots = self.slots.lock().await;
            for slot in slots.iter_mut() {
                *slot = false;
            }
        }

        {
            let mut rx_lock = self.rx.lock().await;
            *rx_lock = None;
        }

        // Close the channel
        drop(self.tx.clone());

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
        let slots = vec![false; config.max_queue_size]; // Initially, all slots are free

        let (tx, rx) = mpsc::channel(config.max_queue_size);

        Ok(Self {
            shmem: RwLock::new(vec![0; shmem_size]),
            slots: Mutex::new(slots),
            tx,
            rx: Mutex::new(Some(rx)),
            config,
            serializer,
            buffer_pool,
        })
    }

    /// Finds and marks the next available free slot atomically
    async fn find_and_mark_next_free_slot(&self) -> Result<usize, MessengerError> {
        let mut slots = self.slots.lock().await;
        if let Some(index) = slots.iter().position(|&slot| !slot) {
            slots[index] = true; // Mark as used while holding the lock
            Ok(index)
        } else {
            Err(MessengerError::NoFreeSlots)
        }
    }

    /// Frees the slot at the given index
    async fn free_slot(&self, slot_index: usize) {
        let mut slots = self.slots.lock().await;
        slots[slot_index] = false;
    }
}
