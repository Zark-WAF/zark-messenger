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

use std::sync::Arc;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use shm::SharedMemory;

pub struct Buffer {
    inner: Arc<BufferInner>,
}

struct BufferInner {
    ptr: NonNull<u8>,
    size: usize,
    shm: SharedMemory,
    destructor: Option<Box<dyn Fn(*mut u8, usize) + Send + Sync>>,
}

impl Buffer {
    pub fn new(size: usize) -> Result<Self, std::io::Error> {
        let shm = SharedMemory::create(size)?;
        let ptr = NonNull::new(shm.as_ptr() as *mut u8).unwrap();
        Ok(Self {
            inner: Arc::new(BufferInner {
                ptr,
                size,
                shm,
                destructor: None,
            }),
        })
    }

    pub fn from_existing(name: &str) -> Result<Self, std::io::Error> {
        let shm = SharedMemory::open(name)?;
        let ptr = NonNull::new(shm.as_ptr() as *mut u8).unwrap();
        let size = shm.len();
        Ok(Self {
            inner: Arc::new(BufferInner {
                ptr,
                size,
                shm,
                destructor: None,
            }),
        })
    }

    pub fn with_destructor<F>(size: usize, destructor: F) -> Result<Self, std::io::Error>
    where
        F: Fn(*mut u8, usize) + Send + Sync + 'static,
    {
        let mut buffer = Self::new(size)?;
        Arc::get_mut(&mut buffer.inner).unwrap().destructor = Some(Box::new(destructor));
        Ok(buffer)
    }

    pub fn len(&self) -> usize {
        self.inner.size
    }

    pub fn is_empty(&self) -> bool {
        self.inner.size == 0
    }
}

impl Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.inner.ptr.as_ptr(), self.inner.size) }
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.inner.ptr.as_ptr(), self.inner.size) }
    }
}

impl Drop for BufferInner {
    fn drop(&mut self) {
        if let Some(destructor) = &self.destructor {
            destructor(self.ptr.as_ptr(), self.size);
        }
    }
}

impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.deref() == other.deref()
    }
}

// Buffering mechanism
pub struct BufferPool {
    buffers: Vec<Buffer>,
    size: usize,
}

impl BufferPool {
    pub fn new(buffer_size: usize, pool_size: usize) -> Result<Self, std::io::Error> {
        let mut buffers = Vec::with_capacity(pool_size);
        for _ in 0..pool_size {
            buffers.push(Buffer::new(buffer_size)?);
        }
        Ok(Self {
            buffers,
            size: buffer_size,
        })
    }

    pub fn get_buffer(&mut self) -> Option<Buffer> {
        self.buffers.pop().or_else(|| Buffer::new(self.size).ok())
    }

    pub fn return_buffer(&mut self, buffer: Buffer) {
        if self.buffers.len() < self.buffers.capacity() {
            self.buffers.push(buffer);
        }
    }
}
