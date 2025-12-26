use common::settings::get_settings;
use cron_processes::notification::run_cron_processes;

mod startup;
mod telemetry;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = get_settings(".env").expect("Error on getting settings.");

    // MYMEMO: introduce tracing_actix_web
    // - Should log some message at the beginning of all main functions.
    // - Logs should be more readable.
    // - More system info should be collected automatically.
    let _guard = telemetry::init_subscriber(settings.debug, settings.application.max_log_files);

    run_cron_processes(&settings).await;

    let application = startup::Application::build(settings).await?;

    tracing::event!(target: "backend", tracing::Level::INFO, "Listening on http:/127.0.0.1:{}/", application.port());

    application.run_until_stopped().await?;

    drop(_guard);
    Ok(())
}
