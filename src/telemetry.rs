use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::layer::SubscriberExt;

pub fn get_subscriber(
    debug: bool,
    max_log_files: usize,
) -> (impl tracing::Subscriber + Send + Sync, WorkerGuard) {
    let env_filter = if debug {
        "debug".to_string()
    } else {
        "info".to_string()
    };
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(env_filter));

    let json_log = if !debug {
        let json_log = tracing_subscriber::fmt::layer().json();
        Some(json_log)
    } else {
        None
    };

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_suffix("log")
        .max_log_files(max_log_files)
        .build("./logs")
        .expect("initializing rolling file appender failed");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file_log = tracing_subscriber::fmt::layer().with_writer(non_blocking);

    let stdout_log = tracing_subscriber::fmt::layer().pretty();
    let subscriber = tracing_subscriber::Registry::default()
        .with(env_filter)
        .with(stdout_log)
        .with(json_log)
        .with(file_log);

    (subscriber, _guard)
}

pub fn init_subscriber(debug: bool, max_log_files: usize) -> WorkerGuard {
    let (subscriber, _guard) = get_subscriber(debug, max_log_files);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
    _guard
}
