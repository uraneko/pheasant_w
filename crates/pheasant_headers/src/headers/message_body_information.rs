extern crate alloc;
use alloc::{borrow::ToOwned, format, string::String};
use chrono::TimeDelta;
use core::fmt::{self, Debug, Display, Formatter};
use hashbrown::{HashMap, HashSet};
use mime::Mime;

use pheasant_core::{Header, HeaderMap, Method, Response, WildCardish};
use pheasant_uri::{Origin, OriginSet};

use crate::{HttpResult, ToHeader, ToHeaders};

pub struct SetContentLength<'a>(&'a [u8]);

impl<'a> SetContentLength<'a> {
    fn new(slice: &'a [u8]) -> Self {
        Self(slice)
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl ToHeader for SetContentLength {
    type Output = (&str, String);

    fn to_header(&self, _header: &str) -> Self::Output {
        ("Content-Length", self.len().to_string())
    }
}

pub struct ContentLength(usize);

impl FromHeader for ContentLength {
    fn from_header(header: String) -> HttpResult<Self> {
        match header.parse::<usize>().unwrap() {
            size if size < 8192 => Ok(Self(size)),
            // TODO validation should be done later since i have no way of knowing
            // server content size limits at this point
            _ => Err(ErrorStatus::Client(ClientError::ContentTooLarge)),
        }
    }
}

pub struct ContentType(Mime);

impl ContentType {
    fn new(mime: &str) -> Self {
        Self(mime.into())
    }

    fn mime(&self) -> &Mime {
        &self.0
    }
}

impl ToHeader for ContentType {
    // const NAME: &str = "Content-Type";
    //
    type Output = [&str; 2];

    fn to_header(&self, header: &str) -> Self::Output {
        (header, self.mime().essence_str())
    }
}

impl FromHeader for ContentType {
    fn from_header(header: String) -> HttpResult<Self> {
        header.parse::<Mime>().unwrap()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub enum Encoding {
    Deflate,
    GZip,
    Zlib,
}

impl FromStr for Encoding {
    type Err = ErrorStatus;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "deflate" => Ok(Self::Deflate),
            "gzip" => Ok(Self::Gzip),
            "zlib" => Ok(Self::Zlib),
            "br" | "zstd" | "dcb" | "dcz" => Err(ErrorStatus::Server(ServerError::Unimplemented)),
            // may be a bad/non-existent algorithm name
            // or
            // an algorithm that this lib doesnt know about
            _ => Err(ErrorStatus::Client(ClientError::UnprocessableContent)),
        }
    }
}

impl Encoding {
    fn to_u8(&self) -> u8 {
        match self {
            Self::Deflate => 1,
            Self::Gzip => 2,
            Self::Zlib => 4,
        }
    }
}

impl Encoding {
    fn encode(&self, slice: &[u8]) -> Vec<u8> {
        match self {
            Self::Deflate => deflate::deflate_bytes(slice),
            Self::Gzip => deflate::deflate_bytes_gzip(slice),
            Self::Zlib => deflate::deflate_bytes_zlib(slice),
        }
    }
}

impl Responses {
    fn encode(&mut self, encoder: Encoding) -> (&mut Self, u8) {
        *self.body = encoder.encode(&self.body);

        (self, encoder.to_u8())
    }
}

trait EncodeBody {
    fn encode(self, encoder: Encoding) -> Self;

    fn content_encoding(self);
}

pub struct ContentEncodingBits(u8);

impl ContentEncodingBits {
    fn encoding_list(self) -> &str {
        match self.0 {
            0 => "",
            1 => "deflate",
            2 => "gzip",
            3 => "deflate, gzip",
            4 => "zlib",
            5 => "deflate, zlib",
            6 => "gzip, zlib",
            7 => "deflate, gzip, zlib",
            _ => unimplemented!("reached unimplemented encodings"),
        }
    }
}

impl ToHeader for ContentEncodingBits {
    type Output = [&str; 2];

    fn to_header(&self, _header: &str) -> Self::Output {
        ["Content-Encoding", self.encoding_list()]
    }
}

impl<'a> EncodeBody for (&'a mut Responses, u8) {
    fn encode(self, encoder: Encoding) -> Self {
        *self.0.body = encoder.encode(&self.0.body);

        (self.0, self.1 | encoder.to_u8())
    }

    fn content_encoding(self) {
        let bits = ContentEncodingBits(self.1);

        self.0.content_encoding(bits.encoding_list());
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct ContentEncoding {
    encodings: Vec<Encoding>,
}

// impl FromIterator<Encoding> for ContentEncoding {
//     fn from_iter<T>(iter: T) -> Self
//     where
//         T: IntoIterator<Item = Encoding>,
//     {
//         Self {
//             encodings: iter.collect(),
//         }
//     }
// }

impl FromHeader for ContentEncoding {
    fn from_header(header: String) -> HttpResult<Self> {
        Ok(Self {
            encodings: header
                .split(',')
                .map(|algo| algo.parse::<Encoding>()?)
                .collect(),
        })
    }
}

// not gonna use
//
// impl Resposne {
//     fn encode_body(&mut self, encoder: Encoding) -> ContentEncoding {
//         ContentEncoding::new(
//             || {
//                 *self.body = encoder.encode(&self.body);
//
//                 &mut self.body
//             },
//             encoder.to_u8(),
//         )
//     }
// }
//
// pub struct ContentEncoding<I> {
//     inner: I,
//     encodings: u8,
// }
//
// impl<I> ContentEncoding<I> {
//     fn new(inner: I, encodings: u8) -> Self {
//         Self { inner, encodings }
//     }
//
//     fn encode(self, encoder: Encoding) -> Self {
//         ContentEncoding::new(
//             || {
//                 let bytes: &mut Vec<u8> = self.inner();
//                 *bytes = encoder.encode(&bytes);
//
//                 bytes
//             },
//             self.encodings | encoder.to_u8(),
//         )
//     }
//
//     fn content_encoding(self) {
//         self.inner()
//     }
// }
//
// impl ToHeader for ContentEncoding {
//     type Output = [&str; 2];
//
//     fn to_header(&self, h: &str) -> Self::Output {}
// }
//
//

// TODO
pub struct ContentLanguage {}

// TODO
pub struct ContentLocation {}
