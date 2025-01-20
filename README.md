# Rustore

Rustore is a simple key-value storage engine written in Rust. It's purpose is to familiarize myself with Rust and to learn more about how databases work.

## Usage

```rust
use rustore::Database;

fn main() {
    use rustore::Database;

    // Create a new database
    let mut db = Database::open("example.db").unwrap();

    // Insert a key-value pair
    db.put("key1", "value1").unwrap();
    db.put("key2", "value2").unwrap();

    assert_eq!(db.get("key2").unwrap(), "value2");
    assert_eq!(db.get("key1").unwrap(), "value1");

    // Remove a key-value pair
    db.delete("key1").unwrap();
}
```
