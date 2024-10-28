use axum::{response::sse::Sse};
use tokio_stream::{Stream, StreamExt};
use std::{convert::Infallible, sync::{Arc, Mutex}};
use tokio::sync::broadcast;
use axum::response::IntoResponse;

// Define a shared broadcast channel
type BroadcastSender = Arc<Mutex<broadcast::Sender<String>>>;

// Handler to return SSE stream
pub fn sse_handler(tx: BroadcastSender) -> impl IntoResponse 
{
    move || 
    {
        let rx = tx.lock().unwrap().subscribe();

        let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
            .filter_map(|result| match result 
            {
                Ok(msg) => Some(Ok::<_, Infallible>(msg.into())),
                Err(_) => None,
            });

        Sse::new(stream)
    }
}

// Handler to send a message to all SSE streams
pub async fn send_message_handler(tx: BroadcastSender) -> impl IntoResponse 
{
    let msg = "Hello, SSE clients!".to_string();
    let _ = tx.lock().unwrap().send(msg);
    "Message sent!"
}