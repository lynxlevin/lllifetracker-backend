use common::settings::get_settings;
use cron_processes::notification::run_cron_processes;
use tracing::{event, Level};

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

    if let Err(e) = run_cron_processes(settings.clone()).await {
        event!(
            Level::ERROR,
            "Some error in run_cron_processes, stopping server. {:?}",
            e
        );
        return Ok(());
    };

    let application = startup::Application::build(settings).await?;

    tracing::event!(target: "backend", tracing::Level::INFO, "Listening on http:/127.0.0.1:{}/", application.port());

    application.run_until_stopped().await?;

    drop(_guard);
    Ok(())
}
