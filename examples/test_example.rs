fn main() {
    use rustore::Database;

    let mut db = Database::open("example.db").unwrap();

    assert_eq!(db.get("key2").unwrap(), "value2");
    assert_eq!(db.get("key1").unwrap(), "other");
}
