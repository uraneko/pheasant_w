use std::collections::{HashMap, HashSet};

use chrono::{DateTime, offset::Utc};
use pheasant_uri::{Origin, Resource};

use crate::{
    ClientError, Cookie, Cors, ErrorStatus, Failure, Header, HeaderMap, Mime, PheasantError,
    PheasantResult, Protocol, Redirection, Request, ResponseStatus, ServerError, Service, Status,
    Successful,
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

impl Response {
    /// generates a new Response template from a protocol value
    // NOTE this should be called `with_proto`
    pub fn with_proto(proto: Protocol) -> Self {
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
    // NOTE any data that is not stored in the Response type has to be set for the response at this
    // point
    // otherwise, data that is stored in the Response type can be injected into the response
    // bytes inside the Response.respond method
    pub async fn payload(req: Request, status: Status, service: &Service) -> Self {
        let mime = mime(&req, service);

        let mut resp = (service.service())(&req).await;
        resp.set_cors(&req, service);
        let mime = if resp.has_header::<Mime>("Content-Type") {
            None
        } else {
            Some(mime)
        };

        let mut resource = req.query().map(|q| q.sequence()).unwrap_or_default();
        resource.insert_str(0, service.route());
        resp.update_status(status, mime, Some(resource));

        resp
    }

    pub fn with_status(code: u16) -> Self {
        Self {
            // TODO handle error
            status: StatusState::Status(code.try_into().unwrap()),
            ..Default::default()
        }
    }

    pub fn preflight(cors: &Cors, origin: Option<&Origin>) -> Self {
        Self {
            headers: cors.to_headers(origin),
            ..Default::default()
        }
    }

    pub fn failing(status: ErrorStatus) -> Self {
        let mut resp = Self::default();
        resp.update_status(status.into(), None, None);

        resp
    }

    /// generates a response from a client/server error
    pub async fn from_err(
        fail: Option<&Failure>,
        proto: Option<Protocol>,
    ) -> Result<Self, PheasantError> {
        let Some(fail) = fail else {
            return Err(PheasantError::ServerError(ServerError::NotImplemented));
        };

        let mut resp = (fail.fail())().await;
        resp.update_mime(fail.mime());
        resp.update_status(fail.code().try_into().unwrap(), None, None);
        resp.update_proto(proto.unwrap_or_default());

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
        println!("{:?}", self);
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

// BUG cors headers were set despite there being no cors attribute in the Service definition
// the cross origin request still failed with client error Origin not set tho
// not sure if they really were set at all, and when were they removed then

impl Response {
    pub fn update_status(
        &mut self,
        status: Status,
        mime: Option<Mime>,
        resource: Option<String>,
    ) -> &mut Self {
        match status {
            Status::Informational(i) => (),
            Status::Successful(s) => self.successful(mime),
            Status::Redirection(r) => self.redirection(resource.unwrap()),
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

    pub fn extend_cookies<CI>(&mut self, cookies: CI) -> &mut Self
    where
        CI: IntoIterator<Item = Cookie>,
    {
        self.cookies.extend(cookies.into_iter());

        self
    }

    // origin comes from the request headers
    // cors comes from the corresponding service
    pub fn set_cors(&mut self, req: &Request, service: &Service) -> &mut Self {
        if let Some(cors) = service.cors()
            && let Some(origin) = req.header::<Origin>("Origin")
        {
            let origin = cors.allows_origin(&origin).then(|| &origin);
            self.headers.extend(cors.to_headers(origin));
        }

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

    fn redirection(&mut self, resource: String) {
        self.set_header::<String>("Location".into(), resource)
            .set_header("Content-Length".into(), 0usize);
    }
}

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

crate::impl_hdfs!(DateTime<Utc>, Origin);
