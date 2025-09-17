//! End-to-End Tests for Project Kenshō Frontend
//! 
//! These tests verify the complete user experience including:
//! - User authentication flow
//! - Search and discovery
//! - Navigation and routing
//! - Streaming initiation
//! - Error handling
//! - Performance metrics

pub mod user_journey;
pub mod test_utils;

#[cfg(test)]
mod test_runner {
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    /// Initialize test environment
    pub fn init_test_env() {
        // Set up console logging
        console_log::init_with_level(log::Level::Debug).ok();
        
        // Set test mode flag
        if let Some(window) = web_sys::window() {
            let local_storage = window.local_storage().unwrap().unwrap();
            local_storage.set_item("test_mode", "true").ok();
        }
    }
    
    /// Clean up after tests
    pub fn cleanup_test_env() {
        if let Some(window) = web_sys::window() {
            let local_storage = window.local_storage().unwrap().unwrap();
            local_storage.remove_item("test_mode").ok();
            local_storage.remove_item("auth_token").ok();
            local_storage.remove_item("refresh_token").ok();
        }
    }
}