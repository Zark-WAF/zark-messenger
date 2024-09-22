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

use serde::Deserialize;


// main configuration struct for the messenger
// this struct holds all the necessary configuration options for the messenger
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // specifies which transport type to use (ipc or tcp)
    pub transport_type: TransportType,
    // configuration for ipc transport, if used
    pub ipc_config: Option<IpcConfig>,
    // configuration for tcp transport, if used
    pub tcp_config: Option<TcpConfig>,
}

// enum to represent the available transport types
// this allows the user to choose between ipc and tcp communication
#[derive(Debug, Clone, Deserialize)]
pub enum TransportType {
    // inter-process communication
    IPC,
    // transmission control protocol
    TCP,
}

// configuration struct for ipc transport
// holds specific settings needed for ipc communication
#[derive(Debug, Clone, Deserialize)]
pub struct IpcConfig {
    // name of the shared memory segment to be used for ipc
    pub shared_memory_name: String,
    // maximum size of messages that can be sent via ipc
    pub max_message_size: usize,

    pub max_queue_size: usize,

    pub max_buffer_size: usize,
}

// configuration struct for tcp transport
// holds specific settings needed for tcp communication
#[derive(Debug, Clone, Deserialize)]
pub struct TcpConfig {
    // host address for tcp connection
    pub host: String,
    // port number for tcp connection
    pub port: u16,

    // maximum size of messages that can be sent via tcp
    pub max_message_size: usize,
}