#[cfg(debug_assertions)]
use std::sync::Arc;

use serde::ser::SerializeStruct as _;
use warp::http::StatusCode;

pub async fn handle_rejection(r: warp::Rejection) -> Result<impl warp::Reply, !> {
    Ok(Error::from(r))
}

#[derive(Debug, Clone)]
pub enum Error {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    LengthRequired,
    ContentTooLarge,
    InternalServerError,

    Unknown,
    Custom(StatusCode, String),

    // This variant only exists in debug, so `Arc` is fine right,,, surely
    #[cfg(debug_assertions)]
    DebugInternal(Arc<dyn core::error::Error + Sync + Send>),
}

impl warp::reject::Reject for Error {}
impl core::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadRequest => write!(f, "bad request"),
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::Forbidden => write!(f, "forbidden"),
            Error::NotFound => write!(f, "not found"),
            Error::MethodNotAllowed => write!(f, "method not allowed"),
            Error::LengthRequired => write!(f, "length required"),
            Error::ContentTooLarge => write!(f, "content too large"),
            Error::InternalServerError => write!(f, "internal server error"),
            Error::Unknown => write!(f, "unknown error"),
            Error::Custom(_, msg) => write!(f, "{}", msg),

            #[cfg(debug_assertions)]
            Error::DebugInternal(msg) => write!(f, "{}", msg),
        }
    }
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut ser = serializer.serialize_struct("Error", 2)?;
        ser.serialize_field("message", &self.to_string())?;
        ser.serialize_field("status", &StatusCode::from(self).as_u16())?;

        ser.end()
    }
}

impl warp::Reply for Error {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::with_status(warp::reply::json(&self), self.into()).into_response()
    }
}

impl From<Error> for StatusCode {
    fn from(value: Error) -> StatusCode {
        match value {
            Error::BadRequest => StatusCode::BAD_REQUEST,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::Forbidden => StatusCode::FORBIDDEN,
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Error::LengthRequired => StatusCode::LENGTH_REQUIRED,
            Error::ContentTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Error::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Custom(code, _) => code,

            #[cfg(debug_assertions)]
            Error::DebugInternal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<&Error> for StatusCode {
    fn from(value: &Error) -> StatusCode {
        value.clone().into()
    }
}

impl From<uuid::Error> for Error {
    fn from(_: uuid::Error) -> Error {
        Error::Custom(StatusCode::BAD_REQUEST, "parsing uuid failed".into())
    }
}

impl From<warp::Rejection> for Error {
    fn from(r: warp::Rejection) -> Error {
        use warp::reject::{
            InvalidHeader, InvalidQuery, LengthRequired, MethodNotAllowed, MissingCookie,
            MissingHeader, PayloadTooLarge, UnsupportedMediaType,
        };

        if r.is_not_found() {
            return Error::NotFound;
        }

        if let Some(e) = r.find::<Error>() {
            return e.clone();
        }

        // Below are all the known `Rejection` reasons
        // (that are public)

        if r.find::<MethodNotAllowed>().is_some() {
            return Error::MethodNotAllowed;
        }

        if let Some(e) = r.find::<InvalidHeader>() {
            return Error::Custom(
                StatusCode::BAD_REQUEST,
                format!(r#"header "{}" invalid"#, e.name()),
            );
        }

        if let Some(e) = r.find::<MissingHeader>() {
            return Error::Custom(
                StatusCode::BAD_REQUEST,
                format!(r#"header "{}" missing"#, e.name()),
            );
        }

        if let Some(e) = r.find::<MissingCookie>() {
            return Error::Custom(
                StatusCode::BAD_REQUEST,
                format!(r#"cookie "{}" missing"#, e.name()),
            );
        }

        if r.find::<InvalidQuery>().is_some() {
            return Error::BadRequest;
        }

        if r.find::<LengthRequired>().is_some() {
            return Error::LengthRequired;
        }

        if r.find::<PayloadTooLarge>().is_some() {
            return Error::ContentTooLarge;
        }

        if r.find::<UnsupportedMediaType>().is_some() {
            return Error::BadRequest;
        }

        Error::Unknown
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Error {
        #[cfg(not(debug_assertions))]
        {
            // Any and all database errors are internal server errors,
            // but this means there is much loss of information,,,
            Self::InternalServerError
        }

        #[cfg(debug_assertions)]
        {
            Self::DebugInternal(Arc::new(value))
        }
    }
}

impl From<heed::Error> for Error {
    fn from(value: heed::Error) -> Error {
        #[cfg(not(debug_assertions))]
        {
            // Any and all database errors are internal server errors,
            // but this means there is much loss of information,,,
            Self::InternalServerError
        }

        #[cfg(debug_assertions)]
        {
            Self::DebugInternal(Arc::new(value))
        }
    }
}
