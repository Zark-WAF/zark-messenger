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

use std::cell::UnsafeCell;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct PoolAllocator<T> {
    chunks: AtomicPtr<Chunk<T>>,
    chunk_size: usize,
}

struct Chunk<T> {
    data: UnsafeCell<[T; 64]>,
    next: AtomicPtr<Chunk<T>>,
    free_list: AtomicPtr<FreeListNode>,
}

struct FreeListNode {
    next: AtomicPtr<FreeListNode>,
}

impl<T> PoolAllocator<T> {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunks: AtomicPtr::new(std::ptr::null_mut()),
            chunk_size,
        }
    }

    pub fn allocate(&self) -> NonNull<T> {
        loop {
            let chunk = self.chunks.load(Ordering::Acquire);
            if !chunk.is_null() {
                if let Some(ptr) = unsafe { (*chunk).allocate() } {
                    return ptr;
                }
            }
            self.add_chunk();
        }
    }

    pub fn deallocate(&self, ptr: NonNull<T>) {
        unsafe {
            let chunk = self.find_chunk(ptr);
            (*chunk).deallocate(ptr);
        }
    }

    fn add_chunk(&self) {
        let new_chunk = Box::into_raw(Box::new(Chunk::new()));
        loop {
            let old_head = self.chunks.load(Ordering::Relaxed);
            unsafe { (*new_chunk).next.store(old_head, Ordering::Relaxed) };
            if self.chunks.compare_exchange(old_head, new_chunk, Ordering::Release, Ordering::Relaxed).is_ok() {
                break;
            }
        }
    }

    unsafe fn find_chunk(&self, ptr: NonNull<T>) -> *mut Chunk<T> {
        let mut current = self.chunks.load(Ordering::Acquire);
        while !current.is_null() {
            if (*current).contains(ptr) {
                return current;
            }
            current = (*current).next.load(Ordering::Acquire);
        }
        panic!("Pointer not found in any chunk");
    }
}

impl<T> Chunk<T> {
    fn new() -> Self {
        let mut chunk = Self {
            data: UnsafeCell::new(unsafe { std::mem::uninitialized() }),
            next: AtomicPtr::new(std::ptr::null_mut()),
            free_list: AtomicPtr::new(std::ptr::null_mut()),
        };
        chunk.initialize_free_list();
        chunk
    }

    fn initialize_free_list(&mut self) {
        let data = self.data.get() as *mut T;
        let num_nodes = 63;
        for i in 0..num_nodes {
            let node = unsafe { data.add(i) } as *mut FreeListNode;
            let next_node = unsafe { data.add(i + 1) } as *mut FreeListNode;
            unsafe {
                (*node).next = AtomicPtr::new(next_node);
            }
        }
        let last_node = unsafe { &mut *(data.add(num_nodes) as *mut FreeListNode) };
        last_node.next = AtomicPtr::new(std::ptr::null_mut());
        self.free_list = AtomicPtr::new(data as *mut FreeListNode);
    }

    unsafe fn allocate(&self) -> Option<NonNull<T>> {
        loop {
            let node = self.free_list.load(Ordering::Acquire);
            if node.is_null() {
                return None;
            }
            let next = (*node).next.load(Ordering::Relaxed);
            if self.free_list.compare_exchange(node, next, Ordering::Release, Ordering::Relaxed).is_ok() {
                return Some(NonNull::new_unchecked(node as *mut T));
            }
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<T>) {
        let node = ptr.as_ptr() as *mut FreeListNode;
        loop {
            let head = self.free_list.load(Ordering::Relaxed);
            (*node).next.store(head, Ordering::Relaxed);
            if self.free_list.compare_exchange(head, node, Ordering::Release, Ordering::Relaxed).is_ok() {
                break;
            }
        }
    }

    fn contains(&self, ptr: NonNull<T>) -> bool {
        let start = self.data.get() as *mut T;
        let end = unsafe { start.add(64) };
        ptr.as_ptr() >= start && ptr.as_ptr() < end
    }
}