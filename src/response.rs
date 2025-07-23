use std::collections::HashMap;

use chrono::offset::Utc;
use mime::Mime;
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::{
    ClientError, PassingStatus, PheasantError, Protocol, Request, ResponseStatus, Route, Server,
    ServerError, Service, Successful,
};

const SERVER: &str = "Pheasant-DevServer / 0.1.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    proto: Protocol,
    body: Option<Vec<u8>>,
    headers: HashMap<String, String>,
    status: PassingStatus,
}

impl Serialize for Response {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Response", 3)?;
        s.serialize_field("body", &self.body)?;
        s.serialize_field("status", &self.status)?;
        s.end()
    }
}

impl Response {
    pub async fn new(mut req: Request, server: &Server) -> Result<Self, PheasantError> {
        // if None then 404 not found
        let service = server.borrow_service(req.method(), req.route());
        if service.is_none() {
            return Err(PheasantError::ClientError(ClientError::NotFound));
        }
        let service = service.unwrap();

        let proto = req.proto();
        let mime = mime(&req, &service);
        let mut headers = req.headers();
        headers.clear();

        let body = (service.service())(req).await;
        let body = deflate::deflate_bytes(&body);
        let body = deflate::deflate_bytes_gzip(&body);
        let len = body.len();

        headers.insert("Content-Encoding".into(), "deflate, gzip".into());
        headers.insert("Content-Type".into(), mime.to_string());
        headers.insert("Content-Length".into(), format!("{}", len));
        headers.insert("Date".into(), Utc::now().to_string());
        headers.insert("Server".into(), SERVER.into());

        Ok(Self {
            headers,
            proto,
            body: if body.is_empty() { None } else { Some(body) },
            status: PassingStatus::Successful(Successful::OK),
        })
    }

    pub fn respond(self) -> Vec<u8> {
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

fn mime(req: &Request, service: &Service) -> Mime {
    let fallback = service.mime().unwrap_or(&mime::TEXT_HTML);

    req.header::<Mime>("Content-Type")
        .unwrap_or(fallback.clone())
}

fn is_alerady_compressed(mime: &Mime) -> bool {
    todo!()
}
