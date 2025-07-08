use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

use super::{HttpMethod, ServerError};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Request {
    method: HttpMethod,
    proto: Protocol,
    uri: String,
    // params: Option<RequestParams<'a>>,
    params: Option<RequestParams>,
    body: Option<String>,
    headers: RequestHeaders,
}

impl Request {
    // parses a new request instance from the request data string
    pub fn parse(req: String) -> Result<Self, ServerError> {
        let has_body = req.contains("\r\n\r\n");
        let (req_line_and_headers, body) = {
            if has_body {
                let mut s = req.splitn(2, "\r\n\r\n");

                (s.next().unwrap(), s.next().map(|s| s.to_string()))
            } else {
                (req.as_ref(), None)
            }
        };
        let mut lines = req_line_and_headers.lines();

        let request_line = lines.next();
        let (method, uri, params, proto) = Self::parse_request_line(request_line)?;
        let headers = RequestHeaders::from_iter(lines);

        // let body = body.map(|b| parse_body(b))

        Ok(Self {
            method,
            proto,
            uri,
            body,
            params,
            headers,
        })
    }

    fn parse_request_line(
        l: Option<&str>,
    ) -> Result<(HttpMethod, String, Option<RequestParams>, Protocol), ServerError> {
        let l = l.ok_or(ServerError::RequestLineNotFound)?;

        let mut iter = l.split(' ');

        let method = iter.next().ok_or(ServerError::BadRequestLine)?.try_into()?;

        let uri = iter.next().ok_or(ServerError::BadRequestLine)?;
        let (uri, params) = Self::parse_uri_and_params(uri)?;

        let proto = iter.next().ok_or(ServerError::BadRequestLine)?.try_into()?;

        Ok((method, uri, params, proto))
    }

    fn parse_uri_and_params(uri: &str) -> Result<(String, Option<RequestParams>), ServerError> {
        let mut iter = uri.splitn(2, '?');

        let uri = iter
            .next()
            .map(|s| s.to_string())
            .unwrap_or(String::from("/"));
        let params = iter.next().map(|s| s.into());
        if uri.is_empty() {
            return Err(ServerError::BadRequestLine);
        }

        Ok((uri, params))
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

impl<'a> From<&'a str> for RequestParams {
    fn from(p: &'a str) -> Self {
        Self {
            params: p
                .split('&')
                .map(|kv| kv.splitn(2, '='))
                .map(|mut i| (i.next(), i.next()))
                .filter(|(a, b)| a.is_some() && b.is_some())
                .map(|(a, b)| {
                    (
                        a.map(|s| s.to_string()).unwrap(),
                        b.map(|s| s.to_string()).unwrap(),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestHeaders {
    headers: HashMap<String, String>,
}

impl<'a> RequestHeaders {
    fn from_iter<T>(iter: T) -> Self
    where
        T: Iterator<Item = &'a str>,
    {
        Self {
            headers: iter
                .map(|kv| kv.splitn(2, ':'))
                .map(|mut i| (i.next(), i.next()))
                .filter(|(a, b)| a.is_some() && b.is_some())
                .map(|(a, b)| {
                    (
                        a.map(|s| s.to_string()).unwrap(),
                        b.map(|s| s.to_string()).unwrap(),
                    )
                })
                .collect(),
        }
    }
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
