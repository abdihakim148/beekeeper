#![allow(unused)]
use std::fmt;
use std::error::Error as StdError;
use argon2::password_hash::errors::Error as HashError;
use actix_web::{http::StatusCode, ResponseError};

/// Module for database-related errors.
mod database_error;

pub use database_error::DatabaseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    LockError(String),
    EmailAlreadyExists,
    UserNotFound,
    TableNotFound,
    InvalidUserId,
    DatabaseConsistencyError,
    SerializationError(String),
    InvalidInput(String),
    ConversionError(String),
    InvalidEmail,
    Database(DatabaseError),
    Unauthorized,
    HashingError(HashError),
    Unknown(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotFound => write!(f, "Not Found"),
            Error::InvalidInput(msg) => write!(f, "Invalid Input: {}", msg),
            Error::Database(err) => write!(f, "Database Error: {}", err),
            Error::Unauthorized => write!(f, "Unauthorized"),
            Error::Unknown(msg) => write!(f, "Unknown Error: {}", msg),
            Error::LockError(msg) => write!(f, "Lock Error: {}", msg),
            Error::EmailAlreadyExists => write!(f, "Email Already Exists"),
            Error::UserNotFound => write!(f, "User Not Found"),
            Error::InvalidUserId => write!(f, "Invalid User ID"),
            Error::TableNotFound => write!(f, "Table Not Found"),
            Error::DatabaseConsistencyError => write!(f, "Database Consistency Error"),
            Error::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            Error::ConversionError(msg) => write!(f, "Conversion Error: {}", msg),
            Error::HashingError(err) => write!(f, "hashing error: {}", err),
            Error::InvalidEmail => write!(f, "Invalid Email"),
        } 
    }
}

impl StdError for Error {}

impl From<DatabaseError> for Error {
    fn from(error: DatabaseError) -> Self {
        Error::Database(error)
    }
}


impl From<HashError> for Error {
    fn from(err: HashError) -> Self {
        Self::HashingError(err)
    }
}


impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        use Error::*;
        match self {
            NotFound | UserNotFound => StatusCode::NOT_FOUND,
            InvalidInput(_) | InvalidUserId | InvalidEmail | SerializationError(_) | ConversionError(_) => StatusCode::BAD_REQUEST,
            Unauthorized => StatusCode::UNAUTHORIZED,
            Database(_) | Unknown(_) | LockError(_) | TableNotFound | DatabaseConsistencyError | SerializationError(_) | HashingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            EmailAlreadyExists => StatusCode::CONFLICT,
        }
    }
}