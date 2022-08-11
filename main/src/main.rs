use actix::*;
use actix_files::{Files, NamedFile};
use actix_web::{
    middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;
use server::messages::GameMessage;
use std::time::{Duration, Instant};
use server::server::GameServer;
use server::{messages, session};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

async fn ws_route(
    req:  HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<GameServer>>,
) -> Result<impl Responder, Error> {
    ws::start(WsGameSession {
        id: 0,
        hb: Instant::now(),
        addr: srv.get_ref().clone()
    }, 
    &req, 
    stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = GameServer::default().start();
    let srv = server.clone();

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("cert/srvs-eu-private.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert/srvs-eu-cert.pem").unwrap();
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .service(web::resource("/game/ws").to(ws_route))
            .wrap(Logger::default())
    })
    .bind_openssl("127.0.0.1:8080", builder)?
    .workers(2)
    .run()
    .await
}

struct WsGameSession {
    id: usize,
    hb: Instant,
    addr: Addr<GameServer>
}

impl WsGameSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                act.addr.do_send(messages::Disconnect { id: act.id });
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for WsGameSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        let addr = ctx.address();
        self.addr
            .send(messages::Connect {addr: addr.recipient()})
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        self.addr.do_send(messages::Disconnect { id: self.id } );
        Running::Stop
    }
}

impl Handler<messages::SessionMessage> for WsGameSession {
    type Result = ();

    fn handle(&mut self, msg: messages::SessionMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsGameSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Text(text) => {
                println!("{} - {}", self.id, text);
                self.addr.do_send(GameMessage {
                    id: self.id,
                    msg: text.trim().to_owned(),
                })
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            _ => ()
        }
    }
}