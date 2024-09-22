use std::env;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
pub fn get_app_data_dir() -> PathBuf {
    let app_data = env::var("APPDATA").expect("Failed to get APPDATA directory");
    PathBuf::from(app_data)
}

#[cfg(target_os = "macos")]
pub fn get_app_data_dir() -> PathBuf {
    let home = env::var("HOME").expect("Failed to get HOME directory");
    PathBuf::from(home).join("Library").join("Application Support")
}

#[cfg(target_os = "linux")]
pub fn get_app_data_dir() -> PathBuf {
    let home = env::var("HOME").expect("Failed to get HOME directory");
    PathBuf::from(home).join(".config")
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn get_app_data_dir() -> PathBuf {
    panic!("Unsupported operating system")
}

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

pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

pub fn is_linux() -> bool {
    cfg!(target_os = "linux")
}
