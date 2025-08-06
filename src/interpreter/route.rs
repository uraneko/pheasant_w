use serde::de::{Deserialize, Deserializer, Error, Visitor};
use serde::ser::{Serialize, SerializeTupleStruct, Serializer};
use std::collections::{HashMap, HashSet};

use super::TransmuteError;
use crate::{Query, Scheme, Url};

/// uri route type,
/// e.g., "/index.html"
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Route(String);

impl std::ops::Deref for Route {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Route {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Route {
    pub fn new(segments: Vec<String>) -> Self {
        Self({
            let mut p = segments.join("/");
            p.insert(0, '/');

            p
        })
    }

    pub fn segments(&self) -> std::str::Split<'_, char> {
        (*self).split('/')
    }

    pub fn is_root(&self) -> bool {
        (*self).as_str() == "/"
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn points_to(&self, route: &str) -> bool {
        self.as_str() == route
    }
}

impl TryFrom<Url> for Route {
    type Error = TransmuteError;

    fn try_from(mut url: Url) -> Result<Self, Self::Error> {
        let Some(path) = url.take_path() else {
            return Err(TransmuteError::RoutePathNotFound);
        };

        Ok(Self::new(path))
    }
}

// serde traits
impl serde::Serialize for Route {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO
        serializer.serialize_str(self.as_str())
    }
}

struct RouteVisitor;

impl<'de> Visitor<'de> for RouteVisitor {
    type Value = Route;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("expected str value of a url route")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let url = v
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let route = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(route)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let url = v
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let route = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(route)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let url = v
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let route = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(route)
    }

    fn visit_bytes<E>(self, b: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(b).map_err(|_| E::custom("invalid bytes"))?;

        let url = s
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let route = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(route)
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(v).map_err(|_| E::custom("invalid bytes"))?;

        let url = s
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let route = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(route)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(&v).map_err(|_| E::custom("invalid bytes"))?;

        let url = s
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let route = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(route)
    }

    // this means that none values would deserialize to mime default
    // fn visit_none<E>(self) -> Result<Self::Value, E>
    // where
    //     E: Error,
    // {
    //     Ok(Mime::default())
    // }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(RouteVisitor)
    }
}

impl<'de> serde::Deserialize<'de> for Route {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Route", &["path"], RouteVisitor)
    }
}
