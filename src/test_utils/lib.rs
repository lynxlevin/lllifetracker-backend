use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DbBackend, DbConn, DbErr};

pub mod factory;

pub use factory::{
    ActionFactory, ActionTrackFactory, AmbitionFactory, DesiredStateFactory, DiaryFactory,
    ReadingNoteFactory, TagFactory, UserFactory,
};

pub async fn init_db() -> Result<DbConn, DbErr> {
    dotenvy::from_filename(".env.testing").ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&database_url)
        .await
        .expect("Failed to open DB connection.");
    let db_conn = match db.get_database_backend() {
        DbBackend::MySql => {
            let url = format!("{}", &database_url);
            Database::connect(&url)
                .await
                .expect("Failed to open DB connection.")
        }
        DbBackend::Postgres => {
            let url = format!("{}", &database_url);
            Database::connect(&url)
                .await
                .expect("Failed to open DB connection.")
        }
        DbBackend::Sqlite => db,
    };
    Migrator::up(&db_conn, None).await.unwrap();
    Ok(db_conn)
}
