use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};

use mime::Mime;
use serde::Serialize;
use url::Url;

use super::{
    ClientError, Method, PheasantError, PheasantResult, Redirection, Request, Response, Route,
    Service, Status, Successful,
};

/// the http server type
pub struct Server {
    /// the server tcp listener socket
    socket: TcpListener,
    /// container for the server services
    services: Vec<Service>,
}

impl Server {
    /// creates a new server
    ///
    /// ```
    /// let (addr, port) = ([127.0.0.1], 8883);
    /// let workers = 90000;
    /// let server = Server::new(workers, addr, port)
    /// ```
    ///
    /// ### Error
    ///
    pub fn new(addr: impl Into<Ipv4Addr>, port: u16, max: usize) -> PheasantResult<Self> {
        Ok(Self {
            socket: {
                let addr = addr.into();
                println!(
                    "\x1b[1;38;2;41;213;244mServer bound at http://{}:{}\x1b[0m",
                    addr, port
                );

                // `impl From<io::Error> for PheasantError` is for this
                // TODO when this errors out
                // we append the port number and try again
                // until we get a free port
                // TODO remove impl From<io::Error> for PheasantError
                TcpListener::bind((addr, port))?
            },
            services: vec![],
        })
    }

    /// pushes a new service to the server
    pub fn service<S>(&mut self, s: S)
    where
        S: Fn() -> Service,
    {
        self.services.push(s());
    }
}

impl Server {
    /// searches for the specified service
    /// returns `(Status, &Service)`
    ///
    /// ### Error
    /// returns an Err, a client error if the service is not found
    pub fn service_status(
        &self,
        method: Method,
        route: &str,
    ) -> PheasantResult<(Status, &Service)> {
        let service = self
            .services
            .iter()
            .find(move |s| s.method() == method && (s.route() == route || s.redirects_to(&route)));

        match service {
            Some(s) if s.route() == route => Ok((Status::Successful(Successful::OK), s)),
            Some(s) if s.redirects_to(&route) => {
                Ok((Status::Redirection(Redirection::SeeOther), s))
            }
            None => Err(PheasantError::ClientError(ClientError::NotFound)),
            Some(_) => unreachable!("filtered out arm not reachable"),
        }
    }

    /// launch the service
    /// listening for incoming tcp streams
    /// and handling them
    pub async fn serve(&mut self) {
        for stream in self.socket.incoming().flatten() {
            if let Err(e) = self.handle_stream(stream).await {
                // TODO log the error or something
                println!("{:?}", e);
            }
        }
    }

    // handles a tcp stream connection
    async fn handle_stream(&self, mut stream: TcpStream) -> PheasantResult<TcpStream> {
        let req = Request::from_stream(&mut stream);
        println!("{:#?}", req);

        let resp = Response::new(req, &self).await;
        let payload = resp.respond();

        stream.write_all(&payload)?;
        stream.flush()?;

        Ok(stream)
    }
}

// #[deprecated(note = "replaced by Request::from_stream")]
// async fn read_stream(s: &mut TcpStream) -> PheasantResult<String> {
//     let mut data = Vec::new();
//     let mut reader = BufReader::new(s);
//     let mut buf = [0; 1024];
//     loop {
//         let Ok(n) = reader.read(&mut buf) else {
//             return Err(PheasantError::StreamReadCrached);
//         };
//         if n < 1024 {
//             break data.extend(&buf[..n]);
//         } else if n > 1024 {
//             return Err(PheasantError::StreamReadWithExcess);
//         }
//         data.extend(buf);
//     }
//
//     String::from_utf8(data).map_err(|e| e.into())
// }

// #[deprecated(note = "replaced by Response::respond")]
// fn format_response(payload: Vec<u8>, ct: &Mime) -> Vec<u8> {
//     let cl = payload.len();
//     let mut res: Vec<u8> = format!(
//         "HTTP/1.1 200 OK\r\nAccept-Range: bytes\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
//         ct, cl
//     )
//     .into_bytes();
//     res.extend(payload);
//     res.extend([13, 10]);
//
//     res
// }

impl From<Request> for () {
    fn from(_p: Request) -> () {
        ()
    }
}
