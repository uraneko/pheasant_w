use chrono::{DateTime, Utc};
use pheasant::{
    Cookie, HeaderMap, Method, Mime, Protocol, Request, Response, Server, Service, fail, get,
};

#[tokio::main]
async fn main() {
    let mut phe = Server::new([127, 0, 0, 1], 8883, 3333).unwrap();
    phe.service(hello).service(favicon).error(not_found);
    // .service(|| Service::new(Method::Get, "/icon", [], "image/svg+xml", svg));

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

#[get("/icon")]
#[mime("image/svg+xml")]
async fn svg(who: Who, p: Protocol) -> Response {
    let mut resp = Response::with_proto(p);
    resp.set_header("Content-Type", "image/svg+xml".parse::<Mime>().unwrap());

    let mut cookie = Cookie::new("test1", "this test cookie should disappear in 1 mins");
    cookie
        .http_only(true)
        .same_site(0)
        .max_age(chrono::TimeDelta::minutes(1));

    resp.update_body(std::fs::read_to_string(who.name).unwrap().into_bytes())
        .set_cookie(cookie);

    resp
}

const NOT_FOUND: &[u8] = include_bytes!("../templates/505.html");

#[fail(404)]
#[mime("text/html")]
async fn not_found() -> Response {
    let mut resp = Response::default();
    let body = NOT_FOUND.to_vec();
    let len = body.len();
    resp.update_body(body);

    resp.set_header("Content-Type", "text/html".parse::<mime::Mime>().unwrap())
        .set_header("Content-Length", len)
        .set_header::<String>("Server", "Phe (devmode)".into())
        .set_header("Date", Utc::now().to_string());

    resp
}

// TODO #[status(200)] macro attr
// server wide preflight req resp
// #[options("*")]
// async fn server_options(_: ()) -> Response {
//     let mut resp = Response::default();
//
//     resp
// }
