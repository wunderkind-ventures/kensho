use dioxus::prelude::*;
use dioxus_router::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use crate::services::api::ApiClient;
use crate::models::LoginResponse;

#[derive(Clone, Debug, Default)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub token: Option<String>,
    pub user_id: Option<String>,
}

impl AuthState {
    const STORAGE_KEY: &'static str = "kensho_auth";

    pub fn new() -> Self {
        // Try to load from localStorage
        Self::load_from_storage().unwrap_or_default()
    }

    pub fn load_from_storage() -> Result<Self, gloo_storage::errors::StorageError> {
        LocalStorage::get::<AuthState>(Self::STORAGE_KEY)
    }

    pub fn save_to_storage(&self) -> Result<(), gloo_storage::errors::StorageError> {
        LocalStorage::set(Self::STORAGE_KEY, self)
    }

    pub fn clear_storage() -> Result<(), gloo_storage::errors::StorageError> {
        LocalStorage::delete(Self::STORAGE_KEY);
        Ok(())
    }

    pub fn login(&mut self, response: LoginResponse) {
        self.is_authenticated = true;
        self.token = Some(response.token);
        self.user_id = Some(response.user_id);
        let _ = self.save_to_storage();
    }

    pub fn logout(&mut self) {
        self.is_authenticated = false;
        self.token = None;
        self.user_id = None;
        let _ = Self::clear_storage();
    }

    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }
}

// Implement serde for localStorage
impl serde::Serialize for AuthState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AuthState", 3)?;
        state.serialize_field("is_authenticated", &self.is_authenticated)?;
        state.serialize_field("token", &self.token)?;
        state.serialize_field("user_id", &self.user_id)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for AuthState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct AuthStateData {
            is_authenticated: bool,
            token: Option<String>,
            user_id: Option<String>,
        }

        let data = AuthStateData::deserialize(deserializer)?;
        Ok(AuthState {
            is_authenticated: data.is_authenticated,
            token: data.token,
            user_id: data.user_id,
        })
    }
}

// Hook for using auth state in components
pub fn use_auth(cx: &ScopeState) -> &UseSharedState<AuthState> {
    use_shared_state::<AuthState>(cx).expect("Auth state not provided")
}

// Auth guard component
#[inline_props]
pub fn AuthGuard<'a>(cx: Scope<'a>, children: Element<'a>) -> Element {
    let auth = use_auth(cx);
    let router = use_navigator(cx);

    if !auth.read().is_authenticated {
        router.push("/login");
        return None;
    }

    children.clone()
}

// Login service hook
pub fn use_login_service(cx: &ScopeState) -> LoginService {
    let auth = use_auth(cx);
    let api = ApiClient::new();
    let router = use_navigator(cx);

    LoginService {
        auth: auth.to_owned(),
        api,
        router: router.clone(),
    }
}

pub struct LoginService {
    auth: UseSharedState<AuthState>,
    api: ApiClient,
    router: Navigator,
}

impl LoginService {
    pub async fn login(&self, username: String, password: String) -> Result<(), String> {
        match self.api.login(username, password).await {
            Ok(response) => {
                self.auth.write().login(response);
                self.router.push("/");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub async fn logout(&self) -> Result<(), String> {
        if let Some(token) = self.auth.read().get_token() {
            let _ = self.api.logout(token).await;
        }
        self.auth.write().logout();
        self.router.push("/login");
        Ok(())
    }
}