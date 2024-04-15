use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};

use embedded_tls::{Certificate, TlsConfig, TlsConnection, TlsContext};
use embedded_websocket::{
    framer_async::{Framer, ReadResult},
    WebSocketClient, WebSocketCloseStatusCode, WebSocketOptions, WebSocketSendMessageType,
};
use rand::rngs::OsRng;

mod from_tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // open a TCP stream to localhost port 1337
    let address = "127.0.0.1:3000";
    println!("Connecting to: {}", address);
    let stream = TcpStream::connect("127.0.0.1:4443").await?;

    println!("Connected.");

    let ca = include_str!("../../certs/CA_cert.crt");
    let ca = pem_parser::pem_to_der(ca);

    let cert = include_str!("../../certs/client.crt");
    let cert = pem_parser::pem_to_der(cert);

    let key = include_str!("../../certs/client.key");
    let key = pem_parser::pem_to_der(key);

    let config = TlsConfig::new()
        .enable_rsa_signatures()
        .with_ca(Certificate::X509(&ca))
        .with_priv_key(&key)
        .with_cert(Certificate::X509(&cert));
    let mut read_record_buffer = [0u8; 16384];
    let mut write_record_buffer = [0u8; 16384];
    let mut tls = TlsConnection::new(
        from_tokio::FromTokio::new(stream),
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

    let mut websocket = WebSocketClient::new_client(rand::thread_rng());

    // initiate a websocket opening handshake
    let websocket_options = WebSocketOptions {
        path: "/ws",
        host: "localhost",
        origin: "http://localhost:3000",
        sub_protocols: None,
        additional_headers: None,
    };

    let mut framer = Framer::new(websocket);

    let mut read_buf = [0; 4000];
    framer
        .connect(&mut tls, &mut read_buf, &websocket_options)
        .await
        .map_err(|_| anyhow::anyhow!("framer error"))?;

    let message = "Hello, World!";
    framer
        .write(
            &mut tls,
            WebSocketSendMessageType::Text,
            true,
            message.as_bytes(),
        )
        .map_err(|_| anyhow::anyhow!("Framer write"))?;

    loop {
        match framer
            .read(&mut stream, &mut frame_buf)
            .map_err(|_| anyhow::anyhow!("Framer read"))?
        {
            ReadResult::Binary(_) => {}
            ReadResult::Text(t) => {
                println!("Received: {}", t);
            }
            ReadResult::Pong(_) => {
                println!("pong")
            }
            ReadResult::Closed => break,
        }
    }

    println!("Connection closed");
    Ok(())
}
