use crate::{Failure, Protocol, Service};
use hashbrown::HashSet;
use pheasant_uri::{Origin, Scheme};
use std::io::Result as IOResult;
use std::net::{Ipv4Addr, SocketAddr, TcpListener};

pub struct HttpSocket {
    /// byte repr of allowed socket protocols
    protos: u8,
    /// tls configuration for use in https requests, if any
    secure: Option<TlsConfig>,
    // TODO once ip and tcp crates are written, replace this with own tcp listener
    /// the tcp listener socket
    socket: TcpListener,
    /// the class of the socket, specifies its functionality
    kind: SocketKind,
    // set of registered socket services
    services: HashSet<Service>,
    // set of registered socket failures (http error status services)
    failures: HashSet<Failure>,
    // socket origin scheme part
    scheme: Scheme,
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SocketKind {
    #[default]
    Origin,
    Gateway,
    Proxy,
}

// placeholder type until tls crate is written
// obviously does nothing
pub struct TlsConfig;

impl HttpSocket {
    /// creates a bew HttpSocket
    ///
    /// ### Error
    /// - returns an std::io::Error when a valid port is not found after u16::MAX is reached  
    pub fn new(
        addr: impl Into<Ipv4Addr>,
        port: u16,
        tls_config: Option<TlsConfig>,
        kind: SocketKind,
        scheme: Scheme,
        protos: &[Protocol],
    ) -> IOResult<Self> {
        Ok(Self {
            secure: tls_config,
            socket: bind_socket(addr, port, scheme)?,
            kind,
            scheme,
            services: HashSet::new(),
            failures: HashSet::new(),
        })
    }

    /// returns a result of the origin this socket is bound to
    ///
    /// ### Error
    /// - errors if std::net::SocketAddr.local_addr() returns an error
    ///
    pub fn origin(&self) -> IOResult<Origin> {
        let addr = self.addr()?;
        let (ip, port) = (addr.ip(), addr.port());

        Ok(Origin::from_parts(self.scheme, ip, port))
    }

    // returns a result of the socket's ip addr
    fn addr(&self) -> IOResult<SocketAddr> {
        self.socket.local_addr()
    }

    /// whether the socket supports secure connections(tls) or not
    ///
    /// > [!WARN]
    /// > tls/https is currently unsupported
    pub fn is_secure(&self) -> bool {
        self.secure.is_some()
    }

    /// returns this socket's kind
    pub fn kind(&self) -> SocketKind {
        self.kind
    }

    /// returns a slice of the protocols this socket supports
    ///
    /// > [!WARN]
    /// > currently only recognizes the http1.1 and http2 protocols
    pub fn supported_protocols(&self) -> &[Protocol] {
        match self.protos {
            0 => unreachable!("an empty protocol slice is an error at HttpSocket::new"),
            1 => &[Protocol::Http1_1],
            2 => &[Protocol::Http2],
            3 => &[Protocol::Http1_1, Protocol::Http2],
            _ => unreachable!("unrecognized u8 protocols repr"),
        }
    }

    /// checks whether this socket supports the http1.1 protocol
    pub fn supports_http1_1(&self) -> bool {
        self.protos && 1 == 1
    }

    /// chechs whether this socket supports the http2 protocol
    ///
    /// > [!WARN]
    /// > http2 is yet unsupported
    ///
    pub fn supports_http2(&self) -> bool {
        self.protos && 2 == 2
    }

    /// gets a shared reference of self
    pub fn as_ref(&self) -> &Self {
        self
    }

    /// gets a mutable borrow of self
    pub fn as_mut(&mut self) -> &mut Self {
        self
    }

    /// registers new service(s) to this socket
    pub fn service<S, B>(&mut self, s: S) -> &mut Self
    where
        S: Fn() -> B,
        B: ServiceBundle,
    {
        let bundle = s();
        match bundle.size() {
            0 => return self,
            1 => {
                let Some(service) = bundle.iter().next() else {
                    unreachable!("size is 1 so we can't fail here");
                };

                self.services.insert(service);
            }
            _ => self.services.extend(bundle.iter()),
        }

        self
    }

    /// registers new http failure to this socket
    pub fn failure<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn() -> Failure,
    {
        self.failures.insert(f());

        self
    }
}

// tries to bind the socket to the passed addr and port
// keeps incrementing port number until it finds a free port
//
// ### Error
// - returns an std::io::Error when port reaches u16::MAX and no free port is found
fn bind_socket(addr: impl Into<Ipv4Addr>, mut port: u16, scheme: Scheme) -> IOResult<TcpListener> {
    let addr = addr.into();
    let socket = loop {
        match TcpListener::bind((addr, port)) {
            Ok(listener) => break listener,
            err if port == u16::MAX => return err,
            _err => port += 1,
        }
    };

    println!(
        "\x1b[1;38;2;237;203;244mSocket listening on origin {:?}://{}:{}\x1b[0m",
        scheme, addr, port
    );

    Ok(socket)
}
