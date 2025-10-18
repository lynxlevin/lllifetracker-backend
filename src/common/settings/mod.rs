use std::env;

use crate::settings::types::{
    ApplicationSettings, DatabaseSettings, EmailSettings, Environment, RedisSettings,
    SecretSettings, Settings,
};

pub mod types;

pub fn get_settings(env_file_name: &str) -> Result<Settings, String> {
    dotenvy::from_filename(env_file_name)
        .map_err(|e| format!("Failed to fetch env file: {}", e.to_string()))?;

    match Environment::try_from(env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "production".into()))
    {
        Ok(env) => match env {
            Environment::Testing => get_development_settings(),
            Environment::Development => get_development_settings(),
            Environment::Production => get_production_settings(),
        },
        Err(e) => return Err(format!("Failed to parse APP_ENVIRONMENT: {}", e)),
    }
}

pub fn get_test_settings() -> Settings {
    get_settings(".env.testing").expect("Error on getting settings.")
}

fn get_development_settings() -> Result<Settings, String> {
    let b = Settings::base_settings();
    merge_env(Settings {
        application: ApplicationSettings {
            protocol: "http".to_string(),
            host: "127.0.0.1".to_string(),
            base_url: "http://127.0.0.1".to_string(),
            frontend_url: "https://localhost:3000".to_string(),
            ..b.application
        },
        debug: true,
        secret: SecretSettings {
            token_expiration: 30,
            ..b.secret
        },
        ..b
    })
}

fn get_production_settings() -> Result<Settings, String> {
    let b = Settings::base_settings();
    merge_env(Settings {
        application: ApplicationSettings {
            protocol: "https".to_string(),
            host: "0.0.0.0".to_string(),
            base_url: "".to_string(),
            frontend_url: "https://localhost:3000".to_string(),
            ..b.application
        },
        debug: false,
        secret: SecretSettings {
            token_expiration: 30,
            ..b.secret
        },
        ..b
    })
}

fn merge_env(s: Settings) -> Result<Settings, String> {
    Ok(Settings {
        application: ApplicationSettings {
            max_login_attempts: get_env_var("MAX_LOGIN_ATTEMPTS")?
                .parse::<u64>()
                .map_err(|e| e.to_string())?,
            login_attempts_cool_time_seconds: get_env_var("LOGIN_ATTEMPTS_COOL_TIME_SECONDS")?
                .parse::<u64>()
                .map_err(|e| e.to_string())?,
            vapid_private_key: get_env_var("VAPID_PRIVATE_KEY")?,
            app_owner_email: get_env_var("APP_OWNER_EMAIL")?,
            ..s.application
        },
        database: DatabaseSettings {
            url: get_env_var("DATABASE_URL")?,
            encryption_key: get_env_var("DATABASE_ENCRYPTION_KEY")?,
            encryption_nonce: get_env_var("DATABASE_ENCRYPTION_NONCE")?,
        },
        debug: match env::var("APP_DEBUG") {
            Ok(debug) => &debug == "true",
            Err(_) => s.debug,
        },
        redis: RedisSettings {
            url: get_env_var("REDIS_URL")?,
            ..s.redis
        },
        secret: SecretSettings {
            secret_key: get_env_var("APP_SECRET__SECRET_KEY")?,
            hmac_secret: get_env_var("APP_SECRET__HMAC_SECRET")?,
            ..s.secret
        },
        email: EmailSettings {
            no_verify: match env::var("APP_EMAIL__NO_VERIFY") {
                Ok(no_verify) => &no_verify == "true",
                Err(_) => s.email.no_verify,
            },
            host: get_env_var("APP_EMAIL__HOST")?,
            host_user: get_env_var("APP_EMAIL__HOST_USER")?,
            host_user_password: get_env_var("APP_EMAIL__HOST_USER_PASSWORD")?,
            sender: get_env_var("APP_EMAIL__SENDER")?,
            ..s.email
        },
        ..s
    })
}

fn get_env_var(key: &str) -> Result<String, String> {
    env::var(key).map_err(|e| e.to_string())
}
