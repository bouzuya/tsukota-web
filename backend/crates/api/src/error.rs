use application::error::ApplicationError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use serde_json::json;

/// API error wrapper for ApplicationError
pub struct ApiError(pub ApplicationError);

impl From<ApplicationError> for ApiError {
    fn from(err: ApplicationError) -> Self {
        ApiError(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type, detail) = match &self.0 {
            ApplicationError::AccountNotFound(id) => (
                StatusCode::NOT_FOUND,
                "account_not_found",
                format!("The specified account was not found: {}", id),
            ),
            ApplicationError::CategoryNotFound(id) => (
                StatusCode::NOT_FOUND,
                "category_not_found",
                format!("The specified category was not found: {}", id),
            ),
            ApplicationError::Device(_err) => (
                StatusCode::UNAUTHORIZED,
                "device_authentication_failed",
                "Device authentication failed".to_owned(),
            ),
            ApplicationError::TransactionNotFound(id) => (
                StatusCode::NOT_FOUND,
                "transaction_not_found",
                format!("The specified transaction was not found: {}", id),
            ),
            ApplicationError::Unauthorized(msg) => (
                StatusCode::FORBIDDEN,
                "unauthorized",
                format!(
                    "You do not have permission to access this resource: {}",
                    msg
                ),
            ),
            ApplicationError::Domain(err) => (
                StatusCode::BAD_REQUEST,
                "domain_error",
                format!("Domain validation error: {}", err),
            ),
            ApplicationError::Repository(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "repository_error",
                format!("Repository error: {}", msg),
            ),
            ApplicationError::InvalidRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "invalid_request",
                format!("Invalid request: {}", msg),
            ),
            ApplicationError::User(_err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "user_error",
                "An internal user error occurred".to_owned(),
            ),
        };

        let body = json!({
            "type": error_type,
            "title": detail,
            "status": status.as_u16(),
        });

        (status, Json(body)).into_response()
    }
}

/// Error for authentication failures
pub struct AuthError;

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = json!({
            "type": "authentication_required",
            "title": "Authentication required",
            "status": 401,
        });

        (StatusCode::UNAUTHORIZED, Json(body)).into_response()
    }
}
