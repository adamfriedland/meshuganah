# Meshuganah
A strongly typed ODM for the official Mongodb Rust driver, providing an easy to use abstraction.

<h1 align="center">Meshuganah</h1>
<div align="center">
    <strong>A strongly typed ODM for the official Mongodb Rust driver, providing an easy to use abstraction.</strong>
</div>
<br />

```rust ,no_run
use futures::stream::StreamExt;

let me = Species {
            id: None,
            name: "some_name".to_string(),
            category: "some_category".to_string(),
            taxonomy: "some_taxonomy".to_string(),
};

let repository = GenericRepository::<User>::new().await;

repository.add_item(None, me).await;

let found = repository.find_one(None, None).await.unwrap();

```