use std::{
    collections::HashMap,
    sync::{
        Arc,
        Mutex,
    },
};
use actix::prelude::*;
use data::mongo_repo::{MongoLangUnitRepo, MongoConnection};
use rand::{self, rngs::ThreadRng, Rng};
use config::{Config, File};

use crate::messages::*;

type GameClient = Arc<Mutex<Recipient<SessionMessage>>>;

pub struct GameServer {
    sessions: HashMap<usize, GameClient>,
    rng: ThreadRng,
    conn: Arc<MongoConnection>,
    repo: Arc<MongoLangUnitRepo>,
}

impl GameServer {
    fn get_session_id(&mut self) -> usize {
        self.rng.gen::<usize>()
    }

    fn send_message(&self, id: usize, msg: &str) {
        if let Some(session) = self.sessions.get(&id) {
            let s = Arc::clone(session);
            let m = String::from(msg);
            let r = Arc::clone(&self.repo);
            tokio::spawn(async move {
                GameServer::process_message(r,s, m).await;
            });
        }
    }

    pub fn new(settings: &Config) -> Self {
        let conn = Arc::new(MongoConnection::new(&settings));
        let repo = Arc::new(MongoLangUnitRepo::new(Arc::clone(&conn)));
        Self {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
            conn,
            repo
        }
    }

    async fn process_message(repo: Arc<MongoLangUnitRepo>, client: GameClient, msg: String) {
        if !msg.starts_with("!") {
            let l = repo.next().await;
            match l {
                Ok(p) => {println!("{:?}", p)},
                Err(e) => {println!("Error: {}", e)}
            }
            client.lock().unwrap().do_send(SessionMessage(msg));
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