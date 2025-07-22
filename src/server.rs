use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};

use mime::Mime;
use serde::Serialize;
use url::Url;

use super::{Method, PheasantError, Request, Service};

pub fn base_url() -> Url {
    Url::parse("http://127.0.0.1:8883").unwrap()
}

pub fn join_path(path: &str) -> Url {
    if path.starts_with("http://127.0.0.1:8883") {
        return Url::parse(path).unwrap();
    }

    base_url().join(path).unwrap()
}

// pub fn into_bytes<S: Serialize>(s: S) -> Vec<u8> {
//     if let Ok(mut res) = serde_json::to_string(&s) {
//         res.remove(0);
//         res.pop();
//         res = res.replacen("\\\"", "\"", res.len());
//         res = res.replacen("\\n", "", res.len());
//
//         return res.into();
//     } else {
//         vec![]
//     }
// }

pub struct Server {
    /// the server tcp listener socket
    socket: TcpListener,
    /// container for the server services
    services: Vec<Service>,
}

impl Server {
    /// creates a new server
    /// ```
    /// let (addr, port) = ([127.0.0.1], 8883);
    /// let max = 90000;
    /// let server = Server::new(max, addr, port)
    /// ```
    pub fn new(addr: impl Into<Ipv4Addr>, port: u16, max: usize) -> Result<Self, PheasantError> {
        Ok(Self {
            socket: {
                let addr = addr.into();
                println!(
                    "\x1b[1;38;2;213;183;214mServer bound at http://{}:{}\x1b[0m",
                    addr, port
                );

                // `impl From<io::Error> for PheasantError` is for this
                TcpListener::bind((addr, port))?
            },
            services: vec![Service::new(
                Method::Get,
                "/not_found404.html",
                "text/html",
                not_found404,
            )],
        })
    }

    /// pushes a new service to the server
    pub fn service(&mut self, service: Service) {
        self.services.push(service);
    }
}

impl Server {
    fn match_service(&self, method: Method, uri: &str) -> Option<&Service> {
        self.services
            .iter()
            .find(move |s| s.method() == method && s.uri() == uri)
    }

    pub async fn serve(&mut self) {
        for stream in self.socket.incoming().flatten() {
            if let Err(e) = self.handle_stream(stream).await {
                // TODO log the error or something
                println!("{:?}", e);
            }
        }
    }

    async fn handle_stream(&self, mut stream: TcpStream) -> Result<TcpStream, PheasantError> {
        let req = Request::from_stream(&mut stream)?;
        println!("{:#?}", req);

        // println!("method: {:?}, uri: {}", req.method(), req.uri());
        let service = self
            .match_service(req.method(), req.uri())
            .unwrap_or(&self.services[0]);

        let payload = (service.service())(req).await;
        let response = format_response(
            payload,
            service.mime().unwrap_or(&mime::APPLICATION_OCTET_STREAM),
        );
        // println!("{}", String::from_utf8_lossy(&response));

        stream.write_all(&response)?;
        // println!("wrote to client; {:?}", stream.take_error());
        stream.flush()?;
        // println!("flushed buffer; {:?}", stream.take_error());

        Ok(stream)
    }
}

async fn read_stream(s: &mut TcpStream) -> Result<String, PheasantError> {
    let mut data = Vec::new();
    let mut reader = BufReader::new(s);
    let mut buf = [0; 1024];
    loop {
        let Ok(n) = reader.read(&mut buf) else {
            return Err(PheasantError::StreamReadCrached);
        };
        if n < 1024 {
            break data.extend(&buf[..n]);
        } else if n > 1024 {
            return Err(PheasantError::StreamReadWithExcess);
        }
        data.extend(buf);
    }

    String::from_utf8(data).map_err(|e| e.into())
}

fn format_response(payload: Vec<u8>, ct: &Mime) -> Vec<u8> {
    let cl = payload.len();
    let mut res: Vec<u8> = format!(
        "HTTP/1.1 200 OK\r\nAccept-Range: bytes\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        ct, cl
    )
    .into_bytes();
    res.extend(payload);
    res.extend([13, 10]);

    res
}

impl From<Request> for () {
    fn from(_p: Request) -> () {
        ()
    }
}

const NOT_FOUND_SVG: &str = include_str!("../assets/404.svg");
const NOT_FOUND_HTML: &str = include_str!("../templates/404.html");

async fn not_found404(_: ()) -> Vec<u8> {
    NOT_FOUND_HTML
        .replace("{404.svg}", NOT_FOUND_SVG)
        .into_bytes()
}
