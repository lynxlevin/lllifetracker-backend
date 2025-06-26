use actix_http::{encoding::Encoder, Request};
use actix_session::SessionMiddleware;
use actix_web::{
    body::{BoxBody, EitherBody},
    dev::{Service, ServiceResponse},
    middleware::Compress,
    test,
    web::Data,
    App, Error,
};
use common::{
    db::init_db,
    redis::init_redis_pool,
    settings::{get_test_settings, types::Settings},
};
use sea_orm::{DbConn, DbErr};
use server::{
    auth_middleware::AuthenticateUser, get_preps_for_redis_session_store, get_routes,
    setup_session_middleware_builder,
};

pub struct Connections<
    S: Service<Request, Response = ServiceResponse<EitherBody<Encoder<BoxBody>>>, Error = Error>,
> {
    pub app: S,
    pub db: DbConn,
    pub settings: Settings,
}

pub async fn init_app() -> Result<
    Connections<
        impl Service<Request, Response = ServiceResponse<EitherBody<Encoder<BoxBody>>>, Error = Error>,
    >,
    DbErr,
> {
    let settings = get_test_settings();
    // let _ = env_logger::try_init();
    let db = init_db(&settings).await;
    let redis_pool = init_redis_pool(&settings)
        .await
        .expect("Error on getting Redis pool.");

    let (redis_store, secret_key) =
        get_preps_for_redis_session_store(&settings, &settings.redis.url).await;

    let app = test::init_service(
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
            .app_data(Data::new(settings.clone())),
    )
    .await;
    Ok(Connections { app, db, settings })
}
