use std::sync::Arc;
use futures::{TryStreamExt};

use mongodb::bson::doc;
use mongodb::options::FindOptions;
use mongodb::{
    Client, 
    options::ClientOptions,
    bson::Document, 
    Database 
};
use crate::model::*;

const DB_NAME: &str = "wdng";
const LANG_UNIT_COLL: &str = "lang_units";
const CONNECTION_PROT: &str = "mongodb://";

pub struct MongoConnection {
    client: Client,
}

impl MongoConnection {

    pub fn new(host: &str, port: &str, user: &str, password: &str) -> Self {
        let conn_str = CONNECTION_PROT.to_owned() 
            + user + ":" + password + "@" 
            + host + ":" + port + "/" + DB_NAME;
        let client_options = 
            futures::executor::block_on(async move {
                ClientOptions::parse(conn_str).await
            });
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
        self.client.database(DB_NAME)
    }

}

pub struct MongoLangUnitRepo {
    connection: Arc<MongoConnection>
}

impl MongoLangUnitRepo {

    pub fn new(conn: Arc<MongoConnection>) -> Self {
        Self { 
            connection: conn
        }
    }

    pub async fn next(&self) -> Result<LangUnit, mongodb::error::Error> {
        let filter = doc! {"en": "implausible"};
        let options = FindOptions::builder().build();
        let mut cursor = self.connection.db()
            .collection::<Document>(LANG_UNIT_COLL)
            .find(filter, options)
            .await?;
        if let Some(unit) = cursor.try_next().await? {
            let mut lu = LangUnit::default();
            for lang in unit.keys() {
                if lang.starts_with("_") {
                    continue;
                }
                let a = unit.get(lang).expect("no lang").as_str().expect("not a string");
                lu.add_meaning(String::from(lang), String::from(a));
            }
            return Ok(lu);
        }
        Ok(LangUnit::default())
    }
}
