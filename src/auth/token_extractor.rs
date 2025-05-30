use crate::auth::data::ApiKey;
use crate::auth::error_handler::AuthError;
use crate::auth::token_management::JwtClaims;
use crate::auth::token_management::JwtToken;
use crate::consts::*;
use crate::state::AppState;
use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use jsonwebtoken::{
    decode,
    jwk::{AlgorithmParameters, JwkSet},
    DecodingKey, Validation,
};
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::RwLock;

// Structure to cache JWKs
#[derive(Debug, Clone)]
pub(crate) struct JwksCache {
    keys: Arc<RwLock<HashMap<String, JwkSet>>>,
}

impl JwksCache {
    pub(crate) fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_or_fetch_jwks(&self, issuer: &str, jwks_uri: &str) -> Result<JwkSet, AuthError> {
        // Check cache first
        if let Some(jwks) = self.keys.read().await.get(issuer) {
            return Ok(jwks.clone());
        }

        // If not in cache, fetch from jwks_uri
        let jwks = reqwest::get(jwks_uri)
            .await
            .map_err(|_| AuthError::JwksError)?
            .json::<JwkSet>()
            .await
            .map_err(|_| AuthError::JwksError)?;

        // Store in cache
        self.keys
            .write()
            .await
            .insert(issuer.to_string(), jwks.clone());

        Ok(jwks)
    }
}

pub enum AuthToken {
    Jwt(JwtToken),
    ApiKey(i32),
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthToken
where
    S: Send + Sync + 'static,
{
    type Rejection = AuthError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        Box::pin(async move {
            let auth_header = parts
                .headers
                .get(AUTHORIZATION)
                .ok_or(AuthError::MissingToken)?;

            let auth_str = auth_header.to_str().map_err(|_| AuthError::InvalidToken)?;

            //region JWT
            if auth_str.starts_with("Bearer ") {
                let token = auth_str.trim_start_matches("Bearer ").to_string();

                match decode::<JwtClaims>(
                    &token,
                    &DecodingKey::from_secret(JWT_SECRET.as_ref()),
                    &Validation::default(),
                ) {
                    Ok(token_data) => {
                        return Ok(AuthToken::Jwt(JwtToken {
                            claims: token_data.claims,
                            token,
                        }));
                    }
                    Err(e) => {
                        info!("in-house jwt failed: {e}");
                        // If validation with JWT_SECRET fails, try JWKS validation
                        // First decode header without verification to get kid
                        let header = jsonwebtoken::decode_header(&token)
                            .map_err(|_| AuthError::InvalidToken)?;

                        // Get kid from header
                        let kid = header.kid.ok_or(AuthError::InvalidToken)?;

                        let app_state = <dyn Any>::downcast_ref::<AppState>(state)
                            .ok_or(AuthError::InvalidState)?;

                        // For Google's JWKS (you can add more issuers as needed)
                        let google_jwks = app_state
                            .jwks_cache
                            .get_or_fetch_jwks(
                                "https://accounts.google.com",
                                "https://www.googleapis.com/oauth2/v3/certs",
                            )
                            .await?;

                        // Find the key with matching kid
                        let jwk = google_jwks
                            .keys
                            .iter()
                            .find(|k| k.common.key_id.as_deref() == Some(&kid))
                            .ok_or(AuthError::InvalidToken)?;

                        // Convert JWK to DecodingKey
                        let decoding_key = match &jwk.algorithm {
                            AlgorithmParameters::RSA(rsa) => {
                                DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                                    .map_err(|_| AuthError::InvalidToken)?
                            }
                            // Add other algorithms as needed
                            _ => return Err(AuthError::InvalidToken),
                        };

                        // Validate the token with the JWKS key
                        let mut validation = Validation::new(header.alg);
                        validation.set_issuer(&["https://accounts.google.com"]);

                        match decode::<JwtClaims>(&token, &decoding_key, &validation) {
                            Ok(token_data) => {
                                return Ok(AuthToken::Jwt(JwtToken {
                                    claims: token_data.claims,
                                    token,
                                }));
                            }
                            Err(_) => return Err(AuthError::InvalidToken),
                        }
                    }
                }
            }
            //endregion

            //region apikey
            if !auth_str.starts_with("ApiKey ") {
                return Err(AuthError::InvalidToken);
            }

            let token = auth_str.trim_start_matches("ApiKey ").to_string();

            // Here you would typically validate the token
            // For example, check if it exists in your database or validate a JWT
            if token.is_empty() {
                return Err(AuthError::InvalidToken);
            }
            let state = <dyn Any>::downcast_ref::<AppState>(state).expect("Invalid state type");
            //TODO: implement auth properly for your template project
            Ok(AuthToken::ApiKey(1))
            // let rs = sqlx::query_as!(ApiKey, "SELECT * FROM api_keys where key = $1", token)
            //     .fetch_optional(&state.pool)
            //     .await;
            // match rs {
            //     Ok(Some(record)) => Ok(AuthToken::ApiKey(record.user_id)),
            //     Ok(None) => Err(AuthError::InvalidToken),
            //     Err(e) => {
            //         error!("invalid auth token on request: {e}");
            //         Err(AuthError::InvalidTokenFormat)
            //     },
            // }
            //endregion
        })
    }
}
