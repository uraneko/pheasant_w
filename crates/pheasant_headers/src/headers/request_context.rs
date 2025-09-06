extern crate alloc;
use alloc::{borrow::ToOwned, format, string::String};
use chrono::{DateTime, Utc};
use core::fmt::{self, Debug, Display, Formatter};
use hashbrown::{HashMap, HashSet};
use mime::Mime;

use pheasant_core::{Header, HeaderMap, Method, Response, WildCardish};
use pheasant_uri::Origin;

use crate::{HttpResult, ToHeader, ToHeaders};

// host is origin without scheme
// TODO make the host type in uri crate
//
// NOTE this is a client only header
// WARN all http/1.1 requests MUST send a host header
// if no header or more than 1 header is found then the server may return a 400 bad req
pub struct Host(Origin);

impl FromHeader for Host {
    fn from_header(header: String) -> HttpResult<Self> {
        header.parse::<Origin>().map(|o| Self(o))
    }
}
