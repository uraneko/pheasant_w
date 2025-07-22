use std::pin::Pin;

use super::{Method, Request};

use mime::Mime;
use url::Url;

use crate::server::join_path;

pub struct ResponseBuilder {
    method: Method,
    uri: Url,
    mime: Mime,
    service: BoxFun,
}

pub struct Response {}

pub struct Service {
    method: Method,
    uri: Url,
    mime: Option<Mime>,
    service: BoxFun,
}

unsafe impl Send for Service {}
unsafe impl Sync for Service {}
// the future return type
type BoxFut<'a> = Pin<Box<dyn Future<Output = Vec<u8>> + Send + 'a>>;

// the wrapper function type
type BoxFun = Box<dyn Fn(Request) -> BoxFut<'static> + Send + Sync>;

impl Service {
    pub fn new<F, O, R>(method: Method, uri: &str, mime: &str, call: F) -> Self
    where
        F: Fn(R) -> O + Send + Sync + 'static,
        O: Future<Output = Vec<u8>> + Send + 'static,
        R: From<Request>,
    {
        Self {
            method,
            uri: join_path(uri),
            mime: mime.parse().ok(),
            service: Box::new(move |req: Request| {
                let input: R = req.into();

                Box::pin(call(input))
            }),
        }
    }

    pub fn service(&self) -> &BoxFun {
        &self.service
    }

    pub fn method(&self) -> Method {
        self.method
    }

    pub fn uri(&self) -> &Url {
        &self.uri
    }

    pub fn mime(&self) -> Option<&Mime> {
        self.mime.as_ref()
    }
}
