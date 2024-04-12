//! This is the simplest possible server using rustls that does something useful:
//! it accepts the default configuration, loads a server certificate and private key,
//! and then accepts a single client connection.
//!
//! Usage: cargo r --bin simpleserver <path/to/cert.pem> <path/to/privatekey.pem>
//!
//! Note that `unwrap()` is used to deal with networking errors; this is not something
//! that is sensible outside of example code.

use rustls::crypto::{aws_lc_rs as provider, CryptoProvider};
use std::env;
use std::error::Error as StdError;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let mut args = env::args();
    args.next();
    let cert_file = args.next().expect("missing certificate file argument");
    let private_key_file = args.next().expect("missing private key file argument");

    let certs = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(cert_file)?))
        .collect::<Result<Vec<_>, _>>()?;
    let private_key =
        rustls_pemfile::private_key(&mut BufReader::new(&mut File::open(private_key_file)?))?
            .unwrap();
    let config = rustls::ServerConfig::builder_with_provider(
        CryptoProvider {
            cipher_suites: [
                provider::cipher_suite::TLS13_AES_256_GCM_SHA384,
                provider::cipher_suite::TLS13_AES_128_GCM_SHA256,
                provider::cipher_suite::TLS13_CHACHA20_POLY1305_SHA256,
            ]
            .to_vec(),
            ..provider::default_provider()
        }
        .into(),
    )
    .with_protocol_versions(&[&rustls::version::TLS13])?
    .with_no_client_auth()
    .with_single_cert(certs, private_key)?;

    let listener = TcpListener::bind(format!("[::]:{}", 4443)).await?;
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
    let (stream, _) = listener.accept().await?;
    // turn stream into an async stream
    let mut stream = acceptor.accept(stream).await?;

    stream.write("From server".as_bytes()).await?;

    // // Get past the handshake
    // while conn.is_handshaking() {
    //     conn.complete_io(&mut stream)?;
    // }
    // println!("alpn protocol: {:?}", conn.alpn_protocol());
    // println!("cipher suite: {:?}", conn.negotiated_cipher_suite());
    // println!("protocol version: {:?}", conn.protocol_version());

    loop {
        let mut buf = [0u8; 1024];

        match stream.read(&mut buf).await {
            Ok(0) => break,
            Ok(len) => {
                let buf = std::str::from_utf8(&buf[..len]).unwrap();
                println!("read {} bytes: {}", len, buf);
                break;
            }
            Err(e) => {
                eprintln!("read error: {:?}", e);
                break;
            }
        }
    }
    // println!("alpn protocol: {:?}", stream.conn.alpn_protocol());
    // println!("cipher suite: {:?}", stream.conn.negotiated_cipher_suite());
    // println!("protocol version: {:?}", stream.conn.protocol_version());

    Ok(())
}
