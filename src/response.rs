use std::collections::HashMap;

use chrono::offset::Utc;
use mime::Mime;
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::{
    ClientError, Fail, Header, HeaderMap, PheasantError, PheasantResult, Protocol, Redirection,
    Request, ResponseStatus, Server, ServerError, Service, Status, Successful,
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
            headers: HashMap::new(),
            body: None,
            status: StatusState::default(),
        }
    }

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
            status: StatusState::Status(code.into()),
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
        resp.update_status(fail.code().into(), None, "");
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
            body: Some(b"{ error: 'NotImplemented', code: 501 }".into()),
            // TODO fill these headers
            headers: HashMap::from([]),
            proto: Protocol::HTTP1_1,
        }
    }

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
            payload.push_str(h);
            payload.push_str(": ");
            payload.push_str(v);
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
    pub fn update_status(&mut self, status: Status, mime: Option<Mime>, route: &str) {
        match status {
            Status::Informational(i) => (),
            Status::Successful(s) => self.successful(mime),
            Status::Redirection(r) => self.redirection(route),
            _ => (),
        }

        self.status = StatusState::Status(status);
    }

    pub fn update_body(&mut self, data: Vec<u8>) {
        self.body = Some(data);
    }

    // updates mime type if it exists
    // returns bool indicating wether the update operation took place or not
    pub fn update_mime(&mut self, mime: Option<&Mime>) -> bool {
        let Some(mime) = mime else { return false };
        // TODO fix header traits
        self.set_header::<Mime>("Content-Type", mime.clone());

        true
    }

    pub fn update_proto(&mut self, proto: Protocol) {
        self.proto = proto;
    }
}

impl Response {
    fn successful(&mut self, mime: Option<Mime>) {
        if let Some(ref mut body) = self.body {
            *body = deflate::deflate_bytes(&body);
            *body = deflate::deflate_bytes_gzip(&body);
            let len = body.len();

            self.set_header::<String>("Content-Encoding".into(), "deflate, gzip".into());
            self.set_header("Content-Length".into(), len);
            if let Some(mime) = mime {
                self.set_header("Content-Type", mime);
            }
            self.set_header("Date".into(), Utc::now());
            self.set_header::<String>("Server".into(), SERVER.into());
        }
    }

    // this doesnt need to be async
    // there is no system IO going on here
    fn redirection(&mut self, route: &str) {
        self.set_header::<String>("Location".into(), route.into());
        self.set_header("Content-Length".into(), 0);
    }
}

// TODO handle OPTIONS request

impl HeaderMap for Response {
    fn header<H: Header>(&self, key: &str) -> Option<H>
    where
        <H as std::str::FromStr>::Err: std::fmt::Debug,
    {
        self.headers.header(key)
    }

    fn set_header<H: Header>(&mut self, key: &str, h: H) -> Option<String> {
        self.headers.set_header(key, h)
    }
}

// resolve the mime type of the response content
// if none is found then falls back to text html
// TODO also consider the `Accept` header
fn mime(req: &Request, service: &Service) -> Mime {
    let fallback = service.mime().unwrap_or(&mime::TEXT_HTML);

    req.header::<Mime>("Content-Type")
        .unwrap_or(fallback.clone())
}

// check if data is already compressed
// could be the case for some archives or some image types
fn is_already_compressed(mime: &Mime) -> bool {
    todo!()
}

/// umbrella fn for client error headers generation fns
async fn client_error(status: &ClientError) -> (HashMap<String, String>, Vec<u8>) {
    match status {
        ClientError::BadRequest => bad_request().await,
        ClientError::NotFound => not_found().await,
        _ => unimplemented!("the rest of the client error statuses are not yet implemented"),
    }
}

const BAD_REQUEST: &[u8] = include_bytes!("../templates/400.html");

/// 400 client error header generation
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

const NOT_FOUND: &[u8] = include_bytes!("../templates/404.html");

/// 404 not found header generation
async fn not_found() -> (HashMap<String, String>, Vec<u8>) {
    let body = NOT_FOUND.to_vec();
    let len = body.len();

    (
        HashMap::from([
            ("Content-Type".into(), "text/html".into()),
            ("Content-Length".into(), format!("{}", len)),
            ("Server".into(), SERVER.into()),
            ("Date".into(), Utc::now().to_string()),
        ]),
        body,
    )
}

/// wrapper fn for server errors headers generation fns
async fn server_error(status: &ServerError) -> (HashMap<String, String>, Vec<u8>) {
    match status {
        ServerError::HTTPVersionNotSupported => http_version_not_supported().await,
        _ => unimplemented!("not yet implemeted status code; {}", file!()),
    }
}

const VERSION_NOT_SUPPORTED: &[u8] = include_bytes!("../templates/505.html");

/// server error 505 response headers generation
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

impl Header for chrono::DateTime<Utc> {}
impl Header for String {}
