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

use lazy_static::lazy_static;
use std::ffi::{c_char, c_void};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

use crate::application::config::{Config, TransportType};
use crate::application::messenger::{Messenger, MessengerImpl};
use crate::domain::message::Message;
use crate::infrastructure::memory::pool_allocator::PoolAllocator;
use crate::infrastructure::serialization::json::JsonSerializer;
use crate::infrastructure::transport::ipc::IpcTransport;
use crate::infrastructure::transport::tcp::TcpTransport;
use crate::infrastructure::transport::Transport;
use crate::application::instance_manager::INSTANCE_MANAGER;

lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().expect("Failed to create Tokio runtime");
    static ref MESSENGER_MUTEX: Mutex<AtomicBool> = Mutex::new(AtomicBool::new(false));
}

#[no_mangle]
pub extern "C" fn zark_messenger_init(config: *const Config) -> *mut c_void {
    if let Some(existing) = INSTANCE_MANAGER.get_messenger() {
        INSTANCE_MANAGER.register_instance();
        return existing;
    }

    let config = unsafe { &*config };
    // Create transport and messenger as before
    let transport: Arc<dyn Transport> = match config.transport_type {
        TransportType::IPC => {
            let ipc_config = config.ipc_config.as_ref().expect("IPC config not provided");
            Arc::new(IpcTransport::new(ipc_config.clone(), Box::new(JsonSerializer), PoolAllocator::new(1024 * 1024))
                .expect("Failed to create IPC transport"))
        }
        TransportType::TCP => {
            let tcp_config = config.tcp_config.as_ref().expect("TCP config not provided");
            Arc::new(RUNTIME.block_on(async {
                TcpTransport::new_client(tcp_config.clone(), Box::new(JsonSerializer)).await
                    .expect("Failed to create TCP transport")
            }))
        }
    };

    let messenger: Box<dyn Messenger> = Box::new(MessengerImpl::new(transport));
    let messenger_ptr = Box::into_raw(messenger) as *mut c_void;
    
    INSTANCE_MANAGER.set_messenger(messenger_ptr);
    INSTANCE_MANAGER.register_instance();
    
    messenger_ptr
}

#[no_mangle]
pub extern "C" fn zark_messenger_send(messenger_param: *mut c_void, message: *const Message) -> bool {
    if messenger_param.is_null() {
        eprintln!("Messenger pointer is null");
        return false;
    }
    if message.is_null() {
        eprintln!("Message pointer is null");
        return false;
    }

    let messenger = unsafe { &*(messenger_param as *mut MessengerImpl) as &dyn Messenger };
    let message = unsafe { &*message };

    RUNTIME.block_on(async {
        messenger.publish(message.topic.clone(), message).await.is_ok()
    })
}


#[no_mangle]
pub extern "C" fn zark_messenger_receive(
    messenger_param: *mut c_void,
    topic: *mut c_char,
    topic_len: usize,
    buffer: *mut c_char,
    buffer_len: usize,
) -> i32 {
    let messenger = unsafe { &*(messenger_param as *mut MessengerImpl) as &dyn Messenger };

    RUNTIME.block_on(async {
        let subscriber = match messenger.subscribe(c_str_to_rust_string(topic)).await {
            Ok(sub) => sub,
            Err(_) => return -1,
        };

        match subscriber.receive().await {
            Ok(msg) => {
                let topic_bytes = msg.topic.as_bytes();
                let payload_bytes = msg.payload.as_slice();

                let topic_copy_len = std::cmp::min(topic_bytes.len(), topic_len.saturating_sub(1));
                let payload_copy_len = std::cmp::min(payload_bytes.len(), buffer_len.saturating_sub(1));

                unsafe {
                    std::ptr::copy_nonoverlapping(topic_bytes.as_ptr(), topic as *mut u8, topic_copy_len);
                    *topic.add(topic_copy_len) = 0;

                    std::ptr::copy_nonoverlapping(payload_bytes.as_ptr(), buffer as *mut u8, payload_copy_len);
                    *buffer.add(payload_copy_len) = 0;
                }

                payload_copy_len as i32
            }
            Err(_) => -1,
        }
    })
}

#[no_mangle]
pub extern "C" fn zark_messenger_cleanup(messenger: *mut c_void) {
    if messenger.is_null() {
        return;
    }

    let messenger = unsafe { &*(messenger as *mut MessengerImpl) as &dyn Messenger };
    RUNTIME.block_on(async {
        let _ = messenger.cleanup().await;
    });
}

#[no_mangle]
pub extern "C" fn zark_messenger_free(messenger: *mut c_void) {
    if messenger.is_null() {
        return;
    }

    INSTANCE_MANAGER.unregister_instance();
}

// Helper function to convert C string to Rust string
fn c_str_to_rust_string(c_str: *const c_char) -> String {
    unsafe {
        std::ffi::CStr::from_ptr(c_str)
            .to_string_lossy()
            .into_owned()
    }
}