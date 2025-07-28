use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

use super::{
    ClientError, Header, HeaderMap, Method, PheasantError, PheasantResult, Route, ServerError,
};
use crate::server::join_path;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Request {
    method: Method,
    proto: Protocol,
    route: Route,
    query: Option<HashMap<String, String>>,
    body: Option<String>,
    headers: HashMap<String, String>,
}

impl Request {
    pub fn from_stream(stream: &mut TcpStream) -> PheasantResult<Self> {
        let mut v = vec![];
        let mut reader = BufReader::new(stream);
        // if error we return 400 bad request
        _ = read_req_line(&mut v, &mut reader)?;
        let (method, route, query, proto) = parse_req_line(&mut v.drain(..))?;
        println!("parsed req line");

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
            route,
            query,
            body,
            headers,
        })
    }

    pub fn method(&self) -> Method {
        self.method
    }

    pub fn route(&self) -> &str {
        &self.route.0
    }

    pub fn take_route(&mut self) -> String {
        std::mem::take(&mut self.route).0
    }

    pub fn query(&mut self) -> Option<&HashMap<String, String>> {
        self.query.as_ref()
    }

    pub fn query_contains(&self) -> bool {
        self.query.is_some()
    }

    pub fn param(&self, key: &str) -> Option<&str> {
        let Some(ref map) = self.query else {
            return None;
        };

        map.get(key).map(|s| s.as_str())
    }

    // pub fn parse_query(&self) -> HashMap<&str, &str> {
    //     self
    //         .query
    //         .unwrap()
    //         .split('&')
    //         .map(|e| -> [&str; 2] { e.splitn(2, '=').collect::<Vec<&str>>().try_into().unwrap() })
    //         .map(|s| (s[0], s[1]))
    //         .collect()
    // }

    // pub fn parse_query_param(&self, p: &str) -> Option<&str> {
    //     self.route
    //         .query()?
    //         .split('&')
    //         .find(|e| e.starts_with(p))
    //         .map(|v| &v[p.len() + 1..])
    // }

    // pub fn parse_query_params(&self, p: &[&str]) -> Vec<&str> {
    //     self.route
    //         .query()
    //         .unwrap()
    //         .split('&')
    //         .filter(|e| p.into_iter().any(|p| e.starts_with(p)))
    //         .map(|v| &v[p.len() + 1..])
    //         .collect()
    // }

    pub fn header<H: Header>(&self, key: &str) -> Option<H>
    where
        <H as std::str::FromStr>::Err: std::fmt::Debug,
    {
        // TODO handle the error
        self.headers.get(key).map(|s| s.parse::<H>().unwrap())
    }

    pub fn proto(&self) -> Protocol {
        self.proto
    }

    pub fn headers(&mut self) -> HashMap<String, String> {
        std::mem::take(&mut self.headers)
    }
}

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

fn read_req_line(v: &mut Vec<u8>, s: &mut BufReader<&mut TcpStream>) -> PheasantResult<usize> {
    s.read_until(10, v)
        .map_err(|_| PheasantError::ClientError(ClientError::BadRequest))
}

fn parse_req_line(
    bytes: &mut impl Iterator<Item = u8>,
) -> Result<(Method, Route, Option<HashMap<String, String>>, Protocol), PheasantError> {
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
    let uri = str::from_utf8(&val)?;
    let (route, query) = if uri.contains('?') {
        let mut s = uri.splitn(2, '?');
        (s.next().unwrap().into(), s.next().map(|q| parse_query(q)))
    } else {
        let route = uri.into();

        (route, None)
    };
    val.clear();

    let proto = bytes
        .filter(|b| *b != 10 && *b != 13)
        .fold(val, |mut acc, b| {
            acc.push(b);
            acc
        });
    println!("p -> {:?}", proto);
    let proto = Protocol::try_from(proto.as_slice())?;

    Ok((method, route, query, proto))
}

fn read_parse_headers(
    v: &mut Vec<u8>,
    s: &mut BufReader<&mut TcpStream>,
) -> PheasantResult<HashMap<String, String>> {
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
            // skip whitespace
            _ = hf.next();
            let mut val = String::new();
            while let Some(b) = hf.next()
                && ![13, 10].contains(&b)
            {
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
fn read_body(v: &mut Vec<u8>, s: &mut BufReader<&mut TcpStream>, len: usize) -> PheasantResult<()> {
    v.resize(len, 0);
    s.read_exact(v)?;

    Ok(())
}

fn parse_query(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        // BUG this crashes the server when uri query is badly formatted
        // TODO scan query after getting request and return ClientError::BadRequest if query is faulty
        .map(|e| -> [&str; 2] { e.splitn(2, '=').collect::<Vec<&str>>().try_into().unwrap() })
        .map(|s| (s[0].to_string(), s[1].to_string()))
        .collect()
}
