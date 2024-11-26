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

use crate::domain::errors::MessengerError;
use crate::domain::message::Message;

pub trait Serializable: Sized {
    fn serialize(&self) -> Result<Vec<u8>, MessengerError>;
    fn deserialize(data: &[u8]) -> Result<Self, MessengerError>;
}

impl Serializable for Message {
    fn serialize(&self) -> Result<Vec<u8>, MessengerError> {
        let mut result = Vec::new();
        
        // serialize topic
        let topic_bytes = self.topic.as_bytes();
        result.extend_from_slice(&(topic_bytes.len() as u32).to_le_bytes());
        result.extend_from_slice(topic_bytes);
        
        // serialize id
        let id_bytes = self.id.as_bytes();
        result.extend_from_slice(&(id_bytes.len() as u32).to_le_bytes());
        result.extend_from_slice(id_bytes);
        
        // serialize payload
        result.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        result.extend_from_slice(&self.payload);
        
        Ok(result)
    }

    fn deserialize(data: &[u8]) -> Result<Self, MessengerError> {
        let mut cursor = 0;
        
        // helper function to read a length-prefixed string
        let read_string = |cursor: &mut usize| -> Result<String, MessengerError> {
            if *cursor + 4 > data.len() {
                return Err(MessengerError::Deserialization("Incomplete data".to_string()));
            }
            let len = u32::from_le_bytes([data[*cursor], data[*cursor+1], data[*cursor+2], data[*cursor+3]]) as usize;
            *cursor += 4;
            if *cursor + len > data.len() {
                return Err(MessengerError::Deserialization("Incomplete data".to_string()));
            }
            let s = String::from_utf8(data[*cursor..*cursor+len].to_vec())
                .map_err(|e| MessengerError::Deserialization(e.to_string()))?;
            *cursor += len;
            Ok(s)
        };
        
        // Read topic
        let topic = read_string(&mut cursor)?;
        
        // Read id
        let id = read_string(&mut cursor)?;
        
        // Read payload
        if cursor + 4 > data.len() {
            return Err(MessengerError::Deserialization("Incomplete data".to_string()));
        }
        let payload_len = u32::from_le_bytes([data[cursor], data[cursor+1], data[cursor+2], data[cursor+3]]) as usize;
        cursor += 4;
        if cursor + payload_len > data.len() {
            return Err(MessengerError::Deserialization("Incomplete data".to_string()));
        }
        let payload = data[cursor..cursor+payload_len].to_vec();
        
        Ok(Message { topic, id, payload })
    }
}

// Implement Serializable for Vec<u8> (raw bytes)
impl Serializable for Vec<u8> {
    fn serialize(&self) -> Result<Vec<u8>, MessengerError> {
        Ok(self.clone())
    }

    fn deserialize(data: &[u8]) -> Result<Self, MessengerError> {
        Ok(data.to_vec())
    }
}