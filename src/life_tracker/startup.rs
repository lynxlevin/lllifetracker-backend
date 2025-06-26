use actix_session::SessionMiddleware;
use actix_web::{dev::Server, middleware::Compress, web::Data, App, HttpServer};
use common::{db::init_db, redis::init_redis_pool, settings::types::Settings};
use sea_orm::DatabaseConnection;
use server::{
    auth_middleware::AuthenticateUser, get_preps_for_redis_session_store, get_routes,
    setup_session_middleware_builder,
};

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(settings: Settings) -> Result<Self, std::io::Error> {
        let db = init_db(&settings).await;
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

async fn run(
    listener: std::net::TcpListener,
    db: DatabaseConnection,
    settings: Settings,
) -> Result<Server, std::io::Error> {
    let redis_pool = init_redis_pool(&settings)
        .await
        .expect("Cannot create deadpool redis.");

    let (redis_store, secret_key) =
        get_preps_for_redis_session_store(&settings, &settings.redis.url).await;

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
            .app_data(Data::new(settings.clone()))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
