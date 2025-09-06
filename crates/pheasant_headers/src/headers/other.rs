extern crate alloc;
use alloc::{borrow::ToOwned, format, string::String};
use chrono::{DateTime, Utc};
use core::fmt::{self, Debug, Display, Formatter};
use hashbrown::{HashMap, HashSet};
use mime::Mime;

use pheasant_core::{Header, HeaderMap, Method, Response, WildCardish};
use pheasant_uri::{Origin, OriginSet};

use crate::{HttpResult, ToHeader, ToHeaders};

pub struct Date(DateTime<Utc>);

impl FromHeader for Date {
    fn from_header(header: String) -> HttpResult<Self> {
        Ok(Self(header.parse::<DateTime<Utc>>().unwrap()))
    }
}

pub struct SetDate;

impl ToHeader for SetDate {
    type Output = (&str, String);

    fn to_header(&self, _header: &str) -> Self::Output {
        ("Date", Utc::now().to_string())
    }
}
