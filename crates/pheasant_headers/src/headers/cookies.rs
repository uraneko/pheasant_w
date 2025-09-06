extern crate alloc;
use alloc::{borrow::ToOwned, format, string::String};
use chrono::{DateTime, TimeDelta, Utc};
use core::fmt::{self, Debug, Display, Formatter};
use hashbrown::{HashMap, HashSet};

use pheasant_core::{Header, HeaderMap, Method, Response, WildCardish};
use pheasant_uri::{Origin, OriginSet};

use crate::{FromHeader, HttpResult, ToHeader, ToHeaders};

// #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
// pub struct Cookies(HashSet<Cookie>);

impl ToHeaders for HashSet<Cookie> {
    type Item = [&str; String];

    fn to_headers(&self) -> impl Iterator<Item = Self::Item> {
        self.iter().map(|cookie| cookie.to_header("Set-Cookie"))
    }
}

impl ToHeader for HashSet<Cookie> {
    type Output = String;

    fn to_header(&self) -> Self::Output {
        let mut header = self
            .iter()
            .fold("".to_owned(), |acc, cookie| acc + cookie.to_string() + "; ");

        header.pop();
        header.pop();

        header
    }
}

// DOCS
// the Cookie header is client specific
// while the Set-Cookie header is server only

// WARN browsers' session restore feature also restores session cookies
// NOTE
// if no expires or max-age attrs are set then the cookie is auto expired at browser session shutdown
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, Hash)]
pub struct Cookie {
    expires: Option<DateTime<Utc>>,
    max_age: Option<TimeDelta>,
    http_only: bool,
    // requires the Secure attr
    partitioned: bool,
    secure: bool,
    path: Option<String>,
    domain: Option<String>,
    same_site: Option<SameSite>,
    key: String,
    val: String,
}

const EXTS: [&str; 7] = [
    "Domain",
    "Expires",
    "HttpOnly",
    "Max-Age",
    "Partitioned",
    "Path",
    "SameSite",
];

fn split_on_pat<P>(s: &mut String, delim: P) -> Option<String>
where
    P: core::str::pattern::Pattern,
{
    if !EXTS.iter().any(|ext| s.starts_with(ext)) {
        return None;
    }

    s.find(delim).map(|idx| s.drain(..idx).collect())
}

fn take_key_val(s: &mut String) -> Option<HttpResult<[String; 2]>> {
    if s.is_empty() {
        return None;
    }

    let [Some(key), Some(val)] = [
        s.find('=')
            .map(|idx| s.drain(..idx).collect())
            .ok_or_else(|| todo!("error")),
        s.find("; ")
            .map(|idx| s.drain(..idx).collect())
            .ok_or_else(|| todo!("error")),
    ] else {
        Some(Err(8));
    };

    Some(Ok([key, val]))
}

// WARN HTTP/2 allows requests to have many Cookie headers for compression optimizations
// HTTP/1.1 tho, doesnt allow this feature
impl FromHeaders for HashSet<Cookie> {
    fn from_headers(mut h: HashSet<String>) -> HttpResult<Self> {
        if h.is_empty() {
            return Err(3);
        }

        Ok(h.into_iter()
            .map(|mut header| {
                let Some(kv) = take_key_val(&mut header) else {
                    return Err(6);
                };

                let [ref k, ref v] = kv?;

                Cookie::new(k, v).fill_out(&mut header)
            })
            .collect())
    }
}

impl FromHeader for HashSet<Cookie> {
    fn from_header(mut header: String) -> HttpResult<Self> {
        let mut set = HashSet::new();
        while let Some(kv) = take_key_val(&mut header) {
            let [ref k, ref v] = kv?;
            set.insert(Cookie::new(k, v).fill_out(&mut header)?);
        }

        Ok(set)
    }
}

impl Cookie {
    pub fn new(k: &str, v: &str) -> Self {
        Self {
            key: k.to_owned(),
            val: v.to_owned(),
            ..Default::default()
        }
    }

    // adds an expiration datetime to the cookie
    // takes a datetim after which the cookie should be expired
    // this sets the Expires cookie attribute
    //
    // WARN the server sets this attribute using its own clock, which may differ from the client
    // side's clock,
    // it is advised to use the less error prone Max-Age attr instead
    pub fn expires(&mut self, datetime: DateTime<Utc>) -> &mut Self {
        self.expires = Some(datetime);

        self
    }

    // adds an expiration datetime to the cookie
    // takes a duration after which the cookie should be expired
    // this sets the Max-Age cookie attribute
    pub fn max_age(&mut self, delta: TimeDelta) -> &mut Self {
        self.max_age = Some(delta);

        self
    }

    // if switch is set to true then the client side can not access this cookie using javascript
    pub fn http_only(&mut self, switch: bool) -> &mut Self {
        self.http_only = switch;

        self
    }

    /// sets the request uri path that triggers this cookie to be send with the request
    /// sub paths will also be considered as matches
    pub fn path(&mut self, path: &str) -> &mut Self {
        self.path = Some(path.into());

        self
    }

    /// sets the domain to which the client will send this cookie with requests
    /// if the domain value does not include the cookie defining server, then
    /// the cookie is rejected
    /// that includes server subdomains;
    /// example.com can not send a cookie with Domain=foo.example.com
    pub fn domain(&mut self, domain: &str) -> &mut Self {
        self.domain = Some(domain.into());

        self
    }

    pub fn same_site<SS>(&mut self, ss: SS) -> &mut Self
    where
        SS: TryInto<SameSite>,
        <SS as TryInto<SameSite>>::Error: std::fmt::Debug,
    {
        self.same_site = Some(ss.try_into().unwrap());

        self
    }

    /// switch for whether the cookie should be stored in partitioned storage
    /// requires the Secure attr
    pub fn partitioned(&mut self, switch: bool) -> &mut Self {
        self.partitioned = switch;

        self
    }

    /// only send this cookie with requests coming from the `https:` scheme
    pub fn secure(&mut self, switch: bool) -> &mut Self {
        self.secure = switch;

        self
    }
}

impl Cookie {
    fn fill_out(mut self, s: &mut String) -> HttpResult<Self> {
        while let Some(ref ext) = split_on_pat(&mut header, '=') {
            match ext {
                "Domain" => cookie.domain(split_on_pat(&mut header, "; ")),
                "Expires" => cookie.expires(split_on_pat(&mut header, "; ")),
                "HttpOnly" => cookie.http_only(true),
                "Max-Age" => cookie.max_age(split_on_pat(&mut header, "; ")),
                "Partitioned" => cookie.partitioned(true),
                "Path" => cookie.max_age(split_on_pat(&mut header, "; ")),
                "SameSite" => cookie.max_age(split_on_pat(&mut header, "; ")),
            }
        }

        Ok(self)
    }
}

impl ToHeader for Cookie {
    type Output = (&str, String);

    fn to_header(&self, header: &str) -> Self::Output {
        (header, self.to_string())
    }
}

impl ToString for Cookie {
    fn to_string(&self) -> String {
        let mut cookie = format!("{}={}", self.key, self.val);
        let mut temp;
        if let Some(ma) = self.max_age {
            let ma = ma.num_seconds();
            temp = format!("; Max-Age={} ", ma);
            cookie.push_str(&temp);
        }

        if let Some(exp) = self.expires {
            temp = format!("; Expires={} ", exp);
            cookie.push_str(&temp)
        }

        if self.http_only {
            cookie.push_str("; HttpOnly");
        }

        if self.secure {
            cookie.push_str("; Secure");

            if self.partitioned {
                cookie.push_str("; Partitioned");
            }
        }

        if let Some(path) = &self.path {
            temp = format!("; Path={}", path);
            cookie.push_str(&temp);
        }

        if let Some(domain) = &self.domain {
            temp = format!("; Domain={}", domain);
            cookie.push_str(&temp);
        }

        if let Some(ss) = &self.same_site {
            temp = format!("; SameSite={:?}", ss);
            cookie.push_str(&temp);
        }

        (header, cookie)
    }
}

// NOTE the same domain with a different scheme is considered a different domain
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, Hash)]
pub enum SameSite {
    // only send this cookie on requests made to the same site that defined it
    Strict = 1,
    // same as strict but also includes cross site requests that
    // - are top level navigation requests; i.e., they move pages
    // - use a safe http method (dont set data in the server)
    Lax = 2,
    // the cookie is send with both same site and cross site requests
    // requires the secure attr
    #[default]
    None = 0,
}

impl TryFrom<u8> for SameSite {
    type Error = ();

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Self::None),
            1 => Ok(Self::Strict),
            2 => Ok(Self::Lax),
            _ => Err(()),
        }
    }
}
