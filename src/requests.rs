use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

use super::{HttpMethod, ServerError};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Request {
    method: HttpMethod,
    proto: Protocol,
    uri: String,
    params: Option<RequestParams>,
    body: Option<RequestBody>,
    headers: RequestHeaders,
}

impl Request {
    // parses a new request instance from the request data string
    pub fn parse_from(req: String) -> Result<Self, ServerError> {
        if req.is_empty() {
            return Err(ServerError::RequestIsEmpty);
        }

        let (hdrs, body) = parse_headers_body(req)?;

        let mut lines = hdrs.lines();

        let request_line = lines.next();
        let (method, uri, params, proto) = parse_request_line(request_line)?;
        let headers = RequestHeaders {
            headers: map_iter(':', lines),
        };

        Ok(Self {
            method,
            proto,
            uri,
            body,
            params,
            headers,
        })
    }

    pub fn method(&self) -> HttpMethod {
        self.method
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn params_ref(&self) -> Option<&RequestParams> {
        self.params.as_ref()
    }

    pub fn take_params(&mut self) -> Option<RequestParams> {
        std::mem::take(&mut self.params)
    }

    fn headers(&self) -> &RequestHeaders {
        &self.headers
    }
}

// NOTE still not sure if this can error
fn parse_headers_body(req: String) -> Result<(String, Option<RequestBody>), ServerError> {
    let has_body = !req.ends_with("\r\n\r\n");

    if has_body {
        let mut s = req.splitn(2, "\r\n\r\n");

        Ok((
            s.next().map(|s| s.to_string()).unwrap(),
            s.next().map(|b| RequestBody {
                body: map_str('&', '=', b),
            }),
        ))
    } else {
        Ok((req, None))
    }
}

fn parse_uri_and_params(uri: &str) -> Result<(String, Option<RequestParams>), ServerError> {
    let mut iter = uri.splitn(2, '?');

    let uri = iter
        .next()
        .map(|s| s.to_string())
        .unwrap_or(String::from("/"));
    let params = iter.next().map(|s| RequestParams {
        params: map_str('&', '=', s),
    });
    if uri.is_empty() {
        return Err(ServerError::BadRequestLine);
    }

    Ok((uri, params))
}

fn parse_request_line(
    l: Option<&str>,
) -> Result<(HttpMethod, String, Option<RequestParams>, Protocol), ServerError> {
    let l = l.ok_or(ServerError::RequestLineNotFound)?;

    let mut iter = l.split(' ');

    let method = iter.next().ok_or(ServerError::BadRequestLine)?.try_into()?;

    let uri = iter.next().ok_or(ServerError::BadRequestLine)?;
    let (uri, params) = parse_uri_and_params(uri)?;

    let proto = iter.next().ok_or(ServerError::BadRequestLine)?.try_into()?;

    Ok((method, uri, params, proto))
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RequestParams {
    params: HashMap<String, String>,
}

impl RequestParams {
    pub fn get_ref(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_ref())
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.params.remove(key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RequestHeaders {
    headers: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RequestBody {
    body: HashMap<String, String>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    V1_1,
    V2,
}

impl TryFrom<&str> for Protocol {
    type Error = ServerError;

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        match v {
            "HTTP/1.1" => Ok(Self::V1_1),
            "HTTP/2" => Ok(Self::V2),
            _ => Err(Self::Error::BadHttpVersion),
        }
    }
}

fn map_iter<'a, T: Iterator<Item = &'a str>>(key_sep: char, iter: T) -> HashMap<String, String> {
    iter.map(|p| p.splitn(2, key_sep))
        .map(|mut i| (i.next(), i.next()))
        // this gets rid of all faulty pairs
        .filter(|(a, b)| a.is_some() && b.is_some())
        .map(|(a, b)| {
            (
                a.map(|s| s.to_string()).unwrap(),
                b.map(|s| s.to_string()).unwrap(),
            )
        })
        .collect()
}

fn map_str(key_sep: char, pair_sep: char, map: &str) -> HashMap<String, String> {
    map.split(pair_sep)
        .map(|kv| kv.splitn(2, key_sep))
        .map(|mut i| (i.next(), i.next()))
        // this gets rid of all faulty pairs
        .filter(|(a, b)| a.is_some() && b.is_some())
        .map(|(a, b)| {
            (
                a.map(|s| s.to_string()).unwrap(),
                b.map(|s| s.to_string()).unwrap(),
            )
        })
        .collect()
}
