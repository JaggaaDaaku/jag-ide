use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use jag_core::types::{UserId, UserRole};
use chrono::{Utc, Duration};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // UserId
    pub email: String,
    pub roles: Vec<UserRole>,
    pub exp: i64,
}

pub struct AuthService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Permission {
    ViewMissions,
    StartMission,
    ApproveArtifacts,
    ManageUsers,
    ExportAudit,
}

impl AuthService {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn generate_token(&self, user_id: &UserId, email: &str, roles: Vec<UserRole>) -> Result<String, jsonwebtoken::errors::Error> {
        let exp = Utc::now() + Duration::minutes(15);
        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            roles,
            exp: exp.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
    }

    pub fn generate_refresh_token(&self, _user_id: &UserId) -> String {
        use rand::RngCore;
        use base64::{Engine as _, engine::general_purpose};
        
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        general_purpose::STANDARD_NO_PAD.encode(bytes)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }

    pub fn require_admin(claims: &Claims) -> bool {
        claims.roles.contains(&UserRole::Admin)
    }

    pub fn has_permission(claims: &Claims, permission: Permission) -> bool {
        for role in &claims.roles {
            let perms = match role {
                UserRole::Admin => vec![
                    Permission::ViewMissions,
                    Permission::StartMission,
                    Permission::ApproveArtifacts,
                    Permission::ManageUsers,
                    Permission::ExportAudit,
                ],
                UserRole::Developer => vec![
                    Permission::ViewMissions,
                    Permission::StartMission,
                    Permission::ApproveArtifacts,
                ],
                UserRole::Viewer => vec![
                    Permission::ViewMissions,
                ],
            };
            if perms.contains(&permission) {
                return true;
            }
        }
        false
    }
}
