//! Run with
//!
//! ```not_rust
//! cargo run -p mydata-backend
//! kill or ctrl-c
//! ```

use std::time::Duration;
use axum::{
    middleware,
    routing::{get, post},
    Json, Router,
};

use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::{broadcast, watch};

use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;

use crate::appstate::RedisMessage;

//use std::sync::Arc;

mod auth;
mod sse;
mod database;
mod appstate;
mod beamline_controls;

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

    let (sse_tx, _rx) = broadcast::channel::<RedisMessage>(100);
    // Broadcast a shutdown signal to long-lived connections (e.g. SSE streams)
    // so they terminate on Ctrl+C instead of blocking graceful shutdown.
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    tracing::debug!("successfully connected to redis and pinged it");
    let db_url = std::env::var("DATABASE_URL").unwrap();
    // set up connection pool
    let db_config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(db_url);
    let diesel_pool = bb8::Pool::builder().build(db_config).await.unwrap();
    let redis_host = std::env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string());
    let redis_client = redis::Client::open(format!("redis://{}", redis_host)).unwrap();
    let app_state = appstate::AppState { diesel_pool, redis_client, sse_tx, shutdown_rx, };

    tokio::spawn(sse::redis_event_listener(app_state.clone()));

    // Create a regular axum app.
    let app = Router::new()
        .route("/api/user_info", get(user_info))
        //.route("/api/authorize", post(auth::authorize))
        .route("/api/authorize", post(database::authorize_user))
        .route("/api/sse/{beamline_id}", get(sse::sse_handler))
        .route("/api/get_user_proposals", get(database::get_user_proposals))
        .route("/api/get_all_proposals", get(database::get_all_proposals))
        .route("/api/get_user_proposals_as/{user_id}", get(database::get_user_proposals_as))
        .route("/api/get_user_proposals_with_datasets/{user_id}", get(database::get_user_proposals_with_datasets))
        .route("/api/search_user_proposals/{field}/{value}", get(database::search_user_proposals))
        .route("/api/search_proposals", get(database::search_proposals))
        .route("/api/get_proposal_search_options", get(database::get_proposal_search_options))
        .route("/api/get_syncotron_runs", get(database::get_syncotron_runs))
        // depricated for get_bio_sample_meta_data_groups
        //.route("/api/bio_sample_types", get(database::get_bio_sample_types)) 
        .route("/api/get_bio_sample_meta_data_groups", get(database::get_bio_sample_meta_data_groups))
        .route("/api/upsert_bio_sample", post(database::upsert_bio_sample))
        .route("/api/get_proposal_datasets/{proposal_id}", get(database::get_proposal_datasets))
        .route("/api/get_proposal_bio_samples/{proposal_id}", get(database::get_proposal_bio_samples))
        .route("/api/get_proposal_experimenters/{proposal_id}", get(database::get_proposal_experimenters))
        .route("/api/get_experiment_roles", get(database::get_experiment_roles))
        .route("/api/get_all_users", get(database::get_all_users))
        .route("/api/get_all_beamlines", get(database::get_all_beamlines))
        .route("/api/get_my_beamlines", get(database::get_my_beamlines))
        .route("/api/add_proposal_experimenter", post(database::add_proposal_experimenter))
        .route("/api/remove_proposal_experimenter", post(database::remove_proposal_experimenter))
        .route("/api/get_beamline_log/{beamline_id}", get(beamline_controls::get_beamline_log))
        .route("/api/get_available_scans/{beamline_id}", get(beamline_controls::get_available_scans))
        .route("/api/get_queued_scans/{beamline_id}", get(beamline_controls::get_queued_scans))
        .route("/api/get_beamline_worker_task_queue_waiting/{beamline_id}", get(beamline_controls::get_beamline_worker_task_queue_waiting))
        .route("/api/get_beamline_worker_task_queue_processing/{beamline_id}", get(beamline_controls::get_beamline_worker_task_queue_processing))
        .route("/api/get_beamline_worker_task_queue_done/{beamline_id}", get(beamline_controls::get_beamline_worker_task_queue_done))
        .route("/api/get_beamline_worker_task_queues/{beamline_id}", get(beamline_controls::get_beamline_worker_task_queues))
        .route("/api/get_beamline_worker_heartbeat/{beamline_id}", get(beamline_controls::get_beamline_worker_heartbeat))
        .route("/api/queue_beamline_worker_task/{beamline_id}", post(beamline_controls::queue_beamline_worker_task))
        
        // Extend the caller's session by an hour after each successful, authenticated request.
        .layer(middleware::from_fn(auth::refresh_token_layer))
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
        .with_graceful_shutdown(shutdown_signal(shutdown_tx))
        .await
        .unwrap();
}

async fn shutdown_signal(shutdown_tx: watch::Sender<bool>)
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

    // Notify long-lived connections (SSE streams) to close so graceful
    // shutdown doesn't hang waiting for clients to disconnect.
    let _ = shutdown_tx.send(true);
}

async fn user_info(claims: auth::Claims) -> Result<Json<auth::Claims>, auth::AuthError> 
{
    // Send the protected data to the user
    Ok(Json(claims))
}

