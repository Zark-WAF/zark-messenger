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

use rand::seq::SliceRandom;

// function to generate a unique identifier for zark-waf
pub fn generate_zark_uid() -> String {
    // create a random number generator
    let mut rng = rand::thread_rng();
    // define the character set for the uid (uppercase letters and numbers)
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
    
    // initialize an empty string to store the uid
    let mut uid = String::new();
    // loop 19 times to create a 19-character uid (including hyphens)
    for i in 0..19 {
        if i > 0 && i % 5 == 0 {
            // add a hyphen after every 5 characters (except at the start)
            uid.push('-');
        } else {
            // randomly choose a character from the defined set and add it to the uid
            uid.push(*chars.choose(&mut rng).unwrap());
        }
    }
    
    // return the generated uid
    uid
}

// this function is needed to:
// 1. create unique identifiers for messages in zark-waf
// 2. ensure that these identifiers are random and hard to predict
// 3. provide a consistent format for identifiers (4 groups of 5 characters separated by hyphens)
// 4. allow for easy tracking and referencing of zark-waf components or sessions
// 5. potentially use in logging, debugging, or tracking specific instances or operations within the waf
