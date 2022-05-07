use std::env;

use actix_web::HttpRequest;
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use service_core::auth::jwt::Claims;
use thiserror::Error;

pub struct Authorization {
    pub claims: Claims,
}

#[derive(Debug, Error)]
pub enum ExtractAuthorizationError {
    #[error("Service has invalid secret.")]
    InvalidSecret,

    #[error("Invalid token.")]
    InvalidToken,
}

impl Authorization {
    pub fn try_from_req(req: &HttpRequest) -> Result<Option<Self>, ExtractAuthorizationError> {
        if let Some(token) = req.headers().get("Authorization") {
            let token = token.to_str().unwrap_or_default();
            if !token.starts_with("Bearer ") {
                return Err(ExtractAuthorizationError::InvalidToken);
            }

            let secret = jwt_secret().ok_or(ExtractAuthorizationError::InvalidSecret)?;
            let secret = DecodingKey::from_base64_secret(secret.as_str())
                .map_err(|_| ExtractAuthorizationError::InvalidSecret)?;
            let token = token.trim_start_matches("Bearer ");
            println!("token = {token}");
            let token_data: TokenData<Claims> =
                jsonwebtoken::decode(token, &secret, &Validation::new(Algorithm::HS512)).map_err(|e| {
                    log::error!("Failed decoding token: {:?}", e);
                    ExtractAuthorizationError::InvalidToken
                })?;
            return Ok(Some(Self {
                claims: token_data.claims,
            }));
        }

        Ok(None)
    }
}

fn jwt_secret() -> Option<String> {
    env::var("ACCESS_TOKEN_SECRET").map_or(None, Some)
}
