#![no_std]
use pheasant_core::ErrorStatus;
extern crate alloc;
use alloc::{borrow::ToOwned, string::String};
use core::fmt::Display;
use core::str::FromStr;
use hashbrown::HashMap;

pub mod headers;
pub use headers::*;

#[macro_export]
macro_rules! impl_hdfs {
    ($($t: ty),*) => {
        $(
            impl Header for $t {
                fn to_string(&self) -> String {
                    <Self as ToString>::to_string(self)
                }

                fn from_str(s: &str) -> Self {
                    s.parse::<Self>().unwrap()
                }
            }
        )*
    }
}

/// HTTP header conversion from/to String
pub trait Header {
    fn to_string(&self) -> String;

    // TODO handle the unwrap error case
    fn from_str(s: &str) -> Self;
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
        self.get(key).map(|s| <H as Header>::from_str(s))
    }

    fn set_header<H: Header>(&mut self, key: &str, h: H) -> &mut Self {
        self.insert(key.to_owned(), h.to_string());

        self
    }
}

pub trait MessageHeaders {
    fn remove(&mut self, h: &str) -> Option<String>;

    fn remove_iter(&mut self, h: &[&str]) -> impl Iterator<Item = String>;

    fn take(&mut self, h: &str) -> Option<String>;

    fn take_iter(&mut self, h: &[&str]) -> impl Iterator<Item = String>;

    fn borrow(&self, h: &str) -> Option<&str>;

    fn borrow_iter(&self, h: &[&str]) -> impl Iterator<Item = &str>;
}

pub type HttpResult<T> = Result<T, ErrorStatus>;

pub trait FromHeaders {
    type Headers;

    fn from_headers(h: Self::Headers) -> HttpResult<Self>;
}

pub trait FromHeader {
    fn from_header(header: String) -> HttpResult<Self>;
}

pub trait ToHeaders {
    type Item;

    fn to_headers(&self) -> impl Iterator<Item = Self::Item>;

    // fn into_headers(self) -> impl Iterator<Item = [String; 2]>;
}

pub trait ToHeader {
    type Output;
    // fn into_header(self) -> [String; 2];

    fn to_header(&self, header: &str) -> Self::Output;
}
