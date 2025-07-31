use pheasant_core::{HeaderMap, Method, Protocol, Request, Response, Server, Service, get};

#[tokio::main]
async fn main() {
    let mut phe = Server::new([127, 0, 0, 1], 8883, 3333).unwrap();
    phe.service(hello);
    phe.service(favicon);
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
