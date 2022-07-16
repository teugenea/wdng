use mongodb::{ Client, options::ClientOptions, error::Error };
use tokio::runtime::Runtime;
use crate::repo::*;

pub struct MongoConnection {
    client: Client,
}

impl MongoConnection {

    pub fn new(host: &String, port: &String, user: String, password: String) -> Self {
        let conn_str = "mongodb://".to_owned() + host + ":" + port;
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

    pub fn client(&self) -> &Client {
        &self.client
    }
}

