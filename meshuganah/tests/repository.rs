use async_std::task;
use fixtures::{DatabaseCollectionFixture, Species};
use futures::StreamExt;
use meshuganah::GenericRepository;
use meshuganah::RepositoryTrait;
use mongodb::bson::doc;

mod fixtures;

#[test]
fn add_document_and_confirm_item() {
    task::block_on(async {
        let fixture = DatabaseCollectionFixture::new().await.drop_database().await;
        let repository = GenericRepository::<Species>::new(fixture.database).await;

        let item = Species {
            id: None,
            name: "some_name".to_string(),
            category: "some_category".to_string(),
            taxonomy: "some_taxonomy".to_string(),
        };

        let added_item = repository
            .add_item(None, item)
            .await
            .expect("failed to insert")
            .expect("no item added");

        let filter = doc! { "name": &added_item.name };

        let found_item = repository
            .find_one(filter, None)
            .await
            .expect("failed to retrieve item")
            .expect("nothing to retrieve");

        assert_eq!(found_item.name, added_item.name);
        assert_ne!(found_item.category, "some_other_category");
    });
}

#[test]
fn find_documents_and_confirm_length() {
    task::block_on(async {
        let fixture = DatabaseCollectionFixture::new().await.drop_database().await;
        let repository = GenericRepository::<Species>::new(fixture.database).await;

        let collection = vec![
            Species {
                id: None,
                name: "some_other_name".to_string(),
                category: "some_category".to_string(),
                taxonomy: "some_taxonomy".to_string(),
            },
            Species {
                id: None,
                name: "some_other_name".to_string(),
                category: "some_category".to_string(),
                taxonomy: "some_taxonomy".to_string(),
            },
        ];

        repository
            .add_many(collection, None)
            .await
            .expect("failed to insert items");

        let filter = doc! { "name": "some_other_name" };

        let cursor: Vec<_> = repository
            .find(filter, None)
            .await
            .expect("failed to retrieve cursor")
            .collect()
            .await;

        assert_eq!(cursor.len(), 2);
    });
}

#[test]
fn delete_document_and_confirm_deletion() {
    task::block_on(async {
        let fixture = DatabaseCollectionFixture::new().await.drop_database().await;
        let repository = GenericRepository::<Species>::new(fixture.database).await;

        let item = Species {
            id: None,
            name: "some_name".to_string(),
            category: "some_category".to_string(),
            taxonomy: "some_taxonomy".to_string(),
        };
        let name = item.name.clone();

        repository
            .add_item(None, item)
            .await
            .expect("failed to insert item");

        let query = doc! { "name": "some_name" };
        let result = repository.delete_item(query, None).await.unwrap().unwrap();

        assert_eq!(result.name, name);
    });
}

#[test]
fn delete_documents_and_confirm_deletion() {
    task::block_on(async {
        let fixture = DatabaseCollectionFixture::new().await.drop_database().await;
        let repository = GenericRepository::<Species>::new(fixture.database).await;

        let collection = vec![
            Species {
                id: None,
                name: "some_other_name_to_delete".to_string(),
                category: "some_category".to_string(),
                taxonomy: "some_taxonomy".to_string(),
            },
            Species {
                id: None,
                name: "some_other_name_to_delete".to_string(),
                category: "some_category".to_string(),
                taxonomy: "some_taxonomy".to_string(),
            },
        ];

        repository
            .add_many(collection, None)
            .await
            .expect("failed to insert items");

        let delete_query = doc! { "name": "some_other_name_to_delete" };
        repository
            .delete_items(delete_query, None)
            .await
            .expect("failed to delete items");

        let find_query = doc! { "name": "some_other_name_to_delete" };
        let found_item = repository
            .find_one(find_query, None)
            .await
            .expect("failed to find item");

        assert!(found_item.is_none(), true);
    });
}
