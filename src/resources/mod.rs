use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

pub mod feature_flags_api;
pub mod environments_api;

#[derive(Error, Debug)]
enum CustomError {
    #[error("Requested file was not found")]
    NotFound,
    #[error("You are forbidden to access requested file.")]
    Forbidden,
    #[error("Unknown Internal Error")]
    Unknown,
    #[error("Application error")]
    ApplicationError,
    #[error("Conflict")]
    Conflict,
    #[error("No Content")]
    NoContent,
    #[error("Creation error")]
    CreationError(String)
}

impl CustomError {
    pub fn name(&self) -> String {
        match self {
            Self::NotFound => "NotFound".to_string(),
            Self::Forbidden => "Forbidden".to_string(),
            Self::Unknown => "Unknown".to_string(),
            Self::ApplicationError => "ApplicationError".to_string(),
            Self::Conflict => "Conflict".to_string(),
            Self::NoContent => "NoContent".to_string(),
            Self::CreationError(err) => format!("Creation error: {}", err.to_string())
        }
    }
}

impl ResponseError for CustomError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ApplicationError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Conflict => StatusCode::CONFLICT,
            Self::NoContent => StatusCode::NO_CONTENT,
            Self::CreationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
            error: self.name(),
        };
        HttpResponse::build(status_code).json(error_response)
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}
