
use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection,
};
use axum::extract::FromRef;
use tokio::sync::{broadcast, watch};

pub type DieselPool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(Clone, Debug)]
pub struct RedisMessage {
    pub channel: String,
    pub payload: String,
}

#[derive(Clone)]
pub struct AppState 
{
    pub diesel_pool: DieselPool,
    pub redis_client: redis::Client,
    pub sse_tx: broadcast::Sender::<RedisMessage>,
    /// Set to `true` when the server is shutting down so long-lived
    /// connections (e.g. SSE streams) can terminate instead of hanging.
    pub shutdown_rx: watch::Receiver<bool>,
}

impl FromRef<AppState> for DieselPool 
{
    fn from_ref(state: &AppState) -> DieselPool 
    {
        state.diesel_pool.clone()
    }
}