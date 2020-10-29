use std::marker::PhantomData;

use async_trait::async_trait;
use mongodb::{
    bson::{self, doc, from_bson, oid::ObjectId, to_bson, Bson, Document},
    options,
    options::{DeleteOptions, FindOneAndDeleteOptions, InsertManyOptions},
    Database,
};

use super::{cursor::TypeCursor, model::Model};

#[async_trait]
pub trait RepositoryTrait<T: Model> {
    async fn new(database: Database) -> Self;

    /// The model's write concern.
    fn write_concern() -> Option<options::WriteConcern> {
        None
    }

    fn get_document(model: &T) -> Result<Document, ()> {
        match to_bson(model) {
            Ok(doc) => match doc {
                Bson::Document(doc) => Ok(doc),
                other => panic!("Returned incorrect type {}", other),
            },
            Err(err) => panic!("couldn't deserialize type {}", err),
        }
    }

    fn get_instance_from_document(document: Document) -> Result<T, ()> {
        match from_bson::<T>(Bson::Document(document)) {
            Ok(doc) => Ok(doc),
            Err(_) => panic!("Something went wrong"),
        }
    }

    fn get_collection(&self) -> mongodb::Collection;

    async fn add_item(&self, filter: Option<bson::Document>, item: T);

    async fn add_many<I: IntoIterator<Item = T> + Send, O: Into<Option<InsertManyOptions>> + Send>(
        &self,
        collection: I,
        options: O,
    );

    async fn find_one<
        F: Into<Option<Document>> + Send,
        O: Into<Option<options::FindOneOptions>> + Send,
    >(
        &self,
        filter: F,
        options: O,
    ) -> Option<T>;

    async fn find<F: Into<Option<Document>> + Send, O: Into<Option<options::FindOptions>> + Send>(
        &self,
        filter: F,
        options: O,
    ) -> Result<TypeCursor<T>, ()>;

    async fn delete_item<O: Into<Option<FindOneAndDeleteOptions>> + Send>(
        &self,
        filter: Document,
        options: O,
    ) -> Result<T, ()>;

    async fn delete_items<O: Into<Option<DeleteOptions>> + Send>(
        &self,
        query: Document,
        options: O,
    );
}

pub struct GenericRepository<T: Model> {
    phantom: PhantomData<T>,
    pub database: Database,
}

#[async_trait]
impl<T: Model> RepositoryTrait<T> for GenericRepository<T> {
    async fn new(database: Database) -> Self {
        GenericRepository {
            phantom: PhantomData,
            database,
        }
    }

    fn get_collection(&self) -> mongodb::Collection {
        self.database.collection(T::COLLECTION_NAME)
    }

    async fn add_item(&self, filter: Option<Document>, mut item: T) {
        let model = Self::get_document(&item).unwrap();

        // Ensure that journaling is set to true for this call, as we need to be able to get an ID back.
        let mut write_concern = Self::write_concern().unwrap_or_default();
        write_concern.journal = Some(true);

        // Handle case where instance already has an ID.
        let filter = match (item.get_id(), filter) {
            (Some(id), _) => doc! {"_id": id},
            (None, None) => {
                let new_id = ObjectId::new();
                item.set_id(new_id.clone());
                doc! {"_id": new_id}
            }
            (None, Some(filter)) => filter,
        };

        // Save the record by replacing it entirely, or upserting if it doesn't already exist.
        let opts = options::FindOneAndReplaceOptions::builder()
            .upsert(Some(true))
            .write_concern(Some(write_concern))
            .return_document(Some(options::ReturnDocument::After))
            .build();

        let updated_doc = self
            .get_collection()
            .find_one_and_replace(filter, model, opts)
            .await;

        match updated_doc {
            Ok(test) => match test {
                Some(document) => println!("document updated {:?}", document),
                None => panic!("wrong document type returned"),
            },
            Err(err) => panic!("failed to update document {}", err),
        };
    }

    async fn find_one<
        F: Into<Option<Document>> + Send,
        O: Into<Option<options::FindOneOptions>> + Send,
    >(
        &self,
        filter: F,
        options: O,
    ) -> Option<T> {
        match self.get_collection().find_one(filter, options).await {
            Ok(success) => match success.map(Self::get_instance_from_document).transpose() {
                Ok(item) => item,
                Err(_) => panic!("Failed on type conversion"),
            },
            Err(_) => panic!("Failed to retrieve document"),
        }
    }

    async fn find<
        F: Into<Option<Document>> + Send,
        O: Into<Option<options::FindOptions>> + Send,
    >(
        &self,
        filter: F,
        options: O,
    ) -> Result<TypeCursor<T>, ()> {
        match self
            .get_collection()
            .find(filter, options)
            .await
            .map(TypeCursor::<T>::new)
        {
            Ok(cursor) => Ok(cursor),
            Err(_) => panic!(""),
        }
    }

    async fn delete_items<O: Into<Option<DeleteOptions>> + Send>(
        &self,
        query: Document,
        options: O,
    ) {
        match self.get_collection().delete_many(query, options).await {
            Ok(success) => println!("Number of items deleted {:?}", success.deleted_count),
            Err(_) => panic!("Delete failed"),
        }
    }

    async fn add_many<
        I: IntoIterator<Item = T> + Send,
        O: Into<Option<InsertManyOptions>> + Send,
    >(
        &self,
        collection: I,
        options: O,
    ) {
        let documents = collection
            .into_iter()
            .map(|item| Self::get_document(&item).expect("failed to serialize"))
            .collect::<Vec<Document>>();

        match self.get_collection().insert_many(documents, options).await {
            Ok(success) => println!("{}", success.inserted_ids.len()),
            Err(_) => panic!("Failed to insert multiple documents"),
        }
    }

    async fn delete_item<O: Into<Option<FindOneAndDeleteOptions>> + Send>(
        &self,
        filter: Document,
        options: O,
    ) -> Result<T, ()> {
        let x = self.database.collection("name");
        match self
            .get_collection()
            .find_one_and_delete(filter, options)
            .await
        {
            Ok(success) => Self::get_instance_from_document(success.unwrap()),
            Err(_) => panic!("Failed to find and delete"),
        }
    }
}
