use std::collections::HashMap;
use futures::{future, Future};

use mongodb::bson::doc;
use mongodb::options::FindOptions;
use mongodb::{
    Client, 
    options::ClientOptions, 
    error::Error, 
    bson::Document, 
    Database 
};
use tokio::runtime::Runtime;
use crate::repo::*;
use crate::model::*;

pub struct MongoConnection {
    client: Client,
}

impl MongoConnection {

    pub fn new(host: &String, port: &String, user: &String, password: &String) -> Self {
        let conn_str = "mongodb://".to_owned() + user + ":" + password + "@" + host + ":" + port;
        let client_options = 
            Runtime::new().unwrap().block_on(ClientOptions::parse(conn_str));
        match client_options {
            Ok(opt) => {
                let client = Client::with_options(opt).unwrap();
                Self{
                    client
                }
            }
            Err(err) => panic!("Cannot connect to DB: {}", err)
        }
    }

    pub fn db(&self) -> Database {
        self.client.database("wdng")
    }

}

pub struct MongoLangUnitRepo {
    connection: MongoConnection
}

impl MongoLangUnitRepo {
    async fn next(&self) -> LangUnit {
        LangUnit::default()
    }
}
