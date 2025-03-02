mod startup;
mod telemetry;
use settings as backend_settings;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let settings = backend_settings::get_settings();

    // MYMEMO: introduce tracing_actix_web
    // - Should log some message at the beginning of all main functions.
    // - Logs should be more readable.
    // - More system info should be collected automatically.
    let _guard =
        telemetry::init_subscriber(settings.debug.clone(), settings.application.max_log_files);

    let application = startup::Application::build(settings).await?;

    tracing::event!(target: "backend", tracing::Level::INFO, "Listening on http:/127.0.0.1:{}/", application.port());

    application.run_until_stopped().await?;

    drop(_guard);
    Ok(())
}
