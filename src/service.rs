use std::pin::Pin;

use super::{Method, Request, Route};

use mime::Mime;

pub struct Service {
    method: Method,
    route: Route,
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
    // wrapper: W
    // W: Fn() -> (Method, Url, Option<Mime>, F)
    pub fn new<F, O, R>(method: Method, route: &str, mime: &str, call: F) -> Self
    where
        F: Fn(R) -> O + Send + Sync + 'static,
        O: Future<Output = Vec<u8>> + Send + 'static,
        R: From<Request>,
    {
        Self {
            method,
            route: route.into(),
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

    pub fn route(&self) -> &str {
        &self.route.0
    }

    pub fn mime(&self) -> Option<&Mime> {
        self.mime.as_ref()
    }
}
