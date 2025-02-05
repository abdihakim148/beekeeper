#[cfg(feature = "http")]
use actix_web::{http::StatusCode, HttpResponse, body::BoxBody, error::ResponseError};
use lettre::address::AddressError as EmailAddressError;
use argon2::password_hash::errors::Error as HashError;
use std::fmt::{Display, Formatter, Result, Debug};
use lettre::transport::smtp::Error as SmtpError;
use lettre::error::Error as LettreError;
use rusty_paseto::core::PasetoError;
use std::error::Error as StdError;
use std::marker::PhantomData;
use std::sync::PoisonError;
use serde::Serialize;
use std::any::TypeId;
use Error::*;

#[cfg(feature = "http")]
pub trait ErrorTrait: ResponseError + StdError + Display + Debug {}
#[cfg(feature = "http")]
impl<T: ResponseError + StdError + Serialize> ErrorTrait for T {}


#[derive(Debug)]
pub enum Error<E = (), F = ()> {
    NotFound(&'static str),
    Conflict(&'static str),
    Internal(Box<dyn StdError>),
    InvalidEmailAddress,
    InvalidToken,
    ExpiredToken,
    /// expected type, found type, field name, http status code, custom message
    ConversionError(TypeId, TypeId, Option<&'static str>, u16, Option<&'static str>),
    #[cfg(feature = "http")]
    Custom(Box<dyn ErrorTrait>),
    Empty(PhantomData<(E, F)>)
}


impl<E: 'static, F: 'static> Error<E, F> {
    fn msg(&self) -> String {
        match self {
            NotFound(name) => format!("{} not found", name),
            Conflict(name) => format!("{} already exists", name),
            Internal(_) => String::from("internal server error"),
            InvalidEmailAddress => String::from("invalid email address"),
            InvalidToken => String::from("invalid token"),
            ExpiredToken => String::from("expired token"),
            ConversionError(_, _, field, status, message) => {
                if *status >= 500u16 {
                    return String::from("internal server Error")
                }
                if let Some(message) = message {
                    if let Some(field) = field {
                        return format!("{message} for field `{field}`")
                    }
                    return String::from(*message)
                }
                if let Some(field) = field {
                    return  format!("invalid data format for field {field}");
                }
                String::from("invalid data format")
            },
            _ => String::new()
        }
    }

    fn set_field(self, field: &'static str) -> Self {
        let field = Some(field);
        match self {
            ConversionError(expected, found, _, status, message) => ConversionError(expected, found, field, status, message),
            _ => self
        }
    }

    fn set_status(self, status: u16) -> Self {
        match self {
            ConversionError(expected, found, field, _, message) => ConversionError(expected, found, field, status, message),
            _ => self
        }
    }


    pub fn conversion_error(message: Option<&'static str>) -> Self {
        ConversionError(TypeId::of::<E>(), TypeId::of::<E>(), None, 400, message)
    }
}


impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            NotFound(name) => write!(f, "{} not found", name),
            Conflict(name) => write!(f, "{} already exists", name),
            Internal(err) => write!(f, "internal: {}", err),
            InvalidEmailAddress => write!(f, "invalid email address"),
            InvalidToken => write!(f, "invalid token"),
            ExpiredToken => write!(f, "expired token"),
            ConversionError(expected, found, field, _status, _) => {
                match field {
                    Some(field) => write!(f, "expected {:?} instead got {:?} for field `{}`", expected, found, field),
                    None => write!(f, "expected {:?} instead got {:?}", expected, found),
                }
            },
            Custom(err) => Display::fmt(err, f),
            Empty(_) => write!(f, "EMPTY")
        }
    }
}


impl StdError for Error {}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        #[derive(Serialize)]
        struct Serializer {
            msg: String
        }

        let ser = Serializer{msg: self.msg()};
        ser.serialize(serializer)
    }
}

#[cfg(feature = "http")]
impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            NotFound(_) => StatusCode::NOT_FOUND,
            Conflict(_) => StatusCode::CONFLICT,
            Internal(_) | Empty(_) => StatusCode::INTERNAL_SERVER_ERROR,
            InvalidEmailAddress => StatusCode::BAD_REQUEST,
            InvalidToken => StatusCode::UNAUTHORIZED,
            ExpiredToken => StatusCode::UNAUTHORIZED,
            ConversionError(_, _, _, status, _) => StatusCode::from_u16(*status).unwrap_or_default(),
            Custom(err) => err.status_code()
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        if let Custom(err) = self {
            return  err.error_response();
        }
        let status = self.status_code();
        let json = serde_json::to_string(&self).unwrap_or(String::from("INTERNAL SERVER ERROR"));
        let body = BoxBody::new(json);
        HttpResponse::build(status).content_type("application/json").body(body)
    }
}


impl From<HashError> for Error {
    fn from(err: HashError) -> Self {
        Self::Internal(err.into())
    }
}


impl From<EmailAddressError> for Error {
    fn from(_: EmailAddressError) -> Self {
        InvalidEmailAddress
    }
}


impl From<LettreError> for Error {
    fn from(err: LettreError) -> Self {
        match err {
            LettreError::Io(err) => Internal(Box::new(err)),
            _ => InvalidEmailAddress
        }
    }
}


impl From<PasetoError> for Error {
    fn from(err: PasetoError) -> Self {
        match err {
            PasetoError::InvalidSignature => InvalidToken,
            _ => Error::Internal(Box::new(err)),
        }
    }
}


impl From<SmtpError> for Error {
    fn from(err: SmtpError) -> Self {
        Self::Internal(Box::new(err))
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Internal("lock poison error".into())
    }
}