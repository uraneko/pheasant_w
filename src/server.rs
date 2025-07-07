use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::str::FromStr;
use std::thread::JoinHandle;

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
    // container for active workers that are currently occupied with handling some clients
    workers: Vec<WorkerHandle>,
    // container of idle workers that are not curretly handling any stream
    idle: Vec<WorkerHandle>,
    /// max number of allowed workers both idle + active
    max: usize,
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
            workers: Vec::with_capacity(max),
            idle: Vec::with_capacity(max),
            max,
            services: vec![],
        })
    }

    pub fn worker(&mut self) {
        let services = &self.services;
        self.serve();
    }

    /// pushes a new service to the server
    pub fn service(&mut self, service: Service<'a>) {
        self.services.push(service);
    }
}

type WorkerHandle = JoinHandle<Result<(), ServerError>>;

impl<'a> Server<'a> {
    fn try_listen(&self, services: &[Service]) {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(600));
            if let Ok((stream, _)) = self.socket.accept() {
                let data = read_stream(&stream);
                println!("{}", data);
            }
        }
    }

    fn serve(&self) {
        self.socket
            .incoming()
            .filter(|stream| stream.is_ok())
            .map(|s| s.unwrap())
            .map(|stream| ( read_stream(&stream), stream))
            .inspect(|(d, s )| println!("{}", d))
            .for_each(|(d, mut stream)| {
                let req = Request::parse(&d).unwrap();
                println!("{:#?}", req);
                self.services
                    .iter()
                    .filter(|s| s.method == req.method() && s.uri == req.uri())
                    .for_each(|s| {
                        let param = req.params().unwrap().get("who").unwrap().to_string();
                        let body = (s.callback)(param);
                        println!("{:?}", body);
                        let mut writer = BufWriter::new(&mut stream);
                        let headers = format!(
                            "HTTP/1.1 200 OK\nAccess-Control-Allow-Origin: *\nServer: pheasant\nContent-Length: {}\nContent-Type: text/html; charset=utf-8\r\n\r\n{}",
                            body.len(), String::from_utf8_lossy(&body)
                        );
                        println!("{}", headers);
                        writer.write(&headers.into_bytes()).unwrap();
                        writer.flush().unwrap();
                    })
            });
    }
}

fn read_stream(s: &TcpStream) -> String {
    // s.set_read_timeout(Some(std::time::Duration::from_secs(4)))
    //     .unwrap();

    let mut reader = BufReader::new(s);
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).unwrap();

    String::from_utf8(buf).unwrap()
}
