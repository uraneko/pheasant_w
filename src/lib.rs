// #![allow(unused_imports)]
// #![allow(dead_code)]
// #![allow(unused_variables)]
use std::str::Utf8Error;
use std::string::FromUtf8Error;

pub mod requests;
pub mod response;
pub mod server;
pub mod service;
pub mod status_codes;

pub use requests::{Protocol, Request};
pub use response::Response;
pub use server::Server;
pub use service::Service;
pub use status_codes::{
    ClientError, Informational, PassingStatus, Redirection, ResponseStatus, ServerError, Successful,
};

pub use pheasant_macro_get::get;

#[derive(Debug)]
pub enum PheasantError {
    ClientError(ClientError),
    ServerError(ServerError),
    RequestLineReadFailed,
    StreamReadCrached,
    StreamReadWithExcess,
    BytesParsingFailed,
    RequestIsEmpty,
    ExpectedRequestBody,
    InvalidIPAddr,
    RequestLineNotFound,
    BadRequestLine,
    BadMethodName,
    BadHttpVersion,
    RequestUriNotFound,
    InitialThreadCapacityHigherThanMaximumThreadsAllowed,
    IO(std::io::Error),
}

impl std::fmt::Display for PheasantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for PheasantError {}

impl From<std::io::Error> for PheasantError {
    fn from(_err: std::io::Error) -> Self {
        Self::ClientError(ClientError::BadRequest)
    }
}

impl From<std::num::ParseIntError> for PheasantError {
    fn from(_err: std::num::ParseIntError) -> Self {
        Self::ClientError(ClientError::BadRequest)
    }
}

impl From<Utf8Error> for PheasantError {
    fn from(_err: Utf8Error) -> Self {
        Self::ClientError(ClientError::BadRequest)
    }
}

impl From<FromUtf8Error> for PheasantError {
    fn from(_err: FromUtf8Error) -> Self {
        Self::ClientError(ClientError::BadRequest)
    }
}

impl From<url::ParseError> for PheasantError {
    fn from(_err: url::ParseError) -> Self {
        Self::ClientError(ClientError::BadRequest)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Route(String);

impl Route {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub trait IntoRoutes {
    fn into_routes(self) -> Vec<Route>;
}

// impl<T> IntoRoutes for T
// where
//     T: IntoIterator<Item = str>,
// {
//     fn into_routes(self) -> Vec<Route> {
//         t.into_iter().map(|r| r.into().collect())
//     }
// }

macro_rules! impl_into_routes {
    ($($t: ty),*) => {
        $(
            impl IntoRoutes for $t {
                fn into_routes(self) -> Vec<Route> {
                    self.into_iter().map(|r| (*r).into()).collect()
                }
            }
        )*
    };
}

impl_into_routes!(&[&str], Vec<&str>);

impl_into_routes!(
    [&str; 0], [&str; 1], [&str; 2], [&str; 3], [&str; 4], [&str; 5], [&str; 6], [&str; 7],
    [&str; 8], [&str; 9], [&str; 10], [&str; 11], [&str; 12]
);

impl<T> IntoRoutes for Option<T>
where
    T: IntoRoutes,
{
    fn into_routes(self) -> Vec<Route> {
        let Some(t) = self else {
            return vec![];
        };

        t.into_routes()
    }
}

impl<'a> IntoRoutes for &'a str {
    fn into_routes(self: &'a str) -> Vec<Route> {
        vec![self.into()]
    }
}

impl IntoRoutes for String {
    fn into_routes(self: String) -> Vec<Route> {
        vec![self.into()]
    }
}

impl From<String> for Route {
    fn from(s: String) -> Self {
        let s = if !s.starts_with('/') {
            format!("/{}", s)
        } else {
            s
        };

        Self(s)
    }
}
impl From<&str> for Route {
    fn from(s: &str) -> Self {
        let s = if !s.starts_with('/') {
            format!("/{}", s)
        } else {
            s.to_string()
        };

        Self(s)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Head,
    #[default]
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Connect,
    Options,
    Trace,
}

impl TryFrom<&[u8]> for Method {
    type Error = PheasantError;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        match s {
            b"HEAD" => Ok(Self::Head),
            b"GET" => Ok(Self::Get),
            b"POST" => Ok(Self::Post),
            b"PUT" => Ok(Self::Put),
            b"PATCH" => Ok(Self::Patch),
            b"DELETE" => Ok(Self::Delete),
            b"CONNECT" => Ok(Self::Connect),
            b"OPTIONS" => Ok(Self::Options),
            b"TRACE" => Ok(Self::Trace),
            _ => Err(Self::Error::ClientError(ClientError::BadRequest)),
        }
    }
}

impl TryFrom<&str> for Method {
    type Error = PheasantError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "HEAD" => Ok(Self::Head),
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "PATCH" => Ok(Self::Patch),
            "DELETE" => Ok(Self::Delete),
            "CONNECT" => Ok(Self::Connect),
            "OPTIONS" => Ok(Self::Options),
            "TRACE" => Ok(Self::Trace),
            _ => Err(Self::Error::ClientError(ClientError::BadRequest)),
        }
    }
}
