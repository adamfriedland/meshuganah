use std::marker::PhantomData;

use async_trait::async_trait;
use mongodb::{
    bson::{self, doc, from_bson, oid::ObjectId, to_bson, Bson, Document},
    error::Result,
    options,
    options::{DeleteOptions, FindOneAndDeleteOptions, InsertManyOptions},
    results::DeleteResult,
    results::InsertManyResult,
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

    fn get_document(model: &T) -> Result<Document> {
        match to_bson(model)? {
            Bson::Document(doc) => Ok(doc),
            other => panic!("Returned incorrect type {:?}", other),
        }
    }

    fn get_instance_from_document(document: Document) -> Result<T> {
        match from_bson::<T>(Bson::Document(document)) {
            Ok(doc) => Ok(doc),
            Err(_) => panic!("Failed to convert instance to type"),
        }
    }

    fn get_collection(&self) -> mongodb::Collection;

    async fn add_item(&self, filter: Option<bson::Document>, item: T) -> Result<Option<T>>;

    async fn add_many<I: IntoIterator<Item = T> + Send, O: Into<Option<InsertManyOptions>> + Send>(
        &self,
        collection: I,
        options: O,
    ) -> Result<InsertManyResult>;

    async fn find_one<
        F: Into<Option<Document>> + Send,
        O: Into<Option<options::FindOneOptions>> + Send,
    >(
        &self,
        filter: F,
        options: O,
    ) -> Result<Option<T>>;

    async fn find<F: Into<Option<Document>> + Send, O: Into<Option<options::FindOptions>> + Send>(
        &self,
        filter: F,
        options: O,
    ) -> Result<TypeCursor<T>>;

    async fn delete_item<O: Into<Option<FindOneAndDeleteOptions>> + Send>(
        &self,
        filter: Document,
        options: O,
    ) -> Result<Option<T>>;

    async fn delete_items<O: Into<Option<DeleteOptions>> + Send>(
        &self,
        query: Document,
        options: O,
    ) -> Result<DeleteResult>;
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

    async fn add_item(&self, filter: Option<Document>, mut item: T) -> Result<Option<T>> {
        let model = Self::get_document(&item).unwrap();

        let mut write_concern = Self::write_concern().unwrap_or_default();
        write_concern.journal = Some(true);

        let filter = match (item.get_id(), filter) {
            (Some(id), _) => doc! {"_id": id},
            (None, None) => {
                let new_id = ObjectId::new();
                item.set_id(new_id.clone());
                doc! {"_id": new_id}
            }
            (None, Some(filter)) => filter,
        };

        let opts = options::FindOneAndReplaceOptions::builder()
            .upsert(Some(true))
            .write_concern(Some(write_concern))
            .return_document(Some(options::ReturnDocument::After))
            .build();

        match self
            .get_collection()
            .find_one_and_replace(filter, model, opts)
            .await?
        {
            Some(document) => Ok(Some(Self::get_instance_from_document(document)?)),
            None => Ok(None),
        }
    }

    async fn add_many<
        I: IntoIterator<Item = T> + Send,
        O: Into<Option<InsertManyOptions>> + Send,
    >(
        &self,
        collection: I,
        options: O,
    ) -> Result<InsertManyResult> {
        let documents = collection
            .into_iter()
            .map(|item| Self::get_document(&item).expect("failed to serialize"))
            .collect::<Vec<Document>>();

        self.get_collection().insert_many(documents, options).await
    }

    async fn find<
        F: Into<Option<Document>> + Send,
        O: Into<Option<options::FindOptions>> + Send,
    >(
        &self,
        filter: F,
        options: O,
    ) -> Result<TypeCursor<T>> {
        self.get_collection()
            .find(filter, options)
            .await
            .map(TypeCursor::<T>::new)
    }

    async fn find_one<
        F: Into<Option<Document>> + Send,
        O: Into<Option<options::FindOneOptions>> + Send,
    >(
        &self,
        filter: F,
        options: O,
    ) -> Result<Option<T>> {
        let document = self.get_collection().find_one(filter, options).await?;

        match document {
            Some(document) => Ok(Some(Self::get_instance_from_document(document)?)),
            None => Ok(None),
        }
    }

    async fn delete_item<O: Into<Option<FindOneAndDeleteOptions>> + Send>(
        &self,
        filter: Document,
        options: O,
    ) -> Result<Option<T>> {
        match self
            .get_collection()
            .find_one_and_delete(filter, options)
            .await?
        {
            Some(doc) => Ok(Some(Self::get_instance_from_document(doc)?)),
            None => Ok(None),
        }
    }

    async fn delete_items<O: Into<Option<DeleteOptions>> + Send>(
        &self,
        query: Document,
        options: O,
    ) -> Result<DeleteResult> {
        self.get_collection().delete_many(query, options).await
    }
}
