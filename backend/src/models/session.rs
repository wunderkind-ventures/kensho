// T026: Session model with JWT handling
// Reference: data-model.md lines 116-143 for Session struct and JWT implementation

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    
    pub user_id: String, // Crunchyroll user ID
    
    pub jwt_token: String,
    
    pub cr_token_key: String, // Redis key for Crunchyroll token
    
    pub expires_at: DateTime<Utc>,
    
    pub refresh_token: Option<String>,
    
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    
    #[serde(default = "Utc::now")]
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // user_id
    pub session_id: Uuid,
    pub exp: i64,           // Expiry timestamp
    pub iat: i64,           // Issued at
    pub cr_token_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionCreate {
    pub user_id: String,
    pub crunchyroll_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub refresh_token: Option<String>,
}

impl Session {
    pub fn new(user_id: String, cr_token_key: String, jwt_secret: &str) -> Result<Self> {
        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::minutes(15);
        
        let claims = Claims {
            sub: user_id.clone(),
            session_id,
            exp: expires_at.timestamp(),
            iat: Utc::now().timestamp(),
            cr_token_key: cr_token_key.clone(),
        };
        
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_ref()),
        )?;
        
        Ok(Session {
            id: session_id,
            user_id,
            jwt_token: token,
            cr_token_key,
            expires_at,
            refresh_token: Some(Uuid::new_v4().to_string()),
            created_at: Utc::now(),
            last_activity: Utc::now(),
        })
    }
    
    pub fn verify_token(token: &str, jwt_secret: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &Validation::default(),
        )?;
        
        Ok(token_data.claims)
    }
    
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    pub fn refresh(&mut self, jwt_secret: &str) -> Result<String> {
        self.expires_at = Utc::now() + Duration::minutes(15);
        self.last_activity = Utc::now();
        
        let claims = Claims {
            sub: self.user_id.clone(),
            session_id: self.id,
            exp: self.expires_at.timestamp(),
            iat: Utc::now().timestamp(),
            cr_token_key: self.cr_token_key.clone(),
        };
        
        let new_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_ref()),
        )?;
        
        self.jwt_token = new_token.clone();
        Ok(new_token)
    }
    
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }
    
    pub fn to_response(&self) -> SessionResponse {
        SessionResponse {
            token: self.jwt_token.clone(),
            expires_at: self.expires_at,
            refresh_token: self.refresh_token.clone(),
        }
    }
}

// Redis key helpers
impl Session {
    pub fn redis_key(&self) -> String {
        format!("session:{}", self.id)
    }
    
    pub fn redis_user_key(user_id: &str) -> String {
        format!("user_session:{}", user_id)
    }
    
    pub fn redis_cr_token_key(&self) -> String {
        self.cr_token_key.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const TEST_SECRET: &str = "test_secret_key_for_jwt_signing";
    
    #[test]
    fn test_session_creation() {
        let session = Session::new(
            "user123".to_string(),
            "cr_token:user123".to_string(),
            TEST_SECRET,
        ).unwrap();
        
        assert_eq!(session.user_id, "user123");
        assert!(!session.is_expired());
        assert!(session.refresh_token.is_some());
    }
    
    #[test]
    fn test_jwt_verification() {
        let session = Session::new(
            "user456".to_string(),
            "cr_token:user456".to_string(),
            TEST_SECRET,
        ).unwrap();
        
        let claims = Session::verify_token(&session.jwt_token, TEST_SECRET).unwrap();
        assert_eq!(claims.sub, "user456");
        assert_eq!(claims.session_id, session.id);
    }
    
    #[test]
    fn test_session_refresh() {
        let mut session = Session::new(
            "user789".to_string(),
            "cr_token:user789".to_string(),
            TEST_SECRET,
        ).unwrap();
        
        let original_token = session.jwt_token.clone();
        let original_expiry = session.expires_at;
        
        // Simulate time passing
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        let new_token = session.refresh(TEST_SECRET).unwrap();
        
        assert_ne!(original_token, new_token);
        assert!(session.expires_at > original_expiry);
    }
    
    #[test]
    fn test_redis_keys() {
        let session = Session::new(
            "user999".to_string(),
            "cr_token:user999".to_string(),
            TEST_SECRET,
        ).unwrap();
        
        assert!(session.redis_key().starts_with("session:"));
        assert_eq!(Session::redis_user_key("user999"), "user_session:user999");
        assert_eq!(session.redis_cr_token_key(), "cr_token:user999");
    }
}