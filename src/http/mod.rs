use std::net::SocketAddr;

use axum::{Extension, Router};
use sqlx::PgPool;

use crate::session::store::Store;

mod error;
pub use error::{Error, Result};

mod auth;
mod users;

pub fn app(db_pool: PgPool, session_store: Store) -> Router
{
    Router::new()
        .merge(auth::router())
        .merge(users::router())
        .layer(Extension(db_pool))
        .layer(Extension(session_store))
}

pub async fn serve(port: u16, db_pool: PgPool, session_store: Store) -> Result<()>
{
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    axum::Server::bind(&addr)
        .serve(app(db_pool, session_store).into_make_service())
        .await?;

    Ok(())
}
