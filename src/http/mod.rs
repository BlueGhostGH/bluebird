use std::net::SocketAddr;

use axum::{Extension, Router};
use sqlx::PgPool;
use tokio::signal;

use thiserror::Error;

mod json;
pub mod session;

mod auth;
mod users;

fn app(pg_pool: PgPool, session_store: session::Store) -> Router
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
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal()
{
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // NOTE: I don't run a Unix machine so I don't actually know if this works
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {}
    }
}

type Result<T> = ::core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error
{
    #[error("{0}")]
    Hyper(#[from] hyper::Error),
}

mod api_error
{
    use std::num::NonZeroU16;

    use serde::Serialize;

    macro_rules! code {
        ($name:ident, $code:expr) => {
            pub(super) const $name: Code = Code(unsafe { NonZeroU16::new_unchecked($code) });
        };
    }

    #[derive(Debug, Serialize)]
    pub(super) struct Code(NonZeroU16);

    // 100 - JSON Syntax Error
    // 110 - JSON Data Error
    // 120 - JSON Missing Content Type
    // 199 - JSON Unknown Error
    impl Code
    {
        #![allow(unsafe_code)]

        code!(JSON_SYNTAX_ERROR, 100);
        code!(JSON_DATA_ERROR, 110);
        code!(JSON_MISSING_CONTENT_TYPE, 120);
        code!(JSON_UNKNOWN_ERROR, 199);
    }
}
