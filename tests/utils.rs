use actix_http::Request;
use actix_web::{
    dev::{Service, ServiceResponse},
    test,
    web::Data,
    App,
};
use common::{db::init_db, redis::init_redis_pool, settings::get_test_settings};
use sea_orm::{DbConn, DbErr};
use server::get_routes;

pub async fn init_app() -> Result<
    (
        impl Service<Request, Response = ServiceResponse, Error = actix_web::Error>,
        DbConn,
    ),
    DbErr,
> {
    let settings = get_test_settings();
    let db = init_db(&settings).await;
    let redis_pool = init_redis_pool(&settings)
        .await
        .expect("Error on getting Redis pool.");

    let app = test::init_service(
        App::new()
            .service(get_routes())
            .app_data(Data::new(db.clone()))
            .app_data(Data::new(redis_pool.clone()))
            .app_data(Data::new(settings)),
    )
    .await;
    Ok((app, db))
}
