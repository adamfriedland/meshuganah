use mongodb::bson;
use serde::{de::DeserializeOwned, Serialize};
pub trait Model
where
    Self: Serialize + DeserializeOwned + Send + Sync,
{
    /// The name of the collection where this model's data is stored.
    const COLLECTION_NAME: &'static str;

    /// Get the ID for this model instance.
    fn get_id(&self) -> Option<bson::oid::ObjectId>;

    /// Set the ID for this model.
    fn set_id(&mut self, id: bson::oid::ObjectId);
}
