use anyhow::Result;
use embedded_tls::{TlsConfig, TlsConnection, TlsContext};
use rand::rngs::OsRng;
mod from_tokio;

use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<()> {
    let client = TcpStream::connect("127.0.0.1:4443").await?;

    let config = TlsConfig::new().enable_rsa_signatures();
    let mut read_record_buffer = [0u8; 16384];
    let mut write_record_buffer = [0u8; 16384];
    let mut tls = TlsConnection::new(
        from_tokio::FromTokio::new(client),
        //embedded_io_adapters::std::FromStd::new(client),
        &mut read_record_buffer,
        &mut write_record_buffer,
    );

    tls.open(TlsContext::new(
        &config,
        embedded_tls::UnsecureProvider::new::<embedded_tls::Aes128GcmSha256>(OsRng),
    ))
    .await
    .map_err(|e| anyhow::anyhow!("Failed to open connection: {:?}", e))?;

    println!("TLS connection established!");

    let mut buf = [0u8; 1024];
    let size = tls
        .read(&mut buf)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read data: {:?}", e))?;
    let buf = std::str::from_utf8(&buf[..size])?;
    println!("read {} bytes: {}", size, buf);

    tls.write("Hello World from client".as_bytes())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to write data: {:?}", e))?;

    tls.close()
        .await
        .map_err(|(_, e)| anyhow::anyhow!("Failed to close connection: {:?}", e))?;

    Ok(())
}
