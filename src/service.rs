use std::collections::HashSet;
use std::pin::Pin;

use crate::{Cors, IntoRoutes, Method, Mime, Protocol, Request, Response};
use pheasant_uri::Route;

// TODO maybe make new type: ResponseTemplate and make that the Service.service return type

/// a http server service type
/// contains the logic that gets executed when a request is made
pub struct Service {
    method: Method,
    route: Route,
    redirects: Option<HashSet<Route>>,
    mime: Option<Mime>,
    service: BoxFun,
    cors: Option<Cors>,
}

unsafe impl Send for Service {}
unsafe impl Sync for Service {}

// the future return type
type BoxFut<'a> = Pin<Box<dyn Future<Output = Response> + Send + 'a>>;

// the wrapper function type
type BoxFun = Box<dyn Fn(&Request) -> BoxFut<'static> + Send + Sync>;

impl Service {
    /// creates a new Service instance
    /// you would only use this function directly if you're not using the http method macros
    ///
    /// ```
    /// let mut phe = Server::new([127, 0, 0, 1], 8883, 3333).unwrap();
    /// phe.service(|| Service::new(Method::Get, "/icon", [], "image/svg+xml", svg));
    ///
    /// async fn svg(who: Who) -> Vec<u8> {
    ///     std::fs::read_to_string(who.name).unwrap().into_bytes()
    /// }
    /// ```
    ///
    /// The macro equivalent of the above code would be
    ///
    /// ```
    /// let mut phe = Server::new([127, 0, 0, 1], 8883, 3333).unwrap();
    /// phe.service(svg);
    ///
    /// #[get("/icon")]
    /// #[mime("image/svg+xml")]
    /// async fn svg(file: StaticFile) -> Vec<u8> {
    ///     std::fs::read_to_string(file.path).unwrap().into_bytes()
    /// }
    /// ```
    ///
    pub fn new<F, O, R>(
        method: Method,
        // TODO convert str to route at the macro level before getting here
        route: Route,
        redirects: Option<HashSet<Route>>,
        mime: Option<Mime>,
        cors: Option<Cors>,
        call: F,
    ) -> Self
    where
        F: Fn(R, Protocol) -> O + Send + Sync + 'static,
        O: Future<Output = Response> + Send + 'static,
        R: for<'a> From<&'a Request>,
    {
        Self {
            method,
            route,
            mime,
            cors,
            redirects,
            service: Box::new(move |req: &Request| {
                let proto = req.proto();

                let input: R = req.into();

                Box::pin(call(input, proto))
            }),
        }
    }

    // returns a ref to the service logic callback
    pub(crate) fn service(&self) -> &BoxFun {
        &self.service
    }

    // returns a copy of the service Method
    pub(crate) fn method(&self) -> Method {
        self.method
    }

    /// returns a reference to the String value of the service route
    pub fn route(&self) -> &str {
        &self.route
    }

    // returns a ref to the Mime type if it was provided
    //
    // otherwise returns None
    pub(crate) fn clone_mime(&self) -> Option<Mime> {
        self.mime.clone()
    }

    // checks if the passed route &str value redirects to this service
    pub(crate) fn redirects_to(&self, route: &str) -> bool {
        let Some(ref re) = self.redirects else {
            return false;
        };
        re.iter().find(|r| r.as_str() == route).is_some()
    }

    pub(crate) fn cors(&self) -> Option<&Cors> {
        self.cors.as_ref()
    }

    /// checks if this service can handle cross origin requests
    pub fn allows_cross_origin_requests(&self) -> bool {
        self.cors.is_some()
    }
}
