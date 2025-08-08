// #![allow(unused_imports)]
// #![allow(dead_code)]
// #![allow(unused_variables)]
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;
use std::str::Utf8Error;
use std::string::FromUtf8Error;

use pheasant_uri::Route;

// NOTE indefinitely experimental
// mod monopoly;

pub mod cookies;
pub mod cors;
pub mod failure;
pub mod headers;
pub mod mime;
pub mod requests;
pub mod response;
pub mod server;
pub mod service;
pub mod status;

pub use cookies::Cookie;
pub use cors::Cors;
pub use failure::Failure;
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

use proc_macro2::{Delimiter, Group, Punct, Spacing, Span, TokenStream as TS2, TokenTree};
use quote::{ToTokens, TokenStreamExt};
use syn::{Ident, Token};

impl ToTokens for Method {
    fn to_tokens(&self, tokens: &mut TS2) {
        tokens.append(<Method as Into<TokenTree>>::into(*self))
    }
}

impl From<Method> for TokenTree {
    fn from(m: Method) -> Self {
        let [ty, var] = {
            let s = m.to_string();
            let mut iter = s.split("::").map(|s| Ident::new(s, Span::call_site()));

            [iter.next().unwrap(), iter.next().unwrap()]
        };

        let mut group = TS2::new();
        group.append(ty);
        group.append(Punct::new(':', Spacing::Joint));
        group.append(Punct::new(':', Spacing::Alone));
        group.append(var);
        let group = Group::new(Delimiter::None, group);

        TokenTree::from(group)
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Method::{}{}",
            self.as_str().chars().next().unwrap(),
            &self.as_str()[1..].to_lowercase(),
        )
    }
}

impl Method {
    pub fn as_str(&self) -> &str {
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

impl FromStr for Method {
    type Err = PheasantError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
            _ => Err(Self::Err::ClientError(ClientError::BadRequest)),
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

impl FromStr for Protocol {
    type Err = PheasantError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.1" => Ok(Self::HTTP1_1),
            "HTTP/2" | "HTTP/3" => {
                Err(Self::Err::ServerError(ServerError::HTTPVersionNotSupported))
            }
            _ => Err(Self::Err::ClientError(ClientError::BadRequest)),
        }
    }
}

pub trait ServiceBundle {
    fn bundle_iter(self) -> std::vec::IntoIter<Service>;
}

impl ServiceBundle for Service {
    fn bundle_iter(self) -> std::vec::IntoIter<Service> {
        vec![self].into_iter()
    }
}

impl ServiceBundle for [Service; 2] {
    fn bundle_iter(self) -> std::vec::IntoIter<Service> {
        Vec::from(self).into_iter()
    }
}

impl ServiceBundle for [Service; 3] {
    fn bundle_iter(self) -> std::vec::IntoIter<Service> {
        Vec::from(self).into_iter()
    }
}

impl ServiceBundle for Vec<Service> {
    fn bundle_iter(self) -> std::vec::IntoIter<Service> {
        self.into_iter()
    }
}

impl_hdfs!(usize, i64, String);
