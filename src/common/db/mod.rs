mod encryptor;

use sea_orm::{Database, DbConn};

use crate::settings::types::Settings;

pub use encryptor::{decode_and_decrypt, encrypt_and_encode};

pub async fn init_db(settings: &Settings) -> DbConn {
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to open DB connection.");

    // MYMEMO: change this code
    // Migrator::up(&db_conn, None).await.unwrap();
    db
}
