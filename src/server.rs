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
    // socket: TcpListener,
    // workers: Vec<ThreadHandle>,
    threads: Vec<ThreadHandle>,
    addr: Ipv4Addr,
    port: u16,
    max_allowed_threads: usize,
    services: Vec<Service<'a>>,
}

impl<'a> Default for Server<'a> {
    fn default() -> Self {
        Self {
            threads: Vec::with_capacity(10_000),
            max_allowed_threads: 10000,
            addr: [127, 0, 0, 1].into(),
            port: 8883,
            services: vec![],
        }
    }
}

impl<'a> Server<'a> {
    fn new(max: usize, init_capa: usize, addr: [u8; 4], port: u16) -> Result<Self, ServerError> {
        if init_capa > max {
            return Err(ServerError::InitialThreadCapacityHigherThanMaximumThreadsAllowed);
        }

        Ok(Self {
            threads: Vec::with_capacity(init_capa),
            max_allowed_threads: max,
            addr: addr.into(),
            port,
            services: vec![],
        })
    }

    pub fn worker(&mut self) {
        let (addr, port) = (self.addr, self.port);
        let services = &self.services;
        let worker = ServiceWorker::new(addr, port).unwrap();
        worker.listen(services);
    }

    pub fn service(&mut self, service: Service<'a>) {
        self.services.push(service);
    }
}

type ThreadHandle = JoinHandle<Result<(), ServerError>>;

#[derive(Debug)]
pub struct ServiceWorker {
    socket: TcpListener,
}

// impl Default for ServiceWorker {
//     fn default() -> Self {
//         Self {
//             socket: TcpListener::bind(DEF_ADDR_PORT).unwrap(),
//         }
//     }
// }

impl ServiceWorker {
    fn new(addr: Ipv4Addr, port: u16) -> Result<Self, ServerError> {
        TcpListener::bind((addr, port))
            .map(|socket| Self { socket })
            .map_err(|e| ServerError::IO(e))
    }

    fn try_listen(&self, services: &[Service]) {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(600));
            if let Ok((stream, _)) = self.socket.accept() {
                let data = read_stream(&stream);
                println!("{}", data);
            }
        }
    }

    fn listen(&self, services: &[Service]) {
        self.socket
            .incoming()
            .filter(|stream| stream.is_ok())
            .map(|s| s.unwrap())
            .map(|stream| (stream.try_clone().unwrap(), read_stream(&stream)))
            .inspect(|(s, d)| println!("{}", d))
            .for_each(|(mut stream, d)| {
                let req = Request::parse(&d).unwrap();
                println!("{:#?}", req);
                services
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
