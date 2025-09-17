//! E2E Test T022: Complete User Journey
//! 
//! Tests the full user experience from landing page through authentication,
//! search, discovery, and streaming initiation.

use dioxus::prelude::*;
use dioxus_web::launch;
use wasm_bindgen_test::*;
use web_sys::{HtmlElement, HtmlInputElement};

wasm_bindgen_test_configure!(run_in_browser);

/// Helper to get element by selector
fn get_element(selector: &str) -> Option<HtmlElement> {
    let document = web_sys::window()?.document()?;
    let element = document.query_selector(selector).ok()??;
    element.dyn_into::<HtmlElement>().ok()
}

/// Helper to get input element
fn get_input(selector: &str) -> Option<HtmlInputElement> {
    let document = web_sys::window()?.document()?;
    let element = document.query_selector(selector).ok()??;
    element.dyn_into::<HtmlInputElement>().ok()
}

/// Helper to click element
fn click_element(selector: &str) {
    if let Some(element) = get_element(selector) {
        element.click();
    }
}

/// Helper to set input value
fn set_input_value(selector: &str, value: &str) {
    if let Some(input) = get_input(selector) {
        input.set_value(value);
    }
}

/// Helper to wait for element to appear
async fn wait_for_element(selector: &str, timeout_ms: i32) -> bool {
    let start = js_sys::Date::now();
    
    loop {
        if get_element(selector).is_some() {
            return true;
        }
        
        if js_sys::Date::now() - start > timeout_ms as f64 {
            return false;
        }
        
        // Sleep for 100ms
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    &resolve,
                    100,
                )
                .unwrap();
        }))
        .await
        .unwrap();
    }
}

#[wasm_bindgen_test]
async fn test_complete_user_journey() {
    // Initialize test environment
    console_log::init_with_level(log::Level::Debug).ok();
    
    // Step 1: User lands on home page
    log::info!("Step 1: Landing on home page");
    assert!(
        wait_for_element(".hero-section", 5000).await,
        "Home page should load with hero section"
    );
    
    assert!(
        get_element(".trending-section").is_some(),
        "Trending section should be visible"
    );
    
    // Step 2: User navigates to login
    log::info!("Step 2: Navigating to login");
    click_element("a[href='/login']");
    
    assert!(
        wait_for_element(".login-form", 3000).await,
        "Login form should appear"
    );
    
    // Step 3: User enters credentials
    log::info!("Step 3: Entering login credentials");
    set_input_value("input[name='email']", "test@example.com");
    set_input_value("input[name='password']", "password123");
    click_element("button[type='submit']");
    
    // Step 4: Wait for authentication and redirect
    log::info!("Step 4: Authenticating and redirecting");
    assert!(
        wait_for_element(".user-profile", 5000).await,
        "User profile should appear after login"
    );
    
    // Step 5: User searches for anime
    log::info!("Step 5: Searching for anime");
    set_input_value(".search-input", "Spy x Family");
    click_element(".search-button");
    
    assert!(
        wait_for_element(".search-results", 3000).await,
        "Search results should appear"
    );
    
    // Step 6: User clicks on anime from search results
    log::info!("Step 6: Selecting anime from results");
    click_element(".search-results .anime-card:first-child");
    
    assert!(
        wait_for_element(".series-detail", 3000).await,
        "Series detail page should load"
    );
    
    // Verify series details are displayed
    assert!(
        get_element(".anime-title").is_some(),
        "Anime title should be displayed"
    );
    
    assert!(
        get_element(".anime-description").is_some(),
        "Anime description should be displayed"
    );
    
    assert!(
        get_element(".episode-list").is_some(),
        "Episode list should be displayed"
    );
    
    // Step 7: User selects an episode to watch
    log::info!("Step 7: Selecting episode to watch");
    click_element(".episode-list .episode-item:first-child .watch-button");
    
    assert!(
        wait_for_element(".video-player", 5000).await,
        "Video player should initialize"
    );
    
    // Step 8: Verify streaming setup
    log::info!("Step 8: Verifying streaming setup");
    let video_element = get_element("video");
    assert!(video_element.is_some(), "Video element should be present");
    
    // Check if stream URL was loaded
    if let Some(video) = video_element {
        let src = video.get_attribute("src");
        assert!(src.is_some(), "Video should have a source URL");
    }
    
    log::info!("✅ Complete user journey test passed!");
}

#[wasm_bindgen_test]
async fn test_browse_seasonal_anime() {
    // Test browsing current season anime
    log::info!("Testing seasonal browse functionality");
    
    // Navigate to browse page
    click_element("a[href='/browse']");
    assert!(
        wait_for_element(".browse-page", 3000).await,
        "Browse page should load"
    );
    
    // Select current season
    click_element(".season-selector .current-season");
    assert!(
        wait_for_element(".seasonal-anime", 3000).await,
        "Seasonal anime should load"
    );
    
    // Verify filtering works
    click_element("input[value='CURRENTLY_AIRING']");
    
    // Wait for filtered results
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 500)
            .unwrap();
    }))
    .await
    .unwrap();
    
    let anime_cards = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector_all(".anime-card")
        .unwrap();
    
    assert!(anime_cards.length() > 0, "Should have filtered results");
}

#[wasm_bindgen_test]
async fn test_authentication_persistence() {
    // Test that authentication persists across page refreshes
    log::info!("Testing authentication persistence");
    
    // Login first
    click_element("a[href='/login']");
    wait_for_element(".login-form", 3000).await;
    
    set_input_value("input[name='email']", "test@example.com");
    set_input_value("input[name='password']", "password123");
    click_element("button[type='submit']");
    
    wait_for_element(".user-profile", 5000).await;
    
    // Simulate page refresh by navigating away and back
    click_element("a[href='/browse']");
    wait_for_element(".browse-page", 3000).await;
    
    click_element("a[href='/']");
    wait_for_element(".hero-section", 3000).await;
    
    // User should still be logged in
    assert!(
        get_element(".user-profile").is_some(),
        "User should remain logged in after navigation"
    );
}

#[wasm_bindgen_test]
async fn test_error_handling() {
    // Test error handling for invalid searches
    log::info!("Testing error handling");
    
    // Search with empty query
    set_input_value(".search-input", "");
    click_element(".search-button");
    
    assert!(
        wait_for_element(".search-error", 2000).await,
        "Should show error for empty search"
    );
    
    // Search with special characters only
    set_input_value(".search-input", "!!!###$$$");
    click_element(".search-button");
    
    assert!(
        wait_for_element(".no-results", 3000).await,
        "Should show no results message"
    );
}

#[wasm_bindgen_test]
async fn test_responsive_navigation() {
    // Test mobile navigation menu
    log::info!("Testing responsive navigation");
    
    // Set viewport to mobile size
    let window = web_sys::window().unwrap();
    window.inner_width().unwrap();
    
    // Mobile menu should be hidden initially
    let mobile_menu = get_element(".mobile-menu");
    assert!(mobile_menu.is_none() || !mobile_menu.unwrap().class_list().contains("open"));
    
    // Click hamburger menu
    click_element(".hamburger-menu");
    
    assert!(
        wait_for_element(".mobile-menu.open", 2000).await,
        "Mobile menu should open"
    );
    
    // Click menu item
    click_element(".mobile-menu a[href='/browse']");
    
    // Menu should close after navigation
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 500)
            .unwrap();
    }))
    .await
    .unwrap();
    
    let mobile_menu_after = get_element(".mobile-menu.open");
    assert!(
        mobile_menu_after.is_none(),
        "Mobile menu should close after navigation"
    );
}

#[wasm_bindgen_test]
async fn test_watchlist_functionality() {
    // Test adding and removing from watchlist
    log::info!("Testing watchlist functionality");
    
    // Navigate to an anime detail page
    click_element(".trending-section .anime-card:first-child");
    wait_for_element(".series-detail", 3000).await;
    
    // Add to watchlist
    click_element(".add-to-watchlist");
    
    assert!(
        wait_for_element(".remove-from-watchlist", 2000).await,
        "Button should change to remove from watchlist"
    );
    
    // Navigate to watchlist
    click_element("a[href='/watchlist']");
    
    assert!(
        wait_for_element(".watchlist-page", 3000).await,
        "Watchlist page should load"
    );
    
    // Verify anime is in watchlist
    let watchlist_items = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector_all(".watchlist-item")
        .unwrap();
    
    assert!(watchlist_items.length() > 0, "Watchlist should have items");
    
    // Remove from watchlist
    click_element(".watchlist-item:first-child .remove-button");
    
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 500)
            .unwrap();
    }))
    .await
    .unwrap();
    
    // Verify removal
    let updated_items = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector_all(".watchlist-item")
        .unwrap();
    
    assert!(
        updated_items.length() == 0 || updated_items.length() < watchlist_items.length(),
        "Item should be removed from watchlist"
    );
}

/// Test module for performance measurements
mod performance {
    use super::*;
    
    #[wasm_bindgen_test]
    async fn test_page_load_performance() {
        let start = js_sys::Date::now();
        
        // Wait for initial page load
        wait_for_element(".hero-section", 5000).await;
        
        let load_time = js_sys::Date::now() - start;
        
        assert!(
            load_time < 3000.0,
            "Initial page load should be under 3 seconds"
        );
        
        // Test navigation performance
        let nav_start = js_sys::Date::now();
        click_element("a[href='/browse']");
        wait_for_element(".browse-page", 3000).await;
        
        let nav_time = js_sys::Date::now() - nav_start;
        
        assert!(
            nav_time < 1000.0,
            "Navigation should be under 1 second"
        );
    }
    
    #[wasm_bindgen_test]
    async fn test_search_performance() {
        let start = js_sys::Date::now();
        
        set_input_value(".search-input", "anime");
        click_element(".search-button");
        
        wait_for_element(".search-results", 3000).await;
        
        let search_time = js_sys::Date::now() - start;
        
        assert!(
            search_time < 2000.0,
            "Search should complete under 2 seconds"
        );
    }
}