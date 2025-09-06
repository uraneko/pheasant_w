// this module is TODO
extern crate alloc;
use alloc::{borrow::ToOwned, format, string::String};
use chrono::{DateTime, Utc};
use core::fmt::{self, Debug, Display, Formatter};
use hashbrown::{HashMap, HashSet};
use mime::Mime;

use pheasant_core::{Header, HeaderMap, Method, Response, WildCardish};
use pheasant_uri::Origin;

use crate::{HttpResult, ToHeader, ToHeaders};

pub struct RequestCacheControl {}

pub struct ResponseCacheControl {}

pub enum ReqDirective {
    MaxAge(i64),
    MaxStale(i64),
    MinFresh(i64),
    NoCache,
    NoStore,
    NoTransform,
    OnlyIfCached,
    StaleIfError,
}

pub enum RespDirective {
    MaxAge(i64),
    SMaxage(i64),
    NoCache,
    NoStore,
    NoTransform,
    MustRevalidate,
    ProxyRevalidate,
    MustUnderstand,
    Private,
    Public,
    Immutable,
    StaleWhileRevalidate,
    StaleIfError,
}
