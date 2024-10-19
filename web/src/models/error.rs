use actix_http::header::ToStrError;
use actix_web::{
    body::BoxBody, error::PayloadError, http::StatusCode, HttpResponse,
    ResponseError,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    num::{ParseFloatError, ParseIntError},
    string::FromUtf8Error,
    sync::PoisonError,
};
use tokio::io;
use utoipa::ToSchema;

#[derive(Clone, Serialize, Debug, ToSchema, Deserialize)]
pub struct AppErr {
    status: u16,
    subject: String,
    content: Option<String>,
}

impl AppErr {
    pub fn new(status: u16, subject: &str) -> Self {
        Self { status, subject: subject.to_string(), content: None }
    }

    pub fn default() -> Self {
        Self { status: 500, subject: "server error".to_string(), content: None }
    }
}

impl fmt::Display for AppErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for AppErr {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.status)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).json(self)
    }
}

impl From<sqlx::Error> for AppErr {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self {
                status: 404,
                subject: "not found".to_string(),
                content: None,
            },
            sqlx::Error::Database(e) => match e.code() {
                Some(c) if c == "2067" => Self {
                    status: 400,
                    subject: "a similar item was found".to_string(),
                    content: None,
                },
                _ => Self::default(),
            },
            _ => Self::default(),
        }
    }
}

impl From<&str> for AppErr {
    fn from(value: &str) -> Self {
        Self::new(500, value)
    }
}

impl From<actix_web::error::Error> for AppErr {
    fn from(value: actix_web::error::Error) -> Self {
        let r = value.error_response();
        Self {
            status: r.status().as_u16(),
            subject: "Err".to_string(),
            content: Some(value.to_string()),
        }
    }
}

impl<T> From<PoisonError<T>> for AppErr {
    fn from(value: PoisonError<T>) -> Self {
        let value = value.to_string();
        log::error!("err poision: {}", value);
        Self { status: 500, subject: value, content: None }
    }
}

macro_rules! impl_from_err {
    ($ty:path) => {
        impl From<$ty> for AppErr {
            fn from(value: $ty) -> Self {
                let value = value.to_string();
                log::error!("err 500: {}", value);
                Self {
                    status: 500,
                    subject: stringify!($ty).to_string(),
                    content: Some(value),
                }
            }
        }
    };
}

impl_from_err!(io::Error);
impl_from_err!(PayloadError);
impl_from_err!(ParseFloatError);
impl_from_err!(ParseIntError);
impl_from_err!(FromUtf8Error);
impl_from_err!(serde_json::Error);
impl_from_err!(ToStrError);

macro_rules! error_helper {
    ($name:ident, $status:ident, $subject:literal) => {
        #[doc = concat!("Helper function that wraps any error and generates a `", stringify!($status), "` response.")]
        #[allow(non_snake_case)]
        pub fn $name(err: &str) -> AppErr {
            AppErr {
                status: StatusCode::$status.as_u16(),
                subject: $subject.to_string(),
                content: Some(err.to_string())
            }
        }
    };
}

error_helper!(AppErrBadRequest, BAD_REQUEST, "bad request");
error_helper!(AppErrForbidden, FORBIDDEN, "forbidden");
error_helper!(AppErrNotFound, NOT_FOUND, "not found");
