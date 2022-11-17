pub mod config;
mod error;

mod root;

use std::{net::SocketAddr, time::Duration};

use axum::{routing::get, Router, Server};
use sqlx::postgres::PgPoolOptions;

use config::Config;
use error::Result;

use root::root;

#[tokio::main]
async fn main() -> Result<()>
{
    tracing_subscriber::fmt::init();

    let config = Config::init()?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(config.database_url())
        .await?;

    let app = Router::with_state(pool).route("/", get(root));

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port()));
    tracing::debug!("listening on {}", addr);

    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}
