use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::str::FromStr;
use std::string::FromUtf8Error;

use serde::Serialize;

use super::{HttpMethod, RequestParams, ServerError, requests::Request};

// const DEF_ADDR_PORT: &str = "127.0.0.1:8883";

pub struct Service<'a> {
    method: HttpMethod,
    uri: String,
    mime: String,
    callback: Box<dyn Fn(Option<RequestParams>) -> Vec<u8> + 'a>,
}

unsafe impl Send for Service<'_> {}
unsafe impl Sync for Service<'_> {}

impl<'a> Service<'a> {
    pub fn new<F, O, P>(method: HttpMethod, uri: &str, mime: &str, f: F) -> Self
    where
        F: Fn(P) -> O + 'a,
        P: From<RequestParams>,
        O: Serialize,
    {
        Self {
            method,
            uri: uri.into(),
            mime: mime.into(),
            callback: Box::new(move |p: Option<RequestParams>| -> Vec<u8> {
                let p = match p {
                    Some(p) => p,
                    None => RequestParams::default(),
                };

                let res = f(p.into());
                serde_json::to_string(&res)
                    .map(|mut res| {
                        res.remove(0);
                        res.pop();
                        res = res.replacen("\\\"", "\"", res.len());
                        res = res.replacen("\\n", "", res.len());

                        res.into()
                    })
                    .unwrap()
            }),
        }
    }
}

pub struct Server<'a> {
    /// the server tcp listener socket
    socket: TcpListener,
    /// container for the server services
    services: Vec<Service<'a>>,
}

impl<'a> Server<'a> {
    /// creates a new server
    /// ```
    /// let (addr, port) = ([127.0.0.1], 8883);
    /// let max = 90000;
    /// let server = Server::new(max, addr, port)
    /// ```
    pub fn new(addr: impl Into<Ipv4Addr>, port: u16, max: usize) -> Result<Self, ServerError> {
        Ok(Self {
            socket: {
                let addr = addr.into();
                println!(
                    "\x1b[1;38;2;213;183;214mServer bound at http://{}:{}\x1b[0m",
                    addr, port
                );

                TcpListener::bind((addr, port))?
            },
            services: vec![Service::new(
                HttpMethod::Get,
                "/not_found404.html",
                "text/html",
                not_found404,
            )],
        })
    }

    /// pushes a new service to the server
    pub fn service(&mut self, service: Service<'a>) {
        self.services.push(service);
    }
}

impl<'a> Server<'a> {
    fn match_service(&self, method: HttpMethod, uri: &str) -> Option<&Service> {
        self.services
            .iter()
            .find(move |s| s.method == method && s.uri == uri)
    }

    pub fn serve(&'a self) {
        for stream in self.socket.incoming().flatten() {
            if let Err(e) = self.handle_stream(stream) {
                // TODO log the error or something
                println!("{:?}", e);
            }
        }
    }

    fn handle_stream(&'a self, mut stream: TcpStream) -> Result<(), ServerError> {
        let data = read_stream(&mut stream)?;
        println!("{{\n{}\n}}", data);
        let mut req = Request::parse_from(data)?;
        println!("{:#?}", req);

        // println!("method: {:?}, uri: {}", req.method(), req.uri());
        let service = self
            .match_service(req.method(), req.uri())
            .unwrap_or(&self.services[0]);
        let param = req.take_params();
        let payload = (service.callback)(param);
        let response = format_response(payload, &service.mime);
        // println!("{}", String::from_utf8_lossy(&response));

        stream.write_all(&response)?;
        // println!("wrote to client; {:?}", stream.take_error());
        stream.flush()?;
        // println!("flushed buffer; {:?}", stream.take_error());

        Ok(())
    }
}

// BUG this will not read request body if any
fn read_stream(s: &mut TcpStream) -> Result<String, ServerError> {
    let mut data = Vec::new();
    let mut reader = BufReader::new(s);
    let mut buf = [0; 1024];
    loop {
        let Ok(n) = reader.read(&mut buf) else {
            return Err(ServerError::StreamReadCrached);
        };
        if n < 1024 {
            break data.extend(&buf[..n]);
        } else if n > 1024 {
            return Err(ServerError::StreamReadWithExcess);
        }
        data.extend(buf);
    }

    String::from_utf8(data).map_err(|e| e.into())
}

impl From<FromUtf8Error> for ServerError {
    fn from(_err: FromUtf8Error) -> Self {
        Self::BytesParsingFailed
    }
}

// TODO response builder
fn format_response(payload: Vec<u8>, ct: &str) -> Vec<u8> {
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

impl From<RequestParams> for () {
    fn from(_p: RequestParams) -> () {
        ()
    }
}

fn not_found404(_: ()) -> String {
    let svg = std::fs::read_to_string("assets/404.svg").unwrap();
    format!(
        "{}",
        std::fs::read_to_string("templates/404.html")
            .unwrap()
            .replace("{404.svg}", &svg)
    )
}
