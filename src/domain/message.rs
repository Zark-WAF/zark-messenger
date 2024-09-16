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

use serde::{Serialize, Deserialize};
use crate::utils::zark_uid::generate_zark_uid;


#[derive(Debug, Clone, Serialize, Deserialize)]
// message struct represents a single message in the messaging system
pub struct Message {
    // topic is used to categorize and route messages to appropriate recipients
    pub topic: String,
    // id is a unique identifier for each message, allowing tracking and deduplication
    pub id: String,
    // payload contains the actual content of the message as a byte vector
    pub payload: Vec<u8>,
}

impl Message {
    // new is a constructor method for creating a new message instance
    pub fn new(topic: String, payload: Vec<u8>) -> Self {
        // create a new message with the given topic and payload
        // generate a unique id for the message using the zark_uid generator
        Self { topic, id: generate_zark_uid(), payload }
    }
}