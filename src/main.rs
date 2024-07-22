use llwinecellar_actix_backend::{settings as backend_settings, startup, telemetry};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let settings = backend_settings::get_settings().expect("Failed to read settings.");

    let subscriber = telemetry::get_subscriber(settings.clone().debug);
    telemetry::init_subscriber(subscriber);

    let application = startup::Application::build(settings).await?;

    tracing::event!(target: "backend", tracing::Level::INFO, "Listening on http:/127.0.0.1:{}/", application.port());

    application.run_until_stopped().await?;
    Ok(())
}
