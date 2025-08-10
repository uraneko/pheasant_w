use serde::de::{Deserializer, Error, Visitor};
use serde::ser::Serializer;
use std::collections::{HashMap, HashSet};

use crate::{ParseError, ParseResult, Parser, Query, TransmuteError, lex, parse::ref_res};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Scheme {
    Http,
    Https,
    Ws,
    Wss,
    File,
    Ftp,
}

impl Scheme {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
            Self::Ws => "ws",
            Self::Wss => "wss",
            Self::File => "file",
            Self::Ftp => "ftp",
        }
    }

    pub fn is_scheme(s: &str) -> bool {
        match s {
            "http" | "https" | "ws" | "wss" | "file" | "ftp" => true,
            _ => false,
        }
    }
}

impl std::str::FromStr for Scheme {
    type Err = ParseError;
    fn from_str(s: &str) -> ParseResult<Self> {
        match s.to_uppercase().as_str() {
            "HTTP" => Ok(Self::Http),
            "HTTPS" => Ok(Self::Https),
            "WS" => Ok(Self::Ws),
            "WSS" => Ok(Self::Wss),
            "FILE" => Ok(Self::File),
            "FTP" => Ok(Self::Ftp),
            _ => ParseError::url(0).map(|_| unsafe { std::mem::zeroed() }),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Url {
    scheme: Option<Scheme>,
    domain: Option<Vec<String>>,
    port: Option<u16>,
    path: Option<Vec<String>>,
    query: Option<Query>,
    fragment: Option<String>,
}

impl Url {
    fn update_scheme(&mut self, scheme: Option<Scheme>) {
        self.scheme = scheme;
    }

    fn update_domain(&mut self, domain: Vec<String>) {
        self.domain = Some(domain);
    }

    fn update_port(&mut self, port: u16) {
        self.port = Some(port);
    }

    fn update_path(&mut self, path: Vec<String>) {
        self.path = Some(path);
    }

    fn update_query(&mut self, query: Query) {
        self.query = Some(query);
    }

    fn update_fragment(&mut self, frag: String) {
        self.fragment = Some(frag);
    }
}

impl Url {
    pub fn matches_any_origin(&self) -> bool {
        let Some(ref domain) = self.domain else {
            return false;
        };

        domain.len() == 1 && domain[0] == "*"
    }
}

// TODO WARN file scheme needs special handling which is not implemented yet

impl std::str::FromStr for Url {
    type Err = ParseError;

    fn from_str(s: &str) -> ParseResult<Self> {
        let Some(parser) = Parser::new(s) else {
            return Err(ref_res(ParseError::url(0)).unwrap());
        };

        parser.parse()
    }
}

impl Url {
    pub fn path_absolute(
        path: Vec<&str>,
        query: Option<(HashMap<&str, &str>, HashSet<&str>)>,
        fragment: Option<String>,
    ) -> Self {
        Self {
            path: Some(path.into_iter().map(|s| s.into()).collect()),
            query: query.map(|(params, attrs)| Query::from_colls(params, attrs)),
            fragment,
            ..Default::default()
        }
    }

    pub fn scheme_relative(
        domain: Vec<String>,
        port: Option<u16>,
        path: Option<Vec<String>>,
        query: Option<(HashMap<&str, &str>, HashSet<&str>)>,
        fragment: Option<String>,
    ) -> Self {
        Self {
            domain: Some(domain),
            port,
            path,
            query: query.map(|(params, attrs)| Query::from_colls(params, attrs)),
            fragment,
            ..Default::default()
        }
    }

    pub fn new(s: &str) -> Result<Self, ParseError> {
        s.parse()
    }

    pub fn from_parts(
        scheme: Option<Scheme>,
        domain: Option<Vec<String>>,
        port: Option<u16>,
        path: Option<Vec<String>>,
        query: Option<Query>,
        fragment: Option<String>,
    ) -> Self {
        Self {
            domain,
            port,
            path,
            query,
            fragment,
            scheme,
        }
    }

    pub fn absolute(
        scheme: Scheme,
        domain: Vec<String>,
        port: Option<u16>,
        path: Option<Vec<String>>,
        query: Option<Query>,
        fragment: Option<String>,
    ) -> Self {
        Self {
            domain: Some(domain),
            port,
            path,
            query,
            fragment,
            scheme: Some(scheme),
        }
    }

    pub fn sequence(&self) -> String {
        let scheme = self
            .scheme
            .map(|s| format!("{}://", s.as_str()))
            .unwrap_or_default();

        let mut domain = if let Some(ref domain) = self.domain {
            let mut domain = domain.into_iter().fold(scheme, |acc, d| acc + d + ".");
            domain.pop();

            domain
        } else {
            scheme
        };

        if let Some(port) = self.port {
            domain.push_str(&format!(":{}", port));
        }

        let mut path = if let Some(ref path) = self.path {
            if path.is_empty() {
                "/".to_owned()
            } else {
                path.into_iter().fold(domain, |acc, s| acc + "/" + s)
            }
        } else {
            domain
        };

        if let Some(ref query) = self.query.as_ref().map(|q| q.sequence()) {
            path.push('?');
            path.push_str(query);
        }

        if let Some(ref fragment) = self.fragment {
            path.push('#');
            path.push_str(fragment);
        }

        path
    }

    /// downcasts the Url instance to sub url type
    pub fn interpret<T>(self) -> Result<T, TransmuteError>
    where
        T: TryFrom<Self, Error = TransmuteError>,
    {
        self.try_into()
    }
}

impl Url {
    pub fn scheme(&self) -> Option<Scheme> {
        self.scheme
    }

    pub fn take_domain(&mut self) -> Option<Vec<String>> {
        let Some(ref mut domain) = self.domain else {
            return None;
        };

        Some(std::mem::take(domain))
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn take_path(&mut self) -> Option<Vec<String>> {
        let Some(ref mut path) = self.path else {
            return None;
        };

        Some(std::mem::take(path))
    }

    pub fn take_query(&mut self) -> Option<Query> {
        let Some(ref mut query) = self.query else {
            return None;
        };

        Some(std::mem::take(query))
    }

    pub fn take_fragment(&mut self) -> Option<String> {
        let Some(ref mut fragment) = self.fragment else {
            return None;
        };

        Some(std::mem::take(fragment))
    }
}

// serde traits
impl serde::Serialize for Url {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.sequence())
    }
}

struct UrlVisitor;

impl<'de> Visitor<'de> for UrlVisitor {
    type Value = Url;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("expected str value of a url query")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let url = v
            .parse::<Url>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(url)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let url = v
            .parse::<Url>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(url)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let url = v
            .parse::<Url>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(url)
    }

    fn visit_bytes<E>(self, b: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(b).map_err(|_| E::custom("invalid bytes"))?;

        let url = s
            .parse::<Url>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(url)
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(v).map_err(|_| E::custom("invalid bytes"))?;

        let url = s
            .parse::<Url>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(url)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(&v).map_err(|_| E::custom("invalid bytes"))?;

        let url = s
            .parse::<Url>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(url)
    }

    // this means that none values would deserialize to mime default
    // fn visit_none<E>(self) -> Result<Self::Value, E>
    // where
    //     E: Error,
    // {
    //     Ok(Mime::default())
    // }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(UrlVisitor)
    }
}

impl<'de> serde::Deserialize<'de> for Url {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "Url",
            &["scheme", "domain", "port", "path", "query", "fragment"],
            UrlVisitor,
        )
    }
}
