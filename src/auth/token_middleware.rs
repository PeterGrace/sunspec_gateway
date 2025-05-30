use crate::auth::error_handler::AuthError;
use crate::auth::token_extractor::AuthToken;
use crate::auth::token_management::JwtClaims;
use crate::modules::users::User;
use crate::state::AppState;
use axum::extract::{Request, State};
use axum::{middleware::Next, response::Response};
use cached::Cached;
use tower_sessions::Session;

pub(crate) async fn auth_middleware(
    State(state): State<AppState>,
    auth_token: Result<AuthToken, AuthError>,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // If token extraction failed, return the error
    let session: &Session = request.extensions().get::<Session>().unwrap();

    let token = auth_token?;
    match token {
        AuthToken::Jwt(jwt) => {
            let sub = jwt.claims.sub;

            // scope down to release cache rwlock as soon as possible
            let user: User = {
                let cached_user: Option<User> = match state.user_cache {
                    Some(r) => {
                        let mut cache = r.write().await;
                        cache.cache_get(&sub).cloned()
                    }
                    None => None,
                };

                let user = match cached_user {
                    Some(s) => {
                        info!("user {sub} is in cache.");
                        s
                    }
                    None => {
                        info!("user {sub} is not in cache, fetching from database.");
                        //TODO: implement auth properly for your template project
                        return Err(AuthError::NotImplemented);
                        // match sqlx::query_as!(User, "SELECT * FROM users where login = $1", sub)
                        //     .fetch_one(&state.pool)
                        //     .await
                        // {
                        //     Ok(u) => {
                        //         info!("user {sub} is in database, caching.");
                        //         if let Err(e) = foo.cache_set(sub.clone(), u.clone()).await {
                        //             error!("Error caching user {sub}: {e}");
                        //             return Err(AuthError::InvalidState);
                        //         }
                        //         u
                        //     },
                        //     Err(e) => {
                        //         error!("Error looking up user {sub} in database: {e}");
                        //         return Err(AuthError::InvalidState);
                        //     }
                        // }
                    }
                };
                user
            };
            match session.insert("user", user.clone()).await {
                Ok(_) => {
                    debug!("Set user in session: {user:#?}");
                }
                Err(e) => {
                    error!("Couldn't set user in session: {e}");
                    return Err(AuthError::InvalidToken);
                }
            }
        }
        AuthToken::ApiKey(user_id) => {
            //TODO: implement auth properly for your template project
            return Err(AuthError::NotImplemented);
            // if let Ok(user) = sqlx::query_as!(User, "SELECT * FROM users where id = $1", user_id)
            //     .fetch_one(&state.pool.clone())
            //     .await
            // {
            //     match session.insert("user", user.clone()).await {
            //         Ok(_) => {
            //             debug!("Set user in session: {user:#?}");
            //         }
            //         Err(e) => {
            //             error!("Couldn't set user in session: {e}");
            //             return Err(AuthError::InvalidToken);
            //         }
            //     }
            // }
        }
    }

    // You can perform additional validation here
    // For example, check token expiration, permissions, etc.

    // If everything is ok, continue with the request
    Ok(next.run(request).await)
}
