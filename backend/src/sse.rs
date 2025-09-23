use axum::response::sse::{Event, Sse, KeepAlive};
use axum::extract::State;
use std::{convert::Infallible};
use futures::stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use std::time::Duration;

use super::appstate;

pub async fn redis_event_listener(state: appstate::AppState) 
{
    let mut pubsub = state.redis_client.get_async_pubsub().await.unwrap();
    pubsub.subscribe("events").await.unwrap();

    println!("Subscribed to Redis channel: events");
    
    while let Some(msg) = pubsub.on_message().next().await 
    {  
        if let Ok(payload) = msg.get_payload::<String>() 
        {
            println!("Received Redis event: {}", payload);

            // Broadcast the event to all connected clients
            let _ = state.sse_tx.send(payload);
        }
    }

}

/// SSE handler: each client gets its own subscription to the broadcast channel.
//async fn sse_handler(State(tx): State<Arc<broadcast::Sender<String>>>) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> 
pub async fn sse_handler(State(state): State<appstate::AppState>) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> 
{
    // Subscribe to our broadcast channel
    let rx =  state.sse_tx.subscribe();

    // Wrap it into a Stream of SSE Events
    let stream = BroadcastStream::new(rx).filter_map(|result| async move 
    {
        match result 
        {
            Ok(msg) => 
            {
                println!("Sending sse event: {}", msg);
                // Wrap each message in a Server‐Sent Event
                let event = Event::default().data(msg);
                Some(Ok(event))
            }
            // On lagging or closed channel, just skip
            Err(E) => 
            {
                println!("Error sse event: {}", E);
                None
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(10)).text("ping"))
}
