use std::{net::SocketAddr, time::Duration};

use axum::{extract::State, http::StatusCode, routing::get, Router};
use sqlx::postgres::{PgPool, PgPoolOptions};

#[tokio::main]
async fn main()
{
    tracing_subscriber::fmt::init();

    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| String::from("postgres://postgres:postgres@localhost/bluebird"));

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("Cannot connect to database");

    let app = Router::with_state(pool).route("/", get(root));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root(State(pool): State<PgPool>) -> Result<String, (StatusCode, String)>
{
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(256_i64)
        .fetch_one(&pool)
        .await
        .map_err(internal_err)?;

    Ok(row.0.to_string())
}

fn internal_err<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
