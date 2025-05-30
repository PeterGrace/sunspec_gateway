// auth_token.rs
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum AuthToken {
    Jwt(JwtToken),
    ApiKey(i32),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String, // subject (user id)
    pub exp: usize,  // expiration time
    pub iat: usize,  // issued at
}

#[derive(Debug)]
pub struct JwtToken {
    pub claims: JwtClaims,
    pub token: String,
}
