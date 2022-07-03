use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use actix::prelude::*;

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

#[derive(Message)]
#[rtype(result = "usize")]
pub struct Connect {
    pub addr: Recipient<SessionMessage>
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SessionMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameMessage {
    pub id: usize,
    pub msg: String,
}