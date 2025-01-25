
use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Form,
};

use crate::models::{ Backend, Credentials};

type AuthSession = axum_login::AuthSession<Backend>;

pub async fn login(
    mut auth_session: AuthSession,
    Form(creds): Form<Credentials>,
) -> impl IntoResponse {
    let user = match auth_session.authenticate(creds.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    return StatusCode::OK.into_response();
    //Redirect::to("/users").into_response()
}
