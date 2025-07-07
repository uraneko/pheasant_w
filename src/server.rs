use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::str::FromStr;

use super::{HttpMethod, ServerError, requests::Request};

// const DEF_ADDR_PORT: &str = "127.0.0.1:8883";

fn service<'a, I, O, T>(f: T) -> Box<dyn Fn(String) -> Vec<u8> + 'a>
where
    T: Fn(I) -> O + 'static,
    I: FromStr,
    <I as FromStr>::Err: std::fmt::Debug,
    O: Into<Vec<u8>>,
{
    Box::new(move |i: String| -> Vec<u8> { f(i.parse::<I>().unwrap()).into() })
}

pub struct Service<'a> {
    method: HttpMethod,
    uri: &'a str,
    callback: Box<dyn Fn(String) -> Vec<u8> + 'a>,
}

unsafe impl Send for Service<'_> {}
unsafe impl Sync for Service<'_> {}

impl<'a> Service<'a> {
    pub fn new<F, I, O>(method: HttpMethod, uri: &'a str, callback: F) -> Self
    where
        F: Fn(I) -> O + 'static,
        I: FromStr,
        <I as FromStr>::Err: std::fmt::Debug,
        O: Into<Vec<u8>>,
    {
        Self {
            method,
            uri,
            callback: service(callback),
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
            socket: TcpListener::bind((addr.into(), port))?,
            services: vec![],
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
            .find(|s| s.method == method && s.uri == uri)
    }

    pub fn serve(&self) {
        for stream in self.socket.incoming().flatten() {
            self.handle_stream(stream);
        }
    }

    fn handle_stream(&self, mut stream: TcpStream) {
        let data = read_stream(&mut stream);
        let req = Request::parse(&data).unwrap();
        println!("{:#?}", req);

        let service = self.match_service(req.method(), req.uri()).unwrap();
        let param = req.params().unwrap().get("who").unwrap().to_string();
        let payload = (service.callback)(param);
        let response = format_response(payload, "text/html; charset=utf-8");
        println!("{}", String::from_utf8_lossy(&response));

        stream.write_all(&response).unwrap();
        println!("wrote to client; {:?}", stream.take_error());

        stream.flush().unwrap();
        println!("flushed buffer; {:?}", stream.take_error());
    }
}

fn read_stream(s: &mut TcpStream) -> String {
    let mut buf = Vec::new();
    let mut reader = BufReader::new(s);
    while buf.len() < 4 || buf[buf.len() - 4..] != [13, 10, 13, 10] {
        reader.read_until(10, &mut buf).unwrap();
        println!("{:?}", buf);
    }

    String::from_utf8(buf).unwrap()
}

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
