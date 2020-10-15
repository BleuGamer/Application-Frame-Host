use util;

use actix::{Actor, StreamHandler};
use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Result};
use actix_web_actors::ws;

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    let mut dir = util::env::get_cwd().unwrap();
    dir.push("app");

    HttpServer::new(move || {
        App::new().service(fs::Files::new("/", dir.to_str().unwrap()).index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

struct WebSocket;

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(WebSocket {}, &req, stream);
    println!("{:?}", resp);
    resp
}

#[actix_web::main]
pub async fn start_web_socket() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/ws/", web::get().to(index)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
