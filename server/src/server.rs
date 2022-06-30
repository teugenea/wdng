use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

use crate::messages::*;

pub struct GameServer {

}

impl Default for GameServer {
    fn default() -> Self {
        Self {
            
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
        0
    }
}

impl Handler<Disconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result {
        println!("Disconnected")
    }
}

impl Handler<GameMessage> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: GameMessage, ctx: &mut Self::Context) -> Self::Result {
        println!("{}", msg.0);
    }
}