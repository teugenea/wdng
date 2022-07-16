use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    id: usize,
    user_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stat {
    user_id: usize,
    unit_id: usize,
    wrong: usize,
    right: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Unit {
    meanings: HashMap<String, String>,
}