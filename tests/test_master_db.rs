#[cfg(feature = "master-db")]
#[tokio::test]
#[allow(non_snake_case)]
async fn test_decrypt_to_file() {
    use rekordcrate::master_db::decrypt_to_file;
    use std::{io, path::PathBuf};

    let source_path = PathBuf::from("data/master_db/master.db");

    // Create temp file for unencrypted dump
    let mut output_path = std::env::temp_dir();
    output_path.push("test_dump.db");

    // Decrypt the database to a new file
    match decrypt_to_file(&source_path, &output_path).await {
        Err(rekordcrate::Error::IOError(e)) if e.kind() == io::ErrorKind::NotFound => {
            // XXX: This should be documented or something.
            todo!("this test requires a RekordBox master.db file to be copied to data/master_db/master.db")
        }
        other => other.unwrap(),
    };

    // Verify we can open and query the unencrypted output without a key
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(&output_path)
        .create_if_missing(true);
    let unencrypted_db = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let count: i32 = sqlx::query_scalar("SELECT count(*) FROM sqlite_master")
        .fetch_one(&unencrypted_db)
        .await
        .unwrap();
    assert!(count > 0, "Decrypted database should contain tables");

    // Ensure database is closed before cleanup
    drop(unencrypted_db);

    // Clean up
    std::fs::remove_file(output_path).unwrap();
}
