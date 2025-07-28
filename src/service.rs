use std::pin::Pin;

use super::{IntoRoutes, Method, Request, Route};

use mime::Mime;

/// a http server service type
/// contains the logic that gets executed when a request is made
pub struct Service {
    method: Method,
    route: Route,
    redirects: Vec<Route>,
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
    pub fn new<F, I, O, R>(method: Method, route: &str, redirects: I, mime: &str, call: F) -> Self
    where
        F: Fn(R) -> O + Send + Sync + 'static,
        O: Future<Output = Vec<u8>> + Send + 'static,
        R: From<Request>,
        I: IntoRoutes,
    {
        Self {
            method,
            route: route.into(),
            redirects: redirects.into_routes(),
            mime: mime.parse().ok(),
            service: Box::new(move |req: Request| {
                let input: R = req.into();

                Box::pin(call(input))
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

    // returns a reference to the String value of the service route
    pub(crate) fn route(&self) -> &str {
        &self.route.0
    }

    // returns a ref to the Mime type if it was provided
    //
    // otherwise returns None
    pub(crate) fn mime(&self) -> Option<&Mime> {
        self.mime.as_ref()
    }

    // checks if the passed route &str value redirects to this service
    pub(crate) fn redirects_to(&self, route: &str) -> bool {
        self.redirects
            .iter()
            .find(|r| r.as_str() == route)
            .is_some()
    }
}
