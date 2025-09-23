
use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection,
};
use axum::extract::FromRef;
use tokio::sync::broadcast;

pub type DieselPool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(Clone)]
pub struct AppState 
{
    pub diesel_pool: DieselPool,
    pub redis_client: redis::Client,
    pub sse_tx: broadcast::Sender::<String>,
}

impl AppState
{
    pub async fn new(diesel_config: AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>, redis_config: &str) -> Self
    {
        let (tx, _rx) = broadcast::channel::<String>(100);
        Self 
        {   
            diesel_pool: bb8::Pool::builder().build(diesel_config).await.unwrap(),
            redis_client: redis::Client::open(redis_config).unwrap(),
            sse_tx: tx,
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