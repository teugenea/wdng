use std::{
    io, net,
    str::FromStr,
    time::{Duration, Instant},
};

use actix::{prelude::*};
use tokio::{
    io::{split, WriteHalf},
    net::{TcpListener, TcpStream},
};
use tokio_util::codec::FramedRead;

use crate::{codec::*, server::GameServer, messages};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct GameSession {
    id: usize,
    hb: Instant,
    addr: Addr<GameServer>,
    framed: actix::io::FramedWrite<ServerResponse, WriteHalf<TcpStream>, ServerCodec>,
}

impl GameSession {

    pub fn new(
        addr: Addr<GameServer>,
        framed:  actix::io::FramedWrite<ServerResponse, WriteHalf<TcpStream>, ServerCodec>,
    ) -> GameSession {
        Self {
            id: 0,
            hb: Instant::now(),
            addr,
            framed
        }
    }

    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");
                // notify chat server
                act.addr.do_send(messages::Disconnect { id: act.id });
                // stop actor
                ctx.stop();
                // don't try to send a ping
                return;
            }
            act.framed.write(ServerResponse::Ping);
        });
    }
}

impl Actor for GameSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        let addr = ctx.address();
        self.addr
            .send(messages::Connect { addr: addr.recipient() })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    _ => ctx.stop(),
                }
                actix::fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        Running::Stop
    }
}

impl Handler<messages::Message> for GameSession {
    type Result = ();

    fn handle(&mut self, msg: messages::Message, ctx: &mut Self::Context) -> Self::Result {
        self.framed.write(ServerResponse::Message(msg.0))
    }
}

impl actix::io::WriteHandler<io::Error> for GameSession {}

impl StreamHandler<Result<ServerRequest, io::Error>> for GameSession {
    fn handle(&mut self, msg: Result<ServerRequest, io::Error>, ctx: &mut Context<Self>) {

    }
}

pub fn tcp_server(_s: &str, server: Addr<GameServer>) {
    let addr = net::SocketAddr::from_str("127.0.0.1:12345").unwrap();
    actix::spawn(async move {
        let listener = TcpListener::bind(&addr).await.unwrap();
        while let Ok((stream, _)) = listener.accept().await {
            let server = server.clone();
            GameSession::create(|ctx|  {
                let (r, w) = split(stream);
                GameSession::add_stream(FramedRead::new(r, ServerCodec), ctx);
                GameSession::new(server, actix::io::FramedWrite::new(w, ServerCodec, ctx))
            });
        }
    });
}