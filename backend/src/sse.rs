use axum::response::sse::{Event, Sse, KeepAlive};
use axum::extract::{Path, State};
use std::{convert::Infallible};
use futures::stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use std::time::Duration;
use futures::future;

use crate::database::{get_beamlines};
use super::database::models::Beamline;
use super::appstate::{self, RedisMessage};

pub async fn redis_event_listener(state: appstate::AppState) 
{
    //let mut conn: bb8::PooledConnection<'_, diesel_async::pooled_connection::AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>> = state.diesel_pool.get_owned().await.unwrap();
    let beamlines: Vec<Beamline> = get_beamlines(&state).await;
    let mut channel_names: Vec<String> = Vec::new();

    println!("Subscribed to Redis channels: "); 
    for beamline in &beamlines
    {
        let mut chan =  String::from("BEAMLINE_SCAN_LOGS_");
        chan.push_str(&beamline.acronym.clone());
        println!("{}", chan);
        channel_names.push(chan);
    }

    let mut pubsub = state.redis_client.get_async_pubsub().await.unwrap();
    pubsub.subscribe(&channel_names).await.unwrap();
    let mut stream_msg = pubsub.on_message();
    while let Some(msg) = stream_msg.next().await 
    {  
        let channel = msg.get_channel_name().to_string();
        let payload: String = msg.get_payload().unwrap();
        let _ = state.sse_tx.send(RedisMessage { channel, payload });
    }
}

/// SSE handler: each client gets its own subscription to the broadcast channel.
pub async fn sse_handler(
    Path(beamline_id): Path<String>,
    State(state): State<appstate::AppState>
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> 
{
    // Subscribe to our broadcast channel
    let rx =  state.sse_tx.subscribe();

    // Wrap it into a Stream of SSE Events
    let stream = BroadcastStream::new(rx).filter_map( move |result| 
    {
        let target = beamline_id.clone();    
        // Return a Future using future::ready
        future::ready(match result 
        {
            Ok(msg) if msg.channel == target => 
            {
                Some(Ok(Event::default().data(msg.payload)))
            }
            _ => None,
        })
    });

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(10)).text("ping"))
}
