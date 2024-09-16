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

use super::Transport;
use crate::application::config::TcpConfig;
use crate::domain::errors::MessengerError;
use crate::domain::message::Message;
use crate::infrastructure::serialization::Serializer;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

// tcp transport struct for handling tcp connections
pub struct TcpTransport {
    // optional listener for server mode
    listener: Option<Arc<TcpListener>>,
    // optional stream for client mode or accepted connection
    stream: Option<Arc<Mutex<TcpStream>>>,
    // configuration for tcp connection
    config: TcpConfig,
    // serializer for message encoding/decoding
    serializer: Box<dyn Serializer>,
}

#[async_trait]
impl Transport for TcpTransport {
    // send a message over tcp
    async fn send(&self, msg: Message) -> Result<(), MessengerError> {
        // serialize the message
        let serialized = self.serializer.serialize(&msg)?;
        // get the length of the serialized message
        let len = serialized.len() as u32;
        // convert length to big-endian bytes
        let len_bytes = len.to_be_bytes();

        // if we have an active stream
        if let Some(stream) = &self.stream {
            let mut stream = stream.lock().await;

            // write the length of the message
            stream.write_all(&len_bytes).await?;

            // write the serialized message
            stream.write_all(&serialized).await?;

            // ensure all data is sent
            stream.flush().await?;
            Ok(())
        } else {
            Err(MessengerError::TransportError("Not connected".into()))
        }
    }

    // receive a message over tcp
    async fn receive(&self) -> Result<Message, MessengerError> {
        // if we have an active stream
        if let Some(stream) = &self.stream {
            // lock the stream
            let mut stream = stream.lock().await;

            // prepare buffer for message length
            let mut len_bytes = [0u8; 4];
            // read the length of the incoming message
            stream.read_exact(&mut len_bytes).await?;
            // convert bytes to length
            let len = u32::from_be_bytes(len_bytes) as usize;

            // prepare buffer for the message
            let mut buffer = vec![0u8; len];
            // read the message
            stream.read_exact(&mut buffer).await?;

            // deserialize and return the message
            self.serializer.deserialize(&buffer)
        } else {
            // return error if not connected
            Err(MessengerError::TransportError("Not connected".into()))
        }
    }

    // cleanup function (no-op for tcp)
    async fn cleanup(&self) {
        // here is nothing to clean up. this is a memoryless transport and it's the responsibility of the network to clean up after itself.
        // this is a no-op.
    }
}

impl TcpTransport {
    // create a new tcp server
    pub async fn new_server(
        config: TcpConfig,
        serializer: Box<dyn Serializer>,
    ) -> Result<Self, MessengerError> {
        // create address string from host and port
        let addr = format!("{}:{}", config.host, config.port);
        // bind to the address
        let listener = TcpListener::bind(&addr).await?;
        // return new TcpTransport instance
        Ok(Self {
            listener: Some(Arc::new(listener)),
            stream: None,
            config,
            serializer,
        })
    }

    // accept a new connection (for server mode)
    pub async fn accept(&mut self) -> Result<(), MessengerError> {
        // if we have a listener
        if let Some(listener) = &self.listener {
            // accept a new connection
            let (stream, _) = listener.accept().await?;
            // store the new stream
            self.stream = Some(Arc::new(Mutex::new(stream)));
            Ok(())
        } else {
            // return error if not in server mode
            Err(MessengerError::TransportError("Not a server".into()))
        }
    }
}
