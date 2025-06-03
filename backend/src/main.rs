//! Run with
//!
//! ```not_rust
//! cargo run -p mydata-backend
//! kill or ctrl-c
//! ```

use std::time::Duration;
use axum::{
    routing::{get, post},
    Json, Router,
};

use tokio::net::TcpListener;
use tokio::signal;
use tokio::time::sleep;
//use tokio_postgres::NoTls;

use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;


//use serde::{Deserialize, Serialize};
//use serde_json::json;
//use std::fmt::Display;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//use ldap3::result::Result;
//use bb8_redis::RedisConnectionManager;
//use redis::AsyncCommands;

//use bb8::{Pool, PooledConnection};
//use bb8_postgres::PostgresConnectionManager;

//use bb8_redis::bb8;

use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection, RunQueryDsl,
};

mod auth;
mod sse;
mod database;


#[tokio::main]
async fn main() 
{
    // Enable tracing.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| 
                {
                "mydata_backend=debug,tower_http=debug,axum=trace".into()
                }),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();
/*
        let postgres_manager = PostgresConnectionManager::new_from_stringlike("host=localhost user=postgres", NoTls).unwrap();
        let postgres_pool = Pool::builder().build(postgres_manager).await.unwrap();
*/
/*
    let redis_manager = RedisConnectionManager::new("redis://localhost").unwrap();
    let redis_pool = bb8::Pool::builder().build(manager).await.unwrap();
    {
        // ping the database before starting
        let mut conn = redis_pool.get().await.unwrap();
        //let keys : Vec<String> = con.hkeys("access_token:*")?;
        //conn.keys::<&str,()> ("access_token:*")
        conn.set::<&str, &str, ()>("foo", "bar").await.unwrap();
        let result: String = conn.get("foo").await.unwrap();
        assert_eq!(result, "bar");
    }
    tracing::debug!("successfully connected to redis and pinged it");
*/
    let db_url = std::env::var("DATABASE_URL").unwrap();
    // set up connection pool
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(db_url);
    let app_state = database::AppState{pool: bb8::Pool::builder().build(config).await.unwrap()};

    // Create a regular axum app.
    let app = Router::new()
        .route("/api/slow", get(|| sleep(Duration::from_secs(5))))
        //.route("/api/forever", get(std::future::pending::<()>))
        .route("/api/user_info", get(user_info))
        .route("/api/authorize", post(auth::authorize))
        .route("/api/sse", get(sse::sse_handler))
        .route("/api/get_user_proposals", get(database::get_user_proposals))
        .layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
            )).with_state(app_state);
        //)).with_state(postgres_pool);

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());    

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() 
{
    let ctrl_c = async 
    {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async 
    {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! 
    {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn user_info(claims: auth::Claims) -> Result<Json<auth::Claims>, auth::AuthError> 
{
    // Send the protected data to the user
    Ok(Json(claims))
}

