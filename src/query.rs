use serde::de::{Deserialize, Deserializer, Error, Visitor};
use serde::ser::{Serialize, SerializeTupleStruct, Serializer};
use std::collections::{HashMap, HashSet};

use crate::{ParseError, ParseResult};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    params: HashMap<String, String>,
    attrs: HashSet<String>,
}

impl Query {
    fn insert_param(&mut self, k: &str, v: &str) {
        self.params.insert(k.to_owned(), v.to_owned());
    }

    fn insert_attr(&mut self, a: &str) {
        self.attrs.insert(a.to_owned());
    }

    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    pub fn attrs(&self) -> &HashSet<String> {
        &self.attrs
    }
}

impl Query {
    pub fn from_colls(map: HashMap<&str, &str>, set: HashSet<&str>) -> Self {
        Query {
            params: map.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
            attrs: set.into_iter().map(|a| a.into()).collect(),
        }
    }

    // returns the str repr of this query
    pub fn sequence(&self) -> String {
        let mut seq = self
            .params
            .iter()
            .fold("".to_owned(), |acc, (k, v)| acc + k + "=" + v + "&");
        seq = self.attrs.iter().fold(seq, |acc, a| acc + a + "&");
        seq.insert(0, '?');

        seq
    }
}

impl std::str::FromStr for Query {
    type Err = ParseError;

    fn from_str(s: &str) -> ParseResult<Self> {
        let mut query = Query::default();
        str_to_pairs(&mut query, s);

        Ok(query)
    }
}

// parses the query params into key -> value pairs
fn str_to_pairs(query: &mut Query, s: &str) {
    s.split('&')
        // BUG this crashes the server when uri query is badly formatted
        // TODO scan query after getting request and return ClientError::BadRequest if query is faulty
        .map(|e| str_to_pair(e))
        .for_each(|[k, v]| {
            if v.is_empty() {
                query.insert_attr(k);
            } else {
                query.insert_param(k, v);
            }
        });
}

// NOTE this handles the pain points of parse_query
// the check for `=` garentees the operation's success
fn str_to_pair(p: &str) -> [&str; 2] {
    if p.contains('=') {
        p.splitn(2, '=').collect::<Vec<&str>>().try_into().unwrap()
    } else {
        // TODO possibly make a HashSet of bool params alongside the HashMap of k -> v pairs
        [p, ""]
    }
}

// serde traits
impl serde::Serialize for Query {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO
        serializer.serialize_str(&self.sequence())
    }
}

struct QueryVisitor;

impl<'de> Visitor<'de> for QueryVisitor {
    type Value = Query;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("expected str value of a url query")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let query = v
            .parse::<Query>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(query)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let query = v
            .parse::<Query>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(query)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let query = v
            .parse::<Query>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(query)
    }

    fn visit_bytes<E>(self, b: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(b).map_err(|_| E::custom("invalid bytes"))?;

        let query = s
            .parse::<Query>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(query)
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(v).map_err(|_| E::custom("invalid bytes"))?;

        let query = s
            .parse::<Query>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(query)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(&v).map_err(|_| E::custom("invalid bytes"))?;

        let query = s
            .parse::<Query>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(query)
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
        deserializer.deserialize_option(QueryVisitor)
    }
}

impl<'de> serde::Deserialize<'de> for Query {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Query", &["params", "attrs"], QueryVisitor)
    }
}
