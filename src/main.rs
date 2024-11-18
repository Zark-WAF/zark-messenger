pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interfaces;
pub mod utils;


use interfaces::ffi::zark_messenger_free;

pub use crate::interfaces::ffi::{zark_messenger_init, zark_messenger_send, zark_messenger_cleanup};

pub use crate::application::messenger::Messenger;
pub use crate::application::config::Config;
pub use crate::domain::errors::MessengerError;
pub use crate::application::config::{IpcConfig, TransportType};
pub use crate::domain::message::Message;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Config struct
    let config = Config {
        transport_type: TransportType::IPC,
        ipc_config: Some(IpcConfig {
            shared_memory_name: "zark_waf_messenger_shm".to_string(),
            max_message_size: 1024,
            max_queue_size: 1024,
            max_buffer_size: 1024,
        }),
        tcp_config: None,
    };

    // Initialize the messenger
    println!("Initializing...");
    let messenger_ptr = unsafe { zark_messenger_init(&config as *const Config) };
    if !messenger_ptr.is_null() {
        println!("Initialization successful");
    } else {
        println!("Initialization failed");
        return Ok(());
    }

    // Create a test message
    let message = Message {
        id: "1".to_string(),
        topic: "test_topic".to_string(),
        payload: vec![1, 2, 3, 4, 5],
    };

    // Send the message
    println!("Sending message...");
    let success = unsafe { zark_messenger_send(messenger_ptr, &message as *const Message) };
    if success {
        println!("Message sent successfully");
    } else {
        println!("Failed to send message");
    }

    // Cleanup and free the messenger
    println!("Cleaning up...");
    unsafe {
        zark_messenger_cleanup(messenger_ptr);
        zark_messenger_free(messenger_ptr);
    }
    println!("Cleanup complete");

    Ok(())
}