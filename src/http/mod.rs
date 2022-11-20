use std::net::SocketAddr;

use axum::{Extension, Router};
use sqlx::PgPool;
use thiserror::Error;

pub mod session;

mod auth;
mod users;

pub fn app(pg_pool: PgPool, session_store: session::Store) -> Router
{
    Router::new()
        .merge(auth::router())
        .merge(users::router())
        .layer(Extension(pg_pool))
        .layer(Extension(session_store))
}

pub async fn serve(port: u16, pg_pool: PgPool, session_store: session::Store) -> Result<()>
{
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    axum::Server::bind(&addr)
        .serve(app(pg_pool, session_store).into_make_service())
        .await?;

    Ok(())
}

type Result<T> = ::core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error
{
    #[error("{0}")]
    Hyper(#[from] hyper::Error),
}
