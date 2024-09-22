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

use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr::NonNull;

struct Node<T> {
    data: T,
    next: AtomicPtr<Node<T>>,
}

pub struct LockFreeQueue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

impl<T> LockFreeQueue<T> {
    pub fn new() -> Self {
        let sentinel = Box::into_raw(Box::new(Node {
            data: unsafe { std::mem::uninitialized() },
            next: AtomicPtr::new(std::ptr::null_mut()),
        }));
        Self {
            head: AtomicPtr::new(sentinel),
            tail: AtomicPtr::new(sentinel),
        }
    }

    pub fn enqueue(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data,
            next: AtomicPtr::new(std::ptr::null_mut()),
        }));

        loop {
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { (*tail).next.load(Ordering::Acquire) };

            if next.is_null() {
                if unsafe { (*tail).next.compare_exchange(next, new_node, Ordering::Release, Ordering::Relaxed).is_ok() } {
                    let _ = self.tail.compare_exchange(tail, new_node, Ordering::Release, Ordering::Relaxed);
                    return;
                }
            } else {
                let _ = self.tail.compare_exchange(tail, next, Ordering::Release, Ordering::Relaxed);
            }
        }
    }

    pub fn dequeue(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { (*head).next.load(Ordering::Acquire) };

            if head == tail {
                if next.is_null() {
                    return None;
                }
                let _ = self.tail.compare_exchange(tail, next, Ordering::Release, Ordering::Relaxed);
            } else {
                if self.head.compare_exchange(head, next, Ordering::Release, Ordering::Relaxed).is_ok() {
                    let data = unsafe { std::ptr::read(&(*next).data) };
                    unsafe { Box::from_raw(head); }
                    return Some(data);
                }
            }
        }
    }
}