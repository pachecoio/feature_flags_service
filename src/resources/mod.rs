use std::fmt::{Display, Formatter};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;
use serde::{Serialize};

pub mod feature_flags_api;

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
    Conflict
}

impl CustomError {
    pub fn name(&self) -> String {
        match self {
            Self::NotFound => "NotFound".to_string(),
            Self::Forbidden => "Forbidden".to_string(),
            Self::Unknown => "Unknown".to_string(),
            Self::ApplicationError => "ApplicationError".to_string(),
            Self::Conflict => "Conflict".to_string(),
        }
    }
}

impl ResponseError for CustomError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NotFound  => StatusCode::NOT_FOUND,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ApplicationError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Conflict => StatusCode::CONFLICT
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

fn map_io_error(e: std::io::Error) -> CustomError {
    match e.kind() {
        std::io::ErrorKind::NotFound => CustomError::NotFound,
        std::io::ErrorKind::PermissionDenied => CustomError::Forbidden,
        _ => CustomError::Unknown,
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}