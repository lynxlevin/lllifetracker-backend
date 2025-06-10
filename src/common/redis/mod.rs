use deadpool_redis::{Config, CreatePoolError, Pool, Runtime};

use crate::settings::types::Settings;

pub async fn init_redis_pool(settings: &Settings) -> Result<Pool, CreatePoolError> {
    let cfg = Config::from_url(&settings.redis.url);
    let redis_pool = cfg.create_pool(Some(Runtime::Tokio1))?;

    Ok(redis_pool)
}
