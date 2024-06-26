use actix::{Actor, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

/// Define HTTP actor
pub struct MyWs {}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                println!("ping");
                ctx.pong(&msg)
            }
            Ok(ws::Message::Text(text)) => {
                println!("text: {}", text);
                ctx.text(text)
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

pub async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    // initiate a connection to our own tcp, this will forward websocket traffic to our badge handler
    // let socket = tokio::net::TcpStream::connect("127.0.0.1:4443").await?;
    // let (reader, writer) = tokio::io::split(socket);
    ws::start(MyWs {}, &req, stream)
}
