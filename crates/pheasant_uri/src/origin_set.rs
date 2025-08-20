use serde::de::{Deserialize, Deserializer, Error, Visitor};
use serde::ser::{Serialize, SerializeTupleStruct, Serializer};
use std::collections::HashSet;
use std::str::FromStr;

use super::{Origin, TransmuteError};
use crate::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OriginSet {
    WhiteList(HashSet<Origin>),
    AnyOrigin,
}

impl Default for OriginSet {
    fn default() -> Self {
        Self::WhiteList(HashSet::new())
    }
}

impl FromStr for OriginSet {
    type Err = TransmuteError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "*" {
            return Ok(Self::AnyOrigin);
        }
        let origin = s.parse::<Origin>().unwrap();

        Ok(Self::WhiteList(HashSet::from([origin])))
    }
}

impl OriginSet {
    pub fn macro_checked(set: HashSet<Origin>) -> Self {
        Self::WhiteList(set)
    }

    pub fn is_white_list(&self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(&Self::default())
    }
    pub fn is_any_origin(&self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(&Self::AnyOrigin)
    }

    pub fn contains(&self, origin: &Origin) -> bool {
        match self {
            Self::WhiteList(wl) => wl.contains(origin),
            Self::AnyOrigin => true,
        }
    }

    pub fn origins_ref(&self) -> Option<&HashSet<Origin>> {
        let Self::WhiteList(wl) = self else {
            return None;
        };

        Some(wl)
    }

    pub fn origins_mut(&mut self) -> Option<&mut HashSet<Origin>> {
        let Self::WhiteList(wl) = self else {
            return None;
        };

        Some(wl)
    }
}
