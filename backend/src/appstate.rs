
use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection,
};
use axum::extract::FromRef;
use redis::{Client as RedisClient};
use tokio::sync::broadcast::{Sender};

pub type DieselPool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(Clone)]
pub struct AppState 
{
    pub diesel_pool: DieselPool,
    pub redis_client: RedisClient,
    pub sse_tx: std::sync::Arc<Sender::<String>>,
}

impl AppState
{
    pub async fn new(diesel_config: AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>, redis_config: &str, tx: &std::sync::Arc<Sender::<String>>) -> Self
    {
        Self 
        {   
            diesel_pool: bb8::Pool::builder().build(diesel_config).await.unwrap(),
            redis_client: RedisClient::open(redis_config).unwrap(),
            sse_tx: tx.clone(),
        }
    }
}

impl FromRef<AppState> for DieselPool 
{
    fn from_ref(state: &AppState) -> DieselPool 
    {
        state.diesel_pool.clone()
    }
}