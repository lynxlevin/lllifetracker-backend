mod encryptor;

use sea_orm::{ConnectionTrait, Database, DbConn, DbErr};

use crate::settings::types::Settings;

pub use encryptor::{decode_and_decrypt, encrypt_and_encode};

#[derive(Clone)]
pub struct Db {
    pub db: DbConn,
}

pub async fn get_db_connection(settings: &Settings) -> Result<Db, DbErr> {
    let db = Database::connect(&settings.database.url).await?;
    Ok(Db { db })
}

async fn db_migration(db: &DbConn) -> Result<(), DbErr> {
    db.get_schema_registry("entities::*").sync(db).await
}

pub async fn init_db(settings: &Settings) -> Result<Db, DbErr> {
    let db = get_db_connection(&settings).await?;
    db_migration(&db.db).await?;
    Ok(db)
}

pub async fn init_test_db(settings: &Settings) -> () {
    let db = get_db_connection(&settings).await.unwrap();

    db.db
        .execute_unprepared(
            "DROP TABLE IF EXISTS
            friendship,
            note,
            note_permission,
            \"user\",
            tag,
            tags_notes,
            CASCADE;
        ",
        )
        .await
        .unwrap();

    db_migration(&db.db).await.unwrap();
    ()
}
