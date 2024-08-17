use actix_web::{cookie, dev::Server, web::Data, App, HttpServer};
use deadpool_redis::Pool;
use sea_orm::*;
use std::env;
pub struct Application {
    port: u16,
    server: Server,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
    pub redis_pool: Pool,
}

impl Application {
    pub async fn build(settings: crate::settings::Settings) -> Result<Self, std::io::Error> {
        let db = get_database_connection().await;
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
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let cfg = deadpool_redis::Config::from_url(redis_url);
    let redis_pool = cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("Cannot create deadpool redis.");
    let state = AppState {
        conn: db,
        redis_pool,
    };

    let secret_key = cookie::Key::from(settings.secret.hmac_secret.as_bytes());
    let server = HttpServer::new(move || {
        App::new()
            .wrap(if settings.debug {
                actix_session::SessionMiddleware::builder(
                    actix_session::storage::CookieSessionStore::default(),
                    secret_key.clone(),
                )
                .cookie_http_only(true)
                .cookie_same_site(cookie::SameSite::None)
                .cookie_secure(true)
                .build()
            } else {
                actix_session::SessionMiddleware::new(
                    actix_session::storage::CookieSessionStore::default(),
                    secret_key.clone(),
                )
            })
            .service(crate::routes::health_check)
            .configure(crate::routes::auth_routes_config)
            .app_data(Data::new(state.clone()))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
