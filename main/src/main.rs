use actix::*;
use actix_web::{
    middleware::Logger, web, App, Error, HttpRequest, HttpServer, Responder,
};
use actix_web_actors::ws;
use clap::Parser;
use server::messages::GameMessage;
use std::time::{Duration, Instant};
use server::server::GameServer;
use server::{messages};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use config::{Config, File};

mod cli;
use common::const_config;
use common::config_utils::*;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
const BIND_ADDR: &str = "127.0.0.1";
const DEFAULT_BIND_PORT: &str = "8080";

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
    let args = cli::Args::parse();
    let settings = Config::builder()
        .add_source(File::with_name(&args.config))
        .build()
        .unwrap();
    let tls_enabled = resolve_bool(&settings, const_config::SERVER_TLS_ENABLED, false);
    match tls_enabled {
        true => run_tls_server(&settings, GameServer::new(&settings).start()).await,
        _ => run_server(&settings, GameServer::new(&settings).start()).await
    }
}

async fn run_tls_server(settings: &Config, server: Addr<GameServer>) -> Result<(), std::io::Error> {
    let port = resolve_string(settings, const_config::SERVER_PORT, DEFAULT_BIND_PORT);
    let private_cert_file = settings.get_string(const_config::CERT_KEY_FILE)
        .expect(format!("TLS is enabled but private key file is not specified [{}]", const_config::CERT_KEY_FILE).as_str());
    let cert_file = settings.get_string(const_config::CERT_FILE)
        .expect(format!("TLS is enabled but certificate chain file is not specified [{}]", const_config::CERT_KEY_FILE).as_str());
    let ws_context = resolve_string(settings, const_config::SERVER_WS_CONTEXT, "/ws");

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file(private_cert_file, SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file(cert_file).unwrap();
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .service(web::resource(ws_context.clone()).to(ws_route))
            .wrap(Logger::default())
    })
    .bind_openssl(BIND_ADDR.to_owned() + ":" + &port, builder)?
    .workers(2)
    .run()
    .await
}

async fn run_server(settings: &Config, server: Addr<GameServer>) -> Result<(), std::io::Error> {
    let port = resolve_string(settings, const_config::SERVER_PORT, DEFAULT_BIND_PORT);
    let ws_context = resolve_string(settings, const_config::SERVER_WS_CONTEXT, "/ws");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .service(web::resource(ws_context.clone()).to(ws_route))
            .wrap(Logger::default())
    })
    .bind(BIND_ADDR.to_owned() + ":" + &port)?
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

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
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