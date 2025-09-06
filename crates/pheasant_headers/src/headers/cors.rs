extern crate alloc;
use alloc::{borrow::ToOwned, format, string::String};
use chrono::TimeDelta;
use core::fmt::{self, Debug, Display, Formatter};
use hashbrown::{HashMap, HashSet};

use pheasant_core::{Header, HeaderMap, Method, Response, WildCardish};
use pheasant_uri::{Origin, OriginSet};

use crate::{HttpResult, ToHeader, ToHeaders};

// TODO Timing-Allow-Origin header

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RequestedCors {
    /// the cors req origin, client MUST provide this header
    origin: Origin,
    /// client MAY send this
    headers: Option<HashSet<String>>,
    /// client MUST send this
    method: Method,
}

impl FromHeaders for RequestedCors {
    type Headers = &mut HashMap<String, String>;

    fn from_headers(h: &mut HashMap<String, String>) -> HttpResult<Self> {
        let [Some(origin), Some(method)] = [
            h.remove("Origin"),
            h.remove("Access_Control_Request_Method"),
        ] else {
            // NOTE could also use 422 maybe
            return Err(ErrorStatus::Client(ClientError::BadRequest));
        };

        let headers = h.remove("Access_Control_Request_Headers");

        Ok(Self {
            origin,
            method,
            headers,
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RegisteredCors {
    /// allowed cors req method
    method: Method,
    /// allowed cors req headers
    headers: HashSet<String>,
    /// the server allows these headers to be exposed to the used in the client side
    /// see https://developer.mozilla.org/en-US/docs/Glossary/CORS-safelisted_response_header
    expose: Option<HashSet<String>>,
    /// set of whitelisted origins or glob '*' to allow any origin
    origins: WildCardish<HashSet<Origin>>,
    /// allow credentials for this cors requests
    credentials: bool,
    /// max-age dictates how long the response of an options request can be cached for
    max_age: Option<TimeDelta>,
}

impl RegisteredCors {
    fn allows_access_for_origin(&self, origin: &str) -> bool {
        if origin == "*" {
            return true;
        }

        self.origins.iter().any(|o| o.as_str() == origin)
    }

    fn origin_ref(&self, origin: &str) -> WildCardish<&Origin> {
        if origin == "*" {
            return WildCardish::Glob;
        }

        let Some(origin) = self.origins.iter().find(|o| o.as_str() == origin) else {
            unreachable!("origin is not allowed by the registered cors")
        };

        WildCardish::Value(origin)
    }
}

pub struct ResponseCors {
    method: Method,
    headers: &HashSet<String>,
    expose: &HashSet<String>,
    origin: &WildCardish<Origin>,
    credentials: bool,
    max_age: Option<String>,
}

impl ResponseCors {
    fn from_service(cors: RegisteredCors, origin: &str) -> HttpResult<Self> {
        if !cors.allows_access_for_origin(origin) {
            return Err(ErrorStatus::Client(ClientError::Forbidden));
        }

        Ok(Self {
            method: cors.method,
            max_age: cors.max_age.map(|ma| ma.to_string()),
            credentials: cors.credentials,
            origin: cors.origin_ref(origin),
            headers: &cors.headers,
            expose: &cors.expose,
        })
    }
}

impl ToHeaders for ResponseCors {
    type Item = [&str; 2];

    fn to_headers(&self) -> impl Iterator<Item = [&str; 2]> {
        Some(self.origin.to_header("Access-Control-Allow-Origin"))
            .chain(Some(self.methods.to_header("Access-Control-Allow-Method")))
            .chain(Some(self.headers.to_header("Access-Control-Allow-Headers")))
            .chain(
                self.expose
                    .map(|expose| expose.to_header("Access-Control-Expose-Headers")),
            )
            .chain(Some(
                self.credintials
                    .to_header("Access-Control-Allow-Credintials"),
            ))
            .chain(Some(self.max_age.to_header("Access-Control-Max-Age")))
    }
}

impl ToHeader for bool {
    type Output = [&str; 2];

    fn to_header(&self, header: &str) -> [&str; 2] {
        [header, if self { "true" } else { "false" }]
    }
}

impl ToHeader for Method {
    type Output = [&str; 2];

    fn to_header(&self, header: &str) -> [&str; 2] {
        [header, self.as_str()]
    }
}

impl ToHeader for HashSet<String> {
    type Output = [&str; 2];

    fn to_header(&self, header: &str) -> Self::Output {
        self.iter()
            .reduce(|acc, h| acc + h + ", ")
            .map(|h| [header, h.trim_end_matches(", ")])
    }
}

impl<T: ToHeader<Output = [&str; 2]>> ToHeader for WildCardish<T> {
    type Output = [&str; 2];

    fn to_header(&self, header: &str) -> Self::Output {
        match self {
            Self::Glob => [header, self.to_str()],
            Self::Value(t) => t.to_header(header),
        }
    }
}

impl ToHeader for String {
    type Output = [&str; 2];

    fn to_header(&self, header: &str) -> Self::Output {
        [header, self]
    }
}

impl RegisteredCors {
    /// no unwrap in this function is bad or dangerous, when used as/where intended
    ///
    /// this function was made to be used inside the http methods macros
    ///
    /// the args it takes are stringified from the correct values parsed and error handled in the
    /// macro
    pub fn macro_checked(
        methods: Method,
        headers: HashSet<String>,
        expose: Option<HashSet<String>>,
        origins: WildCardish<HashSet<Origin>>,
        credentials: bool,
        max_age: Option<i64>,
    ) -> Self {
        Self {
            credentials,
            method,
            headers,
            expose,
            origins,
            max_age: max_age.map(|ma| TimeDelta::new(ma, 0).unwrap()),
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn origin(&mut self, origin: Origin) -> &mut Self {
        if let WildCardish::Value(origins) = self.origins {
            origins.insert(origin)
        }

        self
    }

    /// allows any origin to make the cors request
    /// using the glob operator
    pub fn glob(&mut self) -> &mut Self {
        self.origins = WildCardish::Glob;

        self
    }

    pub fn method(&mut self, method: Method) -> &mut Self {
        todo!("i dont understand the cors method(s) logic")
    }

    pub fn header(&mut self, header: String) -> &mut Self {
        self.headers.insert(header)
    }

    pub fn headers<I>(&mut self, headers: I) -> &mut Self
    where
        I: IntoIterator<Item = String>,
    {
        self.headers.extend(headers)
    }

    pub fn expose(&mut self, header: String) -> &mut Self {
        self.expose.map(|mut ex| ex.insert(header));

        self
    }

    pub fn exposes<I>(&mut self, headers: I) -> &mut Self
    where
        I: IntoIterator<Item = String>,
    {
        self.expose.map(|mut ex| ex.extend(headers));

        self
    }

    pub fn max_age<T>(&mut self, ma: T) -> &mut Self
    where
        T: Into<TimeDelta>,
    {
        self.max_age.map(|_| ma.into());

        self
    }

    pub fn credentials(&mut self, creds: bool) -> &mut Self {
        self.credentials = creds;

        self
    }
}

// PERF the expose field should be: expose: Vec<&str>
// // <- self referencing from the headers field
// should use pin

impl Display for RegisteredCors {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
