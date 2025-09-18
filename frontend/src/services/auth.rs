use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthState {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub user_email: Option<String>,
}

impl Default for AuthState {
    fn default() -> Self {
        // Try to load from localStorage
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            if let Ok(Some(token)) = storage.get_item("auth_token") {
                if let Ok(Some(refresh)) = storage.get_item("refresh_token") {
                    if let Ok(Some(email)) = storage.get_item("user_email") {
                        return Self {
                            access_token: Some(token),
                            refresh_token: Some(refresh),
                            user_email: Some(email),
                        };
                    }
                }
            }
        }
        
        Self {
            access_token: None,
            refresh_token: None,
            user_email: None,
        }
    }
}

impl AuthState {
    pub fn is_authenticated(&self) -> bool {
        self.access_token.is_some()
    }
    
    pub fn login(&mut self, access_token: String, refresh_token: String) {
        self.access_token = Some(access_token.clone());
        self.refresh_token = Some(refresh_token.clone());
        
        // Save to localStorage
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.set_item("auth_token", &access_token);
            let _ = storage.set_item("refresh_token", &refresh_token);
        }
    }
    
    pub fn logout(&mut self) {
        self.access_token = None;
        self.refresh_token = None;
        self.user_email = None;
        
        // Clear localStorage
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.remove_item("auth_token");
            let _ = storage.remove_item("refresh_token");
            let _ = storage.remove_item("user_email");
        }
    }
}