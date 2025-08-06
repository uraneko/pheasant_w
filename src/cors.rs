use std::collections::HashSet;

use chrono::TimeDelta;

use crate::{Header, HeaderMap, Method, Response};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Cors {
    methods: HashSet<Method>,
    headers: HashSet<String>,
    expose: Option<HashSet<String>>,
    // TODO once Route is fixed/updated, use it instead of String
    origins: HashSet<String>,
    /// max-age dictates how long the response of an options request can be cached for
    max_age: Option<TimeDelta>,
}

impl Cors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allows_origin(&self, origin: &str) -> bool {
        self.origins.contains(origin)
    }

    pub fn method<T>(&mut self, m: T) -> &mut Self
    where
        T: TryInto<Method>,
        <T as TryInto<Method>>::Error: std::fmt::Debug,
    {
        self.methods.insert(m.try_into().unwrap());

        self
    }

    pub fn methods(&mut self, m: &[Method]) -> &mut Self {
        self.methods.extend(m);

        self
    }

    pub fn headers<H: Iterator<Item = String>>(&mut self, h: H) -> &mut Self {
        self.headers.extend(h.into_iter());

        self
    }

    pub fn expose<E: Iterator<Item = String>>(&mut self, e: E) -> &mut Self {
        self.expose.as_mut().map(|ex| ex.extend(e.into_iter()));

        self
    }

    pub fn origin(&mut self, o: &str) -> &mut Self {
        self.origins.insert(o.into());

        self
    }

    pub fn max_age(&mut self, duration: TimeDelta) -> &mut Self {
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
