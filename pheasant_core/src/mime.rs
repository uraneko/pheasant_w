use std::fmt;

use serde::de::{Deserialize, Deserializer, Error, Visitor};
use serde::ser::{Serialize, SerializeTupleStruct, Serializer};

use crate::Header;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mime(mime::Mime);

impl Mime {
    // safe unwrap as long as this function is used as intended,
    // which is from the http methods macros
    pub fn macro_checked(s: &str) -> Self {
        s.parse::<Mime>().unwrap()
    }
}

impl std::ops::Deref for Mime {
    type Target = mime::Mime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Mime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl serde::Serialize for Mime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.essence_str())
    }
}

struct MimeVisitor;

impl<'de> Visitor<'de> for MimeVisitor {
    type Value = Mime;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("expecting str value of a mime")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let mime = v
            .parse::<mime::Mime>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(Mime(mime))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let mime = v
            .parse::<mime::Mime>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(Mime(mime))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let mime = v
            .parse::<mime::Mime>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(Mime(mime))
    }

    fn visit_bytes<E>(self, b: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(b).map_err(|_| E::custom("invalid bytes"))?;

        let mime = s
            .parse::<mime::Mime>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(Mime(mime))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(v).map_err(|_| E::custom("invalid bytes"))?;

        let mime = s
            .parse::<mime::Mime>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(Mime(mime))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = str::from_utf8(&v).map_err(|_| E::custom("invalid bytes"))?;

        let mime = s
            .parse::<mime::Mime>()
            .map_err(|_| E::custom("invalid str value"))?;

        Ok(Mime(mime))
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
        deserializer.deserialize_option(MimeVisitor)
    }
}

impl<'de> serde::Deserialize<'de> for Mime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple_struct("Mime", 1, MimeVisitor)
    }
}

impl From<mime::Mime> for Mime {
    fn from(m: mime::Mime) -> Self {
        Self(m)
    }
}

impl From<Mime> for mime::Mime {
    fn from(m: Mime) -> Self {
        m.0
    }
}

impl std::str::FromStr for Mime {
    type Err = mime::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse::<mime::Mime>()?))
    }
}

impl fmt::Display for Mime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.essence_str())
    }
}

impl Default for Mime {
    fn default() -> Self {
        Self(mime::APPLICATION_OCTET_STREAM)
    }
}

crate::impl_hdfs!(Mime);

#[derive(Debug)]
enum MimeError {
    MimeError,
}

impl std::error::Error for MimeError {}

impl std::fmt::Display for MimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for MimeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        match msg {
            _ => Self::MimeError,
        }
    }
}

use proc_macro2::{Delimiter, Group, Literal, Span, TokenStream as TS2, TokenTree};
use quote::{ToTokens, TokenStreamExt};
use syn::Ident;

impl ToTokens for Mime {
    fn to_tokens(&self, tokens: &mut TS2) {
        tokens.append(<&Mime as Into<TokenTree>>::into(self))
    }
}

impl From<&Mime> for TokenTree {
    fn from(m: &Mime) -> Self {
        let mut ts = TS2::new();
        let ident = Ident::new("Mime", Span::call_site());
        ts.append(ident);

        let lit = Group::new(
            Delimiter::Parenthesis,
            TokenTree::Literal(Literal::string(m.essence_str())).into(),
        );
        ts.append(lit);

        let group = Group::new(Delimiter::None, ts);
        TokenTree::from(group)
    }
}
