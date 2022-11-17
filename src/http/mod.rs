use std::net::SocketAddr;

use axum::{Extension, Router};
use sqlx::PgPool;

mod error;
pub use error::{Error, Result};

mod users;

pub fn app(db_pool: PgPool) -> Router
{
    Router::new()
        .merge(users::router())
        .layer(Extension(db_pool))
}

pub async fn serve(port: u16, db_pool: PgPool) -> Result<()>
{
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    axum::Server::bind(&addr)
        .serve(app(db_pool).into_make_service())
        .await?;

    Ok(())
}
