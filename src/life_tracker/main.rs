use common::settings::get_settings;
use tracing::{event, Level};

mod startup;
mod telemetry;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = get_settings(".env").expect("Error on getting settings.");

    let _guard = telemetry::init_subscriber(settings.debug, settings.application.max_log_files);

    let application = startup::Application::build(settings).await?;

    event!(target: "backend", Level::INFO, "Listening on http:/127.0.0.1:{}/", application.port());

    application.run_until_stopped().await?;

    drop(_guard);
    Ok(())
}
