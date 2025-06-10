use actix_http::Request;
use actix_web::{
    dev::{Service, ServiceResponse},
    test,
    web::{scope, Data},
    App,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DbBackend, DbConn, DbErr};
use server::get_routes;

async fn init_db() -> Result<DbConn, DbErr> {
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

pub async fn init_app() -> Result<
    (
        impl Service<Request, Response = ServiceResponse, Error = actix_web::Error>,
        DbConn,
    ),
    DbErr,
> {
    let db = init_db().await?;
    let app = test::init_service(
        App::new()
            .service(get_routes())
            .app_data(Data::new(db.clone())),
    )
    .await;
    Ok((app, db))
}
