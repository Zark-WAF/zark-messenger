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

use std::sync::{Arc, Mutex, Condvar};
use std::time::Duration;

pub struct Waiter {
    mutex: Mutex<bool>,
    condvar: Condvar,
}

impl Waiter {
    pub fn new() -> Self {
        Self {
            mutex: Mutex::new(false),
            condvar: Condvar::new(),
        }
    }

    pub fn wait(&self) {
        let mut notified = self.mutex.lock().unwrap();
        while !*notified {
            notified = self.condvar.wait(notified).unwrap();
        }
        *notified = false;
    }

    pub fn wait_timeout(&self, timeout: Duration) -> bool {
        let mut notified = self.mutex.lock().unwrap();
        if !*notified {
            let result = self.condvar.wait_timeout(notified, timeout).unwrap();
            notified = result.0;
            if result.1.timed_out() {
                return false;
            }
        }
        *notified = false;
        true
    }

    pub fn notify_one(&self) {
        let mut notified = self.mutex.lock().unwrap();
        *notified = true;
        self.condvar.notify_one();
    }

    pub fn notify_all(&self) {
        let mut notified = self.mutex.lock().unwrap();
        *notified = true;
        self.condvar.notify_all();
    }
}