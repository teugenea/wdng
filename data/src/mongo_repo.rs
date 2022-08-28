use std::sync::Arc;
use config::Config;
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
use common::config_utils::*;
use common::const_config;

const DB_NAME: &str = "wdng";
const LANG_UNIT_COLL: &str = "lang_units";
const CONNECTION_PROT: &str = "mongodb://";

pub struct MongoConnection {
    client: Client,
}

impl MongoConnection {

    pub fn new(settings: &Config) -> Self {
        let user = settings.get_string(const_config::DB_USER)
            .expect("Cannot read database user");
        let password = settings.get_string(const_config::DB_PASSWORD)
            .expect("Cannot read database password");
        let host = resolve_string(settings, const_config::DB_HOST, "localhost");
        let port = resolve_string(settings, const_config::DB_PORT, "27017");
        let db_name = resolve_string(settings, const_config::DB_NANE, "test");
        let tls_enabled = resolve_bool(settings, const_config::DB_TLS_ENABLED, false);

        let base_conn_str = CONNECTION_PROT.to_owned() 
            + user.as_str() + ":" + password.as_str()
            + "@" + host.as_str() + ":" + port.as_str()
            + "/" + db_name.as_str();
        let conn_str= match tls_enabled {
            true => {
                let srv_pub_key = settings.get_string(const_config::DB_SRV_PUB_KEY)
                    .expect("error");
                let client_cert = settings.get_string(const_config::DB_CLIENT_CERT)
                    .expect("error");
                base_conn_str + "?tls=true" 
                    + format!("&authMechanism=SCRAM-SHA-256&tlsCAFile={}&tlsCertificateKeyFile={}&directConnection=true", srv_pub_key, client_cert).as_str()
            },
            _ => base_conn_str
        };

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
