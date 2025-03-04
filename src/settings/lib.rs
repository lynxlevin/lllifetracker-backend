#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub debug: bool,
    pub redis: RedisSettings,
    pub secret: Secret,
    pub email: EmailSettings,
    pub frontend_url: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub protocol: String,
    pub max_log_files: usize,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct RedisSettings {
    pub pool_max_open: u64,
    pub pool_max_idle: u64,
    pub pool_timeout_seconds: u64,
    pub pool_expire_seconds: u64,
}

#[derive(serde::Deserialize, Clone)]
pub struct Secret {
    pub secret_key: String,
    pub token_expiration: i64,
    pub hmac_secret: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailSettings {
    pub no_verify: bool,
    pub host: String,
    pub host_user: String,
    pub host_user_password: String,
    pub sender: String,
}

pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either 'development' or 'production'.",
                other
            )),
        }
    }
}

pub fn get_settings() -> Settings {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory.");
    let settings_directory = base_path.join("settings");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(settings_directory.join("base.yaml")))
        .add_source(config::File::from(
            settings_directory.join(environment_filename),
        ))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. 'APP_APPLICATION__PORT=5001' would set 'Settings.application.port'
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()
        .expect("Failed to build config.");

    settings
        .try_deserialize::<Settings>()
        .expect("Failed to read settings.")
}

pub static ENV: once_cell::sync::Lazy<minijinja::Environment<'static>> =
    once_cell::sync::Lazy::new(|| {
        let mut env = minijinja::Environment::new();
        env.set_loader(minijinja::path_loader("templates"));
        env
    });