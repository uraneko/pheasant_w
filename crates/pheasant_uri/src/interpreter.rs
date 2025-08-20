use std::collections::{HashMap, HashSet};

use crate::{Query, Scheme, Url};

pub mod origin;
pub mod resource;
pub mod route;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransmuteError {
    RoutePathNotFound,
    NotAValidRoute,
    NotAValidOrigin,
    OriginSchemeNotFound,
    OriginDomainNotFound,
}
