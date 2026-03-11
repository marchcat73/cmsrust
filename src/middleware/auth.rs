use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::{StatusCode, header::AUTHORIZATION},
};
use crate::utils::jwt::verify_token;

pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, "Missing token".to_string()))?;

    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Config error".to_string()))?;

    let claims = verify_token(token, &secret)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    // Сохраняем claims в extensions для использования в хендлерах
    let mut request = request;
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

// Middleware для проверки ролей
pub async fn require_role(
    request: Request,
    next: Next,
    required_role: &str,
) -> Result<Response, (StatusCode, String)> {
    let claims = request
        .extensions()
        .get::<crate::utils::jwt::Claims>()
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

    if claims.role != required_role && claims.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "Insufficient permissions".to_string()));
    }

    Ok(next.run(request).await)
}
