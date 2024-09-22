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
use serde_json;
use crate::domain::message::Message;
use crate::domain::errors::MessengerError;
use super::Serializer;

// json serializer implementation
pub struct JsonSerializer;

#[async_trait]
impl Serializer for JsonSerializer {
    // convert message to json bytes
    fn serialize(&self, msg: &Message) -> Result<Vec<u8>, MessengerError> {
        serde_json::to_vec(msg)
            .map_err(|e| MessengerError::Serialization(e.to_string()))
    }

    // convert json bytes to message
    fn deserialize(&self, data: &[u8]) -> Result<Message, MessengerError> {
        serde_json::from_slice(data)
            .map_err(|e| MessengerError::Serialization(e.to_string()))
    }
}