use pheasant_core::{Method, Request, Server, Service};

#[tokio::main]
async fn main() {
    let mut phe = Server::new([127, 0, 0, 1], 8883, 3333).unwrap();
    phe.service(Service::new(Method::Get, "/hello", "text/html", hello));
    phe.service(Service::new(
        Method::Get,
        "/favicon.ico",
        "image/svg+xml",
        favicon,
    ));
    phe.service(Service::new(Method::Get, "/icon", "image/svg+xml", svg));
    phe.serve().await;
}

struct Who {
    who: String,
}

impl From<Request> for Who {
    fn from(mut req: Request) -> Self {
        Self {
            who: req.parse_query_param("who").unwrap().into(),
        }
    }
}

async fn hello(who: Who) -> Vec<u8> {
    format!("<h1>hello {}</h1>", who.who).into_bytes()
}

async fn favicon(_: ()) -> Vec<u8> {
    std::fs::read_to_string("assets/404.svg")
        .unwrap()
        .into_bytes()
}

// #[get("/icon")]
async fn svg(who: Who) -> Vec<u8> {
    std::fs::read_to_string(who.who).unwrap().into_bytes()
}
