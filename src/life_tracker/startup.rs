use actix_session::{config::PersistentSession, storage, SessionMiddleware};
use actix_web::{
    cookie,
    dev::Server,
    middleware::Compress,
    web::{scope, Data},
    App, HttpServer,
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend};
use server::{get_preps_for_redis_session_store, get_routes, setup_session_middleware_builder};
use std::env;

use migration::{Migrator, MigratorTrait};
use routes::{
    action_routes, action_track_routes, ambition_routes, auth_routes, desired_state_routes,
    diary_routes, mindset_routes, reading_note_routes, tag_routes,
};
use utils::auth::auth_middleware::AuthenticateUser;
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(settings: settings::Settings) -> Result<Self, std::io::Error> {
        let db = get_database_connection().await;
        Migrator::up(&db, None).await.unwrap();
        let address = format!(
            "{}:{}",
            settings.application.host, settings.application.port
        );

        let listener = std::net::TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, db, settings).await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub async fn get_database_connection() -> DatabaseConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&database_url)
        .await
        .expect("Failed to open DB connection.");
    match db.get_database_backend() {
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
    }
}

async fn run(
    listener: std::net::TcpListener,
    db: DatabaseConnection,
    settings: settings::Settings,
) -> Result<Server, std::io::Error> {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let cfg = deadpool_redis::Config::from_url(redis_url.clone());
    let redis_pool = cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("Cannot create deadpool redis.");

    let (redis_store, secret_key) = get_preps_for_redis_session_store(&settings, &redis_url).await;

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .wrap(AuthenticateUser)
            .wrap(
                setup_session_middleware_builder(
                    SessionMiddleware::builder(redis_store.clone(), secret_key.clone()),
                    &settings,
                )
                .build(),
            )
            .service(get_routes())
            .app_data(Data::new(db.clone()))
            .app_data(Data::new(redis_pool.clone()))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
