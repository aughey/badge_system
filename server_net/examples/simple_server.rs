use anyhow::Result;
use rustls::crypto::{aws_lc_rs as provider, CryptoProvider};
use rustls::server::WebPkiClientVerifier;
use rustls::RootCertStore;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut args = env::args();
    args.next();
    let ca_file = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing ca file argument"))?;
    let cert_file = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing certificate file argument"))?;
    let private_key_file = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing private key file argument"))?;

    info!("Loading CA file: {:?}", ca_file);
    let roots = {
        let mut filebuf = BufReader::new(File::open(ca_file)?);
        let root_ca = rustls_pemfile::certs(&mut filebuf);
        let mut roots = RootCertStore::empty();
        for cert in root_ca {
            roots
                .add(cert?)
                .map_err(|_| anyhow::anyhow!("failed to add cert to root store"))?;
        }
        roots
    };

    info!("Loading cert file: {:?}", cert_file);
    let certs = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(cert_file)?))
        .collect::<Result<Vec<_>, _>>()?;
    info!("Loading private key file: {:?}", private_key_file);
    let private_key =
        rustls_pemfile::private_key(&mut BufReader::new(&mut File::open(&private_key_file)?))?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "no private key found in {:?} (encrypted keys not supported)",
                    private_key_file
                )
            })?;

    info!("Building client verifier");
    let client_verifier = WebPkiClientVerifier::builder(roots.into()).build()?;

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
    .with_client_cert_verifier(client_verifier)
    .with_single_cert(certs, private_key)?;

    let listener = TcpListener::bind(format!("[::]:{}", 4443)).await?;
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));

    loop {
        info!("Waiting for incoming connection");
        let (stream, _) = listener.accept().await?;
        info!("Got incoming connection");

        // turn stream into an async stream
        let mut stream = match acceptor.accept(stream).await {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("accept error: {:?}", e);
                continue;
            }
        };

        info!("Writing to stream");
        stream.write("From server".as_bytes()).await?;

        // // Get past the handshake
        // while conn.is_handshaking() {
        //     conn.complete_io(&mut stream)?;
        // }
        // println!("alpn protocol: {:?}", conn.alpn_protocol());
        // println!("cipher suite: {:?}", conn.negotiated_cipher_suite());
        // println!("protocol version: {:?}", conn.protocol_version());

        info!("Reading from stream");
        loop {
            let mut buf = [0u8; 1024];

            match stream.read(&mut buf).await {
                Ok(0) => break,
                Ok(len) => {
                    let buf = std::str::from_utf8(&buf[..len]).map_err(|e| {
                        anyhow::anyhow!("Could not convert buf into utf8 string: {e:?}")
                    })?;
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
    }

    #[allow(unreachable_code)]
    Ok(())
}
