
use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection,
};
use axum::extract::FromRef;
use tokio::sync::broadcast;

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
}

impl FromRef<AppState> for DieselPool 
{
    fn from_ref(state: &AppState) -> DieselPool 
    {
        state.diesel_pool.clone()
    }
}