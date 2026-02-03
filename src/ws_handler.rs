use futures_util::{SinkExt, StreamExt as _};
use tokio::sync::broadcast;
use warp::filters::ws::{Message, WebSocket};
use crate::db::ChatMessage;


pub async fn handle_connection(pool: sqlx::MySqlPool, ws: WebSocket, tx: broadcast::Sender<String>) {
    let (mut ws_sender, mut ws_receiver) = ws.split();
    let mut rx = tx.subscribe();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_sender.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });
    // for every item in the web socket stream
    while let Some(result) = ws_receiver.next().await {
        //do this
        match result {
            Ok(message) => {
                if let Ok(text) = message.to_str() {
                    if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(text) {
                        if crate::tables::user_db::find_user_by_username(&pool, &chat_msg.username).await.is_ok() {
                            //lock thread until sender mutex acquired, then sends received message to the broadcast
                            tx.send(text.to_string()).expect("Failed to broadcast message");
                        } else {
                            println!("Unauthorized user_id: {}", chat_msg.username);
                        }
                    } else {
                        println!("Failed to parse message: {}", text);
                    }
                }
            },
            Err(_e) => break,
        }
    }
}