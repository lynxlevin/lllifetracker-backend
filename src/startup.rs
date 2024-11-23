use actix_session::{config::PersistentSession, storage, SessionMiddleware};
use actix_web::{cookie, dev::Server, web::{Data, scope}, App, HttpServer};
use sea_orm::*;
use std::env;

use migration::{Migrator, MigratorTrait};
use crate::{
    routes::{action_routes, ambition_routes, auth_routes, objective_routes},
    utils::auth::auth_middleware::AuthenticateUser,
};
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(settings: crate::settings::Settings) -> Result<Self, std::io::Error> {
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
    settings: crate::settings::Settings,
) -> Result<Server, std::io::Error> {
    // MYMEMO: refactor redis usage referencing deadpool redis official. Would like to remove boiler plates at getting connections.
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let cfg = deadpool_redis::Config::from_url(redis_url.clone());
    let redis_pool = cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("Cannot create deadpool redis.");

    let secret_key = cookie::Key::from(settings.secret.hmac_secret.as_bytes());
    let redis_store = storage::RedisSessionStore::new(redis_url)
        .await
        .expect("Cannot unwrap redis session.");
    let server = HttpServer::new(move || {
        App::new()
            .wrap(AuthenticateUser)
            .wrap(if settings.debug {
                SessionMiddleware::builder(redis_store.clone(), secret_key.clone())
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(cookie::time::Duration::days(7)),
                    )
                    .cookie_name("sessionId".to_string())
                    .cookie_same_site(cookie::SameSite::None)
                    .cookie_secure(false)
                    .build()
            } else {
                SessionMiddleware::builder(redis_store.clone(), secret_key.clone())
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(cookie::time::Duration::days(7)),
                    )
                    .cookie_name("sessionId".to_string())
                    .build()
            })
            .service(scope("/api")
                .service(crate::routes::health_check)
                .configure(auth_routes)
                .configure(ambition_routes)
                .configure(objective_routes)
                .configure(action_routes)
            )
            .app_data(Data::new(db.clone()))
            .app_data(Data::new(redis_pool.clone()))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
