use env;
use meshuganah::Model;
use mongodb::{bson, Database};
use serde::{Deserialize, Serialize};

pub struct DatabaseCollectionFixture {
    pub database: Database,
}

impl DatabaseCollectionFixture {
    pub async fn new() -> Self {
        let mongo_host =
            env::var("MONGODB_HOST").expect("environment variable MONGODB_HOST not defined");
        let database = mongodb::Client::with_uri_str(mongo_host)
            .await
            .unwrap()
            .database("test");

        DatabaseCollectionFixture { database }
    }

    pub async fn drop_database(self) -> Self {
        self.database
            .drop(None)
            .await
            .expect("Failed to drop database");

        self
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Species {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub name: String,
    pub category: String,
    pub taxonomy: String,
}

unsafe impl Send for Species {}

impl Model for Species {
    const COLLECTION_NAME: &'static str = "species";

    fn get_id(&self) -> Option<bson::oid::ObjectId> {
        self.id.clone()
    }

    fn set_id(&mut self, oid: bson::oid::ObjectId) {
        self.id = Some(oid);
    }
}
