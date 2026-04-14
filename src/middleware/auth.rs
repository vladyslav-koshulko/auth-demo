use crate::oauth::client::ensure_valid_token;
use crate::session::file::{get_current_session_id, get_current_user, Session};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use std::pin::Pin;
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub session_id: String,
    pub session: Session,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);
    fn from_request_parts<'life0, 'life1, 'async_trait>(
        _parts: &'life0 mut Parts,
        _state: &'life1 S,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let session_id =
                get_current_session_id().ok_or((StatusCode::UNAUTHORIZED, "Not authenticated"))?;

            let mut session =
                get_current_user().ok_or((StatusCode::UNAUTHORIZED, "Session not found"))?;

            ensure_valid_token(&session_id, &mut session)
                .await
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token"))?;

            Ok(AuthUser {
                session_id,
                session,
            })
        })
    }
}

pub async fn auth_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let session_id = get_current_session_id().ok_or(StatusCode::UNAUTHORIZED)?;

    let mut session = get_current_user().ok_or(StatusCode::UNAUTHORIZED)?;

    if let Err(_e) = ensure_valid_token(&session_id, &mut session).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let response = next.run(request).await;
    Ok(response)
}
