use mongodb::bson;
use serde::{de::DeserializeOwned, Serialize};
pub trait Model
where
    Self: Serialize + DeserializeOwned + Send + Sync,
{
    const COLLECTION_NAME: &'static str;

    fn get_id(&self) -> Option<bson::oid::ObjectId>;
    
    fn set_id(&mut self, id: bson::oid::ObjectId);
}
