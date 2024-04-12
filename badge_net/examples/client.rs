use anyhow::Result;
use embedded_tls::{blocking::TlsConnection, TlsConfig, TlsContext};
use rand::{rngs::OsRng, RngCore};
use tokio::signal;

use std::net::TcpStream;

fn main() -> Result<()> {
    let client = TcpStream::connect("127.0.0.1:4443")?;

    let config = TlsConfig::new().enable_rsa_signatures();
    let mut read_record_buffer = [0u8; 16384];
    let mut write_record_buffer = [0u8; 16384];
    let mut tls = TlsConnection::new(
        embedded_io_adapters::std::FromStd::new(client),
        &mut read_record_buffer,
        &mut write_record_buffer,
    );

    tls.open(TlsContext::new(
        &config,
        embedded_tls::UnsecureProvider::new::<embedded_tls::Aes128GcmSha256>(OsRng),
    ))
    .map_err(|e| anyhow::anyhow!("Failed to open connection: {:?}", e))?;

    println!("TLS connection established!");

    let mut buf = [0u8; 1024];
    let size = tls
        .read(&mut buf)
        .map_err(|e| anyhow::anyhow!("Failed to read data: {:?}", e))?;
    let buf = std::str::from_utf8(&buf[..size]).unwrap();
    println!("read {} bytes: {}", size, buf);

    tls.write("Hello World from client".as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to write data: {:?}", e))?;

    tls.close()
        .map_err(|(_, e)| anyhow::anyhow!("Failed to close connection: {:?}", e))?;

    Ok(())
}
