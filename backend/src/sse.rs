use axum::response::sse::{Event, Sse};
use axum::extract::State;
use std::{convert::Infallible};
use futures::stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use std::time::Duration;

use super::appstate;
use crate::{auth};


//use std::{convert::Infallible, sync::{Arc, Mutex}};
//use tokio::sync::broadcast;
//use axum::response::IntoResponse;
//use tokio_stream::{Stream, StreamExt};
//use std::sync::Arc;
//use tokio_stream::StreamExt as _;
//use tokio::sync::broadcast;
//use axum_extra::TypedHeader;

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
    std::thread::sleep(Duration::from_millis(10));
}

/// SSE handler: each client gets its own subscription to the broadcast channel.
//async fn sse_handler(State(tx): State<Arc<broadcast::Sender<String>>>) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> 
pub async fn sse_handler(State(state): State<appstate::AppState>, claims: auth::Claims) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> 
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
                // Wrap each message in a Server‐Sent Event
                let event = Event::default().data(msg);
                Some(Ok(event))
            }
            // On lagging or closed channel, just skip
            Err(_) => None,
        }
    });

    Sse::new(stream)
}

// Define a shared broadcast channel
//type BroadcastSender = Arc<Mutex<broadcast::Sender<String>>>;
/* 
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

*/
/*
pub async fn sse_handler(TypedHeader(user_agent): TypedHeader<headers::UserAgent>,) -> Sse<impl Stream<Item = Result<Event, Infallible>>> 
{
    println!("`{}` connected", user_agent.as_str());

    // A `Stream` that repeats an event every second
    //
    // You can also create streams from tokio channels using the wrappers in
    // https://docs.rs/tokio-stream
    let stream = stream::repeat_with(|| Event::default().data("hi!"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive( KeepAlive::default() )
   //Sse::new(stream).keep_alive(
    //    axum::response::sse::KeepAlive::new()
     //       .interval(Duration::from_secs(1))
     //       .text("keep-alive-text"),
    //)
}

*/
/*
async fn add_sse_client(State(mut sse_handler): State(mut sse::SseHandler), id: String,) -> Result<(), Box<dyn std::error::Error>> 
{
    sse_handler.add_client(id)?;
    Ok(())
}

async fn sse_handler(TypedHeader(user_agent): TypedHeader<headers::UserAgent>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> 
{
    println!("`{}` connected", user_agent.as_str());

    // A `Stream` that repeats an event every second
    //
    // You can also create streams from tokio channels using the wrappers in
    // https://docs.rs/tokio-stream
    let stream = stream::repeat_with(|| Event::default().data("hi!"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
*/