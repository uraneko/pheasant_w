use chrono::{DateTime, Utc};
use pheasant_core::{
    HeaderMap, Method, Protocol, Request, Response, Server, Service, error_code, get,
};

#[tokio::main]
async fn main() {
    let mut phe = Server::new([127, 0, 0, 1], 8883, 3333).unwrap();
    phe.service(hello);
    phe.service(favicon);
    phe.error(not_found);
    // phe.service(|| Service::new(Method::Get, "/icon", [], "image/svg+xml", svg));
    phe.serve().await;
}

struct Who {
    name: String,
}

impl From<Request> for Who {
    fn from(req: Request) -> Self {
        Self {
            name: req.param("who").unwrap().into(),
        }
    }
}

#[get("/hello")]
async fn hello(who: Who) -> Vec<u8> {
    format!("<h1>hello {}</h1>", who.name).into_bytes()
}

#[get("favicon.ico")]
#[re("bad")]
#[mime("image/svg+xml")]
async fn favicon(_: ()) -> Vec<u8> {
    std::fs::read_to_string("assets/404.svg")
        .unwrap()
        .into_bytes()
}

// #[get("/icon")]
// #[mime("image/svg+xml")]
async fn svg(who: Who, p: Protocol) -> Response {
    let mut resp = Response::template(p);
    resp.set_header(
        "Content-Type",
        "image/svg+xml".parse::<mime::Mime>().unwrap(),
    );
    resp.update_body(std::fs::read_to_string(who.name).unwrap().into_bytes());

    resp
}

const NOT_FOUND: &[u8] = include_bytes!("../templates/505.html");

#[error_code(404)]
#[mime("text/html")]
async fn not_found() -> Response {
    let mut resp = Response::default();
    let body = NOT_FOUND.to_vec();
    let len = body.len();
    resp.update_body(body);

    resp.set_header("Content-Type", "text/html".parse::<mime::Mime>().unwrap());
    resp.set_header("Content-Length", len);
    resp.set_header::<String>("Server", "Phe (devmode)".into());
    resp.set_header("Date", Utc::now().to_string());

    resp
}
