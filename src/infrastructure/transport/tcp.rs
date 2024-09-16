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

use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use crate::domain::message::Message;
use crate::domain::errors::MessengerError;
use super::Transport;
use crate::application::config::TcpConfig;
use crate::infrastructure::serialization::Serializer;

pub struct TcpTransport {
    listener: Option<Arc<TcpListener>>,
    stream: Option<Arc<TcpStream>>,
    config: TcpConfig,
    serializer: Box<dyn Serializer>,
}


#[async_trait]
impl Transport for TcpTransport {
    async fn send(&self, msg: Message) -> Result<(), MessengerError> {
        let serialized = self.serializer.serialize(&msg)?;
        let len = serialized.len() as u32;
        let len_bytes = len.to_be_bytes();

        if let Some(stream) = &self.stream {
            stream.write_all(&len_bytes).await?;
            stream.write_all(&serialized).await?;
            stream.flush().await?;
            Ok(())
        } else {
            Err(MessengerError::TransportError("Not connected".into()))
        }
    }

    async fn receive(&self) -> Result<Message, MessengerError> {
        if let Some(stream) = &self.stream {
            let mut len_bytes = [0u8; 4];
            stream.read_exact(&mut len_bytes).await?;
            let len = u32::from_be_bytes(len_bytes) as usize;

            let mut buffer = vec![0u8; len];
            stream.read_exact(&mut buffer).await?;

            self.serializer.deserialize(&buffer)
        } else {
            Err(MessengerError::TransportError("Not connected".into()))
        }
    }

    fn cleanup(&self) {
       // here is nothing to clean up. this is a memoryless transport and it's the responsibility of the network to clean up after itself. 
       // this is a no-op.
    }
}

impl TcpTransport {
    pub async fn new_server(config: TcpConfig, serializer: Box<dyn Serializer>) -> Result<Self, MessengerError> {
        let addr = format!("{}:{}", config.host, config.port);
        let listener = TcpListener::bind(&addr).await?;
        Ok(Self {
            listener: Some(Arc::new(listener)),
            stream: None,
            config,
            serializer,
        })
    }

    pub async fn accept(&mut self) -> Result<(), MessengerError> {
        if let Some(listener) = &self.listener {
            let (stream, _) = listener.accept().await?;
            self.stream = Some(Arc::new(stream));
            Ok(())
        } else {
            Err(MessengerError::TransportError("Not a server".into()))
        }
    }
}