// #![allow(unused_imports)]
// #![allow(dead_code)]
// #![allow(unused_variables)]
use std::str::Utf8Error;
use std::string::FromUtf8Error;

// NOTE indefinitely experimental
mod monopoly;

pub mod cookies;
pub mod cors;
pub mod fail;
pub mod headers;
pub mod mime;
pub mod requests;
pub mod response;
pub mod server;
pub mod service;
pub mod status;

pub use cookies::Cookie;
pub use cors::Cors;
pub use fail::Fail;
pub use headers::{Header, HeaderMap};
pub use mime::Mime;
pub use requests::Request;
pub use response::Response;
pub use server::Server;
pub use service::Service;
pub use status::{
    ClientError, ErrorStatus, Informational, Redirection, ResponseStatus, ServerError, Status,
    Successful,
};

pub type PheasantResult<T> = Result<T, PheasantError>;

/// crate's main error type
#[derive(Debug)]
pub enum PheasantError {
    ClientError(ClientError),
    ServerError(ServerError),
}

impl std::fmt::Display for PheasantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for PheasantError {}

// WARN this is senseless, should be PortIsTaken error variant
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

/// uri route type,
/// e.g., "/index.html"
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Route(String);

impl Route {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// used for service redirections generations
pub trait IntoRoutes {
    /// consumes self and returns the routes
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

// impl IntoRoutes for T: IntoIterator<Item = &str>
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
        let s = if !s.starts_with('/') && s != "*" {
            format!("/{}", s)
        } else {
            s.to_string()
        };

        Self(s)
    }
}

/// HTTP Method enum
/// only Get method is somewhat supported at the moment
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
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

impl Method {
    fn as_str(&self) -> &str {
        match self {
            Self::Head => "HEAD",
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
            Self::Connect => "CONNECT",
            Self::Options => "OPTIONS",
            Self::Trace => "TRACE",
        }
    }
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
        match s.to_uppercase().as_str() {
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

/// Http protocol version
///
/// currently only http 1.1 is supported
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protocol {
    #[default]
    HTTP1_1,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::HTTP1_1 => "HTTP/1.1",
            }
        )
    }
}

impl TryFrom<&[u8]> for Protocol {
    type Error = PheasantError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        match v {
            b"HTTP/1.1" => Ok(Self::HTTP1_1),
            b"HTTP/2" | b"HTTP/3" => Err(Self::Error::ServerError(
                ServerError::HTTPVersionNotSupported,
            )),
            _ => Err(Self::Error::ClientError(ClientError::BadRequest)),
        }
    }
}

impl TryFrom<&str> for Protocol {
    type Error = PheasantError;

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        match v {
            "HTTP/1.1" => Ok(Self::HTTP1_1),
            "HTTP/2" | "HTTP/3" => Err(Self::Error::ServerError(
                ServerError::HTTPVersionNotSupported,
            )),
            _ => Err(Self::Error::ClientError(ClientError::BadRequest)),
        }
    }
}
