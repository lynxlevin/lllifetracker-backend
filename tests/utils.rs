use actix_http::Request;
use actix_web::{
    dev::{Service, ServiceResponse},
    test,
    web::Data,
    App,
};
use common::settings::{get_test_settings, types::Settings};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DbBackend, DbConn, DbErr};
use server::get_routes;

async fn init_db(settings: &Settings) -> Result<DbConn, DbErr> {
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
    Ok(db_conn)
}

pub async fn init_app() -> Result<
    (
        impl Service<Request, Response = ServiceResponse, Error = actix_web::Error>,
        DbConn,
    ),
    DbErr,
> {
    let settings = get_test_settings();
    let db = init_db(&settings).await?;
    let app = test::init_service(
        App::new()
            .service(get_routes())
            .app_data(Data::new(db.clone()))
            .app_data(Data::new(settings)),
    )
    .await;
    Ok((app, db))
}
