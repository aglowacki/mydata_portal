//! Run with
//!
//! ```not_rust
//! cargo run -p mydata-backend
//! kill or ctrl-c
//! ```

use std::time::Duration;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, RequestPartsExt, Router,
};

use tokio::net::TcpListener;
use tokio::signal;
use tokio::time::sleep;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;


use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Display;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//use ldap3::result::Result;
use ::bb8::{Pool, PooledConnection};
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;

use bb8_redis::bb8;

mod auth;


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

    let manager = RedisConnectionManager::new("redis://localhost").unwrap();
    let pool = bb8::Pool::builder().build(manager).await.unwrap();

    {
        // ping the database before starting
        let mut conn = pool.get().await.unwrap();
        //let keys : Vec<String> = con.hkeys("access_token:*")?;
        ///conn.keys::<&str,()> ("access_token:*")
        conn.set::<&str, &str, ()>("foo", "bar").await.unwrap();
        let result: String = conn.get("foo").await.unwrap();
        assert_eq!(result, "bar");
    }
    tracing::debug!("successfully connected to redis and pinged it");

    // Create a regular axum app.
    let app = Router::new()
        .route("/api/slow", get(|| sleep(Duration::from_secs(5))))
        .route("/api/forever", get(std::future::pending::<()>))
        .route("/api/user_info", get(protected))
        .route("/api/authorize", post(auth::authorize))
        .layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
        ));

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

async fn protected(claims: auth::Claims) -> Result<Json<auth::Claims>, auth::AuthError> 
{
    // Send the protected data to the user
    Ok(Json(claims))
}


