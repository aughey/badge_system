use std::net::TcpStream;

use embedded_websocket::{
    framer::{Framer, ReadResult},
    WebSocketClient, WebSocketCloseStatusCode, WebSocketOptions, WebSocketSendMessageType,
};

fn main() -> anyhow::Result<()> {
    // open a TCP stream to localhost port 1337
    let address = "127.0.0.1:3000";
    println!("Connecting to: {}", address);
    let mut stream = TcpStream::connect(address)?;
    println!("Connected.");

    let mut read_buf = [0; 4000];
    let mut read_cursor = 0;
    let mut write_buf = [0; 4000];
    let mut frame_buf = [0; 4000];
    let mut websocket = WebSocketClient::new_client(rand::thread_rng());

    // initiate a websocket opening handshake
    let websocket_options = WebSocketOptions {
        path: "/ws",
        host: "localhost",
        origin: "http://localhost:3000",
        sub_protocols: None,
        additional_headers: None,
    };

    let mut framer = Framer::new(
        &mut read_buf,
        &mut read_cursor,
        &mut write_buf,
        &mut websocket,
    );
    framer
        .connect(&mut stream, &websocket_options)
        .map_err(|_| anyhow::anyhow!("framer error"))?;

    let message = "Hello, World!";
    framer
        .write(
            &mut stream,
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
