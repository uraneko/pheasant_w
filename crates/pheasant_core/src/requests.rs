use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

use super::{ClientError, Header, HeaderMap, Method, PheasantError, PheasantResult, Protocol};
use pheasant_uri::{Query, Route, Scheme, Url};

/// HTTP Request type
/// used in services to generate service input type; R: From<Request>
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Request {
    method: Method,
    proto: Protocol,
    route: Route,
    scheme: Option<Scheme>,
    query: Option<Query>,
    body: Option<String>,
    headers: HashMap<String, String>,
}

impl Request {
    /// parse the tcp stream request bytes into a http Request instance
    ///
    /// ### Error
    ///
    /// returns a `PheasantError` in case of a bad request
    pub(crate) fn from_stream<R: Read>(stream: &mut R) -> PheasantResult<Self> {
        let mut v = vec![];
        let mut reader = BufReader::new(stream);
        // if error we return 400 bad request
        _ = read_req_line(&mut v, &mut reader)?;
        let (method, mut url, proto) = parse_req_line(&mut v.drain(..))?;
        println!("{}", url.to_string());
        let (scheme, route, query) = (url.scheme(), url.take_path(), url.take_query());
        let route = Route::new(route.unwrap());

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
            scheme,
            route,
            query,
            body,
            headers,
        })
    }

    /// returns a copy of this request's http Method
    pub fn method(&self) -> Method {
        self.method
    }

    /// returns a reference `&str` of this request's route value
    pub fn route(&self) -> &str {
        // WARN this uses Deref
        &self.route
    }

    /// returns this request's route value String
    /// doesn't clone, uses std::mem::take
    /// Note that the original request route value becomes String::new() once this is used
    pub fn take_route(&mut self) -> String {
        // WARN this uses DerefMut
        std::mem::take(&mut self.route)
    }

    /// if the request has a query, this returns a reference to it,
    /// otherwise, returns `None`
    pub fn query(&self) -> Option<&Query> {
        self.query.as_ref()
    }

    pub fn take_query(&mut self) -> Option<Query> {
        std::mem::take(&mut self.query)
    }

    /// checks if this request has a query
    pub fn has_query(&self) -> bool {
        self.query.is_some()
    }

    /// returns a reference to the param in the request query if both exist
    /// Otherwise, returns `None`
    pub fn param(&self, key: &str) -> Option<&str> {
        let Some(ref query) = self.query else {
            return None;
        };

        query.param(key)
    }

    /// returns a bool indicating wether this request's query
    /// contains a param named `key`
    pub fn contains_param(&self, key: &str) -> bool {
        let Some(ref query) = self.query else {
            return false;
        };

        query.contains_param(key)
    }

    pub fn contains_attr(&self, key: &str) -> bool {
        let Some(ref query) = self.query else {
            return false;
        };

        query.contains_attr(key)
    }

    pub fn body(&self) -> Option<&str> {
        self.body.as_ref().map(|s| s.as_str())
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

    /// returns a copy of this request's protocol
    pub fn proto(&self) -> Protocol {
        self.proto
    }

    /// takes this request's headers map and returns them
    ///
    /// once this is used, self.headers becomes an empty `HashMap`
    pub fn headers(&mut self) -> HashMap<String, String> {
        std::mem::take(&mut self.headers)
    }
}

impl HeaderMap for Request {
    fn header<H: Header>(&self, key: &str) -> Option<H> {
        self.headers.header(key)
    }

    fn set_header<H: Header>(&mut self, key: &str, h: H) -> &mut Self {
        self.headers.set_header(key, h);

        self
    }
}

fn read_req_line<R: Read>(v: &mut Vec<u8>, s: &mut BufReader<&mut R>) -> PheasantResult<usize> {
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
    let Ok(url) = str::from_utf8(&val).unwrap().parse::<Url>() else {
        return Err(PheasantError::ClientError(ClientError::BadRequest));
    };

    val.clear();

    let proto = bytes
        .filter(|b| *b != 10 && *b != 13)
        .fold(val, |mut acc, b| {
            acc.push(b);
            acc
        });
    let proto = Protocol::try_from(proto.as_slice())?;

    Ok((method, url, proto))
}

fn read_parse_headers<R: Read>(
    v: &mut Vec<u8>,
    s: &mut BufReader<&mut R>,
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
fn read_body<R: Read>(
    v: &mut Vec<u8>,
    s: &mut BufReader<&mut R>,
    len: usize,
) -> PheasantResult<()> {
    v.resize(len, 0);
    s.read_exact(v)?;

    Ok(())
}

impl From<&Request> for Request {
    fn from(req: &Request) -> Self {
        req.clone()
    }
}
