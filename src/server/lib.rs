use actix_session::{
    config::{PersistentSession, SessionMiddlewareBuilder},
    storage::RedisSessionStore,
};
use actix_web::{cookie, web::scope, Scope};
use common::settings::types::Settings;
use routes::{auth_routes, desired_state_routes, diary_routes, reading_note_routes, tag_routes};
use web_adapters::{
    action_routes, action_track_routes, ambition_routes, desired_state_category_routes,
};

pub async fn get_preps_for_redis_session_store(
    settings: &Settings,
    redis_url: &str,
) -> (RedisSessionStore, cookie::Key) {
    let secret_key = cookie::Key::from(settings.secret.hmac_secret.as_bytes());
    let redis_store = RedisSessionStore::new(redis_url)
        .await
        .expect("Error on getting RedisSessionStore");
    (redis_store, secret_key)
}

pub fn setup_session_middleware_builder(
    builder: SessionMiddlewareBuilder<RedisSessionStore>,
    settings: &Settings,
) -> SessionMiddlewareBuilder<RedisSessionStore> {
    if settings.debug {
        builder
            .session_lifecycle(
                PersistentSession::default().session_ttl(cookie::time::Duration::days(7)),
            )
            .cookie_name("sessionId".to_string())
            .cookie_same_site(cookie::SameSite::None)
            .cookie_secure(false)
    } else {
        builder
            .session_lifecycle(
                PersistentSession::default().session_ttl(cookie::time::Duration::days(7)),
            )
            .cookie_name("sessionId".to_string())
    }
}

pub fn get_routes() -> Scope {
    scope("/api")
        .service(routes::health_check)
        .configure(auth_routes)
        .configure(ambition_routes)
        .configure(desired_state_routes)
        .configure(action_routes)
        .configure(reading_note_routes)
        .configure(tag_routes)
        .configure(action_track_routes)
        .configure(diary_routes)
        .configure(desired_state_category_routes)
}

#[actix_web::get("/health-check")]
pub async fn health_check() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json("Application is safe and healthy.")
}
