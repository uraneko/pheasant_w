use std::collections::HashSet;

use chrono::TimeDelta;

use crate::Method;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Cors {
    methods: HashSet<Method>,
    headers: HashSet<String>,
    expose: Option<HashSet<String>>,
    // TODO once Route is fixed/updated, use it instead of String
    origin: Option<String>,
    /// max-age dictates how long the response of an options request can be cached for
    max_age: Option<TimeDelta>,
}

impl Cors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn methods(&mut self) -> &mut HashSet<Method> {
        &mut self.methods
    }

    pub fn headers(&mut self) -> &mut HashSet<String> {
        &mut self.headers
    }

    pub fn origin(&mut self, o: &str) -> &mut Self {
        self.origin = Some(o.into());

        self
    }

    pub fn max_age(&mut self, duration: TimeDelta) -> &mut Self {
        self.max_age = Some(duration);

        self
    }

    pub fn format(&self) -> String {
        let mut cors = "".to_owned();

        if let Some(o) = &self.origin {
            cors.push_str("Access-Control-Allow-Origin: ");
            cors.push_str(o);
            cors.push('\n');
        }

        if !self.methods.is_empty() {
            cors.push_str("Access-Control-Allow-Methods: ");
            let methods = self
                .methods
                .iter()
                .fold("".to_owned(), |acc, m| acc + ", " + &format!("{:?}", m));

            cors.push_str(&methods);
            cors.push('\n');
        }

        if !self.headers.is_empty() {
            cors.push_str("Access-Control-Allow-Headers: ");
            let headers = self
                .headers
                .iter()
                .fold("".to_owned(), |acc, h| acc + ", " + &h);

            cors.push_str(&headers);
            cors.push('\n');
        }

        if let Some(expose) = &self.expose
            && !expose.is_empty()
        {
            cors.push_str("Access-Control-Expose-Headers: ");
            let headers = self
                .headers
                .iter()
                .fold("".to_owned(), |acc, h| acc + ", " + &h);

            cors.push_str(&headers);
            cors.push('\n');
        }

        if let Some(ma) = self.max_age {
            let ma = ma.num_seconds();
            cors.push_str("Access-Control-Max-Age: ");
            cors.push_str(&format!("{}", ma));
            cors.push('\n');
        }

        cors
    }
}

impl std::fmt::Display for Cors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}
