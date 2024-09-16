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

use std::ffi::{CStr, CString, c_char, c_void};
use std::sync::Arc;
use crate::application::messenger::Messenger;
use crate::application::config::{Config, TransportType};
use crate::domain::message::Message;
use crate::infrastructure::transport::{IpcTransport, TcpTransport};
use crate::infrastructure::serialization::json::JsonSerializer;


#[no_mangle]
pub extern "C" fn zark_messenger_init(config_json: *const c_char) -> *mut c_void {
    let config_str = unsafe { CStr::from_ptr(config_json).to_str().unwrap() };
    let config: Config = serde_json::from_str(config_str).unwrap();

    let transport: Arc<dyn crate::infrastructure::transport::Transport> = match config.transport_type {
        TransportType::IPC => {
            let ipc_config = config.ipc_config.unwrap();
            Arc::new(IpcTransport::new(ipc_config, Box::new(JsonSerializer)).unwrap())
        },
        TransportType::TCP => {
            let tcp_config = config.tcp_config.unwrap();
            Arc::new(TcpTransport::new_client(tcp_config, Box::new(JsonSerializer)).unwrap())
        },
    };

    let messenger = Box::new(Messenger::new(transport));
    Box::into_raw(messenger) as *mut c_void
}

#[no_mangle]
pub extern "C" fn zark_messenger_send(messenger: *mut c_void, topic: *const c_char, message: *const c_char) -> bool {
    let messenger = unsafe { &*(messenger as *const Messenger) };
    let topic = unsafe { CStr::from_ptr(topic).to_str().unwrap().to_string() };
    let message = unsafe { CStr::from_ptr(message).to_str().unwrap().as_bytes().to_vec() };

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            messenger.send(topic, message).await.is_ok()
        })
}

#[no_mangle]
pub extern "C" fn zark_messenger_receive(messenger: *mut c_void, topic: *mut c_char, topic_len: usize, buffer: *mut c_char, buffer_len: usize) -> i32 {
    let messenger = unsafe { &*(messenger as *const Messenger) };

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            match messenger.receive().await {
                Ok(msg) => {
                    let topic_bytes = msg.topic.as_bytes();
                    let payload_bytes = msg.payload.as_slice();

                    let topic_copy_len = std::cmp::min(topic_bytes.len(), topic_len - 1);
                    let payload_copy_len = std::cmp::min(payload_bytes.len(), buffer_len - 1);

                    unsafe {
                        std::ptr::copy_nonoverlapping(topic_bytes.as_ptr(), topic as *mut u8, topic_copy_len);
                        *topic.add(topic_copy_len) = 0;

                        std::ptr::copy_nonoverlapping(payload_bytes.as_ptr(), buffer as *mut u8, payload_copy_len);
                        *buffer.add(payload_copy_len) = 0;
                    }

                    payload_copy_len as i32
                },
                Err(_) => -1,
            }
        })
}

#[no_mangle]
pub extern "C" fn zark_messenger_cleanup(messenger: *mut c_void) {
    let messenger = unsafe { &*(messenger as *const Messenger) };
    messenger.cleanup();
}

#[no_mangle]
pub extern "C" fn zark_messenger_free(messenger: *mut c_void) {
    unsafe {
        drop(Box::from_raw(messenger as *mut Messenger));
    }
}