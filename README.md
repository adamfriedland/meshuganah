<h1 align="center">Meshuganah</h1>
<div align="center">
    <strong>A strongly typed ODM for the official Mongodb Rust driver, providing an easy to use abstraction.</strong>
</div>
<br />

```rust ,no_run
use meshuganah::GenericRepository;
use meshuganah::RepositoryTrait;
use futures::stream::StreamExt;

let me = Species {
            id: None,
            name: "some_name".to_string(),
            category: "some_category".to_string(),
            taxonomy: "some_taxonomy".to_string(),
};

let repository = GenericRepository::<Species>::new().await;

// Inserting a document
repository.add_item(None, me).await;

// Find single document
let found = repository.find_one(None, None).await.unwrap();

// Find multiple documents
let mut cursor = repository.find(None, None).await.unwrap();

while let Some(item) = cursor.next().await {
    println!("{:?}", item);
}
```

# Work still needed to be done
- Correct error handling
- Migrations
- Indexing
- Extending more of the base client in a type safe way