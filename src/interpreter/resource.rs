use serde::de::{Deserialize, Deserializer, Error, Visitor};
use serde::ser::{Serialize, SerializeTupleStruct, Serializer};
use std::collections::{HashMap, HashSet};

use super::TransmuteError;
use super::route::Route;
use crate::{Query, Url};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    route: Route,
    query: Option<Query>,
}

impl Resource {
    pub fn query(&self) -> Option<&Query> {
        let Some(ref query) = self.query else {
            return None;
        };

        Some(query)
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

    /// takes route from self
    pub fn take_route(&mut self) -> Route {
        std::mem::take(&mut self.route)
    }

    /// takes query from self
    pub fn take_query(&mut self) -> Option<Query> {
        std::mem::take(&mut self.query)
    }

    pub fn sequence(&self) -> String {
        let Some(ref query) = self.query else {
            return self.route.as_str().to_owned();
        };

        let mut seq = query.sequence();
        seq.insert_str(0, self.route.as_str());

        seq
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

// serde traits
impl serde::Serialize for Resource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO
        serializer.serialize_str(&self.sequence())
    }
}

struct ResourceVisitor;

impl<'de> Visitor<'de> for ResourceVisitor {
    type Value = Resource;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("expecting str value of an absolute path url")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let url = v
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let resource = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(resource)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let url = v
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let resource = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(resource)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let url = v
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let resource = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(resource)
    }

    fn visit_bytes<E>(self, b: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(b).map_err(|_| E::custom("invalid bytes"))?;

        let url = s
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let resource = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(resource)
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(v).map_err(|_| E::custom("invalid bytes"))?;

        let url = s
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let resource = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(resource)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(&v).map_err(|_| E::custom("invalid bytes"))?;

        let url = s
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let resource = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(resource)
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
        deserializer.deserialize_option(ResourceVisitor)
    }
}

impl<'de> serde::Deserialize<'de> for Resource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Resource", &["route", "query"], ResourceVisitor)
    }
}
