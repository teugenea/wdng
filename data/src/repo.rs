use mongodb::{ Client, options::ClientOptions, error::Error };
use tokio::runtime::Runtime;

pub struct MongoConnection {
    client: Client,
}

impl MongoConnection {

    pub fn new() -> Result<Self, Error> {
        let client_options = 
            Runtime::new().unwrap().block_on(ClientOptions::parse("mongodb://localhost:27017"));
        match client_options {
            Ok(opt) => {
                let client = Client::with_options(opt)?;
                Ok(Self{
                    client
                })
            }
            Err(err) => Err(err)
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}