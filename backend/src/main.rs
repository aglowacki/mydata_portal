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
use tokio::sync::broadcast;

use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;

//use std::sync::Arc;

mod auth;
mod sse;
mod database;
mod appstate;

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

    let (sse_tx, _rx) = broadcast::channel::<String>(100);
    tracing::debug!("successfully connected to redis and pinged it");
    let db_url = std::env::var("DATABASE_URL").unwrap();
    // set up connection pool
    let db_config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(db_url);
    let diesel_pool = bb8::Pool::builder().build(db_config).await.unwrap();
    let redis_client = redis::Client::open("redis://localhost").unwrap();
    let app_state = appstate::AppState { diesel_pool, redis_client, sse_tx };

    //let app_state = appstate::AppState::new(db_config, "redis://localhost", tx.clone()).await;

    tokio::spawn(sse::redis_event_listener(app_state.clone()));

    // Create a regular axum app.
    let app = Router::new()
        .route("/api/slow", get(|| sleep(Duration::from_secs(5))))
        //.route("/api/forever", get(std::future::pending::<()>))
        .route("/api/user_info", get(user_info))
        .route("/api/authorize", post(auth::authorize))
        .route("/api/sse", get(sse::sse_handler))
        .route("/api/get_user_proposals", get(database::get_user_proposals))
        .route("/api/get_user_proposals_as/{user_id}", get(database::get_user_proposals_as))
        .route("/api/get_user_proposals_with_datasets/{user_id}", get(database::get_user_proposals_with_datasets))
        .layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
            )).with_state(app_state);

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

