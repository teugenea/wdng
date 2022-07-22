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
    lang_unit_id: usize,
    wrong: usize,
    right: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LangUnit {
    meanings: HashMap<String, String>,
}

impl Default for LangUnit {
    fn default() -> Self {
        Self { 
            meanings: Default::default() 
        }
    }
}