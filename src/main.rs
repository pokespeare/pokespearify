use anyhow::Context;

use pokespearify::config::Config;
use pokespearify::telemetry::{get_subscriber, init_subscriber};
use pokespearify::Application;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("pokespeare".into(), "info".into());
    init_subscriber(subscriber);

    let config = Config::collect().context("Failed to collect config")?;

    let app = Application::new(
        (config.host, config.port),
        config.poke_api_base_url,
        config.translator_api_base_url,
    )
    .await?;
    tracing::info!("Service is listening under {}", app.addr());
    app.run().await?;
    Ok(())
}
