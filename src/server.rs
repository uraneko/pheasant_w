use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};

use mime::Mime;
use serde::Serialize;
use url::Url;

use super::{
    ClientError, Fail, Method, PheasantError, PheasantResult, Protocol, Redirection, Request,
    Response, ResponseStatus, Route, ServerError, Service, Status, Successful,
};

/// the http server type
pub struct Server {
    /// the server tcp listener socket
    socket: TcpListener,
    /// container for the server services
    services: Vec<Service>,
    // container for the server error responses (client/server errors)
    errors: Vec<Fail>,
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
                    "\x1b[1;38;2;237;203;244mServer bound at http://{}:{}\x1b[0m",
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
            errors: vec![],
        })
    }

    /// pushes a new service to the server
    pub fn service<S>(&mut self, s: S)
    where
        S: Fn() -> Service,
    {
        self.services.push(s());
    }

    pub fn error<E>(&mut self, e: E)
    where
        E: Fn() -> Fail,
    {
        self.errors.push(e());
    }
}

impl Server {
    /// searches for the specified service
    /// returns `(Status, &Service)`
    ///
    /// ### Error
    /// returns an Err, a client error (404 not found) if the service is not found
    // TODO this should return Result<(Status, &Service), &Fail>
    pub fn service_status(
        &self,
        method: Method,
        route: &str,
    ) -> PheasantResult<(Status, &Service)> {
        match self
            .services
            .iter()
            .find(move |s| s.method() == method && (s.route() == route || s.redirects_to(&route)))
        {
            Some(s) if s.route() == route => Ok((Status::Successful(Successful::OK), s)),
            Some(s) if s.redirects_to(&route) => {
                Ok((Status::Redirection(Redirection::SeeOther), s))
            }
            None => Err(PheasantError::ClientError(ClientError::NotFound)),
            Some(_) => unreachable!("unimplemented"),
        }
    }

    /// searches for the speficied `Fail` (error status fallback service)
    /// returns `Some(&Fail)` if found
    /// else returns `None`
    pub fn fail_status(&self, status_code: u16) -> Option<&Fail> {
        self.errors.iter().find(move |e| e.code() == status_code)
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
        println!("{:#?}", req); // if req is err we return a status error response
        let Ok(req) = req else {
            let resp = self.error_template(400, None).await;

            return send_response(stream, resp);
        };

        let resp = match self.service_status(req.method(), req.route()) {
            Ok((status, service)) => Response::payload(req, status, service).await,
            Err(PheasantError::ClientError(ClientError::NotFound)) => {
                self.error_template(404, Some(req.proto())).await
            }
            _ => unimplemented!("not implemented yet"),
        };

        send_response(stream, resp)
    }

    pub async fn error_template(&self, code: u16, proto: Option<Protocol>) -> Response {
        let fail = self.fail_status(code);
        Response::from_err(fail, proto)
            .await
            .unwrap_or(Response::not_implemented().await)
    }
}

// sends the response to the client and returns the connection tcp stream
fn send_response(mut stream: TcpStream, resp: Response) -> PheasantResult<TcpStream> {
    let payload = resp.respond();

    stream.write_all(&payload)?;
    stream.flush()?;

    Ok(stream)
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
