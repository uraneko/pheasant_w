use std::collections::HashMap;
use std::str::FromStr;

use super::{HttpMethod, ServerError};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Request<'a> {
    method: HttpMethod,
    proto: Protocol,
    uri: &'a str,
    params: Option<RequestParams<'a>>,
    headers: RequestHeaders<'a>,
}

impl<'a> Request<'a> {
    // parses a new request instance from the request data string
    pub fn parse(req: &'a str) -> Result<Self, ServerError> {
        let mut lines = req.lines();
        let request_line = lines.next();
        let (method, uri, params, proto) = Self::parse_request_line(request_line)?;
        let headers = RequestHeaders::from_iter(lines);

        Ok(Self {
            method,
            proto,
            uri,
            params,
            headers,
        })
    }

    fn parse_request_line(
        l: Option<&'a str>,
    ) -> Result<(HttpMethod, &'a str, Option<RequestParams<'a>>, Protocol), ServerError> {
        let l = l.ok_or(ServerError::RequestLineNotFound)?;

        let mut iter = l.split(' ');

        let method = iter.next().ok_or(ServerError::BadRequestLine)?.try_into()?;

        let uri = iter.next().ok_or(ServerError::BadRequestLine)?;
        let (uri, params) = Self::parse_uri_and_params(uri)?;

        let proto = iter.next().ok_or(ServerError::BadRequestLine)?.try_into()?;

        Ok((method, uri, params, proto))
    }

    fn parse_uri_and_params(
        uri: &'a str,
    ) -> Result<(&'a str, Option<RequestParams<'a>>), ServerError> {
        let mut iter = uri.splitn(2, '?');

        let uri = iter.next().unwrap_or_default();
        let params = iter.next().map(|s| s.into());
        if uri.is_empty() {
            return Err(ServerError::BadRequestLine);
        }

        Ok((uri, params))
    }

    pub fn method(&self) -> HttpMethod {
        self.method
    }

    pub fn uri(&self) -> &'a str {
        self.uri
    }

    pub fn params(&self) -> Option<RequestParams<'a>> {
        self.params.clone()
    }

    fn headers(&self) -> RequestHeaders<'a> {
        self.headers.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestParams<'a> {
    params: HashMap<&'a str, &'a str>,
}

impl<'a> RequestParams<'a> {
    pub fn get(&self, key: &str) -> Option<&&str> {
        self.params.get(key)
    }
}

impl<'a> From<&'a str> for RequestParams<'a> {
    fn from(p: &'a str) -> Self {
        Self {
            params: p
                .split('&')
                .map(|kv| kv.splitn(2, '='))
                .map(|mut i| (i.next(), i.next()))
                .filter(|(a, b)| a.is_some() && b.is_some())
                .map(|(a, b)| (a.unwrap(), b.unwrap()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestHeaders<'a> {
    headers: HashMap<&'a str, &'a str>,
}

impl<'a> RequestHeaders<'a> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: Iterator<Item = &'a str>,
    {
        Self {
            headers: iter
                .map(|kv| kv.splitn(2, ':'))
                .map(|mut i| (i.next(), i.next()))
                .filter(|(a, b)| a.is_some() && b.is_some())
                .map(|(a, b)| (a.unwrap(), b.unwrap()))
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
