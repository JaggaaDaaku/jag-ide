use actix_web::{get, post, web, HttpResponse, Responder};
use oauth2::AuthorizationCode;
use openidconnect::core::{CoreClient, CoreProviderMetadata, CoreResponseType};
use openidconnect::{
    AuthenticationFlow, ClientId, ClientSecret, CsrfToken, IssuerUrl,
    Nonce, RedirectUrl, Scope, TokenResponse,
};
use crate::AppState;
use jag_core::types::UserRole;
use serde::Deserialize;
use tracing::{error, info};
use chrono::{Utc, Duration};

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[get("/login")]
pub async fn login(state: web::Data<AppState>) -> impl Responder {
    let config = &state.config.auth;
    
    if !config.oidc_enabled {
        return HttpResponse::BadRequest().body("OIDC is disabled");
    }

    let provider_metadata = match CoreProviderMetadata::discover_async(
        IssuerUrl::new(config.oidc_issuer_url.clone()).unwrap(),
        &openidconnect::reqwest::async_http_client,
    ).await {
        Ok(m) => m,
        Err(e) => {
            error!("OIDC Discovery failed: {}", e);
            return HttpResponse::InternalServerError().body("Discovery failed");
        }
    };

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(config.oidc_client_id.clone()),
        Some(ClientSecret::new(config.oidc_client_secret.clone())),
    )
    .set_redirect_uri(RedirectUrl::new(config.oidc_redirect_url.clone()).unwrap());

    let (auth_url, _csrf_token, _nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    HttpResponse::Found()
        .append_header(("Location", auth_url.as_str().to_string()))
        .finish()
}

#[get("/callback")]
pub async fn callback(
    state: web::Data<AppState>,
    params: web::Query<CallbackParams>,
) -> impl Responder {
    let config = &state.config.auth;

    if !config.oidc_enabled {
        // Development Mode Shortcut
        let email = "admin@jag-ide.com";
        return complete_login(&state, email, "dev_mode").await;
    }

    let provider_metadata = match CoreProviderMetadata::discover_async(
        IssuerUrl::new(config.oidc_issuer_url.clone()).unwrap(),
        &openidconnect::reqwest::async_http_client,
    ).await {
        Ok(m) => m,
        Err(e) => {
            error!("OIDC Discovery failed: {}", e);
            return HttpResponse::InternalServerError().body("Discovery failed");
        }
    };

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(config.oidc_client_id.clone()),
        Some(ClientSecret::new(config.oidc_client_secret.clone())),
    )
    .set_redirect_uri(RedirectUrl::new(config.oidc_redirect_url.clone()).unwrap());

    let token_response: openidconnect::core::CoreTokenResponse = match client
        .exchange_code(AuthorizationCode::new(params.code.clone()))
        .request_async(&openidconnect::reqwest::async_http_client)
        .await {
            Ok(t) => t,
            Err(e) => {
                error!("Token exchange failed: {}", e);
                return HttpResponse::InternalServerError().body("Token exchange failed");
            }
        };

    let id_token: &openidconnect::core::CoreIdToken = match token_response.id_token() {
        Some(id) => id,
        None => return HttpResponse::InternalServerError().body("No ID token returned"),
    };

    let claims: &openidconnect::core::CoreIdTokenClaims = match id_token.claims(&client.id_token_verifier(), &Nonce::new("dummy".to_string())) {
        Ok(c) => c,
        Err(e) => {
            error!("ID token verification failed: {}", e);
            return HttpResponse::Unauthorized().body("Invalid ID token");
        }
    };

    let email = match claims.email() {
        Some(e) => e.to_string(),
        None => return HttpResponse::BadRequest().body("No email claim in ID token"),
    };

    complete_login(&state, &email, "oidc").await
}

async fn complete_login(state: &AppState, email: &str, provider: &str) -> HttpResponse {
    let user = match state.db.get_user_by_email(email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            // Auto-provisioning with default Viewer role
            match state.db.create_user(email, &UserRole::Viewer).await {
                Ok(u) => {
                    info!("Auto-provisioned new user: {}", email);
                    u
                },
                Err(e) => return HttpResponse::InternalServerError().body(format!("{}", e)),
            }
        }
        Err(e) => return HttpResponse::InternalServerError().body(format!("{}", e)),
    };

    let roles = vec![user.role.clone()];
    let access_token = state.auth_service.generate_token(&user.id, &user.email, roles.clone()).unwrap();
    let raw_refresh_token = state.auth_service.generate_refresh_token(&user.id);
    
    // Store session in DB (using hashed token)
    let token_hash = sha256_hash(&raw_refresh_token);
    let expires_at = Utc::now() + Duration::days(7);
    
    if let Err(e) = state.db.create_session(&user.id, &token_hash, expires_at, None).await {
        error!("Failed to create user session: {}", e);
        return HttpResponse::InternalServerError().finish();
    }

    state.audit_logger.log_signed(
        None,
        Some(user.id.clone()),
        None,
        "user_login",
        "auth",
        &format!("User {} logged in via {}", email, provider),
        jag_sandbox::audit::AuditResult::Success,
    ).await;

    HttpResponse::Ok().json(serde_json::json!({
        "access_token": access_token,
        "refresh_token": raw_refresh_token,
        "user": user,
    }))
}

#[post("/refresh")]
pub async fn refresh(
    state: web::Data<AppState>,
    req: web::Json<RefreshRequest>,
) -> impl Responder {
    let token_hash = sha256_hash(&req.refresh_token);
    
    match state.db.validate_session(&token_hash).await {
        Ok(Some(session)) => {
            let user = match state.db.get_user_by_id(&session.user_id).await {
                Ok(Some(u)) => u,
                _ => return HttpResponse::Unauthorized().finish(),
            };
            
            let access_token = state.auth_service.generate_token(&user.id, &user.email, vec![user.role]).unwrap();
            
            state.audit_logger.log_signed(
                None,
                Some(user.id.clone()),
                None,
                "token_refresh",
                "auth",
                &format!("User {} refreshed access token", user.email),
                jag_sandbox::audit::AuditResult::Success,
            ).await;

            HttpResponse::Ok().json(serde_json::json!({
                "access_token": access_token,
                "status": "success"
            }))
        }
        _ => HttpResponse::Unauthorized().finish(),
    }
}

#[post("/logout")]
pub async fn logout(
    state: web::Data<AppState>,
    req: web::Json<LogoutRequest>,
) -> impl Responder {
    let token_hash = sha256_hash(&req.refresh_token);
    
    // Attempt to find user for auditing before revoking
    if let Ok(Some(session)) = state.db.validate_session(&token_hash).await {
        state.audit_logger.log_signed(
            None,
            Some(session.user_id),
            None,
            "user_logout",
            "auth",
            "User logged out successfully",
            jag_sandbox::audit::AuditResult::Success,
        ).await;
    }

    let _ = state.db.revoke_session(&token_hash).await;
    
    HttpResponse::Ok().json(serde_json::json!({ "status": "logged_out" }))
}

fn sha256_hash(input: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}
