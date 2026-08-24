mod encryptor;

use sea_orm::{Database, DbConn};

use crate::settings::types::Settings;

pub use encryptor::{decode_and_decrypt, encrypt_and_encode};

pub async fn init_db(settings: &Settings) -> DbConn {
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to open DB connection.");

    db.get_schema_registry("entities::*")
        .sync(&db)
        .await
        .expect("Failed in DB migration.");
    db
}
