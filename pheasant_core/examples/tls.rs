use pheasant_core::tls::{make_cert, tls_conn};
use rustls::Stream;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() -> Result<(), std::io::Error> {
    make()
}

fn make() -> Result<(), std::io::Error> {
    // make_cert(&["localhost".into()]);
    conn()
}

fn conn() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("localhost:7878")?;
    let (mut stream, _) = listener.accept()?;

    let mut serverconn = tls_conn(&["localhost".into()]).unwrap();
    let mut tlss = Stream::new(&mut serverconn, &mut stream);
    println!("2");

    tlss.write_all(b"hello from tls secured server connection")?;
    tlss.flush()?;
    println!("3");

    let mut buf = [0; 2048];
    let len = tlss.read(&mut buf)?;
    println!("4");

    println!("{:?}", str::from_utf8(&buf).unwrap());

    Ok(())
}
