use chrono::TimeDelta;
use std::collections::HashSet;

use crate::{Header, HeaderMap, Method, Response};
use pheasant_uri::{Origin, OriginSet};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Cors {
    methods: HashSet<Method>,
    headers: HashSet<String>,
    expose: Option<HashSet<String>>,
    origins: OriginSet,
    /// max-age dictates how long the response of an options request can be cached for
    max_age: Option<TimeDelta>,
}

impl Cors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allows_origin(&self, origin: &Origin) -> bool {
        self.origins.contains(origin)
    }

    pub fn methods(&mut self) -> &mut HashSet<Method> {
        &mut self.methods
    }

    pub fn headers(&mut self) -> &mut HashSet<String> {
        &mut self.headers
    }

    pub fn alloc_expose(&mut self) {
        self.expose = Some(HashSet::new());
    }

    pub fn expose(&mut self) -> Option<&mut HashSet<String>> {
        self.expose.as_mut()
    }

    pub fn origins(&mut self) -> Option<&mut HashSet<Origin>> {
        self.origins.origins()
    }

    pub fn overwrite_origins(&mut self, origins: OriginSet) {
        self.origins = origins;
    }

    pub fn update_max_age(&mut self, duration: TimeDelta) -> &mut Self {
        self.max_age = Some(duration);

        self
    }

    pub fn cors_methods(&self) -> &HashSet<Method> {
        &self.methods
    }

    pub fn cors_headers(&self) -> &HashSet<String> {
        &self.headers
    }

    // PERF the expose field should be: expose: Vec<&str>
    // // <- self referencing from the headers field
    // should use pin
    pub fn cors_expose(&self) -> Option<&HashSet<String>> {
        self.expose.as_ref()
    }

    pub fn cors_max_age(&self) -> Option<i64> {
        self.max_age.as_ref().map(|td| td.num_seconds())
    }

    // pub fn set_headers(&self, resp: &mut Response) {
    //     self.cors_methods(resp);
    //     self.cors_headers(resp);
    //     self.cors_expose(resp);
    //     self.cors_max_age(resp);
    // }

    // pub fn format(&self, origin: &str) -> String {
    //     let mut cors = "".to_owned();
    //
    //     // if let Some(o) = &self.origins {
    //     cors.push_str("Access-Control-Allow-Origin: ");
    //     cors.push_str(origin);
    //     cors.push('\n');
    //     // }
    //
    //     if !self.methods.is_empty() {
    //         cors.push_str("Access-Control-Allow-Methods: ");
    //         let methods = self
    //             .methods
    //             .iter()
    //             .fold("".to_owned(), |acc, m| acc + ", " + &format!("{:?}", m));
    //
    //         cors.push_str(&methods);
    //         cors.push('\n');
    //     }
    //
    //     if !self.headers.is_empty() {
    //         cors.push_str("Access-Control-Allow-Headers: ");
    //         let headers = self
    //             .headers
    //             .iter()
    //             .fold("".to_owned(), |acc, h| acc + ", " + &h);
    //
    //         cors.push_str(&headers);
    //         cors.push('\n');
    //     }
    //
    //     if let Some(expose) = &self.expose
    //         && !expose.is_empty()
    //     {
    //         cors.push_str("Access-Control-Expose-Headers: ");
    //         let headers = self
    //             .headers
    //             .iter()
    //             .fold("".to_owned(), |acc, h| acc + ", " + &h);
    //
    //         cors.push_str(&headers);
    //         cors.push('\n');
    //     }
    //
    //     if let Some(ma) = self.max_age {
    //         let ma = ma.num_seconds();
    //         cors.push_str("Access-Control-Max-Age: ");
    //         cors.push_str(&format!("{}", ma));
    //         cors.push('\n');
    //     }
    //
    //     cors
    // }
}

impl std::fmt::Display for Cors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Header for HashSet<String> {
    fn to_string(&self) -> String {
        let mut s = self
            .into_iter()
            .fold("".to_owned(), |acc, s| acc + s + ", ");
        s.pop();
        s.pop();

        s
    }

    fn from_str(s: &str) -> Self {
        let s = format!("[ {} ]", s);

        serde_json::from_str(&s).unwrap()
    }
}

impl Header for HashSet<Method> {
    fn to_string(&self) -> String {
        let mut s = self
            .into_iter()
            .fold("".to_owned(), |acc, m| acc + m.as_str() + ", ");
        s.pop();
        s.pop();

        s
    }

    fn from_str(s: &str) -> Self {
        let s = format!("[ {} ]", s);

        serde_json::from_str(&s).unwrap()
    }
}

impl Header for i64 {}
impl Header for String {}
