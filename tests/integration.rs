use rustore::*;
use tempfile::NamedTempFile;

// Helper function to create a temporary database
fn create_temp_db() -> (Database, NamedTempFile) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = Database::open(temp_file.path().to_str().unwrap()).unwrap();
    (db, temp_file)
}

#[test]
fn test_put_and_get() {
    let (mut db, _temp_file) = create_temp_db();

    // Test putting and getting a single value
    db.put("key1", "value1").unwrap();
    assert_eq!(db.get("key1").unwrap(), "value1");

    // Test putting and getting multiple values
    db.put("key2", "value2").unwrap();
    db.put("key3", "value3").unwrap();

    assert_eq!(db.get("key2").unwrap(), "value2");
    assert_eq!(db.get("key3").unwrap(), "value3");
}

#[test]
fn test_get_non_existent_key() {
    let (db, _temp_file) = create_temp_db();

    match db.get("nonexistent") {
        Err(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        Ok(_) => panic!("Expected error for non-existent key"),
    }
}

#[test]
fn test_delete() {
    let (mut db, _temp_file) = create_temp_db();

    // Put and then delete a value
    db.put("key1", "value1").unwrap();
    assert_eq!(db.get("key1").unwrap(), "value1");

    db.delete("key1").unwrap();

    // Verify the key is no longer accessible
    match db.get("key1") {
        Err(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        Ok(_) => panic!("Expected error after deletion"),
    }
}

#[test]
fn test_delete_non_existent_key() {
    let (mut db, _temp_file) = create_temp_db();

    match db.delete("nonexistent") {
        Err(Error::NotFound) => (),
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_persistence() {
    let temp_path = NamedTempFile::new().unwrap();
    let file_path = temp_path.path().to_str().unwrap().to_string();

    // Write data
    {
        let mut db = Database::open(file_path.as_str()).unwrap();
        db.put("key1", "value1").unwrap();
        db.close().unwrap();
    }

    // Read data from a new instance
    {
        let db = Database::open(file_path.as_str()).unwrap();
        assert_eq!(db.get("key1").unwrap(), "value1");
    }
}

#[test]
fn test_update_existing_key() {
    let (mut db, _temp_file) = create_temp_db();

    db.put("key1", "value1").unwrap();
    assert_eq!(db.get("key1").unwrap(), "value1");

    // Update the value
    db.put("key1", "new_value").unwrap();
    assert_eq!(db.get("key1").unwrap(), "new_value");
}

#[test]
fn test_large_values() {
    let (mut db, _temp_file) = create_temp_db();

    let large_value = "x".repeat(1000000); // 1MB string
    db.put("large_key", &large_value).unwrap();
    assert_eq!(db.get("large_key").unwrap(), large_value);
}
