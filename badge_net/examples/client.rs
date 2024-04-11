use anyhow::Result;
use embedded_tls::{TlsConfig, TlsContext};
use std::net::TcpStream;

fn main() -> Result<()> {
    let client = TcpStream::connect("127.0.0.1:4443")?;

    let config: TlsConfig<embedded_tls::Aes256GcmSha384> =
        TlsConfig::new().with_server_name("badger");
    let mut read_record_buffer = [0u8; 16384];
    let mut write_record_buffer = [0u8; 16384];
    let mut tls = embedded_tls::blocking::TlsConnection::new(
        client.into(),
        &mut read_record_buffer,
        &mut write_record_buffer,
    );

    let mut r = rand::thread_rng();

    tls.open(TlsContext::new(&config, &mut r))
        .map_err(|e| anyhow::anyhow!("Failed to open connection: {:?}", e))?;

    println!("TLS connection established!");

    Ok(())
}
