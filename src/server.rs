use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::pin::Pin;
use std::string::FromUtf8Error;

use serde::Serialize;

use super::{HttpMethod, PheasantError, Request, RequestBody, RequestParams};

pub fn into_bytes<S: Serialize>(s: S) -> Vec<u8> {
    if let Ok(mut res) = serde_json::to_string(&s) {
        res.remove(0);
        res.pop();
        res = res.replacen("\\\"", "\"", res.len());
        res = res.replacen("\\n", "", res.len());

        return res.into();
    } else {
        vec![]
    }
}

enum ServiceDivergence {
    Params(Box<dyn Fn(RequestParams) -> Vec<u8>>),
    Body(Box<dyn Fn(RequestBody) -> Vec<u8>>),
    ParamsAndBody(Box<dyn Fn(RequestParams, RequestBody) -> Vec<u8>>),
    None(Box<dyn Fn(RequestParams) -> Vec<u8>>),
}

pub struct Service {
    method: HttpMethod,
    uri: String,
    mime: String,
    callback: BoxFun,
}

unsafe impl Send for Service {}
unsafe impl Sync for Service {}
// the future return type
type BoxFut<'a> = Pin<Box<dyn Future<Output = Vec<u8>> + Send + 'a>>;

// the wrapper function type
type BoxFun = Box<dyn Fn(Request) -> BoxFut<'static> + Send + Sync>;

impl Service {
    pub fn new<F, O, R>(method: HttpMethod, uri: &str, mime: &str, call: F) -> Self
    where
        F: Fn(R) -> O + Send + Sync + 'static,
        O: Future<Output = Vec<u8>> + Send + 'static,
        R: From<Request>,
    {
        Self {
            method,
            uri: uri.into(),
            mime: mime.into(),
            callback: Box::new(move |req: Request| {
                let input: R = req.into();

                Box::pin(call(input))
            }),
        }
    }
}

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
    pub fn service(&mut self, service: Service) {
        self.services.push(service);
    }
}

impl Server {
    fn match_service(&self, method: HttpMethod, uri: &str) -> Option<&Service> {
        self.services
            .iter()
            .find(move |s| s.method == method && s.uri == uri)
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
        let data = read_stream(&mut stream).await?;
        println!("{{\n{}\n}}", data);
        let req = Request::parse_from(data)?;
        println!("{:#?}", req);

        // println!("method: {:?}, uri: {}", req.method(), req.uri());
        let service = self
            .match_service(req.method(), req.uri())
            .unwrap_or(&self.services[0]);

        let payload = (service.callback)(req).await;
        let response = format_response(payload, &service.mime);
        // println!("{}", String::from_utf8_lossy(&response));

        stream.write_all(&response)?;
        // println!("wrote to client; {:?}", stream.take_error());
        stream.flush()?;
        // println!("flushed buffer; {:?}", stream.take_error());

        Ok(stream)
    }
}

// BUG this will not read request body if any
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

impl From<FromUtf8Error> for PheasantError {
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
