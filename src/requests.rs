use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

use url::Url;

use super::{ClientError, Method, PheasantError, ServerError};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Request {
    method: Method,
    proto: Protocol,
    uri: Url,
    body: Option<String>,
    headers: HashMap<String, String>,
}

impl Request {
    pub fn from_stream(stream: &mut TcpStream) -> Result<Self, PheasantError> {
        let mut v = vec![];
        let mut reader = BufReader::new(stream);
        // if error we return 400 bad request
        _ = read_req_line(&mut v, &mut reader)?;
        let (method, uri, proto) = parse_req_line(&mut v.drain(..))?;

        let headers = read_parse_headers(&mut v, &mut reader)?;

        let len = headers.header::<usize>("Content-Length");

        let body = if let Some(len) = len {
            read_body(&mut v, &mut reader, len)?;
            let b = String::from_utf8(v)?;

            Some(b)
        } else {
            None
        };

        Ok(Self {
            method,
            proto,
            uri,
            body,
            headers,
        })
    }

    pub fn method(&self) -> Method {
        self.method
    }

    pub fn uri(&self) -> &Url {
        &self.uri
    }

    pub fn query(&mut self) -> Option<&str> {
        self.uri.query()
    }

    pub fn contains_query(&self) -> bool {
        self.uri.query().is_some()
    }

    pub fn parse_query(&self) -> HashMap<&str, &str> {
        self.uri
            .query()
            .unwrap()
            .split('&')
            .map(|e| -> [&str; 2] { e.splitn(2, '=').collect::<Vec<&str>>().try_into().unwrap() })
            .map(|s| (s[0], s[1]))
            .collect()
    }

    pub fn parse_query_param(&self, p: &str) -> Option<&str> {
        self.uri
            .query()?
            .split('&')
            .find(|e| e.starts_with(p))
            .map(|v| &v[p.len() + 1..])
    }

    pub fn parse_query_params(&self, p: &[&str]) -> Vec<&str> {
        self.uri
            .query()
            .unwrap()
            .split('&')
            .filter(|e| p.into_iter().any(|p| e.starts_with(p)))
            .map(|v| &v[p.len() + 1..])
            .collect()
    }

    pub fn header<H: Header>(&self, key: &str) -> Option<H>
    where
        <H as std::str::FromStr>::Err: std::fmt::Debug,
    {
        // TODO handle the error
        self.headers.get(key).map(|s| s.parse::<H>().unwrap())
    }
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    #[default]
    HTTP1_1,
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

fn read_req_line(
    v: &mut Vec<u8>,
    s: &mut BufReader<&mut TcpStream>,
) -> Result<usize, PheasantError> {
    s.read_until(10, v)
        .map_err(|_| PheasantError::ClientError(ClientError::BadRequest))
}

fn parse_req_line(
    bytes: &mut impl Iterator<Item = u8>,
) -> Result<(Method, Url, Protocol), PheasantError> {
    let mut val = vec![];
    while let Some(b) = bytes.next()
        && b != 32
    {
        val.push(b);
    }
    let method = Method::try_from(val.as_slice())?;
    val.clear();

    while let Some(b) = bytes.next()
        && b != 32
    {
        val.push(b);
    }
    let uri = Url::parse(str::from_utf8(&val)?)?;
    val.clear();

    let proto = bytes.fold(val, |mut acc, b| {
        acc.push(b);
        acc
    });
    let proto = Protocol::try_from(proto.as_slice())?;

    Ok((method, uri, proto))
}

fn read_parse_headers(
    v: &mut Vec<u8>,
    s: &mut BufReader<&mut TcpStream>,
) -> Result<HashMap<String, String>, PheasantError> {
    let mut map = HashMap::new();

    while let Ok(n) = s.read_until(10, v) {
        if v.len() <= 2 && Some(&10) == v.last() {
            break;
        }
        let [n, v] = {
            let mut hf = v.drain(..);
            let mut name = String::new();
            while let Some(b) = hf.next() {
                if b == b':' {
                    break;
                }
                name.push(b as char);
            }
            let mut val = String::new();
            while let Some(b) = hf.next() {
                val.push(b as char);
            }

            [name, val]
        };

        map.insert(n, v);
    }

    Ok(map)
}

// WARN rn, if no content len header is found, server ignores request body
// TODO handle body with missing content length
fn read_body(
    v: &mut Vec<u8>,
    s: &mut BufReader<&mut TcpStream>,
    len: usize,
) -> Result<(), PheasantError> {
    v.resize(len, 0);
    s.read_exact(v)?;

    Ok(())
}

pub trait Header: std::str::FromStr {}

impl Header for usize {}

trait MapHeader {
    fn header<H: Header>(&self, key: &str) -> Option<H>
    where
        <H as std::str::FromStr>::Err: std::fmt::Debug;
}

impl MapHeader for HashMap<String, String> {
    fn header<H: Header>(&self, key: &str) -> Option<H>
    where
        <H as std::str::FromStr>::Err: std::fmt::Debug,
    {
        // TODO handle the error
        self.get(key).map(|s| s.parse::<H>().unwrap())
    }
}
