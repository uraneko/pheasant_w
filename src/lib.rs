// #![allow(unused_imports)]
// #![allow(dead_code)]
// #![allow(unused_variables)]

use std::borrow::Borrow;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle, scope, spawn};

pub mod requests;
pub mod server;

pub use requests::{Request, RequestBody, RequestParams};
pub use server::{Server, Service};

#[derive(Debug)]
pub enum ServerError {
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

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for ServerError {}

impl From<std::io::Error> for ServerError {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
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

impl TryFrom<&str> for HttpMethod {
    type Error = ServerError;

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
            _ => Err(Self::Error::BadMethodName),
        }
    }
}

pub enum MimeType {
    TextHtml,
    TextJs,
    TextCss,
    ApplicationJson,
    ImageSvgXml,
}
