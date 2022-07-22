use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};
use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

use crate::messages::*;

type Rec = Arc<Mutex<Recipient<SessionMessage>>>;

pub struct GameServer {
    sessions: HashMap<usize, Rec>,
    rng: ThreadRng,
}

impl GameServer {
    fn get_session_id(&mut self) -> usize {
        self.rng.gen::<usize>()
    }

    fn send_message(&self, id: usize, message: &str) {
        if let Some(session) = self.sessions.get(&id) {
            let s = Arc::clone(session);
            let m = String::from(message);
            tokio::spawn(async move {
                GameServer::process_message(s, m).await;
            });
        }
    }

    async fn process_message(r: Rec, message: String) {
        r.lock().unwrap().do_send(SessionMessage(message));
    }
}

impl Default for GameServer {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }
}

impl Actor for GameServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for GameServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result {
        println!("Connected");
        let id = self.get_session_id();
        self.sessions.insert(id, Arc::new(Mutex::new(msg.addr)));
        id
    }
}

impl Handler<Disconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result {
        println!("Disconnected");
        self.sessions.remove(&msg.id);
    }
}

impl Handler<GameMessage> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: GameMessage, ctx: &mut Self::Context) -> Self::Result {
        self.send_message(msg.id, &msg.msg)
    }
}