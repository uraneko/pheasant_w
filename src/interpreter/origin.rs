use serde::de::{Deserialize, Deserializer, Error, Visitor};
use serde::ser::{Serialize, SerializeTupleStruct, Serializer};
use std::fmt;
use std::str::FromStr;

use super::TransmuteError;
use crate::{Scheme, Url};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Origin {
    scheme: Scheme,
    domain: String,
    port: Option<u16>,
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

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.sequence())
    }
}

impl FromStr for Origin {
    type Err = TransmuteError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Url>().unwrap().interpret::<Self>()
    }
}

impl Origin {
    pub fn domains(&self) -> std::str::Split<'_, char> {
        self.domain.split('/')
    }

    /// top level domain, a common example is com
    pub fn tld(&self) -> &str {
        self.domains().last().unwrap()
    }

    /// bottom level domain, a common example is www
    pub fn bld(&self) -> &str {
        self.domains().next().unwrap()
    }

    pub fn sld(&self) -> &str {
        self.domains().rev().skip(1).next().unwrap()
    }

    pub fn scheme(&self) -> Scheme {
        self.scheme
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn count(&self) -> usize {
        self.domains().count()
    }

    pub fn as_str(&self) -> &str {
        &self.domain
    }

    pub fn sequence(&self) -> String {
        let Some(port) = self.port else {
            return format!("{}://{}", self.scheme.as_str(), self.domain);
        };

        format!("{}://{}:{}", self.scheme.as_str(), self.domain, port)
    }

    pub fn is_any_origin(&self) -> bool {
        self.domain == "*"
    }
}

// serde traits
impl serde::Serialize for Origin {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO
        serializer.serialize_str(&self.sequence())
    }
}

struct OriginVisitor;

impl<'de> Visitor<'de> for OriginVisitor {
    type Value = Origin;

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

        let origin = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(origin)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let url = v
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let origin = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(origin)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let url = v
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let origin = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(origin)
    }

    fn visit_bytes<E>(self, b: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(b).map_err(|_| E::custom("invalid bytes"))?;

        let url = s
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let origin = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(origin)
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(v).map_err(|_| E::custom("invalid bytes"))?;

        let url = s
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let origin = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(origin)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(&v).map_err(|_| E::custom("invalid bytes"))?;

        let url = s
            .parse::<Url>()
            .map_err(|_| E::custom("invalid url string"))?;

        let origin = url
            .try_into()
            .map_err(|_| E::custom("url is not a valid Route"))?;

        Ok(origin)
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
        deserializer.deserialize_option(OriginVisitor)
    }
}

impl<'de> serde::Deserialize<'de> for Origin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Origin", &["scheme", "domain", "port"], OriginVisitor)
    }
}
