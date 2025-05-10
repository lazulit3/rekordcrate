//! Decrypts and reads the Rekordbox `master.db` database.
use sqlx::sqlite::SqliteConnectOptions;
use std::fs;
use std::str::FromStr;
use tempfile::NamedTempFile;

/// Key for decrypting RekordBox `master.db`
// TODO: This may vary depending on RekordBox versions.
pub const MASTER_DB_KEY: &str = "402fd482c38817c35ffa8ffb8c7d93143b749e7d315df7a81732a1ff43608497";
use std::path::Path;

/// Creates an unencrypted copy of a Rekordbox master.db database file.
///
/// # Arguments
///
/// * `master_db` - Path to the encrypted master.db file
/// * `output` - Path where the unencrypted database should be written
pub async fn decrypt_to_file(master_db: &Path, output: &Path) -> crate::Result<()> {
    // Copy the input database to our temporary file (paranoid about affecting it)
    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path().to_str().unwrap();
    fs::copy(master_db, temp_path)?;

    let options = SqliteConnectOptions::from_str(temp_path)?
        .pragma("key", format!("\"{}\"", MASTER_DB_KEY))
        .create_if_missing(true);
    let pool = sqlx::SqlitePool::connect_with(options).await?;

    let output = output.to_str().unwrap();

    // Export to unencrypted database
    let unencrypted_db = "unencrypted";
    sqlx::raw_sql(&format!(
        "
           ATTACH DATABASE '{}' AS {unencrypted_db} KEY '';
           SELECT sqlcipher_export('{unencrypted_db}');
           DETACH DATABASE {unencrypted_db};
        ",
        output,
    ))
    .execute(&pool)
    .await?;

    pool.close().await;
    Ok(())
}
