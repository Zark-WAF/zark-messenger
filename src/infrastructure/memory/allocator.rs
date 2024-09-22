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

use std::alloc::{GlobalAlloc, Layout};
use std::cell::UnsafeCell;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MemoryAllocator {
    heap: UnsafeCell<*mut u8>,
    heap_size: usize,
    allocated: AtomicUsize,
}

unsafe impl Sync for MemoryAllocator {}

impl MemoryAllocator {
    pub fn new(heap_size: usize) -> Self {
        let layout = Layout::from_size_align(heap_size, 8).unwrap();
        let heap = unsafe { std::alloc::alloc(layout) };
        Self {
            heap: UnsafeCell::new(heap),
            heap_size,
            allocated: AtomicUsize::new(0),
        }
    }
}

unsafe impl GlobalAlloc for MemoryAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        let allocated = self.allocated.load(Ordering::Relaxed);

        if allocated + size > self.heap_size {
            return std::ptr::null_mut();
        }

        let heap = *self.heap.get();
        let offset = (heap as usize + allocated + align - 1) & !(align - 1);
        self.allocated.store(offset - heap as usize + size, Ordering::Relaxed);

        offset as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
       
    }
}