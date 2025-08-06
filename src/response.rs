use std::collections::{HashMap, HashSet};

use chrono::{DateTime, offset::Utc};
// use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::{
    ClientError, Cookie, Cors, Fail, Header, HeaderMap, Mime, PheasantError, PheasantResult,
    Protocol, Redirection, Request, ResponseStatus, ServerError, Service, Status, Successful,
};

const SERVER: &str = "Pheasant (dev/0.1.0)";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum StatusState {
    Status(Status),
    #[default]
    Pending,
}

impl StatusState {
    fn code(&self) -> Option<u16> {
        let Self::Status(s) = self else {
            return None;
        };

        Some(s.code())
    }

    fn text(&self) -> Option<&str> {
        let Self::Status(s) = self else {
            return None;
        };

        Some(s.text())
    }
}

/// Http Response type
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Response {
    proto: Protocol,
    body: Option<Vec<u8>>,
    headers: HashMap<String, String>,
    status: StatusState,
    cookies: HashSet<Cookie>,
}

// impl Serialize for Response {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut s = serializer.serialize_struct("Response", 3)?;
//         s.serialize_field("body", &self.body)?;
//         s.serialize_field("status", &self.status)?;
//         s.end()
//     }
// }

// struct Template;
// struct Payload;
// Response<Template>;
// Response<Payload>;

impl Response {
    /// generates a new Response template from a protocol value
    pub fn template(proto: Protocol) -> Self {
        Self {
            proto,
            ..Default::default()
        }
    }

    // TONOTDO just like redirects/not implemented
    // make options requests builtin
    // RE actually, that would be undesirable
    // different resources could have different access levels/components
    // there is a reason an options request is only cached for 5 seconds by default

    /// returns a filled response ready for consumption
    ///
    /// dont use this directly
    // NOTE &Service contains the function that returns the Response template
    pub async fn payload(req: Request, status: Status, service: &Service) -> Self {
        let mime = mime(&req, service);

        let mut resp = (service.service())(req).await;
        let mime = if resp.has_header::<Mime>("Content-Type") {
            None
        } else {
            Some(mime)
        };
        resp.update_status(status, mime, service.route());

        resp
    }

    pub fn with_status(code: u16) -> Self {
        Self {
            // TODO handle error
            status: StatusState::Status(code.try_into().unwrap()),
            ..Default::default()
        }
    }

    /// generates a response from a client/server error
    pub async fn from_err(
        fail: Option<&Fail>,
        proto: Option<Protocol>,
    ) -> Result<Self, PheasantError> {
        let Some(fail) = fail else {
            return Err(PheasantError::ServerError(ServerError::NotImplemented));
        };

        let mut resp = (fail.fail())().await;
        resp.update_mime(fail.mime());
        resp.update_status(fail.code().try_into().unwrap(), None, "");
        resp.update_proto(proto.unwrap_or(Protocol::default()));

        Ok(resp)
    }

    /// impl of server error not implemented
    /// this http error response is the fallback of all http error responses
    /// when the user hasnt defined a service(fail) for a needed error response
    /// a response gets generated from this error
    pub async fn not_implemented() -> Self {
        Self {
            status: StatusState::Status(Status::ServerError(ServerError::NotImplemented)),
            body: Some(b"{ error: 'NotImplemented', code: 501 }".to_vec()),
            ..Default::default()
        }
    }

    // pub async fn preflight() -> Self {
    //     Self {
    //         status: StatusState::Status(Status::Successful(Successful::NoContent)),
    //         body: None,
    //         cors: {
    //             let mut cors = Cors::new();
    //             cors.methods()
    //                 .extend(&[Method::Get, Method::Options, Method::Head, Method::Post]);
    //             cors.headers().insert("*".into());
    //             cors.origin("*");
    //
    //             Some(cors)
    //         },
    //         ..Default::default()
    //     }
    // }

    /// format the response into bytes to be sent to the client
    pub fn respond(self) -> Vec<u8> {
        println!("{:#?}", self);
        // serde_json::to_string(&self).unwrap().into_bytes()
        let mut payload = format!(
            "{} {} {}\n",
            self.proto,
            self.status.code().unwrap(),
            self.status.text().unwrap(),
        );
        let mut iter = self.headers.into_iter();
        while let Some((ref h, ref v)) = iter.next() {
            // NOTE this
            // ```
            // if h.starts_with("Set-Cookie") {
            //     h.trim_end_matches(char::is_numeric)
            // } else {
            //     h
            // }
            // ```
            // should be cheaper than storing the
            // cookies in a vec (would require another allocation)
            // RE: seems like http2 allows many Cookie headers
            // a vec would just be a better representative of Cookie/Set-Cookie headers
            payload.push_str(h);
            payload.push_str(": ");
            payload.push_str(v);
            payload.push('\n');
        }

        let mut iter = self.cookies.into_iter();
        while let Some(cookie) = iter.next() {
            payload.push_str("Set-Cookie: ");
            payload.push_str(&cookie.to_string());
            payload.push('\n');
        }

        payload.push('\n');
        let mut payload = payload.into_bytes();
        if let Some(body) = self.body {
            payload.extend(body);
        }

        payload
    }
}

impl Response {
    pub fn update_status(&mut self, status: Status, mime: Option<Mime>, route: &str) -> &mut Self {
        match status {
            Status::Informational(i) => (),
            Status::Successful(s) => self.successful(mime),
            Status::Redirection(r) => self.redirection(route),
            _ => (),
        }

        self.status = StatusState::Status(status);

        self
    }

    pub fn update_body(&mut self, data: Vec<u8>) -> &mut Self {
        self.body = Some(data);

        self
    }

    // updates mime type if it exists
    // returns bool indicating wether the update operation took place or not
    pub fn update_mime(&mut self, mime: Option<&Mime>) -> &mut Self {
        let Some(mime) = mime else { return self };
        // TODO fix header traits
        self.set_header::<Mime>("Content-Type", mime.clone());

        self
    }

    pub fn update_proto(&mut self, proto: Protocol) -> &mut Self {
        self.proto = proto;

        self
    }

    pub fn set_cookie(&mut self, cookie: Cookie) -> &mut Self {
        self.cookies.insert(cookie);

        self
    }

    pub fn set_cookies<CI>(&mut self, cookies: CI) -> &mut Self
    where
        CI: Iterator<Item = Cookie>,
    {
        self.cookies.extend(cookies);

        self
    }

    // works only if origin is amongst the cors.origins field values
    // origin comes from the request headers
    // cors comes from the corresponding service
    pub fn set_cors(&mut self, cors: &Cors, origin: &str) -> &mut Self {
        if !cors.allows_origin(origin) {
            return self;
        }

        self.set_header("Access-Control-Allow-Origin", origin.to_owned());
        cors.set_headers(self);

        self
    }
}

impl Response {
    fn successful(&mut self, mime: Option<Mime>) {
        if let Some(ref mut body) = self.body {
            *body = deflate::deflate_bytes(&body);
            *body = deflate::deflate_bytes_gzip(&body);
            let len = body.len();

            self.set_header::<String>("Content-Encoding".into(), "deflate, gzip".into())
                .set_header("Content-Length".into(), len);
            if let Some(mime) = mime {
                self.set_header("Content-Type", mime);
            }
            self.set_header("Date".into(), Utc::now())
                .set_header::<String>("Server".into(), SERVER.into());
        }
    }

    // this doesnt need to be async
    // there is no system IO going on here
    fn redirection(&mut self, route: &str) {
        self.set_header::<String>("Location".into(), route.into())
            .set_header("Content-Length".into(), 0usize);
    }
}

// TODO handle OPTIONS request

impl HeaderMap for Response {
    fn header<H: Header>(&self, key: &str) -> Option<H> {
        self.headers.header(key)
    }

    fn set_header<H: Header>(&mut self, key: &str, h: H) -> &mut Self {
        self.headers.set_header(key, h);

        self
    }
}

// resolve the mime type of the response content
// if none is found then falls back to text html
// TODO also consider the `Accept` header
fn mime(req: &Request, service: &Service) -> Mime {
    req.header::<Mime>("Content-Type")
        .unwrap_or_else(|| service.clone_mime().unwrap_or_default())
}

// check if data is already compressed
// could be the case for some archives or some image types
fn is_already_compressed(mime: &Mime) -> bool {
    todo!()
}

/// umbrella fn for client error headers generation fns
#[deprecated]
async fn client_error(status: &ClientError) -> (HashMap<String, String>, Vec<u8>) {
    match status {
        ClientError::BadRequest => bad_request().await,
        ClientError::NotFound => not_found().await,
        _ => unimplemented!("the rest of the client error statuses are not yet implemented"),
    }
}

#[deprecated]
const BAD_REQUEST: &[u8] = include_bytes!("../templates/400.html");

/// 400 client error header generation
#[deprecated]
async fn bad_request() -> (HashMap<String, String>, Vec<u8>) {
    // let body = b"{\n\t'error': 'Bad Request'\n\t'message': 'Some redundant response body'\n}";
    let body = BAD_REQUEST.to_vec();
    let len = body.len();

    (
        HashMap::from([
            ("Content-Type".into(), "text/html".into()),
            ("Content-Length".into(), format!("{}", len)),
        ]),
        body,
    )
}

#[deprecated]
const NOT_FOUND: &[u8] = include_bytes!("../templates/404.html");

/// 404 not found header generation
#[deprecated]
async fn not_found() -> (HashMap<String, String>, Vec<u8>) {
    let body = NOT_FOUND.to_vec();
    let len = body.len();

    (
        HashMap::from([
            ("Content-Type".into(), "text/html".into()),
            ("Content-Length".into(), format!("{}", len)),
            ("Server".into(), SERVER.into()),
            ("Date".into(), Header::to_string(&Utc::now())),
        ]),
        body,
    )
}

/// wrapper fn for server errors headers generation fns
#[deprecated]
async fn server_error(status: &ServerError) -> (HashMap<String, String>, Vec<u8>) {
    match status {
        ServerError::HTTPVersionNotSupported => http_version_not_supported().await,
        _ => unimplemented!("not yet implemeted status code; {}", file!()),
    }
}

#[deprecated]
const VERSION_NOT_SUPPORTED: &[u8] = include_bytes!("../templates/505.html");

/// server error 505 response headers generation
#[deprecated]
async fn http_version_not_supported() -> (HashMap<String, String>, Vec<u8>) {
    let body = VERSION_NOT_SUPPORTED.to_vec();
    let len = body.len();

    (
        HashMap::from([
            ("Content-Type".into(), "text/html".into()),
            ("Content-Length".into(), format!("{}", len)),
        ]),
        body,
    )
}

impl Header for DateTime<Utc> {}
