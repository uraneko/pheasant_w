use std::collections::HashMap;

use chrono::offset::Utc;
use mime::Mime;
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::{
    ClientError, Header, HeaderMap, PheasantError, PheasantResult, Protocol, Redirection, Request,
    ResponseStatus, Server, ServerError, Service, Status, Successful,
};

const SERVER: &str = "Pheasant (dev/0.1.0)";

/// Http Response type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    proto: Protocol,
    body: Option<Vec<u8>>,
    headers: HashMap<String, String>,
    status: Status,
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

impl Response {
    // Generates a new response instance
    pub(crate) async fn new(req: PheasantResult<Request>, server: &Server) -> Self {
        if let Err(err) = req {
            return Self::from_err(err).await;
        }

        let req = req.unwrap();
        let proto = req.proto();

        let ss = server.service_status(req.method(), req.route());
        if let Err(err) = ss {
            return Self::from_err(err).await;
        }
        let (status, service) = ss.unwrap();

        let (headers, body) = match status {
            Status::Successful(Successful::OK) => {
                let (h, b) = successful(req, service).await;

                (h, Some(b))
            }
            Status::Redirection(Redirection::SeeOther) => {
                let h = redirection(req, service).await;

                (h, None)
            }
            _ => unimplemented!("other status codes are not yet implemented; {}", file!()),
        };

        Self {
            headers,
            proto,
            body,
            status,
        }
    }

    /// generates a response from a client/server error
    async fn from_err(err: PheasantError) -> Self {
        let (headers, body) = match err {
            PheasantError::ServerError(se) => server_error(&se).await,
            PheasantError::ClientError(ce) => client_error(&ce).await,
        };
        let body = Some(body);

        return Self {
            proto: Protocol::HTTP1_1,
            status: Status::from(err),
            headers,
            body,
        };
    }

    /// format the response into bytes to be sent to the client
    pub(crate) fn respond(self) -> Vec<u8> {
        println!("{:#?}", self);
        // serde_json::to_string(&self).unwrap().into_bytes()
        let mut payload = format!(
            "{} {} {}\n",
            self.proto,
            self.status.code(),
            self.status.text(),
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

fn mime(req: &Request, service: &Service) -> Mime {
    let fallback = service.mime().unwrap_or(&mime::TEXT_HTML);

    req.header::<Mime>("Content-Type")
        .unwrap_or(fallback.clone())
}

fn is_already_compressed(mime: &Mime) -> bool {
    todo!()
}

async fn successful(mut req: Request, service: &Service) -> (HashMap<String, String>, Vec<u8>) {
    let mime = mime(&req, &service);

    let mut headers = req.headers();
    headers.clear();

    let body = (service.service())(req).await;
    let body = deflate::deflate_bytes(&body);
    let body = deflate::deflate_bytes_gzip(&body);
    let len = body.len();

    // these are success headers
    headers.insert("Content-Encoding".into(), "deflate, gzip".into());
    headers.insert("Content-Type".into(), mime.to_string());
    headers.insert("Content-Length".into(), format!("{}", len));
    headers.insert("Date".into(), Utc::now().to_string());
    // add server.name/version field
    headers.insert("Server".into(), SERVER.into());

    (headers, body)
}

async fn redirection(mut req: Request, service: &Service) -> HashMap<String, String> {
    let mime = mime(&req, &service);
    let route = service.route().into();

    let mut headers = req.headers();
    headers.clear();

    headers.insert("Location".into(), route);
    headers.insert("Content-Type".into(), mime.to_string());
    headers.insert("Content-Length".into(), String::from("0"));

    headers
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
