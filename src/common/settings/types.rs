use serde::Deserialize;

#[derive(Deserialize, Clone, Default)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub debug: bool,
    pub redis: RedisSettings,
    pub secret: SecretSettings,
    pub email: EmailSettings,
}

impl Settings {
    pub fn base_settings() -> Self {
        Self {
            application: ApplicationSettings {
                port: 5000,
                max_log_files: 14,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Clone, Default)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub frontend_url: String,
    pub protocol: String,
    pub max_log_files: usize,
    pub max_login_attempts: u64,
    pub login_attempts_cool_time_seconds: u64,
    pub vapid_private_key: String,
    pub app_owner_email: String,
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct DatabaseSettings {
    pub url: String,
    pub encryption_key: String,
    pub encryption_nonce: String,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct RedisSettings {
    pub url: String,
}

#[derive(Deserialize, Clone, Default)]
pub struct SecretSettings {
    pub secret_key: String,
    pub token_expiration: i64,
    pub hmac_secret: String,
}

#[derive(Deserialize, Clone, Default)]
pub struct EmailSettings {
    pub no_verify: bool,
    pub host: String,
    pub host_user: String,
    pub host_user_password: String,
    pub sender: String,
}

pub enum Environment {
    Testing,
    Development,
    Production,
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "testing" => Ok(Self::Testing),
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(format!("{} is not a supported environment.", other)),
        }
    }
}
