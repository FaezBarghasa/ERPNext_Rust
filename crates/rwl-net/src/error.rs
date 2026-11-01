//! Typed API Error Model (`rwl-net::error::ApiError`).
//!
//! Provides RFC 7807 compliant problem details with sanitized error responses.
//! Internal database and runtime details are masked behind unique trace IDs.

use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use uuid::Uuid;

/// Typed API Error with automatic RFC 7807 JSON representation.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ProblemDetails {
    #[serde(rename = "type")]
    problem_type: &'static str,
    title: &'static str,
    status: u16,
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    incident_id: Option<String>,
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let status = self.status_code();
        let (detail, incident_id) = match self {
            Self::NotFound(msg)
            | Self::Forbidden(msg)
            | Self::Unauthorized(msg)
            | Self::BadRequest(msg)
            | Self::Conflict(msg) => (msg.clone(), None),
            Self::Database(internal) => {
                let id = Uuid::new_v4().to_string();
                tracing::error!(incident_id = %id, error = %internal, "Internal database error occurred");
                (
                    format!("An internal database error occurred. Reference ID: {id}"),
                    Some(id),
                )
            }
            Self::Internal(internal) => {
                let id = Uuid::new_v4().to_string();
                tracing::error!(incident_id = %id, error = %internal, "Internal server error occurred");
                (
                    format!("An unexpected error occurred. Reference ID: {id}"),
                    Some(id),
                )
            }
        };

        let title = match status {
            StatusCode::NOT_FOUND => "Not Found",
            StatusCode::FORBIDDEN => "Forbidden",
            StatusCode::UNAUTHORIZED => "Unauthorized",
            StatusCode::BAD_REQUEST => "Bad Request",
            StatusCode::CONFLICT => "Conflict",
            _ => "Internal Server Error",
        };

        let body = ProblemDetails {
            problem_type: "https://datatracker.ietf.org/doc/html/rfc7807",
            title,
            status: status.as_u16(),
            detail,
            incident_id,
        };

        HttpResponse::build(status).json(body)
    }
}

impl From<surrealdb::Error> for ApiError {
    fn from(err: surrealdb::Error) -> Self {
        Self::Database(err.to_string())
    }
}
