use pheasant_uri::Origin;
use rcgen::{Certificate, CertifiedKey, KeyPair, generate_simple_self_signed};
use rustls::crypto::ring::default_provider as ring_provider;
use rustls::{
    ConfigBuilder, ProtocolVersion, ServerConfig, ServerConnection, SupportedProtocolVersion,
};
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::fs;
use std::net::TcpStream;
use std::sync::Arc;

pub fn make_cert(subject_alt_names: &[String]) -> Result<(), rcgen::Error> {
    let (cert, key) = generate_certificate(subject_alt_names)?;
    let cert_pem = cert.pem();
    let key_pem = key.serialize_pem();
    fs::write("tls/cert.pem", cert_pem.into_bytes()).unwrap();
    fs::write("tls/key.pem", key_pem.into_bytes()).unwrap();

    Ok(())
}

pub fn tls_conn(subject_alt_names: &[String]) -> Result<ServerConnection, rcgen::Error> {
    let (cert, key) = generate_certificate(subject_alt_names)?;
    let cert_der = cert.der().clone();
    let key_der: PrivatePkcs8KeyDer<'_> =
        <Vec<u8> as Into<PrivatePkcs8KeyDer<'_>>>::into(key.serialize_der());
    let key_der: PrivateKeyDer<'_> = key_der.into();

    let config = generate_configs(vec![cert_der], key_der);

    Ok(ServerConnection::new(Arc::new(config)).unwrap())
}

fn generate_certificate(
    subject_alt_names: &[String],
) -> Result<(Certificate, KeyPair), rcgen::Error> {
    let CertifiedKey { cert, signing_key } = generate_simple_self_signed(subject_alt_names)?;

    Ok((cert, signing_key))
}

fn generate_configs(
    cert: Vec<CertificateDer<'static>>,
    private_key: PrivateKeyDer<'static>,
) -> ServerConfig {
    ServerConfig::builder_with_provider(Arc::new(ring_provider()))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(cert, private_key)
        .unwrap()
}
