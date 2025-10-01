mod encryptor;

use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DbBackend, DbConn};

use crate::settings::types::Settings;

pub use encryptor::{decode_and_decrypt, encrypt_and_encode};

pub async fn init_db(settings: &Settings) -> DbConn {
    let db = Database::connect(&settings.database.url)
        .await
        .expect("Failed to open DB connection.");
    let db_conn = match db.get_database_backend() {
        DbBackend::MySql => Database::connect(&settings.database.url)
            .await
            .expect("Failed to open DB connection."),
        DbBackend::Postgres => Database::connect(&settings.database.url)
            .await
            .expect("Failed to open DB connection."),
        DbBackend::Sqlite => db,
    };
    Migrator::up(&db_conn, None).await.unwrap();
    db_conn
}
