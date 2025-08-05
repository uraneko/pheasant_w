use std::collections::{HashMap, HashSet};

use crate::{Query, Scheme, Url};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransmuteError {
    RoutePathNotFound,
    NotAValidRoute,
    NotAValidOrigin,
    OriginSchemeNotFound,
    OriginDomainNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Route {
    path: String,
}

impl Route {
    pub fn new(segments: Vec<String>) -> Self {
        Self {
            path: segments.join("/"),
        }
    }

    pub fn segments(&self) -> std::str::Split<'_, char> {
        self.path.split('/')
    }

    pub fn is_root(&self) -> bool {
        self.as_str() == "/"
    }

    pub fn is_glob(&self) -> bool {
        self.as_str() == "*"
    }

    pub fn as_str(&self) -> &str {
        &self.path
    }

    pub fn routes_to(&self, route: &str) -> bool {
        self.as_str() == route
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Resource {
    route: Route,
    query: Option<Query>,
}

impl Resource {
    pub fn query(&self) -> Option<(&HashMap<String, String>, &HashSet<String>)> {
        let Some(ref query) = self.query else {
            return None;
        };

        Some((query.params(), query.attrs()))
    }

    pub fn contains_query(&self) -> bool {
        self.query.is_some()
    }

    pub fn params(&self) -> Option<&HashMap<String, String>> {
        let Some(ref query) = self.query else {
            return None;
        };

        Some(query.params())
    }

    pub fn attrs(&self) -> Option<&HashSet<String>> {
        let Some(ref query) = self.query else {
            return None;
        };

        Some(query.attrs())
    }

    pub fn contains_param(&self, k: &str) -> bool {
        let Some(params) = self.params() else {
            return false;
        };

        params.contains_key(k)
    }

    pub fn contains_attr(&self, k: &str) -> bool {
        let Some(attrs) = self.attrs() else {
            return false;
        };

        attrs.contains(k)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Origin {
    scheme: Scheme,
    domain: String,
    port: Option<u16>,
}

impl TryFrom<Url> for Route {
    type Error = TransmuteError;

    fn try_from(mut url: Url) -> Result<Self, Self::Error> {
        let Some(path) = url.take_path() else {
            return Err(TransmuteError::RoutePathNotFound);
        };

        Ok(Self {
            path: path.join("/"),
        })
    }
}

impl TryFrom<Url> for Resource {
    type Error = TransmuteError;

    fn try_from(mut url: Url) -> Result<Self, Self::Error> {
        let Some(path) = url.take_path() else {
            return Err(TransmuteError::RoutePathNotFound);
        };

        Ok(Self {
            route: Route::new(path),
            query: url.take_query(),
        })
    }
}

impl TryFrom<Url> for Origin {
    type Error = TransmuteError;

    fn try_from(mut url: Url) -> Result<Self, Self::Error> {
        let Some(scheme) = url.scheme() else {
            return Err(TransmuteError::OriginSchemeNotFound);
        };

        let Some(domain) = url.take_domain() else {
            return Err(TransmuteError::OriginDomainNotFound);
        };

        Ok(Self {
            scheme,
            domain: domain.join("."),
            port: url.port(),
        })
    }
}

impl Origin {
    pub fn domains(&self) -> std::str::Split<'_, char> {
        self.domain.split('/')
    }

    pub fn tld(&self) -> &str {
        self.domains().last().unwrap()
    }

    pub fn bld(&self) -> &str {
        self.domains().next().unwrap()
    }

    pub fn scheme(&self) -> Scheme {
        self.scheme
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    // pub fn scheme_default_port(&self) -> u16 {
    //     match self.scheme {
    //         Scheme::Http | Scheme::Ws => 80,
    //         Scheme::Https | Scheme::Wss => 443,
    //         Scheme::Ftp => 21,
    //         // WARN this is outright wrong
    //         Scheme::File => 20,
    //     }
    // }
}
