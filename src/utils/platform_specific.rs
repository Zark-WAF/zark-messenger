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

use std::env;
use std::path::PathBuf;

/// Returns the application data directory based on the operating system.
///
/// # Panics
///
/// Panics if the environment variable for the application data directory is not found.
pub fn get_app_data_dir() -> PathBuf {
    match env::var("APPDATA") {
        Ok(app_data) => PathBuf::from(app_data),
        Err(_) => {
            match env::var("HOME") {
                Ok(home) => {
                    #[cfg(target_os = "macos")]
                    return PathBuf::from(home).join("Library").join("Application Support");
                    #[cfg(target_os = "linux")]
                    return PathBuf::from(home).join(".config");
                    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
                    return PathBuf::from(home).join(".config"); // Default to .config for unknown Unix-like systems
                }
                Err(_) => panic!("Failed to get HOME directory"),
            }
        }
    }
}

/// Returns the name of the current operating system.
pub fn get_platform_name() -> &'static str {
    #[cfg(target_os = "windows")]
    { "windows" }
    #[cfg(target_os = "macos")]
    { "macos" }
    #[cfg(target_os = "linux")]
    { "linux" }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    { "unknown" }
}

/// Checks if the current operating system is Windows.
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

/// Checks if the current operating system is macOS.
pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

/// Checks if the current operating system is Linux.
pub fn is_linux() -> bool {
    cfg!(target_os = "linux")
}