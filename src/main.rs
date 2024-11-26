pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interfaces;
pub mod utils;

use std::sync::Arc;
use infrastructure::transport::Transport;
use tokio::sync::Barrier;
use tokio::task;

use crate::application::config::IpcConfig;
use crate::domain::message::Message;
use crate::infrastructure::serialization::json::JsonSerializer;
use crate::infrastructure::memory::pool_allocator::PoolAllocator;
use crate::infrastructure::transport::ipc::IpcTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Config struct
    let ipc_config = IpcConfig {
        shared_memory_name: "zark_waf_messenger_shm".to_string(),
        max_message_size: 1024,
        max_queue_size: 10000,
        max_buffer_size: 1024,
    };

    // Initialize the serializer and buffer pool
    let serializer = Box::new(JsonSerializer {});
    let buffer_pool = PoolAllocator::new(ipc_config.max_buffer_size);

    // Initialize the IpcTransport
    println!("Initializing IpcTransport...");
    let ipc_transport = Arc::new(IpcTransport::new(
        ipc_config,
        serializer,
        buffer_pool,
    )?);

    // Number of concurrent tasks
    let num_tasks = 100;
    let messages_per_task = 100;

    // A barrier to synchronize the start of all tasks
    let barrier = Arc::new(Barrier::new(num_tasks * 2));

    // Shared vector to collect sent messages
    let sent_messages = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // Spawn sender tasks
    let mut send_handles = Vec::new();

    for i in 0..num_tasks {
        let ipc_transport = ipc_transport.clone();
        let barrier = barrier.clone();
        let sent_messages = sent_messages.clone();
        let handle = task::spawn(async move {
            // Wait for all tasks to be ready
            barrier.wait().await;

            for j in 0..messages_per_task {
                let message = Message {
                    id: format!("sender-{}-message-{}", i, j),
                    topic: "test_topic".to_string(),
                    payload: vec![i as u8, j as u8],
                };

                ipc_transport.send(&message).await.unwrap();

                // Record the sent message
                let mut sent = sent_messages.lock().await;
                sent.push(message);
            }
        });

        send_handles.push(handle);
    }

    // Shared vector to collect received messages
    let received_messages = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // Spawn receiver tasks
    let mut receive_handles = Vec::new();

    for _i in 0..num_tasks {
        let ipc_transport = ipc_transport.clone();
        let barrier = barrier.clone();
        let received_messages = received_messages.clone();
        let handle = task::spawn(async move {
            // Wait for all tasks to be ready
            barrier.wait().await;

            for _ in 0..messages_per_task {
                match ipc_transport.receive().await {
                    Ok(message) => {
                        // Record the received message
                        let mut received = received_messages.lock().await;
                        received.push(message);
                    }
                    Err(e) => {
                        eprintln!("Receive error: {:?}", e);
                        // Handle the error appropriately
                    }
                }
            }
        });

        receive_handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in send_handles {
        handle.await.unwrap();
    }

    for handle in receive_handles {
        handle.await.unwrap();
    }

    // Verify that all messages sent are received
    let sent = sent_messages.lock().await;
    let mut received = received_messages.lock().await;

    // Sort the messages for comparison
    let mut sent_sorted = sent.clone();
    sent_sorted.sort_by(|a, b| a.id.cmp(&b.id));

    received.sort_by(|a, b| a.id.cmp(&b.id));

    assert_eq!(sent_sorted, *received);

    println!("All messages sent and received successfully.");

    // Cleanup
    println!("Cleaning up...");
    ipc_transport.cleanup().await?;
    println!("Cleanup complete");

    Ok(())
}
