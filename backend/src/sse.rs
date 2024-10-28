use axum::response::sse::{Event, Sse};
use std::{convert::Infallible, time::Duration};
//use std::{convert::Infallible, sync::{Arc, Mutex}};
//use tokio::sync::broadcast;
//use axum::response::IntoResponse;
//use tokio_stream::{Stream, StreamExt};


use axum_extra::TypedHeader;
use futures::stream::{self, Stream};
use tokio_stream::StreamExt as _;
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

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}


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