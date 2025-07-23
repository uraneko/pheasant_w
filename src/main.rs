use pheasant_core::{Method, Request, Server, Service};
use pheasant_macro_get::get;

#[tokio::main]
async fn main() {
    let mut phe = Server::new([127, 0, 0, 1], 8883, 3333).unwrap();
    phe.service(hello);
    phe.service(favicon);
    phe.service(|| Service::new(Method::Get, "/icon", "image/svg+xml", svg));
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
#[mime("text/html")]
async fn hello(who: Who) -> Vec<u8> {
    format!("<h1>hello {}</h1>", who.name).into_bytes()
}

#[get("favicon.ico")]
async fn favicon(_: ()) -> Vec<u8> {
    std::fs::read_to_string("assets/404.svg")
        .unwrap()
        .into_bytes()
}

// #[get("/icon")]
// #[mime("image/svg+xml")]
async fn svg(who: Who) -> Vec<u8> {
    std::fs::read_to_string(who.name).unwrap().into_bytes()
}
