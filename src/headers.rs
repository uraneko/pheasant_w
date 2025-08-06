use std::collections::HashMap;

/// HTTP header conversion from/to String
pub trait Header: serde::Serialize + serde::de::DeserializeOwned {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    fn from_str(s: &str) -> Self {
        serde_json::from_str(s).unwrap()
    }
}

/// read and write headers of a request/response
pub trait HeaderMap {
    /// get a header value from a request/response
    ///
    /// ```
    /// let mime: Mime = req.header("Content-Type");
    /// ```
    fn header<H: Header>(&self, key: &str) -> Option<H>;

    /// set a header value for a request/response
    ///
    /// ```
    /// let len = content.len();
    /// let maybe_old: Option<usize> = response.set_header("Content-Length", len);
    /// ```
    fn set_header<H: Header>(&mut self, key: &str, h: H) -> &mut Self;

    fn has_header<H: Header>(&self, key: &str) -> bool {
        self.header::<H>(key).is_some()
    }
}

impl HeaderMap for HashMap<String, String> {
    fn header<H: Header>(&self, key: &str) -> Option<H> {
        // TODO handle the unwrap error case
        self.get(key).map(|s| H::from_str(s))
    }

    fn set_header<H: Header>(&mut self, key: &str, h: H) -> &mut Self {
        self.insert(key.to_owned(), h.to_string());

        self
    }
}
