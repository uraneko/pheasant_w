use pheasant::{HttpMethod, RequestParams, Server, Service};

#[tokio::main]
async fn main() {
    let mut phe = Server::new([127, 0, 0, 1], 8883, 3333).unwrap();
    phe.service(Service::new(HttpMethod::Get, "/hello", "text/html", hello));
    phe.service(Service::new(
        HttpMethod::Get,
        "/favicon.ico",
        "image/svg+xml",
        favicon,
    ));
    phe.service(Service::new(HttpMethod::Get, "/icon", "image/svg+xml", svg));
    phe.serve();
}

struct Who {
    who: String,
}

impl From<RequestParams> for Who {
    fn from(mut p: RequestParams) -> Self {
        Self {
            who: p.remove("who").unwrap(),
        }
    }
}

fn hello(p: RequestParams) -> String {
    let who: Who = p.into();

    format!("<h1>hello {}</h1>", who.who)
}

fn favicon(_: ()) -> String {
    std::fs::read_to_string("assets/404.svg").unwrap()
}

fn svg(p: RequestParams) -> String {
    let who: Who = p.into();

    std::fs::read_to_string(who.who).unwrap()
}
