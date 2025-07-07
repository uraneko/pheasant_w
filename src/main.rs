use pheasant::{HttpMethod, Server, Service};

#[tokio::main]
async fn main() {
    let mut phe = Server::default();
    phe.service(Service::new(HttpMethod::Get, "/hello", hello));
    phe.worker();
}

fn hello(who: String) -> String {
    format!("<h1>hello {}</h1>", who)
}
