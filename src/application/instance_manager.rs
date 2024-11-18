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

use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicPtr, Ordering};
use std::ffi::c_void;
use lazy_static::lazy_static;

pub struct InstanceManager {
    pub active_connections: AtomicUsize,
    pub is_shutting_down: AtomicBool,
    messenger_ptr: AtomicPtr<c_void>,
}

impl InstanceManager {
    pub fn new() -> Self {
        Self {
            active_connections: AtomicUsize::new(0),
            is_shutting_down: AtomicBool::new(false),
            messenger_ptr: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    pub fn get_messenger(&self) -> Option<*mut c_void> {
        let ptr = self.messenger_ptr.load(Ordering::SeqCst);
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    pub fn set_messenger(&self, messenger: *mut c_void) {
        self.messenger_ptr.store(messenger, Ordering::SeqCst);
    }

    pub fn register_instance(&self) {
        self.active_connections.fetch_add(1, Ordering::SeqCst);
    }

    pub fn unregister_instance(&self) {
        self.active_connections.fetch_sub(1, Ordering::SeqCst);
    }
}

lazy_static! {
    pub static ref INSTANCE_MANAGER: InstanceManager = InstanceManager::new();
}

