//! Test utilities for E2E tests

use std::time::Duration;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement, Document};

/// Test configuration
pub struct TestConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub retry_interval: Duration,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            timeout: Duration::from_secs(10),
            retry_interval: Duration::from_millis(100),
        }
    }
}

/// Page object for common interactions
pub struct PageObject {
    document: Document,
}

impl PageObject {
    pub fn new() -> Self {
        let window = web_sys::window().expect("Window should exist");
        let document = window.document().expect("Document should exist");
        
        Self { document }
    }
    
    /// Find element by selector
    pub fn find_element(&self, selector: &str) -> Option<HtmlElement> {
        self.document
            .query_selector(selector)
            .ok()
            .flatten()
            .and_then(|el| el.dyn_into::<HtmlElement>().ok())
    }
    
    /// Find all elements by selector
    pub fn find_elements(&self, selector: &str) -> Vec<HtmlElement> {
        let node_list = self.document
            .query_selector_all(selector)
            .expect("Selector should be valid");
        
        let mut elements = Vec::new();
        for i in 0..node_list.length() {
            if let Some(node) = node_list.item(i) {
                if let Ok(element) = node.dyn_into::<HtmlElement>() {
                    elements.push(element);
                }
            }
        }
        
        elements
    }
    
    /// Click on element
    pub fn click(&self, selector: &str) -> Result<(), String> {
        self.find_element(selector)
            .ok_or_else(|| format!("Element not found: {}", selector))
            .map(|el| el.click())
    }
    
    /// Set input value
    pub fn set_input_value(&self, selector: &str, value: &str) -> Result<(), String> {
        self.document
            .query_selector(selector)
            .ok()
            .flatten()
            .and_then(|el| el.dyn_into::<HtmlInputElement>().ok())
            .ok_or_else(|| format!("Input not found: {}", selector))
            .map(|input| input.set_value(value))
    }
    
    /// Get element text
    pub fn get_text(&self, selector: &str) -> Option<String> {
        self.find_element(selector)
            .map(|el| el.text_content())
            .flatten()
    }
    
    /// Check if element exists
    pub fn element_exists(&self, selector: &str) -> bool {
        self.find_element(selector).is_some()
    }
    
    /// Check if element is visible
    pub fn is_visible(&self, selector: &str) -> bool {
        self.find_element(selector)
            .map(|el| {
                let style = web_sys::window()
                    .unwrap()
                    .get_computed_style(&el)
                    .unwrap()
                    .unwrap();
                
                style.get_property_value("display").unwrap() != "none"
                    && style.get_property_value("visibility").unwrap() != "hidden"
            })
            .unwrap_or(false)
    }
    
    /// Wait for element to appear
    pub async fn wait_for_element(&self, selector: &str, timeout_ms: u32) -> bool {
        let start = js_sys::Date::now();
        
        loop {
            if self.element_exists(selector) {
                return true;
            }
            
            if js_sys::Date::now() - start > timeout_ms as f64 {
                return false;
            }
            
            // Sleep for retry interval
            sleep_ms(100).await;
        }
    }
    
    /// Wait for element to be visible
    pub async fn wait_for_visible(&self, selector: &str, timeout_ms: u32) -> bool {
        let start = js_sys::Date::now();
        
        loop {
            if self.is_visible(selector) {
                return true;
            }
            
            if js_sys::Date::now() - start > timeout_ms as f64 {
                return false;
            }
            
            sleep_ms(100).await;
        }
    }
    
    /// Wait for element to disappear
    pub async fn wait_for_disappear(&self, selector: &str, timeout_ms: u32) -> bool {
        let start = js_sys::Date::now();
        
        loop {
            if !self.element_exists(selector) {
                return true;
            }
            
            if js_sys::Date::now() - start > timeout_ms as f64 {
                return false;
            }
            
            sleep_ms(100).await;
        }
    }
}

/// Sleep for specified milliseconds
pub async fn sleep_ms(ms: u32) {
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                ms as i32,
            )
            .unwrap();
    }))
    .await
    .unwrap();
}

/// Mock data for testing
pub mod mock_data {
    use serde_json::json;
    
    pub fn mock_anime() -> serde_json::Value {
        json!({
            "id": "test-anime-1",
            "title": "Test Anime",
            "description": "A test anime for E2E testing",
            "poster_url": "/images/test-poster.jpg",
            "episode_count": 12,
            "status": "CURRENTLY_AIRING",
            "rating": 8.5,
            "tags": ["Action", "Adventure"]
        })
    }
    
    pub fn mock_episode() -> serde_json::Value {
        json!({
            "id": "test-episode-1",
            "anime_id": "test-anime-1",
            "episode_number": 1,
            "title": "Episode 1",
            "duration_ms": 1440000,
            "thumbnail_url": "/images/test-episode.jpg"
        })
    }
    
    pub fn mock_user() -> serde_json::Value {
        json!({
            "email": "test@example.com",
            "password": "password123"
        })
    }
    
    pub fn mock_auth_response() -> serde_json::Value {
        json!({
            "access_token": "mock-access-token",
            "refresh_token": "mock-refresh-token",
            "expires_in": 3600
        })
    }
}

/// Test assertions
pub mod assertions {
    use super::PageObject;
    
    pub fn assert_element_exists(page: &PageObject, selector: &str, message: &str) {
        assert!(
            page.element_exists(selector),
            "{}: Element '{}' not found",
            message,
            selector
        );
    }
    
    pub fn assert_element_visible(page: &PageObject, selector: &str, message: &str) {
        assert!(
            page.is_visible(selector),
            "{}: Element '{}' not visible",
            message,
            selector
        );
    }
    
    pub fn assert_text_equals(page: &PageObject, selector: &str, expected: &str, message: &str) {
        let actual = page.get_text(selector).unwrap_or_default();
        assert_eq!(
            actual.trim(),
            expected,
            "{}: Text mismatch for '{}'",
            message,
            selector
        );
    }
    
    pub fn assert_text_contains(page: &PageObject, selector: &str, expected: &str, message: &str) {
        let actual = page.get_text(selector).unwrap_or_default();
        assert!(
            actual.contains(expected),
            "{}: Text '{}' not found in '{}'",
            message,
            expected,
            selector
        );
    }
}

/// Performance metrics collector
pub struct PerformanceMetrics {
    marks: std::collections::HashMap<String, f64>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            marks: std::collections::HashMap::new(),
        }
    }
    
    pub fn mark(&mut self, name: &str) {
        self.marks.insert(name.to_string(), js_sys::Date::now());
    }
    
    pub fn measure(&self, start_mark: &str, end_mark: &str) -> Option<f64> {
        match (self.marks.get(start_mark), self.marks.get(end_mark)) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
    
    pub fn assert_performance(&self, start_mark: &str, end_mark: &str, max_ms: f64, operation: &str) {
        if let Some(duration) = self.measure(start_mark, end_mark) {
            assert!(
                duration <= max_ms,
                "{} took {}ms, expected under {}ms",
                operation,
                duration,
                max_ms
            );
        }
    }
}